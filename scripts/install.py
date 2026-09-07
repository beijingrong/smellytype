#!/usr/bin/env python3
"""Stage SmellyType; optionally activate, migrate from the former fork, or roll back."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time

HOME = Path.home()
SOURCE = Path(__file__).resolve().parents[1]
PREFIX = HOME/'.local/lib/smellytype'
CONFIG = HOME/'.config/smellytype/config.toml'
SERVICE = HOME/'.config/systemd/user/smellytype.service'
RUNTIME = Path(os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}'))
SNAPSHOT = PREFIX/'migration.json'

def run(*args):
    return subprocess.run(args, check=True, capture_output=True, text=True, timeout=30)

def idle(name):
    state = RUNTIME/name/'state'
    pid = RUNTIME/name/'pid'
    try:
        value = int(pid.read_text())
        if value <= 0: return
        os.kill(value, 0)
    except (FileNotFoundError, ProcessLookupError, ValueError): return
    if state.exists() and state.read_text().strip() not in ('idle','stopped',''):
        raise RuntimeError(f'{name} is recording or recognizing; finish first')

def quoted(value):
    return '"'+str(value).replace('\\','\\\\').replace('"','\\"').replace('%','%%')+'"'

def restore(data):
    previous=data.get('previous_app','voxtype')
    idle('smellytype')
    run('systemctl','--user','disable','--now','smellytype.service')
    for filename, backup in data['files'].items():
        p=Path(filename)
        if backup is None:p.unlink(missing_ok=True)
        else:
            p.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(backup,p)
    run('systemctl','--user','daemon-reload')
    if data['voxtype_enabled']:run('systemctl','--user','enable',previous+'.service')
    if data['voxtype_active']:run('systemctl','--user','start',previous+'.service')
    run('hyprctl','reload')
    run('omarchy-shell','shell','rescanPlugins')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifacts',type=Path,default=SOURCE/'target/release')
    parser.add_argument('--activate',action='store_true')
    group=parser.add_mutually_exclusive_group()
    group.add_argument('--migrate-voxtype',action='store_true')
    group.add_argument('--migrate-yuntype',action='store_true')
    parser.add_argument('--rollback',action='store_true')
    args=parser.parse_args()
    if args.rollback:
        restore(json.loads(SNAPSHOT.read_text()));SNAPSHOT.unlink();print('Previous setup restored');return
    previous='yuntype' if args.migrate_yuntype else 'voxtype'
    migrating=args.migrate_voxtype or args.migrate_yuntype
    if migrating and not args.activate:parser.error('Migration requires --activate')
    for variable, expected in [('XDG_CONFIG_HOME', HOME/'.config'), ('XDG_DATA_HOME', HOME/'.local/share'), ('XDG_STATE_HOME', HOME/'.local/state')]:
        if os.environ.get(variable) and Path(os.environ[variable]) != expected:
            parser.error('This installer currently requires the standard Omarchy XDG directories')
    idle('smellytype')
    if args.activate and SNAPSHOT.exists():parser.error('Already activated; stage updates without --activate')
    names=['smellytype','smellytype-osd','smellytype-osd-quickshell','smellytype-audio-bridge']
    for name in names:
        if not (args.artifacts/name).is_file():parser.error(f'Missing built binary: {name}')
    # Stage never replaces a live executable or changes a service.
    if subprocess.run(['systemctl','--user','is-active','--quiet','smellytype.service']).returncode==0:
        parser.error('Stop the idle SmellyType service before staging an update')
    (PREFIX/'bin').mkdir(parents=True,exist_ok=True)
    for name in names:
        target=PREFIX/'bin'/name
        shutil.copy2(args.artifacts/name,target.with_suffix('.new'))
        if shutil.which('strip'):run('strip','--strip-debug',str(target.with_suffix('.new')))
        target.with_suffix('.new').replace(target)
    shutil.copytree(SOURCE/'quickshell',PREFIX/'quickshell',dirs_exist_ok=True)
    shutil.copytree(SOURCE/'plugin',PREFIX/'plugin',dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__'))
    if not args.activate:
        print('SmellyType staged; use --activate when ready');return
    idle(previous)
    secret=CONFIG.with_name('doubao.env')
    oldconfig=HOME/'.config'/previous/'config.toml'
    oldsecret=oldconfig.with_name('doubao.env')
    if not secret.exists() and not (migrating and oldsecret.exists()):
        parser.error('Create ~/.config/smellytype/doubao.env privately with DOUBAO_API_KEY first, or migrate an existing key')
    bindings=HOME/'.config/hypr/bindings.lua'
    desktop=HOME/'.local/share/applications'/(previous+'-configure.desktop')
    override=HOME/'.config/systemd/user'/(previous+'.service.d')/'90-doubao.conf'
    usage=HOME/'.local/state/smellytype/dictation-usage.json'
    paths=[CONFIG,secret,SERVICE,bindings,HOME/'.config/omarchy/shell.json',HOME/'.local/share/applications/smellytype-configure.desktop',usage]
    if migrating:paths += [desktop,override]
    backup=PREFIX/'backups'/time.strftime('%Y%m%d-%H%M%S');backup.mkdir(parents=True,mode=0o700)
    data={'files':{},'previous_app':previous,'voxtype_active':subprocess.run(['systemctl','--user','is-active','--quiet',previous+'.service']).returncode==0,'voxtype_enabled':subprocess.run(['systemctl','--user','is-enabled','--quiet',previous+'.service'],stdout=subprocess.DEVNULL).returncode==0}
    for index,p in enumerate(paths):
        dest=backup/str(index)
        if p.exists():shutil.copy2(p,dest);dest.chmod(0o600);data['files'][str(p)]=str(dest)
        else:data['files'][str(p)]=None
    SNAPSHOT.write_text(json.dumps(data));SNAPSHOT.chmod(0o600)
    try:
        CONFIG.parent.mkdir(parents=True,exist_ok=True)
        if not CONFIG.exists():
            content=oldconfig.read_text() if migrating and oldconfig.exists() else 'state_file="auto"\n[hotkey]\nenabled=false\n[audio]\nsample_rate=16000\nmax_duration_secs=180\n[output]\nmode="type"\n[osd]\nenabled=true\nfrontend="quickshell"\n'
            content=re.sub(r'^engine\s*=.*\n','',content,flags=re.M)
            CONFIG.write_text('engine="doubao"\n'+content)
        if not secret.exists():shutil.copy2(oldsecret,secret)
        secret.chmod(0o600)
        if migrating and not usage.exists():
            oldusage=HOME/'.local/state'/previous/'dictation-usage.json'
            if oldusage.exists():usage.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(oldusage,usage)
        SERVICE.parent.mkdir(parents=True,exist_ok=True)
        SERVICE.write_text('[Unit]\nDescription=SmellyType cloud dictation\nAfter=graphical-session.target\nPartOf=graphical-session.target\n[Service]\nExecStart='+quoted(PREFIX/'bin/smellytype')+' daemon\nEnvironmentFile='+str(secret).replace('%','%%')+'\nEnvironment='+quoted('SMELLYTYPE_OSD_FRONTEND=quickshell')+'\nEnvironment='+quoted('SMELLYTYPE_OSD_QML_PATH='+str(PREFIX/'quickshell'))+'\nEnvironment='+quoted('PATH='+str(PREFIX/'bin')+':'+str(HOME/'.local/bin')+':/usr/local/bin:/usr/bin')+'\nRestart=on-failure\nRestartSec=2\n[Install]\nWantedBy=default.target\n')
        run('systemd-analyze','--user','verify',str(SERVICE))
        binary=PREFIX/'bin/smellytype'
        userbin=HOME/'.local/bin';userbin.mkdir(parents=True,exist_ok=True)
        # Do not overwrite a unrelated user command.
        link=userbin/'smellytype'
        if link.exists() and link.resolve()!=binary:raise RuntimeError('An unrelated smellytype command already exists')
        if not link.exists():link.symlink_to(binary)
        command=str(binary)
        block='\n-- BEGIN SMELLYTYPE\nhl.unbind("F9")\nhl.unbind("SUPER + CTRL + X")\n'
        for key,label,action,release in [('F9','Start cloud dictation','start',False),('F9','Stop cloud dictation','stop',True),('SUPER + CTRL + X','Toggle cloud dictation','toggle',False)]:
            block+='o.bind('+json.dumps(key)+','+json.dumps(label)+','+json.dumps(command+' record '+action)+(', { release = true }' if release else '')+')\n'
        block+='-- END SMELLYTYPE\n'
        bindings.parent.mkdir(parents=True,exist_ok=True)
        text=bindings.read_text() if bindings.exists() else ''
        if args.migrate_yuntype:
            text=re.sub(r'\n-- BEGIN YUNTYPE\n.*?-- END YUNTYPE\n','\n',text,flags=re.S)
        bindings.write_text(text+block)
        run('hyprctl','reload')
        errors=run('hyprctl','configerrors').stdout.strip()
        if errors and errors!='ok':raise RuntimeError('Hyprland rejected bindings')
        if data['voxtype_active'] or data['voxtype_enabled']:run('systemctl','--user','disable','--now',previous+'.service')
        if migrating:
            if desktop.exists() and ('beijingrong.'+previous+'/configure.py') in desktop.read_text():desktop.unlink()
            override.unlink(missing_ok=True)
            run('omarchy','plugin','disable','beijingrong.'+previous)
        run('systemctl','--user','daemon-reload')
        run('systemctl','--user','enable','--now','smellytype.service')
        run('python3',str(PREFIX/'plugin/install.py'))
        for _ in range(50):
            state=RUNTIME/'smellytype/state'
            if state.exists() and state.read_text().strip()=='idle':break
            time.sleep(.1)
        else:raise RuntimeError('SmellyType did not become ready')
        print('SmellyType activated. Prior files and service states saved for --rollback.')
    except Exception:
        restore(data)
        SNAPSHOT.unlink(missing_ok=True)
        raise

if __name__=='__main__':main()
