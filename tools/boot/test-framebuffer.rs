#[path="../../drivers/framebuffer/mod.rs"] mod framebuffer;
#[path="../../drivers/framebuffer/font.rs"] mod font;
#[test]
fn glyphs_palette_and_framebuffer_bounds(){
 let chars=(32..127).filter_map(char::from_u32).chain("ąćęłńóśźżĄĆĘŁŃÓŚŹŻ—×".chars());
 for ch in chars{let pixels:Vec<u8>=(0..20).flat_map(|y|(0..12).map(move|x|font::coverage(ch,x,y))).collect();
  assert!(pixels.iter().all(|&a|a<=15));if ch==' '{assert!(pixels.iter().all(|&a|a==0));}else{assert!(pixels.iter().any(|&a|a>0));}
  assert_eq!(font::coverage(ch,12,0),0);assert_eq!(font::coverage(ch,0,20),0);
 }
 let p:Vec<u8>=(0..20).flat_map(|y|(0..12).map(move|x|font::coverage('p',x,y))).collect();
 let upper:Vec<u8>=(0..20).flat_map(|y|(0..12).map(move|x|font::coverage('P',x,y))).collect();assert_ne!(p,upper);
 for y in 0..20{for x in 0..12{assert_eq!(font::coverage('🦀',x,y),font::coverage('?',x,y));}}
 for ch in ['—','×'] {let pixels:Vec<u8>=(0..20).flat_map(|y|(0..12).map(move|x|font::coverage(ch,x,y))).collect();let fallback:Vec<u8>=(0..20).flat_map(|y|(0..12).map(move|x|font::coverage('?',x,y))).collect();assert_ne!(pixels,fallback);}
 let width=1280;let height=800;let stride=1296;let guard=0xdeadbeefu32;
 let mut pixels=vec![guard;stride*height+2];let address=unsafe{pixels.as_mut_ptr().add(1)}as usize;
 for red in [16,0]{let blue=16-red;
  framebuffer::init(framebuffer::Config{address,width,height,pitch:stride*4,red,green:8,blue});
  framebuffer::write("\x1b[2J\x1b[HWitaj: p P Zażółć gęślą jaźń ĄĆĘŁŃÓŚŹŻ".as_bytes());framebuffer::flush();
  assert_eq!(pixels[0],guard);assert_eq!(*pixels.last().unwrap(),guard);
  for y in 0..height{assert!(pixels[1+y*stride+width..1+(y+1)*stride].iter().all(|&c|c==guard));}
  let bg=(15<<red)|(24<<8)|(34<<blue);let fg=(214<<red)|(229<<8)|(247<<blue);
  assert!(pixels[1..1+stride*20].iter().any(|&c|c!=bg&&c!=fg&&c!=guard));
  for &c in &pixels[1..1+stride*20]{if c==guard{continue;}
   assert!((15..=214).contains(&((c>>red)&255)));assert!((24..=229).contains(&((c>>8)&255)));assert!((34..=247).contains(&((c>>blue)&255)));
  }
 }
}
