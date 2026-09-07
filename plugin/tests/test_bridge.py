import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('bridge', Path(__file__).parents[1] / 'bridge.py')
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)

class BridgeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name)
        self.patcher = patch.multiple(bridge, RUNTIME=root, CONFIG=root/'config.toml', USAGE=root/'usage.json', BINARY=root/'smellytype')
        self.patcher.start()
        self.addCleanup(self.patcher.stop)
        bridge.CONFIG.write_text('[audio]\nmax_duration_secs = 180\n')
        (root/'state').write_text('idle')
        (root/'pid').write_text(str(os.getpid()))

    def test_timeout_targets_cloud_field_and_rejects_invalid_values(self):
        with patch.object(bridge, 'run') as run:
            with self.assertRaises(ValueError): bridge.main(['timeout', '301'])
            run.assert_not_called()
            bridge.main(['timeout', '120'])
            self.assertIn('doubao.final_timeout_secs', run.call_args_list[0].args)
            self.assertEqual(run.call_args_list[0].args[-1], '120')

    def test_live_and_persisted_time(self):
        bridge.USAGE.write_text(json.dumps({'total_ms': 60000, 'sessions': 2}))
        (bridge.RUNTIME/'state').write_text('streaming')
        (bridge.RUNTIME/'dictation-active.json').write_text(json.dumps({'pid': os.getpid(), 'started_ms': int(time.time()*1000)-2000}))
        result = bridge.status()
        self.assertTrue(result['recording'])
        self.assertEqual(result['total_ms'], 60000)
        self.assertGreaterEqual(result['elapsed_ms'], 2000)
        (bridge.RUNTIME/'pid').unlink()
        self.assertEqual(bridge.status()['state'], 'offline')
        self.assertFalse(bridge.status()['recording'])

    def test_no_restart_while_busy_or_out_of_range(self):
        with patch.object(bridge, 'run') as run:
            with self.assertRaises(ValueError): bridge.main(['limit', '901'])
            (bridge.RUNTIME/'state').write_text('streaming')
            with self.assertRaises(ValueError): bridge.main(['limit', '60'])
            run.assert_not_called()

    def test_restart_failure_restores_config(self):
        original = bridge.CONFIG.read_bytes()
        def fail(*args):
            if args[0] == str(bridge.BINARY): bridge.CONFIG.write_text('[audio]\nmax_duration_secs = 60\n')
            elif 'restart' in args: raise subprocess.CalledProcessError(1, args)
        with patch.object(bridge, 'run', side_effect=fail):
            with self.assertRaises(subprocess.CalledProcessError): bridge.main(['limit', '60'])
        self.assertEqual(bridge.CONFIG.read_bytes(), original)

    def test_recording_start_during_setting_does_not_restart(self):
        original = bridge.CONFIG.read_bytes()
        def change(*args):
            bridge.CONFIG.write_text('[audio]\nmax_duration_secs = 60\n')
            (bridge.RUNTIME/'state').write_text('streaming')
        with patch.object(bridge, 'run', side_effect=change) as run:
            with self.assertRaises(ValueError): bridge.main(['limit', '60'])
            self.assertEqual(run.call_count, 1)
        self.assertEqual(bridge.CONFIG.read_bytes(), original)

    def test_final_recognition_cannot_start_again(self):
        (bridge.RUNTIME/'state').write_text('transcribing')
        with patch.object(bridge, 'run') as run:
            with self.assertRaises(ValueError): bridge.main(['toggle'])
            run.assert_not_called()

if __name__ == '__main__': unittest.main()
