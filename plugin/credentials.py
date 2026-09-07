#!/usr/bin/env python3
"""Private terminal-only key replacement; never returns credentials to QML."""
import getpass
import os
import subprocess
import tempfile
from pathlib import Path
import bridge


def save_key(key):
    if not key or any(ord(c) < 33 or ord(c) > 126 for c in key):
        raise ValueError('密钥不能为空，也不能包含空格或控制字符')
    if bridge.status()['state'] not in ('idle', 'offline'):
        raise ValueError('录音或识别正在进行，请结束后再试')
    path = bridge.CONFIG.with_name('doubao.env')
    previous = path.read_bytes() if path.exists() else None
    # Preserve unrelated environment settings; replace only the credential.
    lines = path.read_text().splitlines() if path.exists() else []
    lines = [line for line in lines if not line.startswith('DOUBAO_API_KEY=')]
    quoted = key.replace('\\', '\\\\').replace('"', '\\"')
    lines.append('DOUBAO_API_KEY="' + quoted + '"')
    def write(data):
        fd, temp = tempfile.mkstemp(prefix='.doubao-', dir=path.parent)
        try:
            with os.fdopen(fd, 'wb') as f: f.write(data)
            os.replace(temp, path)
        finally:
            Path(temp).unlink(missing_ok=True)
    restarted = False
    try:
        write(('\n'.join(lines) + '\n').encode())
        if bridge.status()['state'] not in ('idle', 'offline'):
            raise ValueError('录音已开始，密钥更换已撤销')
        restarted = True
        bridge.run('systemctl', '--user', 'restart', 'smellytype.service')
        bridge.run('systemctl', '--user', 'is-active', '--quiet', 'smellytype.service')
    except Exception:
        if previous is None: path.unlink(missing_ok=True)
        else: write(previous)
        if restarted: bridge.run('systemctl', '--user', 'restart', 'smellytype.service')
        raise

if __name__ == '__main__':
    try:
        print('豆包流式语音 2.0：设置 / 更换 API Key。输入不会显示；Ctrl+C 取消。')
        save_key(getpass.getpass('API Key：').strip())
        print('已保存并重启服务。请用一次短录音验证云端鉴权。')
    except (KeyboardInterrupt, EOFError):
        print('\n已取消。')
    except Exception as e:
        print(str(e) if isinstance(e, ValueError) else '保存失败；已尝试恢复原密钥，请检查服务状态。')
    input('按回车关闭…')
