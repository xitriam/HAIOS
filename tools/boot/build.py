#!/usr/bin/env python3
"""Build in a separate output directory; no host disk installation."""
import argparse, hashlib, json, shutil, subprocess
from pathlib import Path
root = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser()
p.add_argument('--out', type=Path, required=True)
p.add_argument('--missing-hhdm', action='store_true')
a = p.parse_args(); out = a.out.resolve(); out.mkdir(parents=True, exist_ok=True)
if out == root or root in out.parents: raise SystemExit('Output must be outside source tree')
limine = Path.home()/'.local/share/haios/toolchain/limine-12.9.2/limine-binary'
# The signed release was verified during provisioning. Recheck installed notices/binaries
# by comparing each packaged input to the pinned release archive, not a mutable checksum file.
import tarfile
archive = limine.parents[1]/'limine-12.9.2-binary.tar.xz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == '07d01fd0139f76afd510d9a8d7805af730ef7f6a55eb5cf23500f132283829de'
with tarfile.open(archive) as tar:
    for name in ['limine-bios.sys','limine-bios-cd.bin','limine-uefi-cd.bin','BOOTX64.EFI','LICENSE','3RDPARTY.md']:
        assert (limine/name).read_bytes() == tar.extractfile('limine-binary/'+name).read()
command = [str(Path.home()/'.cargo/bin/rustc'), '+1.99.0', '--edition=2024', '--target', 'x86_64-unknown-none', '-C', 'panic=abort', '-C', 'opt-level=2', '-C', 'relocation-model=static', '-C', 'link-arg=-T'+str(root/'boot/linker.ld'), '-C', 'link-arg=--build-id=none', str(root/'kernel/main.rs'), '-o', str(out/'haios.elf')]
if a.missing_hhdm: command += ['--cfg', 'haios_test_missing_hhdm']
command += ['--remap-path-prefix',str(root)+'=HAIOS']
subprocess.run(command, check=True)
stage = out/'iso-root'; stage.mkdir(exist_ok=True)
(stage/'boot').mkdir(exist_ok=True); (stage/'EFI/BOOT').mkdir(parents=True, exist_ok=True)
shutil.copy2(out/'haios.elf', stage/'boot/haios.elf')
shutil.copy2(root/'boot/limine.conf', stage/'limine.conf')
for name in ['limine-bios.sys','limine-bios-cd.bin','limine-uefi-cd.bin']:
    shutil.copy2(limine/name, stage/name)
shutil.copy2(limine/'BOOTX64.EFI', stage/'EFI/BOOT/BOOTX64.EFI')

# Preserve each component's notices without relabeling upstream code as HAIOS.
for name in ['LICENSE', 'NOTICE', 'THIRD_PARTY.md']:
    shutil.copy2(root/name, stage/name)
(stage/'licenses/limine').mkdir(parents=True, exist_ok=True)
for name in ['LICENSE', '3RDPARTY.md']:
    shutil.copy2(limine/name, stage/'licenses/limine'/name)
sysroot = Path(subprocess.check_output([str(Path.home()/'.cargo/bin/rustc'), '+1.99.0', '--print', 'sysroot'], text=True).strip())
rust_notices = sysroot/'share/doc/rust'
(stage/'licenses/rust').mkdir(parents=True, exist_ok=True)
shutil.copy2(rust_notices/'COPYRIGHT-library.html', stage/'licenses/rust/COPYRIGHT-library.html')
shutil.copytree(rust_notices/'licenses', stage/'licenses/rust/texts', dirs_exist_ok=True)
notice_hashes = {str(f.relative_to(stage)): hashlib.sha256(f.read_bytes()).hexdigest()
                 for f in sorted((stage/'licenses').rglob('*')) if f.is_file()}
for name in ['LICENSE', 'NOTICE', 'THIRD_PARTY.md']:
    notice_hashes[name] = hashlib.sha256((stage/name).read_bytes()).hexdigest()

iso = out/'haios.iso'
subprocess.run(['xorriso','-as','mkisofs','-R','-r','-J','-b','limine-bios-cd.bin','-no-emul-boot','-boot-load-size','4','-boot-info-table','--efi-boot','limine-uefi-cd.bin','-efi-boot-part','--efi-boot-image','--protective-msdos-label',str(stage),'-o',str(iso)], check=True, capture_output=True)
# CD BIOS path works without installing a bootloader into any host disk.
report={'notice_sha256':notice_hashes,'rust_command':command,'missing_hhdm':a.missing_hhdm,'artifacts':{n:hashlib.sha256((out/n).read_bytes()).hexdigest() for n in ['haios.elf','haios.iso']}}
(out/'build.json').write_text(json.dumps(report,indent=2)+'\n'); print(json.dumps(report,indent=2))
