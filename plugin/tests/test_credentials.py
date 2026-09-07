import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parents[1]))
import credentials

class CredentialsTests(unittest.TestCase):
    def test_secret_write_and_rollback(self):
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory)/'config.toml'
            secret = config.with_name('doubao.env')
            original = b'DOUBAO_API_KEY="old-test-key"\nOTHER=keep\n'
            secret.write_bytes(original)
            with patch.object(credentials.bridge, 'CONFIG', config), patch.object(credentials.bridge, 'status', return_value={'state':'idle'}):
                with patch.object(credentials.bridge, 'run'):
                    credentials.save_key('new-test-key')
                self.assertIn('OTHER=keep', secret.read_text())
                self.assertEqual(secret.stat().st_mode & 0o777, 0o600)
                saved = secret.read_bytes()
                with patch.object(credentials.bridge, 'run', side_effect=RuntimeError('failure')):
                    with self.assertRaises(RuntimeError): credentials.save_key('bad-test-key')
                self.assertEqual(secret.read_bytes(), saved)

    def test_busy_does_not_replace_key(self):
        with patch.object(credentials.bridge, 'status', return_value={'state':'streaming'}):
            with self.assertRaises(ValueError): credentials.save_key('test-only')
