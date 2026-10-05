#!/usr/bin/env python3
"""Pinned HAIOS tools for the current user; no sudo and no kernel build."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import urllib.request

ROOT = Path.home() / '.local/share/haios/toolchain'
ROOT.mkdir(parents=True, exist_ok=True)
RUSTUP_SHA = 'dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71'
LIMINE_SHA = '07d01fd0139f76afd510d9a8d7805af730ef7f6a55eb5cf23500f132283829de'
PROTOCOL = '3a0526b700e356f0eac1b71a77697b3fd1c707a3'
KEY = '05D29860D0A0668AAEFB9D691F3C021BECA23821'
receipts = []

def fetch(url, name, expected=None):
    data = urllib.request.urlopen(url, timeout=120).read()
    digest = hashlib.sha256(data).hexdigest()
    if expected and digest != expected:
        raise RuntimeError(f'Checksum mismatch: {name}')
    p = ROOT / name
    p.write_bytes(data)
    receipts.append({'file': name, 'url': url, 'sha256': digest,
                     'expected_sha256': expected, 'bytes': len(data)})
    return p

installer = fetch('https://static.rust-lang.org/rustup/archive/1.29.1/x86_64-unknown-linux-gnu/rustup-init', 'rustup-init-1.29.1', RUSTUP_SHA)
installer.chmod(0o700)
subprocess.run([str(installer), '-y', '--no-modify-path', '--profile', 'minimal',
                '--default-toolchain', '1.99.0', '--target', 'x86_64-unknown-none'], check=True)
base = 'https://github.com/Limine-Bootloader/Limine/releases/download/v12.9.2/'
fetch(base + 'limine-binary.tar.xz', 'limine-12.9.2-binary.tar.xz', LIMINE_SHA)
fetch(base + 'limine-binary.tar.xz.sig', 'limine-12.9.2-binary.tar.xz.sig', 'c6a3faa572fffd90ff4de7c48b9b43bdab198d3fa42e8843db7c2f74a5acde4c')
fetch('https://keyserver.ubuntu.com/pks/lookup?op=get&search=0x' + KEY, 'limine-signing-key.asc')
for file, out in [('PROTOCOL.md', 'limine-PROTOCOL.md'), ('include/limine.h', 'limine.h')]:
    expected = {'limine-PROTOCOL.md': '9e69fdd01af4c31f419f1e06a74c7371e4cd8f14021d9c49dd3a4fa9f2359cac', 'limine.h': '0aadf2633c85d8cb145344c968add158aeb1deb68af567d55ba8431c492b6d4c'}[out]
    fetch('https://raw.githubusercontent.com/Limine-Bootloader/limine-protocol/' + PROTOCOL + '/' + file, out, expected)
fetch('https://raw.githubusercontent.com/Limine-Bootloader/Limine/v12.9.2/3RDPARTY.md', 'limine-3RDPARTY.md')
fetch('https://raw.githubusercontent.com/Limine-Bootloader/Limine/v12.9.2/COPYING', 'limine-COPYING')
result = {'rust': '1.99.0', 'rustup': '1.29.1', 'target': 'x86_64-unknown-none',
          'limine': '12.9.2', 'protocol_commit': PROTOCOL, 'signing_primary_fingerprint': KEY,
          'limine_signature_verified': False, 'downloads': receipts,
          'rustc_version': subprocess.check_output([str(Path.home()/'.cargo/bin/rustc'), '+1.99.0', '-Vv'], text=True)}
(ROOT / 'download-receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
