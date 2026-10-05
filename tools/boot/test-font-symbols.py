#!/usr/bin/env python3
"""Check actual QEMU framebuffer cells against the pinned grayscale glyph atlas."""
import argparse,hashlib,json,re,time
from pathlib import Path
from vm_test import Guest
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');a=p.parse_args();out=a.out.resolve();firmware='uefi'if a.uefi else'bios'
root=Path(__file__).resolve().parents[2];atlas=(root/'drivers/framebuffer/font-coverage.bin').read_bytes();checks=[]
def glyph(index):
 pixels=[]
 for n in range(index*240,(index+1)*240):
  alpha=(atlas[n//2]>>4)if n%2==0 else(atlas[n//2]&15)
  pixels.append(bytes((fg*alpha+bg*(15-alpha)+7)//15 for bg,fg in zip([15,24,34],[214,229,247])))
 return b''.join(pixels)
def assert_screen(file,indices):
 data=file.read_bytes();m=re.match(rb'P6\s+(\d+)\s+(\d+)\s+255\s',data);assert m
 width,height=map(int,m.groups());pixels=data[m.end():];assert len(pixels)==width*height*3
 matches={}
 for index in indices:
  wanted=glyph(index);assert wanted!=glyph(31)
  found=[]
  for y in range(height//20):
   for x in range(width//12):
    cell=b''.join(pixels[((y*20+j)*width+x*12)*3:((y*20+j)*width+x*12+12)*3]for j in range(20))
    if cell==wanted:found.append([x,y])
  assert found,(index,file);matches[str(index)]=found
 return matches
with Guest(out,uefi=a.uefi)as g:
 g.send('clear');g.send('credits',rb'Mateusz');time.sleep(.1);file=out/f'font-symbols-credits-{firmware}.ppm';g.qmp('screendump',{'filename':str(file)});credits=assert_screen(file,[113,114]);checks.append('Actual credits pixels contain em dash and multiplication sign, distinct from question-mark fallback')
 g.send('clear');g.send('help calc',rb'Sk');time.sleep(.1);file=out/f'font-symbols-help-{firmware}.ppm';g.qmp('screendump',{'filename':str(file)});help_cells=assert_screen(file,[113]);checks.append('Actual help topic separator renders em dash')
 g.shutdown()
report={'passed':True,'firmware':firmware,'iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'atlas_sha256':hashlib.sha256(atlas).hexdigest(),'credits_cells':credits,'help_cells':help_cells,'checks':checks};(out/f'font-symbols-{firmware}.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
