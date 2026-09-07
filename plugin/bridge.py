#!/usr/bin/env python3
"""Small JSON bridge. No audio, transcript, API key, or billing access."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import tomllib

HOME_DIR = Path.home()
RUNTIME = Path(os.environ.get('XDG_RUNTIME_DIR') or f'/run/user/{os.geteuid()}') / 'smellytype'
CONFIG = Path(os.environ.get('XDG_CONFIG_HOME', str(HOME_DIR / '.config'))) / 'smellytype/config.toml'
USAGE = Path(os.environ.get('XDG_STATE_HOME', str(HOME_DIR / '.local/state'))) / 'smellytype/dictation-usage.json'
BINARY = HOME_DIR / '.local/lib/smellytype/bin/smellytype'

def load_json(path):
    try: return json.loads(path.read_text())
    except FileNotFoundError: return {}

def daemon_alive():
    try:
        pid = int((RUNTIME / 'pid').read_text().strip())
        if pid <= 0: return False
        os.kill(pid, 0)
        return True
    except (FileNotFoundError, ProcessLookupError, ValueError): return False

def status():
    try: state = (RUNTIME / 'state').read_text().strip()
    except FileNotFoundError: state = 'offline'
    if not daemon_alive(): state = 'offline'
    try:
        with CONFIG.open('rb') as f: cfg = tomllib.load(f)
    except FileNotFoundError: cfg = {}
    usage = load_json(USAGE)
    active = load_json(RUNTIME / 'dictation-active.json')
    recording = False
    if active and state == 'streaming':
        try: os.kill(int(active['pid']), 0); recording = True
        except (ProcessLookupError, ValueError, KeyError): pass
    elapsed = max(0, int(time.time()*1000) - int(active.get('started_ms', 0))) if recording else 0
    return {'ok': True, 'state': state, 'recording': recording,
            'final_timeout': cfg.get('doubao', {}).get('final_timeout_secs', 60),
            'key_configured': CONFIG.with_name('doubao.env').is_file(),
            'limit': cfg.get('audio', {}).get('max_duration_secs', 60),
            'total_ms': usage.get('total_ms', 0), 'sessions': usage.get('sessions', 0),
            'since': usage.get('since', ''), 'elapsed_ms': elapsed,
            'installed': BINARY.is_file(), 'configured': CONFIG.is_file()}

def run(*args):
    subprocess.run(args, check=True, capture_output=True, text=True, timeout=15)

def main(argv):
    action = argv[0] if argv else 'status'
    if action == 'status': return status()
    current = status()
    if not current['installed'] or not current['configured']:
        raise ValueError('请先安装并配置 SmellyType：github.com/beijingrong/smellytype')
    if action in ('limit', 'timeout'):
        if current['state'] != 'idle': raise ValueError('请先结束录音并等待识别完成')
        seconds = int(argv[1])
        if action == 'limit' and not 5 <= seconds <= 900: raise ValueError('时长必须在 5–900 秒之间')
        if action == 'timeout' and not 1 <= seconds <= 300: raise ValueError('等待时间必须在 1–300 秒之间')
        field = 'audio.max_duration_secs' if action == 'limit' else 'doubao.final_timeout_secs'
        before = CONFIG.read_bytes()
        import shutil
        backup = CONFIG.with_name('config.toml.before-plugin-' + action)
        shutil.copy2(CONFIG, backup)
        restarted = False
        try:
            run(str(BINARY), '--config', str(CONFIG), 'config', 'set', field, str(seconds))
            if status()['state'] != 'idle':
                raise ValueError('录音已开始，本次设置未应用')
            restarted = True
            run('systemctl', '--user', 'restart', 'smellytype.service')
            run('systemctl', '--user', 'is-active', '--quiet', 'smellytype.service')
        except Exception:
            CONFIG.write_bytes(before)
            if restarted: run('systemctl', '--user', 'restart', 'smellytype.service')
            raise
        return {'ok': True, 'message': '设置已保存并应用'}
    if action == 'credentials':
        if current['state'] not in ('idle', 'offline'): raise ValueError('请先结束录音并等待识别完成')
        subprocess.Popen(['xdg-terminal-exec', '--title=豆包语音密钥', '-e', 'python3', str(Path(__file__).with_name('credentials.py'))], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        return {'ok': True, 'message': '请在终端中填写密钥'}
    if action == 'restart':
        if current['state'] not in ('idle', 'offline'): raise ValueError('请先结束录音并等待识别完成')
        run('systemctl', '--user', 'restart', 'smellytype.service')
        run('systemctl', '--user', 'is-active', '--quiet', 'smellytype.service')
        return {'ok': True, 'message': '语音服务已重启'}
    if action == 'toggle':
        if current['state'] == 'idle': run(str(BINARY), 'record', 'start')
        elif current['recording']: run(str(BINARY), 'record', 'stop')
        else: raise ValueError('正在等待识别结果，请稍候')
    elif action == 'cancel': run(str(BINARY), 'record', 'cancel')
    else: raise ValueError('Unknown action')
    return {'ok': True}

if __name__ == '__main__':
    try: print(json.dumps(main(sys.argv[1:]), ensure_ascii=False))
    except Exception as e:
        # Subprocess stderr can contain transcription; never forward it.
        message = str(e) if isinstance(e, ValueError) else '操作失败，请检查 SmellyType 服务或配置'
        print(json.dumps({'ok': False, 'error': message}, ensure_ascii=False))
        sys.exit(1)
