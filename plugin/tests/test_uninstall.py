import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('uninstall', Path(__file__).parents[1]/'uninstall.py')
uninstall = importlib.util.module_from_spec(spec)
spec.loader.exec_module(uninstall)

class UninstallTests(unittest.TestCase):
    def test_removal_preserves_unrelated_entry_and_cancelled_operation(self):
        for owned, cancelled in ((True, False), (False, False), (True, True)):
            with self.subTest(owned=owned, cancelled=cancelled), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                desktop = root/'data/applications/smellytype-configure.desktop'
                desktop.parent.mkdir(parents=True)
                target = root/'config/omarchy/plugins/beijingrong.smellytype/configure.py'
                desktop.write_text('Exec=' + (str(target) if owned else '/custom/configure'))
                error = subprocess.CalledProcessError(1, 'omarchy') if cancelled else None
                with patch.dict(os.environ, {'XDG_DATA_HOME': str(root/'data'), 'XDG_CONFIG_HOME': str(root/'config')}), patch.object(uninstall.subprocess, 'run', side_effect=error):
                    if cancelled:
                        with self.assertRaises(subprocess.CalledProcessError): uninstall.main()
                    else: uninstall.main()
                self.assertEqual(desktop.exists(), cancelled or not owned)
