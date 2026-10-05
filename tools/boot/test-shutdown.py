#!/usr/bin/env python3
"""Guest PS/2 shutdown, QMP cause and actual exit; unsupported ACPI stays usable."""
import argparse,hashlib,json,re,select,shutil,socket,subprocess,tempfile,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');p.add_argument('--unsupported-machine',action='store_true');a=p.parse_args();out=a.out.resolve()
with tempfile.TemporaryDirectory(prefix='haios-power-') as td:
 w=Path(td);ss=w/'serial';qs=w/'qmp';transcript=bytearray();events=[];checks=[]
 cmd=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-smp','1','-m','256M','-display','none','-monitor','none','-serial',f'unix:{ss},server=on,wait=on','-qmp',f'unix:{qs},server=on,wait=off','-nic','none','-no-reboot','-boot','d','-drive',f'file={out}/haios.iso,media=cdrom,readonly=on,format=raw']
 if a.unsupported_machine:cmd[2]='pc,accel=kvm'
 if a.uefi:
  shutil.copyfile('/usr/share/OVMF/OVMF_VARS_4M.fd',w/'vars.fd');cmd+=['-drive','if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd','-drive',f'if=pflash,format=raw,unit=1,file={w}/vars.fd']
 vm=subprocess.Popen(cmd,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE);serial=None;q=None;f=None
 try:
  deadline=time.monotonic()+10
  while not ss.exists():
   assert vm.poll() is None and time.monotonic()<deadline;time.sleep(.02)
  serial=socket.socket(socket.AF_UNIX);serial.connect(str(ss));serial.setblocking(False)
  q=socket.socket(socket.AF_UNIX);q.settimeout(15);q.connect(str(qs));f=q.makefile('rwb',buffering=0);json.loads(f.readline())
  def qmp(name,args=None):
   f.write((json.dumps({'execute':name,'arguments':args or {}})+'\n').encode())
   while True:
    r=json.loads(f.readline())
    if 'event'in r:events.append(r)
    if 'error'in r:raise RuntimeError(r)
    if 'return'in r:return r['return']
  qmp('qmp_capabilities')
  def wait(pattern,start=0):
   deadline=time.monotonic()+25
   while time.monotonic()<deadline:
    found=re.search(pattern,re.sub(rb'\x1b\[[0-9;]*[A-Za-z]',b'',bytes(transcript[start:])))
    if found:return found
    if select.select([serial],[],[],.02)[0]:
     data=serial.recv(65536)
     if not data:raise RuntimeError('Serial closed before '+repr(pattern))
     transcript.extend(data)
   raise RuntimeError('Timeout '+repr(pattern))
  def send(s,pattern):
   mark=len(transcript);serial.sendall(s.encode()+b'\n');return wait(pattern,mark)
  def key(code):qmp('send-key',{'keys':[{'type':'qcode','data':code}],'hold-time':15});time.sleep(.03)
  wait(rb'haios> ')
  send('shutdown extra',rb'ERROR command');send('version',rb'HAIOS 0.2.0-dev');checks.append('Invalid arguments leave console usable')
  send('edit /home/power-test',rb'Nowa tre');send('shutdown',rb'edit> ');send('.cancel',rb'Anulowano');checks.append('Editor treats shutdown as text')
  send('run spin',rb'PROC:START pid=\d+ name=spin')
  for code in ['s','h','u','t','tab']:key(code)
  time.sleep(.1);qmp('screendump',{'filename':str(out/'shutdown-input.ppm')});key('ret')
  if a.unsupported_machine:
   wait(rb'ERROR shutdown:');send('version',rb'HAIOS 0.2.0-dev');assert vm.poll() is None and not any(e['event']=='SHUTDOWN' for e in events)
   checks.append('Unsupported chipset rejects power-off and preserves console');qmp('quit');vm.wait(timeout=10)
  else:
   # No host quit/powerdown: guest command must cause the event and process exit.
   deadline=time.monotonic()+10
   while time.monotonic()<deadline:
    line=f.readline()
    if not line:break
    event=json.loads(line)
    if 'event'in event:events.append(event)
   assert any(e['event']=='SHUTDOWN' and e.get('data',{}).get('guest') is True and e['data'].get('reason')=='guest-shutdown' for e in events),events
   assert vm.wait(timeout=10)==0
   while select.select([serial],[],[],.05)[0]:
    data=serial.recv(65536)
    if not data:break
    transcript.extend(data)
   assert b'HAIOS:SHUTDOWN' in transcript
   checks.append('PS/2 Tab completes shutdown; guest shuts down with active ring3 process; QEMU exits 0')
  result={'passed':True,'firmware':'uefi' if a.uefi else 'bios','unsupported_machine':a.unsupported_machine,'iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'checks':checks,'events':events,'exit':vm.returncode,'serial':transcript.decode(errors='replace')}
  name='shutdown-'+result['firmware']+('-unsupported' if a.unsupported_machine else '')+'.json';(out/name).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='serial'},indent=2))
 finally:
  if f:f.close()
  if q:q.close()
  if serial:serial.close()
  if vm.poll() is None:vm.terminate();vm.wait(timeout=5)
