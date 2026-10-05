//! Bounded text rasterizer for a trusted Limine RGB framebuffer; no allocation.
mod font;
use core::ptr;
const CELL_WIDTH:usize=12;const CELL_HEIGHT:usize=20;
const COLS:usize=128;const ROWS:usize=64;const CELLS:usize=COLS*ROWS;
pub struct Config{pub address:usize,pub width:usize,pub height:usize,pub pitch:usize,pub red:u8,pub green:u8,pub blue:u8}
struct Terminal{enabled:bool,base:usize,pitch:usize,cols:usize,rows:usize,x:usize,y:usize,cells:[char;CELLS],dirty:[bool;CELLS],fg:u32,bg:u32,colors:[u32;16],escape:u8,param:usize,esc_len:usize,utf:[u8;4],used:usize,need:usize}
static mut TERM:Terminal=Terminal{enabled:false,base:0,pitch:0,cols:0,rows:0,x:0,y:0,cells:[' ';CELLS],dirty:[false;CELLS],fg:0,bg:0,colors:[0;16],escape:0,param:0,esc_len:0,utf:[0;4],used:0,need:0};
fn term()->&'static mut Terminal{unsafe{&mut *ptr::addr_of_mut!(TERM)}}
pub fn init(c:Config){let t=term();t.base=c.address;t.pitch=c.pitch/4;t.cols=(c.width/CELL_WIDTH).min(COLS);t.rows=(c.height/CELL_HEIGHT).min(ROWS);t.fg=(214u32<<c.red)|(229u32<<c.green)|(247u32<<c.blue);t.bg=(15u32<<c.red)|(24u32<<c.green)|(34u32<<c.blue);for alpha in 0..16{let mut color=0;for shift in [0,8,16,24]{let foreground=(t.fg>>shift)&255;let background=(t.bg>>shift)&255;color|=((foreground*alpha+background*(15-alpha)+7)/15)<<shift;}t.colors[alpha as usize]=color;}
t.enabled=true;t.dirty.fill(true);
// Safety: config validated before this call; the trusted bootloader maps the full extent.
for y in 0..c.height{for x in 0..c.width{unsafe{(t.base as *mut u32).add(y*t.pitch+x).write_volatile(t.bg);}}}}
impl Terminal{
 fn mark(&mut self){if self.x<self.cols&&self.y<self.rows{self.dirty[self.y*COLS+self.x]=true;}}
 fn clear_line(&mut self){for x in 0..self.cols{let i=self.y*COLS+x;self.cells[i]=' ';self.dirty[i]=true;}}
 fn clear(&mut self){self.cells.fill(' ');self.dirty.fill(true);self.x=0;self.y=0;}
 fn newline(&mut self){self.x=0;self.y+=1;if self.y==self.rows{self.cells.copy_within(COLS..self.rows*COLS,0);self.cells[(self.rows-1)*COLS..self.rows*COLS].fill(' ');self.dirty.fill(true);self.y-=1;}}
 fn character(&mut self,c:char){match c{'\n'=>self.newline(),'\r'=>self.x=0,'\u{8}'=>self.x=self.x.saturating_sub(1),c if !c.is_control()=>{let i=self.y*COLS+self.x;self.cells[i]=c;self.dirty[i]=true;self.x+=1;if self.x==self.cols{self.newline();}},_=>{}}}
 fn byte(&mut self,b:u8){if self.escape!=0{self.esc_len+=1;if self.esc_len>16{self.escape=0;return;}if self.escape==1{self.escape=if b==b'['{2}else{0};return;}if b.is_ascii_digit(){self.param=(self.param*10+(b-b'0')as usize).min(999);return;}let n=self.param.max(1);match b{b'K'=>self.clear_line(),b'J' if self.param==2=>self.clear(),b'H'=>{self.x=0;self.y=0;},b'G'=>self.x=(n-1).min(self.cols-1),b'D'=>self.x=self.x.saturating_sub(n),b'C'=>self.x=(self.x+n).min(self.cols-1),_=>{}}self.escape=0;return;}
 if self.used>0{if b&0xc0!=0x80{self.used=0;self.byte(b);return;}self.utf[self.used]=b;self.used+=1;if self.used==self.need{let ch=core::str::from_utf8(&self.utf[..self.used]).ok().and_then(|s|s.chars().next());self.used=0;if let Some(c)=ch{self.character(c);}}return;}
 match b{27=>{self.escape=1;self.param=0;self.esc_len=0;},0..=127=>self.character(b as char),0xc2..=0xf4=>{self.utf[0]=b;self.used=1;self.need=if b<0xe0{2}else if b<0xf0{3}else{4};},_=>{}}
 }
}
pub fn write(bytes:&[u8]){let t=term();if !t.enabled{return;}t.mark();for &b in bytes{t.byte(b);}t.mark();}
pub fn flush(){let t=term();if !t.enabled{return;}for y in 0..t.rows{for x in 0..t.cols{let i=y*COLS+x;if !t.dirty[i]{continue;}t.dirty[i]=false;let ch=t.cells[i];for py in 0..CELL_HEIGHT{for px in 0..CELL_WIDTH{let alpha=if x==t.x&&y==t.y&&py==CELL_HEIGHT-1&&px<CELL_WIDTH-1{15}else{font::coverage(ch,px,py)};let color=t.colors[alpha as usize];unsafe{(t.base as *mut u32).add((y*CELL_HEIGHT+py)*t.pitch+x*CELL_WIDTH+px).write_volatile(color);}}}}}}
