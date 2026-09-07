#!/usr/bin/env python3
"""Open the same settings panel from a desktop launcher or the Omarchy bar."""
import subprocess

try:
    subprocess.run(['omarchy-shell', 'beijingrong.smellytype', 'open'], check=True, capture_output=True, timeout=8)
except (subprocess.SubprocessError, OSError):
    subprocess.run(['notify-send', 'SmellyType Configuration', '无法打开豆包设置。请确认 Omarchy Shell 正在运行，并启用了 beijingrong.smellytype 插件。'], check=False)
    raise SystemExit(1)
