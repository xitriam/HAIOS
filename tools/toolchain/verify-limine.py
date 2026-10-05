#!/usr/bin/env python3
"""Verify a pinned release, build its host tool as the user; never touch disks."""
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

root = Path.home() / '.local/share/haios/toolchain'
archive = root / 'limine-12.9.2-binary.tar.xz'
fingerprint = '05D29860D0A0668AAEFB9D691F3C021BECA23821'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == '07d01fd0139f76afd510d9a8d7805af730ef7f6a55eb5cf23500f132283829de'
with tempfile.TemporaryDirectory(prefix='haios-gpg-') as home:
    def gpg(*args):
        return subprocess.run(['gpg', '--homedir', home, '--batch', *args], capture_output=True, text=True, check=True)
    gpg('--import', str(root/'limine-signing-key.asc'))
    keys = gpg('--with-colons', '--fingerprint', '--list-keys').stdout
    assert any(line.startswith('fpr:') and line.split(':')[9] == fingerprint for line in keys.splitlines())
    status = gpg('--status-fd', '1', '--verify', str(archive)+'.sig', str(archive)).stdout
    valid = [line for line in status.splitlines() if line.startswith('[GNUPG:] VALIDSIG ')]
    assert len(valid) == 1 and fingerprint in valid[0].split(), 'Unexpected signer'
with tarfile.open(archive) as tar:
    tar.extractall(root/'limine-12.9.2', filter='data')
build = root/'limine-12.9.2/limine-binary'
subprocess.run(['make', '-C', str(build)], check=True, capture_output=True)
version = subprocess.run([str(build/'limine'), '--version'], capture_output=True, text=True, check=True)
result = {'host': subprocess.check_output(['hostname'], text=True).strip(), 'limine': '12.9.2',
          'archive_sha256_verified': True, 'signature_verified': True,
          'signing_primary_fingerprint': fingerprint, 'gpg_validsig': valid[0],
          'make_exit': 0, 'version_exit': version.returncode,
          'version_output': version.stdout, 'host_tool_sha256': hashlib.sha256((build/'limine').read_bytes()).hexdigest(),
          'disk_install_invoked': False, 'kernel_boot_tested': False}
(root/'limine-verification.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result, indent=2))
