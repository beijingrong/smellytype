import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    'startup', Path(__file__).resolve().parents[1] / 'fix_service_startup.py')
startup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(startup)


class StartupTests(unittest.TestCase):
    def exercise(self, enabled, fail=False):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            units = home / '.config/systemd/user'
            units.mkdir(parents=True)
            service = units / 'smellytype.service'
            original = '[Service]\nExecStart=/usr/bin/true\n[Install]\nWantedBy=default.target\n'
            service.write_text(original)
            old_link = units / 'default.target.wants/smellytype.service'
            new_link = units / 'graphical-session.target.wants/smellytype.service'
            if enabled:
                old_link.parent.mkdir()
                old_link.symlink_to(service)
            calls = []
            def run(args, **kwargs):
                calls.append(args)
                if 'is-enabled' in args:
                    return subprocess.CompletedProcess(args, 0 if enabled else 1,
                                                       stdout='enabled\n' if enabled else 'disabled\n')
                if args[0] == 'systemd-analyze' and fail:
                    raise subprocess.CalledProcessError(1, args)
                if 'reenable' in args:
                    old_link.unlink(missing_ok=True)
                    new_link.unlink(missing_ok=True)
                    target = new_link if (units / 'smellytype.service.d/90-graphical-session.conf').exists() else old_link
                    target.parent.mkdir(exist_ok=True)
                    target.symlink_to(service)
                return subprocess.CompletedProcess(args, 0, stdout='')
            with patch.object(subprocess, 'run', side_effect=run):
                if fail:
                    with self.assertRaises(subprocess.CalledProcessError):
                        startup.migrate(home)
                else:
                    startup.migrate(home)
            self.assertEqual(service.read_text(), original)
            self.assertEqual(old_link.exists(), enabled and fail)
            self.assertEqual(new_link.exists(), enabled and not fail)
            self.assertFalse(any(word in args for args in calls
                                 for word in ('--now', 'start', 'stop', 'restart')))

    def test_enabled_service_moves_without_interrupting_dictation(self):
        self.exercise(True)

    def test_disabled_service_stays_disabled(self):
        self.exercise(False)

    def test_failed_validation_restores_old_startup(self):
        self.exercise(True, fail=True)
