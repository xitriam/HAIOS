#!/usr/bin/env python3
"""Real guest checks for every documented command, examples and PS2 help completion."""
import argparse,hashlib,json,re,time,subprocess,tempfile
from pathlib import Path
from vm_test import Guest
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');a=p.parse_args()
source=(Path(__file__).resolve().parents[2]/'kernel/help.rs').read_text();names=re.findall(r'Topic \{name:"([a-z]+)"',source)
assert len(names)==27 and len(set(names))==27
temporary=tempfile.TemporaryDirectory(prefix='haios-help-');disk=Path(temporary.name)/'data.img'
subprocess.run(['python3',str(Path(__file__).with_name('create-data-disk.py')),str(disk)],check=True)
with Guest(a.out,disk,uefi=a.uefi) as g:
 start=len(g.data);g.send('help',rb'help calc');reply=bytes(g.data[start:])
 for name in names:
  assert re.search(('\n'+name+r'(?: [^\n]*)? — ').encode(),reply),name
  start=len(g.data);g.send('help '+name,rb'Sk');response=bytes(g.data[start:])
  assert 'Przykład: '.encode()in response,name
 g.send('help missing',rb'ERROR help');g.send('help calc extra',rb'ERROR command')
 start=len(g.data)
 for code in ['h','e','l','p','spc','c','a','l','tab','ret']:
  g.qmp('send-key',{'keys':[{'type':'qcode','data':code}],'hold-time':15});time.sleep(.03)
 g.wait(rb'Sk',start);g.wait(rb'\nhaios> ',start)
 g.send('mkdir /home/Proba');g.send('cd /home/Proba');g.send('pwd',rb'/home/Proba');g.send('cd /');g.send('touch /home/notatka');g.send('write /home/notatka Witaj');g.send('append /home/notatka !');g.send('cat /home/notatka',rb'Witaj!')
 start=len(g.data);g.send('calc 12 + 3');g.wait(rb'USER pid=[0-9]+ 15\n',start)
 g.send('edit /home/notatka');g.send('help calc',rb'edit> ');g.send('.cancel',rb'Anulowano');g.send('cat /home/notatka',rb'Witaj!')
 g.shutdown()
temporary.cleanup()
report={'passed':True,'firmware':'uefi'if a.uefi else'bios','commands':names,'iso_sha256':hashlib.sha256((a.out/'haios.iso').read_bytes()).hexdigest(),'checks':['All 27 commands have overview, syntax and example','Unknown topics and extra arguments rejected','Actual PS2 Tab completes help calc','mkdir/cd/touch/write/append/cat/calc examples work','Help inside editor remains text, cancel preserves file']};(a.out/('help-'+report['firmware']+'.json')).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
