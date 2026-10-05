"""Bounded QEMU guest test harness; dedicated regular disk image only."""
import json,re,select,shutil,socket,subprocess,tempfile,time
from pathlib import Path
class Guest:
 def __init__(self,out,disk=None,uefi=False,readonly=False,io_error=None,allow_reboot=False,machine="q35"):
  self.tmp=tempfile.TemporaryDirectory(prefix='haios-storage-test-');w=Path(self.tmp.name);self.data=bytearray();self.events=[]
  ss=w/'serial';qs=w/'qmp';cmd=['qemu-system-x86_64','-machine',machine+',accel=kvm','-cpu','host','-smp','1','-m','256M','-display','none','-monitor','none','-serial',f'unix:{ss},server=on,wait=on','-qmp',f'unix:{qs},server=on,wait=off','-nic','none','-no-reboot','-boot','d','-drive',f'file={out}/haios.iso,media=cdrom,readonly=on,format=raw']
  if allow_reboot:cmd.remove('-no-reboot')
  if disk:
   assert disk.is_file()and not disk.is_symlink()and disk.stat().st_size==64*1024*1024
   backend=str(disk)
   if io_error:
    config=w/'blkdebug.conf';config.write_text('[inject-error]\nevent = "none"\niotype = "'+io_error+'"\nerrno = "5"\n');backend='blkdebug:'+str(config)+':'+str(disk)
   cmd+=['-drive',f'file={backend},id=data,if=none,format=raw,cache=writeback,werror=report,rerror=report,readonly={"on" if readonly else "off"}','-device','virtio-blk-pci,drive=data,disable-modern=on,disable-legacy=off']
  if uefi:
   shutil.copyfile('/usr/share/OVMF/OVMF_VARS_4M.fd',w/'vars.fd');cmd+=['-drive','if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd','-drive',f'if=pflash,format=raw,unit=1,file={w}/vars.fd']
  self.vm=subprocess.Popen(cmd,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
  deadline=time.monotonic()+10
  while not ss.exists():
   if self.vm.poll()is not None:raise RuntimeError(self.vm.stderr.read().decode())
   assert time.monotonic()<deadline;time.sleep(.02)
  self.serial=socket.socket(socket.AF_UNIX);self.serial.connect(str(ss));self.serial.setblocking(False)
  self.q=socket.socket(socket.AF_UNIX);self.q.settimeout(10);self.q.connect(str(qs));self.f=self.q.makefile('rwb',buffering=0);json.loads(self.f.readline());self.qmp('qmp_capabilities');self.wait(rb'\nhaios> ',timeout=35)
 def qmp(self,name,args=None):
  self.f.write((json.dumps({'execute':name,'arguments':args or {}})+'\n').encode())
  while True:
   line=self.f.readline()
   if not line:raise RuntimeError('QMP closed')
   r=json.loads(line)
   if 'event'in r:self.events.append(r)
   if 'error'in r:raise RuntimeError(r)
   if 'return'in r:return r['return']
 def wait(self,pattern,start=0,timeout=25):
  deadline=time.monotonic()+timeout
  while time.monotonic()<deadline:
   found=re.search(pattern,re.sub(rb'\x1b\[[0-9;]*[A-Za-z]',b'',bytes(self.data[start:])))
   if found:return found
   if select.select([self.serial],[],[],.02)[0]:
    chunk=self.serial.recv(65536)
    if not chunk:raise RuntimeError('Serial closed '+repr(pattern)+' '+repr(self.data[-600:]))
    self.data.extend(chunk)
  raise RuntimeError('Timeout '+repr(pattern)+' '+repr(self.data[-800:]))
 def send(self,text,pattern=rb'haios> '):
  start=len(self.data);self.serial.sendall(text.encode()+b'\n');self.wait(rb'\n(?:haios|edit)> ',start)
  reply=re.sub(rb'\x1b\[[0-9;]*[A-Za-z]',b'',bytes(self.data[start:]))
  # Redraws echo a prompt on every key. Only the prompt after Enter ends a command.
  reply=reply[reply.find(b'\n'):]
  found=re.search(pattern,reply);assert found,(text,pattern,reply[-1000:]);return found
 def shutdown(self):
  start=len(self.data);self.serial.sendall(b'shutdown\n');self.wait(rb'HAIOS:SHUTDOWN',start);self.vm.wait(timeout=20);assert self.vm.returncode==0
  while select.select([self.serial],[],[],.05)[0]:
   chunk=self.serial.recv(65536)
   if not chunk:break
   self.data.extend(chunk)
  assert b'HAIOS:SHUTDOWN' in self.data[start:]
  while True:
   line=self.f.readline()
   if not line:break
   event=json.loads(line)
   if 'event' in event:self.events.append(event)
  assert any(e.get('event')=='SHUTDOWN' and e.get('data',{}).get('guest') for e in self.events),self.events
 def close(self):
  if self.vm.poll()is None:self.vm.kill();self.vm.wait(timeout=5)
  self.f.close();self.q.close();self.serial.close();self.tmp.cleanup()
 def __enter__(self):return self
 def __exit__(self,*args):self.close()
