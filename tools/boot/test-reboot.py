#!/usr/bin/env python3
"""Guest-only reset, same QEMU PID, persisted cache, failures, credits/help/Tab."""
import argparse,hashlib,json,re,subprocess,tempfile,time
from pathlib import Path
from vm_test import Guest
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');a=p.parse_args();checks=[]
with tempfile.TemporaryDirectory(prefix='haios-reboot-')as td:
 disk=Path(td)/'data.img';subprocess.run(['python3',str(Path(__file__).with_name('create-data-disk.py')),str(disk)],check=True)
 with Guest(a.out,disk,a.uefi,allow_reboot=True)as g:
  g.send('help reboot',rb'Sk');g.send('help credits',rb'Sk');g.send('credits',rb'Mateusz \(xitriam\)');g.send('credits',rb'ChatGPT / Codex \(OpenAI\)');g.send('credits extra',rb'ERROR command');g.send('reboot extra',rb'ERROR command')
  # Actual completion on the PS/2 keyboard, then observe complete command execution.
  mark=len(g.data)
  for c in ['c','r','e','tab','ret']:g.qmp('send-key',{'keys':[{'type':'qcode','data':c}],'hold-time':15});time.sleep(.03)
  g.wait(rb'\nTw',mark);g.wait(rb'\nhaios> ',mark);g.send('edit /home/reboot',rb'Nowa tre');g.send('reboot',rb'edit> ');g.send('.cancel',rb'Anulowano')
  checks.append('Help/credits, PS2 Tab for credits, argument rejection, editor treats reboot as text')
  pid=g.vm.pid
  for attempt in range(3):
   g.send('write /home/reboot Próba '+str(attempt));g.send('run spin',rb'PROC:START');mark=len(g.data)
   if attempt==0:
    for c in ['r','e','b','tab','ret']:g.qmp('send-key',{'keys':[{'type':'qcode','data':c}],'hold-time':15});time.sleep(.03)
   else:g.serial.sendall(b'reboot\n')
   g.wait(rb'HAIOS:REBOOT',mark);g.wait(rb'HAIOS:CONSOLE:READY',mark,timeout=40);g.wait(rb'\nhaios> ',mark);assert g.vm.pid==pid and g.vm.poll()is None
   g.qmp('query-status');assert any(e.get('event')=='RESET'and e.get('data',{}).get('guest')and e['data'].get('reason')=='guest-reset'for e in g.events),g.events
   g.send('cat /home/reboot',rb'\n'+'Próba '.encode()+str(attempt).encode()+rb'\n');g.send('disk',rb'dirty=0 generation='+str(attempt+2).encode());g.send('ps',rb'PID NAME SWITCHES');assert b'spin'not in bytes(g.data[mark:]);g.send('credits',rb'Mateusz')
  g.shutdown()
  checks.append('Three real guest resets (PS2 Tab and COM1) retain QEMU PID, file, generation; active process clears; shutdown still works')
 for failure in ['ro','absent','write','flush']:
  before=hashlib.sha256(disk.read_bytes()).hexdigest()
  with Guest(a.out,None if failure=='absent'else disk,a.uefi,readonly=failure=='ro',io_error=failure if failure in ('write','flush')else None,allow_reboot=True)as g:
   g.send('write /home/unsaved pending');g.send('reboot',rb'ERROR reboot: zapis');g.send('cat /home/unsaved',rb'\npending\n');g.send('credits',rb'Mateusz');g.qmp('query-status');assert not any(e.get('event')=='RESET'for e in g.events)
  if failure!='flush':assert hashlib.sha256(disk.read_bytes()).hexdigest()==before
  checks.append('Failed '+failure+' sync refuses reboot and preserves live cache')
 with Guest(a.out,disk,allow_reboot=True,machine='pc')as g:g.send('reboot',rb'ERROR reboot: restart');g.send('credits',rb'Mateusz');g.qmp('query-status');assert not any(e.get('event')=='RESET'for e in g.events);checks.append('Unsupported chipset refuses reset and keeps console')
report={'passed':True,'firmware':'uefi'if a.uefi else'bios','iso_sha256':hashlib.sha256((a.out/'haios.iso').read_bytes()).hexdigest(),'checks':checks};(a.out/('reboot-'+report['firmware']+'.json')).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
