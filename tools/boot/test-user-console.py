#!/usr/bin/env python3
"""Actual PS/2 input, framebuffer capture and bounded RAM filesystem acceptance."""
import argparse,hashlib,json,re,select,shutil,socket,subprocess,tempfile,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');p.add_argument('--stress-seconds',type=int,default=0);a=p.parse_args();out=a.out.resolve();firmware='uefi' if a.uefi else 'bios'
checks=[];transcript=bytearray()
with tempfile.TemporaryDirectory(prefix='haios-user-console-') as td:
 w=Path(td);ss=w/'serial';qs=w/'qmp'
 cmd=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-smp','1','-m','256M','-display','none','-monitor','none','-serial',f'unix:{ss},server=on,wait=on','-qmp',f'unix:{qs},server=on,wait=off','-nic','none','-no-reboot','-no-shutdown','-boot','d','-drive',f'file={out}/haios.iso,media=cdrom,readonly=on,format=raw']
 if a.uefi:
  shutil.copyfile('/usr/share/OVMF/OVMF_VARS_4M.fd',w/'vars.fd');cmd+=['-drive','if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd','-drive',f'if=pflash,format=raw,unit=1,file={w}/vars.fd']
 vm=subprocess.Popen(cmd,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
 try:
  deadline=time.monotonic()+10
  while not ss.exists():
   assert time.monotonic()<deadline and vm.poll() is None;time.sleep(.02)
  serial=socket.socket(socket.AF_UNIX);serial.connect(str(ss));serial.setblocking(False)
  q=socket.socket(socket.AF_UNIX);q.connect(str(qs));qf=q.makefile('rwb',buffering=0);json.loads(qf.readline())
  def qmp(name,args=None):
   qf.write((json.dumps({'execute':name,'arguments':args or {}})+'\n').encode())
   while True:
    r=json.loads(qf.readline())
    if 'error'in r:raise RuntimeError(r)
    if 'return'in r:return r['return']
  qmp('qmp_capabilities')
  def wait(pattern,start=0,timeout=15):
   end=time.monotonic()+timeout
   while time.monotonic()<end:
    data=bytes(transcript[start:]);data=re.sub(rb'\x1b\[[0-9;]*[A-Za-z]',b'',data)
    found=re.search(pattern,data)
    if found:return found
    if select.select([serial],[],[],.02)[0]:
     chunk=serial.recv(65536)
     if not chunk:raise RuntimeError('Serial closed')
     transcript.extend(chunk)
   raise RuntimeError('Timeout '+repr(pattern)+' '+repr(bytes(transcript[-800:])))
  def send(text,pattern):
   start=len(transcript);serial.sendall(text.encode()+b'\n');return wait(pattern,start)
  def key(*codes):
   qmp('send-key',{'keys':[{'type':'qcode','data':code} for code in codes],'hold-time':15});time.sleep(.025)
  def type_ps2(text):
   for ch in text:key('spc' if ch==' ' else ch)
  wait(rb'haios> ',timeout=25)
  assert b'HAIOS:VIDEO:OK' in transcript and b'HAIOS:KEYBOARD:OK' in transcript,bytes(transcript)
  start=len(transcript);type_ps2('versio');key('n');key('ret');wait(rb'HAIOS 0.3.0-dev',start);checks.append('PS/2 text input enters guest command')
  start=len(transcript);type_ps2('verxion');key('home');key('right');key('right');key('right');key('delete');key('s');key('end');key('ret');wait(rb'HAIOS 0.3.0-dev',start);checks.append('PS/2 cursor, Home/End and Delete insertion')
  start=len(transcript);key('up');key('ret');wait(rb'HAIOS 0.3.0-dev',start);checks.append('PS/2 history replay')
  start=len(transcript);type_ps2('vers');key('tab');key('ret');wait(rb'HAIOS 0.3.0-dev',start);checks.append('PS/2 command completion')
  # Actual Shift produces a distinct uppercase directory; both case variants work.
  start=len(transcript);type_ps2('mkdir ');key('shift','p');type_ps2('roba');key('ret');time.sleep(.1)
  send('cd /Proba',rb'haios> ');send('pwd',rb'\n/Proba\n');send('cd /',rb'haios> ')
  send('mkdir /proba',rb'haios> ');send('cd /proba',rb'haios> ');send('pwd',rb'\n/proba\n');send('cd /',rb'haios> ');send('rm /Proba',rb'haios> ');send('rm /proba',rb'haios> ')
  checks.append('PS/2 Shift uppercase P and lowercase p create and access distinct directories')
  send('cd /home',rb'haios> ');send('mkdir dane',rb'haios> ');send('write dane/test Zażółć gęślą jaźń',rb'OK zapisano');send('cat dane/test','Zażółć gęślą jaźń'.encode());send('append dane/test !',rb'OK zapisano');send('cat ./dane/../dane/test','Zażółć gęślą jaźń!'.encode());checks.append('RAMFS directories, relative paths, UTF-8, overwrite and append')
  start=len(transcript);type_ps2('write polski ')
  for code in ['z','o','l','c']:key('alt_r',code)
  key('ret');wait(rb'OK zapisano',start);send('cat polski','żółć'.encode());checks.append('PS/2 AltGr Polish programmer characters')
  send('edit dane/test',rb'Nowa tre');send('.clear',rb'edit> ');send('Pierwszy wiersz',rb'edit> ');send('Drugi wiersz',rb'edit> ');send('.save',rb'OK zapisano');send('cat dane/test',rb'Pierwszy wiersz\nDrugi wiersz');send('edit dane/test',rb'Nowa tre');send('Zmienione',rb'edit> ');send('.cancel',rb'Anulowano');send('cat dane/test',rb'Pierwszy wiersz\nDrugi wiersz');checks.append('Multiline edit commit/cancel preserves file')
  send('write /bin/hello zmiana',rb'ERROR RAMFS');send('touch /bin/nowy',rb'ERROR RAMFS');send('rm dane',rb'ERROR RAMFS');send('cat nieistnieje',rb'ERROR RAMFS');checks.append('Read-only program files and invalid operations')
  def program(command,pattern):
   mark=len(transcript);serial.sendall(command.encode()+b'\n');wait(pattern,mark);wait(rb'PROC:EXIT pid=\d+\n',mark)
  program('echo Witaj użytkowniku','Witaj użytkowniku'.encode())
  program('uptime',rb'USER pid=\d+ \d+\n')
  for expression,result in [('2 + 3','5'),('-9 / 2','-4'),('7 * 6','42'),('2 - 8','-6'),('-9223372036854775808 + 0','-9223372036854775808')]:
   program('calc '+expression,rb'USER pid=\d+ '+result.encode()+rb'\n')
  for expression in ['1 / 0','9223372036854775807 + 1','-9223372036854775808 / -1','9223372036854775808 + 0','abc','1 * 9223372036854775807x']:
   program('calc '+expression,rb'ERROR calc:')
  program('wc dane/test',rb'USER pid=\d+ 2 4 29\n')
  program('wc brak',rb'ERROR wc:')
  program('run fsbadptr',rb'fsbadptr:ok\n')
  checks.append('Ring3 echo/uptime/wc/calc, signed limits and readfile pointer/output validation')
  send('clear',rb'haios> ');send('help sync',rb'sync utrwala');send('cat polski','żółć'.encode());time.sleep(.2);program('echo p P proba Proba g G q Q y Y','p P proba Proba g G q Q y Y'.encode());program('echo Zażółć gęślą jaźń ĄĆĘŁŃÓŚŹŻ','Zażółć gęślą jaźń ĄĆĘŁŃÓŚŹŻ'.encode());time.sleep(.2);qmp('screendump',{'filename':str(out/f'user-console-{firmware}.ppm')});checks.append('Actual framebuffer captured after guest keyboard and commands')
  send('rm dane/test',rb'haios> ');send('rm dane',rb'haios> ');send('rm polski',rb'haios> ');send('cd /',rb'haios> ')
  before=send('stat',rb'RAMFS nodes=(\d+)/32 bytes=(\d+)\n').groups();base=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))
  # Whole-file and node capacity failures must not change data or leak a slot.
  send('write /home/limit '+'x'*64,rb'OK zapisano')
  for _ in range(63):send('append /home/limit '+'x'*64,rb'OK zapisano')
  send('append /home/limit x',rb'ERROR RAMFS')
  program('wc /home/limit',rb'USER pid=\d+ 0 1 4096\n')
  send('rm /home/limit',rb'haios> ')
  count=int(before[0]);names=['/home/n'+str(i) for i in range(32-count)]
  for name in names:send('touch '+name,rb'haios> ')
  send('touch /home/full',rb'ERROR RAMFS')
  for name in names:send('rm '+name,rb'haios> ')
  assert send('stat',rb'RAMFS nodes=(\d+)/32 bytes=(\d+)\n').groups()==before
  checks.append('4096-byte content and 32-node limits, rejected changes preserve data and capacity')

  start=time.monotonic();cycles=0
  while a.stress_seconds and time.monotonic()-start<a.stress_seconds:
   send('mkdir /temp',rb'haios> ');send('write /temp/data próba',rb'OK zapisano');send('append /temp/data x',rb'OK zapisano');send('cat /temp/data','próbax'.encode());send('rm /temp/data',rb'haios> ');send('rm /temp',rb'haios> ')
   mark=len(transcript);serial.sendall(b'run hello\n');wait(rb'hello:world\n',mark);wait(rb'PROC:EXIT pid=',mark)
   assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
   assert send('stat',rb'RAMFS nodes=(\d+)/32 bytes=(\d+)\n').groups()==before
   cycles+=1
  if cycles:checks.append('Mixed RAMFS/process stress with complete memory and node balance')
  report={'passed':True,'firmware':firmware,'checks':checks,'iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'stress_seconds':time.monotonic()-start if cycles else 0,'stress_cycles':cycles,'free_frames':base,'ramfs_usage':[int(v) for v in before]}
  (out/f'user-console-{firmware}.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
 finally:
  (out/f'user-console-{firmware}.serial').write_bytes(transcript)
  if vm.poll() is None:
   try:qmp('quit')
   except Exception:vm.terminate()
  vm.wait(timeout=10)
