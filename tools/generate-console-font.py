#!/usr/bin/env python3
"""Rasterize pinned DejaVu Sans Mono 2.37 into a bounded 4-bit coverage atlas.

Normal kernel builds consume the committed atlas and do not require Pillow/TTF.
"""
import argparse,hashlib,json
from pathlib import Path
import PIL
from PIL import Image,ImageDraw,ImageFont,features
ROOT=Path(__file__).resolve().parents[1]
SHA='b4a6c3e4faab8773f4ff761d56451646409f29abedd68f05d38c2df667d3c582'
CHARS=''.join(chr(i) for i in range(32,127))+'ąćęłńóśźżĄĆĘŁŃÓŚŹŻ'
p=argparse.ArgumentParser();p.add_argument('ttf',type=Path);p.add_argument('--check',action='store_true');a=p.parse_args()
assert hashlib.sha256(a.ttf.read_bytes()).hexdigest()==SHA,'Unexpected source font'
font=ImageFont.truetype(str(a.ttf),16,layout_engine=ImageFont.Layout.BASIC)
atlas=bytearray()
for ch in CHARS:
 image=Image.new('L',(12,20));box=font.getbbox(ch,anchor='ls')
 assert 0<=1+box[0] and 1+box[2]<=12 and 0<=16+box[1] and 16+box[3]<=20,(ch,box)
 ImageDraw.Draw(image).text((1,16),ch,font=font,fill=255,anchor='ls',stroke_width=0)
 pixels=[(value*15+127)//255 for value in image.get_flattened_data()]
 atlas.extend((pixels[i]<<4)|pixels[i+1] for i in range(0,240,2))
target=ROOT/'drivers/framebuffer/font-coverage.bin'
if a.check:assert target.read_bytes()==atlas,'Atlas differs; generator runtime must match receipt'
else:
 target.write_bytes(atlas)
 receipt={'font':'DejaVu Sans Mono 2.37','source_sha256':SHA,'source_archive_url':'https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2','source_archive_sha256':'fa9ca4d13871dd122f61258a80d01751d603b4d3ee14095d65453b4e846e17d7','pillow':PIL.__version__,'freetype':features.version_module('freetype2'),'pixel_size':16,'cell':[12,20],'origin':[1,16],'chars':CHARS,'coverage_bits':4,'atlas_sha256':hashlib.sha256(atlas).hexdigest()}
 (ROOT/'drivers/framebuffer/font-source.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n')
print('OK: 113 glyphs, 13560 bytes; clipping bounds and source hash checked')
