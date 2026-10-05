"""Refuse device files, symlinks, incorrect size and concurrent HAIOS launchers."""
import fcntl,os,stat
from pathlib import Path
def locked_disk(path):
 path=Path(path).absolute()
 if ','in str(path) or any(ord(c)<32 for c in str(path)):raise SystemExit('Unsupported disk filename')
 try:fd=os.open(path,os.O_RDWR|os.O_NOFOLLOW)
 except OSError:raise SystemExit('Create a dedicated data image first with create-data-disk.py')
 f=os.fdopen(fd,'r+b');info=os.fstat(fd)
 if not stat.S_ISREG(info.st_mode)or info.st_size!=64*1024*1024:f.close();raise SystemExit('Expected a regular 64 MiB disk image')
 try:fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
 except BlockingIOError:f.close();raise SystemExit('This HAIOS data image is already in use')
 return f,path
def device_args(path):return ['-drive',f'file={path},id=haiosdata,if=none,format=raw,cache=writeback','-device','virtio-blk-pci,drive=haiosdata,disable-modern=on,disable-legacy=off']
