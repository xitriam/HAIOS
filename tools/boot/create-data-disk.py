#!/usr/bin/env python3
"""Create a NEW, exclusive 64 MiB HAIOS data image. Never format an existing file."""
import argparse,os,struct,zlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('image',type=Path);a=p.parse_args()
record=4608;payload=bytearray(record*32)
for i,(name,directory,data)in enumerate([('/home',True,b''),('/about',False,'HAIOS 0.3.0: trwałe pliki. sync zapisuje; shutdown zapisuje i wyłącza VM.\n'.encode()),('/home/czytaj.txt',False,'Witaj w HAIOS! Użyj help, disk, mkdir, write, cat, edit i sync.\n'.encode())]):
 b=memoryview(payload)[i*record:(i+1)*record];n=name.encode();b[0]=2 if directory else 1;b[1]=len(n);struct.pack_into('<H',b,2,len(data));b[4:4+len(n)]=n;b[132:132+len(data)]=data
superblock=bytearray(512);superblock[:8]=b'HAIOSD03';struct.pack_into('<IIQII',superblock,8,1,512,131072,record,32);struct.pack_into('<I',superblock,32,zlib.crc32(superblock[:32]))
header=bytearray(512);header[:8]=b'HAIOSC03';struct.pack_into('<QII',header,8,1,zlib.crc32(payload),len(payload));struct.pack_into('<I',header,24,zlib.crc32(header[:24]))
# O_EXCL refuses symlinks and existing regular files/devices alike.
fd=os.open(a.image,os.O_CREAT|os.O_EXCL|os.O_RDWR,0o600)
try:
 with os.fdopen(fd,'r+b')as f:
  f.truncate(64*1024*1024);f.write(superblock);f.seek(8*512);f.write(header);f.write(payload);f.flush();os.fsync(f.fileno())
except BaseException:
 a.image.unlink(missing_ok=True);raise
print('Utworzono nowy dysk danych HAIOS: 64 MiB, generacja 1.')
