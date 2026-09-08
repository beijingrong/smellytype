import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1]/'install.py'

class MigrationTests(unittest.TestCase):
    def test_voxtype_migration_and_rollback(self):
        self.migrate_and_restore('voxtype')

    def test_yuntype_rename_and_rollback(self):
        self.migrate_and_restore('yuntype')

    def migrate_and_restore(self, previous):
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {'XDG_CONFIG_HOME':'', 'XDG_DATA_HOME':'', 'XDG_STATE_HOME':''}):
            home=Path(d)
            with patch.object(Path,'home',return_value=home):
                spec=importlib.util.spec_from_file_location('installer',SCRIPT)
                m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
            m.RUNTIME=home/'run'
            cfg=home/'.config'/previous/'config.toml';cfg.parent.mkdir(parents=True)
            cfg.write_text('state_file="auto"\n[audio]\nmax_duration_secs=300\n')
            key=cfg.with_name('doubao.env');key.write_text('DOUBAO_API_KEY="test-only"\n')
            bindings=home/'.config/hypr/bindings.lua';bindings.parent.mkdir(parents=True);bindings.write_text('-- original\n'+('\n-- BEGIN YUNTYPE\nold bindings\n-- END YUNTYPE\n' if previous=='yuntype' else '')); original_bindings=bindings.read_text()
            override=home/'.config/systemd/user'/(previous+'.service.d')/'90-doubao.conf';override.parent.mkdir(parents=True);override.write_text('old override')
            desktop=home/'.local/share/applications'/(previous+'-configure.desktop');desktop.parent.mkdir(parents=True);desktop.write_text('Exec=/test/beijingrong.'+previous+'/configure.py')
            artifacts=home/'artifacts';artifacts.mkdir()
            for name in ('smellytype','smellytype-osd','smellytype-osd-quickshell','smellytype-audio-bridge'):(artifacts/name).write_text('fake')
            calls=[]
            def run(args, **kwargs):
                args=list(args)
                calls.append(args)
                if args[:4]==['systemctl','--user','is-active','--quiet']:
                    return subprocess.CompletedProcess(args,0 if args[-1]==previous+'.service' else 1,stdout='')
                if args[:4]==['systemctl','--user','enable','--now']:
                    (m.RUNTIME/'smellytype').mkdir(parents=True);(m.RUNTIME/'smellytype/state').write_text('idle')
                return subprocess.CompletedProcess(args,0,stdout='')
            with patch.object(subprocess,'run',side_effect=run), patch.object(sys,'argv',[str(SCRIPT),'--artifacts',str(artifacts),'--activate','--migrate-'+previous]):m.main()
            self.assertIn('EnvironmentFile='+str(m.CONFIG.with_name('doubao.env')),m.SERVICE.read_text())
            self.assertNotIn('EnvironmentFile="',m.SERVICE.read_text())
            self.assertIn('WantedBy=graphical-session.target',m.SERVICE.read_text())
            self.assertNotIn('WantedBy=default.target',m.SERVICE.read_text())
            self.assertIn('engine="doubao"',m.CONFIG.read_text())
            self.assertIn('max_duration_secs=300',m.CONFIG.read_text())
            self.assertEqual(m.CONFIG.with_name('doubao.env').stat().st_mode & 0o777,0o600)
            self.assertIn('SMELLYTYPE',bindings.read_text())
            self.assertNotIn('BEGIN YUNTYPE',bindings.read_text())
            self.assertFalse(override.exists());self.assertFalse(desktop.exists())
            with patch.object(subprocess,'run',side_effect=run), patch.object(sys,'argv',[str(SCRIPT),'--rollback']):m.main()
            self.assertEqual(bindings.read_text(),original_bindings)
            self.assertEqual(override.read_text(),'old override')
            self.assertTrue(desktop.exists());self.assertFalse(m.CONFIG.exists())
            self.assertTrue(key.exists())
