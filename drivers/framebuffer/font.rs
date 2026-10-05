//! DejaVu Sans Mono 2.37, pre-rasterized 4-bit grayscale in 12x20 cells.
//! Font data retains its upstream license: licenses/dejavu/LICENSE.
const ATLAS:&[u8;13560]=include_bytes!("font-coverage.bin");
pub fn coverage(ch:char,x:usize,y:usize)->u8{
 if x>=12||y>=20{return 0;}
 let glyph=if (' '..='~').contains(&ch){ch as usize-32}else{match ch{
 'ą'=>95,'ć'=>96,'ę'=>97,'ł'=>98,'ń'=>99,'ó'=>100,'ś'=>101,'ź'=>102,'ż'=>103,
 'Ą'=>104,'Ć'=>105,'Ę'=>106,'Ł'=>107,'Ń'=>108,'Ó'=>109,'Ś'=>110,'Ź'=>111,'Ż'=>112,_=>31}};
 let pixel=20*12*glyph+y*12+x;let packed=ATLAS[pixel/2];
 if pixel%2==0{packed>>4}else{packed&15}
}
