#!/usr/bin/env python3
"""Check actual ISO contents against build receipt and project notices."""
import argparse, hashlib, json, subprocess, tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser(); p.add_argument('--out',type=Path,required=True); a=p.parse_args(); out=a.out.resolve()
receipt=json.loads((out/'build.json').read_text())
required={'licenses/dejavu/LICENSE','LICENSE','NOTICE','THIRD_PARTY.md','licenses/limine/LICENSE','licenses/limine/3RDPARTY.md','licenses/rust/COPYRIGHT-library.html','licenses/rust/texts/Apache-2.0.txt','licenses/rust/texts/MIT.txt','licenses/rust/texts/LLVM-exception.txt'}
if not required <= receipt['notice_sha256'].keys(): raise RuntimeError('Incomplete notice inventory')
with tempfile.TemporaryDirectory(prefix='haios-notices-') as tmp:
    destination=Path(tmp)/'iso'
    subprocess.run(['xorriso','-osirrox','on','-indev',str(out/'haios.iso'),'-extract','/',str(destination)],check=True,capture_output=True)
    for name,digest in receipt['notice_sha256'].items():
        if hashlib.sha256((destination/name).read_bytes()).hexdigest()!=digest: raise RuntimeError('Notice mismatch: '+name)
    for name in ['LICENSE','NOTICE','THIRD_PARTY.md']:
        if (destination/name).read_bytes()!=(root/name).read_bytes(): raise RuntimeError('Project notice mismatch: '+name)
    if b'Apache License' not in (destination/'LICENSE').read_bytes(): raise RuntimeError('Wrong project license')
result={'passed':True,'iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'notice_files_verified':len(receipt['notice_sha256']),'notice_sha256':receipt['notice_sha256']}
(out/'test-notices.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
