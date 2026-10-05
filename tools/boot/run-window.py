#!/usr/bin/env python3
"""User framebuffer through password-protected loopback VNC and SSH."""
import argparse,fcntl,json,os,secrets,signal,socket,subprocess,tempfile,time
from pathlib import Path
from data_disk import locked_disk,device_args
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--disk',type=Path);p.add_argument('--vnc-port',type=int,default=5901);p.add_argument('--password-file',type=Path);a=p.parse_args();out=a.out.resolve();disk_lock,disk_path=locked_disk(a.disk or out/'haios-data.img')
if not 5900<=a.vnc_port<=5999:raise SystemExit('VNC port must be 5900..5999')
lock=(out/'window.lock').open('w')
try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
except BlockingIOError:raise SystemExit('HAIOS window VM already running')
password_path=a.password_file or out/'window-password'
if a.password_file is None:
 fd=os.open(password_path,os.O_CREAT|os.O_TRUNC|os.O_WRONLY,0o600)
 with os.fdopen(fd,'w')as f:f.write(secrets.token_hex(4)+'\n')
if password_path.stat().st_mode&0o077:raise SystemExit('Password file must have mode 600')
password=password_path.read_text().strip()
if len(password)!=8 or not password.isascii():raise SystemExit('Expected eight ASCII characters')
with tempfile.TemporaryDirectory(prefix='haios-window-') as td:
 w=Path(td);qmp=w/'qmp';session_file=out/'window-session.json'
 cmd=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-smp','1','-m','256M','-vnc',f'127.0.0.1:{a.vnc_port-5900},password=on','-monitor','none','-serial',f'file:{out}/window.serial','-qmp',f'unix:{qmp},server=on,wait=off','-nic','none','-boot','d','-drive',f'file={out}/haios.iso,media=cdrom,readonly=on,format=raw']
 cmd+=device_args(disk_path)
 vm=subprocess.Popen(cmd)
 def stop(signum,frame):raise KeyboardInterrupt
 for sig in [signal.SIGTERM,signal.SIGHUP]:signal.signal(sig,stop)
 try:
  deadline=time.monotonic()+10
  while not qmp.exists():
   if vm.poll() is not None or time.monotonic()>deadline:raise RuntimeError('No QMP socket')
   time.sleep(.02)
  with socket.socket(socket.AF_UNIX) as s:
   s.settimeout(5);s.connect(str(qmp));f=s.makefile('rwb',buffering=0);json.loads(f.readline())
   for command,args in [('qmp_capabilities',{}),('set_password',{'protocol':'vnc','password':password,'connected':'keep'})]:
    f.write((json.dumps({'execute':command,'arguments':args})+'\n').encode())
    while True:
     result=json.loads(f.readline())
     if 'error'in result:raise RuntimeError(result['error'])
     if 'return'in result:break
   f.close()
  session_file.write_text(json.dumps({'pid':vm.pid,'qmp':str(qmp),'vnc_port':a.vnc_port,'command':cmd},indent=2)+'\n')
  print('HAIOS: okno VM przez SSH. Ctrl-C zamyka tę VM. Hasło w lokalnym oknie iMaca.',flush=True)
  vm.wait()
 except KeyboardInterrupt:pass
 finally:
  if vm.poll() is None:
   vm.terminate()
   try:vm.wait(timeout=5)
   except subprocess.TimeoutExpired:vm.kill();vm.wait()
  session_file.unlink(missing_ok=True)
  password_path.unlink(missing_ok=True)
