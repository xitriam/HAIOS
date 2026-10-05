#!/usr/bin/env python3
"""Interactive HAIOS in QEMU. Ctrl-A then X exits; no host disk or NIC."""
import argparse,shutil,subprocess,tempfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');a=p.parse_args();out=a.out.resolve()
if not (out/'haios.iso').is_file():raise SystemExit('Build haios.iso first')
with tempfile.TemporaryDirectory(prefix='haios-interactive-')as tmp:
    command=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-smp','1','-m','256M','-display','none','-serial','mon:stdio','-nic','none','-no-reboot','-no-shutdown','-boot','d','-drive','file='+str(out/'haios.iso')+',media=cdrom,readonly=on,format=raw']
    if a.uefi:
        vars_file=Path(tmp)/'vars.fd';shutil.copyfile('/usr/share/OVMF/OVMF_VARS_4M.fd',vars_file)
        command+=['-drive','if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd','-drive','if=pflash,format=raw,unit=1,file='+str(vars_file)]
    print('HAIOS console. Type help. Exit QEMU: Ctrl-A, release, then X.',flush=True)
    raise SystemExit(subprocess.call(command))
