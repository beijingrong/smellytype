#!/usr/bin/env python3
"""Export the canonical plugin to its standalone distribution checkout."""
import argparse
from pathlib import Path
import shutil
import subprocess

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('destination', type=Path)
args=parser.parse_args()
root=Path(__file__).resolve().parents[1]
source=root/'plugin'
if subprocess.check_output(['git','-C',str(root),'status','--porcelain','--','plugin']):
    parser.error('Commit the canonical plugin changes before exporting')
revision=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
args.destination.mkdir(parents=True,exist_ok=True)
for p in source.rglob('*'):
    if not p.is_file() or '__pycache__' in p.parts or p.suffix=='.pyc': continue
    target=args.destination/p.relative_to(source)
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(p,target)
(args.destination/'SOURCE.md').write_text('# Source revision\n\nExported from https://github.com/beijingrong/smellytype/tree/'+revision+'/plugin\n\nEdit the canonical plugin directory and rerun scripts/export_plugin.py for releases.\n')
print('Exported plugin from',revision)
