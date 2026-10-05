#!/usr/bin/env python3
import argparse,hashlib,json,subprocess,tempfile,shutil
from pathlib import Path
from vm_test import Guest
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');a=p.parse_args();out=a.out.resolve();checks=[];logs=[]
with tempfile.TemporaryDirectory(prefix='haios-durable-')as td:
 disk=Path(td)/'data.img';subprocess.run(['python3',str(Path(__file__).with_name('create-data-disk.py')),str(disk)],check=True)
 with Guest(out,disk,a.uefi)as g:
  assert b'HAIOS:DISK:READY' in g.data,repr(g.data)
  g.send('disk',rb'ready=1 readonly=0 dirty=0 generation=1')
  g.send('mkdir /home/Proba');g.send('write /home/Proba/test Zażółć gęślą jaźń',rb'OK zapisano');g.send('disk',rb'dirty=1');g.send('sync',rb'OK zsynchronizowano');g.send('disk',rb'dirty=0 generation=2')
  g.send('edit /home/Proba/test',rb'Nowa tre');g.send('.line 1 Zmieniony tekst',rb'edit> ');g.send('.cancel',rb'Anulowano');g.send('cat /home/Proba/test','Zażółć gęślą jaźń'.encode())
  g.send('edit /home/Proba/test',rb'Nowa tre');g.send('.line 1 Trwały tekst',rb'edit> ');g.send('.save',rb'OK zapisano');g.shutdown();logs.append(g.data.decode(errors='replace'))
 checks.append('VirtIO mount/write/flush/sync, Polish paths/content, edit load/change/cancel, shutdown commits')
 with Guest(out,disk,a.uefi)as g:
  g.send('cat /home/Proba/test','Trwały tekst'.encode());g.send('disk',rb'dirty=0 generation=3');g.send('rm /home/Proba/test');g.send('rm /home/Proba');g.shutdown();logs.append(g.data.decode(errors='replace'))
 with Guest(out,disk,a.uefi)as g:
  g.send('cat /home/Proba/test',rb'ERROR RAMFS');g.send('disk',rb'dirty=0 generation=4');g.shutdown()
 checks.append('Cold restart preserves exact file and subsequent deletions')
 for offset in (520*512,520*512+512+132):
  damaged=Path(td)/('damaged-'+str(offset)+'.img');shutil.copyfile(disk,damaged)
  with damaged.open('r+b')as f:f.seek(offset);byte=f.read(1);f.seek(offset);f.write(bytes([byte[0]^1]))
  with Guest(out,damaged,a.uefi)as g:g.send('disk',rb'dirty=0 generation=3');g.send('cat /home/Proba/test','Trwały tekst'.encode());g.shutdown()
 both=Path(td)/'both.img';shutil.copyfile(disk,both)
 with both.open('r+b')as f:
  for offset in (8*512,520*512):f.seek(offset);f.write(b'BROKEN!!')
 before_both=hashlib.sha256(both.read_bytes()).hexdigest()
 with Guest(out,both,a.uefi)as g:
  assert b'HAIOS:DISK:UNAVAILABLE' in g.data;g.send('write /home/broken test');g.send('sync',rb'ERROR dysk');g.send('shutdown',rb'ERROR shutdown')
 assert hashlib.sha256(both.read_bytes()).hexdigest()==before_both
 checks.append('Damaged latest header/payload fall back to generation 3; two damaged slots refuse all writes')
 with Guest(out,disk,a.uefi,readonly=True)as g:
  g.send('disk',rb'ready=1 readonly=1');g.send('write /home/ro test',rb'OK zapisano');g.send('sync',rb'ERROR dysk');g.send('shutdown',rb'ERROR shutdown');g.send('cat /home/ro',rb'test');g.send('version',rb'HAIOS 0.3.0');logs.append(g.data.decode(errors='replace'))
 checks.append('Read-only disk permits reads; failed sync/shutdown retain RAM and live console')
 for io_error in ('write','flush','read'):
  before=hashlib.sha256(disk.read_bytes()).hexdigest()
  with Guest(out,disk,a.uefi,io_error=io_error)as g:
   if io_error=='read':assert b'HAIOS:DISK:UNAVAILABLE' in g.data
   g.send('write /home/io test');g.send('sync',rb'ERROR dysk');g.send('shutdown',rb'ERROR shutdown');g.send('version',rb'HAIOS 0.3.0')
  if io_error in ('write','read'):assert hashlib.sha256(disk.read_bytes()).hexdigest()==before
  with Guest(out,disk,a.uefi)as g:g.send('cat /home/io',rb'ERROR RAMFS');g.shutdown()
 checks.append('Actual VirtIO read/write/flush EIO status is reported; failed shutdown remains live; old checkpoint recovers')
 before=hashlib.sha256(disk.read_bytes()).hexdigest()
 with disk.open('r+b')as f:f.write(b'UNKNOWN!')
 corrupt=hashlib.sha256(disk.read_bytes()).hexdigest()
 with Guest(out,disk,a.uefi)as g:
  assert b'HAIOS:DISK:UNAVAILABLE' in g.data;g.send('write /home/failed test');g.send('sync',rb'ERROR dysk');g.send('shutdown',rb'ERROR shutdown');g.send('version',rb'HAIOS 0.3.0')
 assert hashlib.sha256(disk.read_bytes()).hexdigest()==corrupt
 checks.append('Unknown superblock refuses writes, never autoformats, console remains usable')
 with Guest(out,None,a.uefi)as g:
  assert b'HAIOS:DISK:UNAVAILABLE' in g.data;g.send('write /home/absent test');g.send('sync',rb'ERROR dysk');g.send('shutdown',rb'ERROR shutdown');g.send('version',rb'HAIOS 0.3.0')
 checks.append('Absent disk fails explicitly without hanging or losing RAM changes')
report={'passed':True,'firmware':'uefi'if a.uefi else'bios','iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'checks':checks,'serial':logs};(out/('storage-'+report['firmware']+'.json')).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='serial'},indent=2))
