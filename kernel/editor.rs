//! Character-based line editor with fixed storage and eight-entry history.
#[derive(Clone,Copy,Debug,PartialEq)]pub enum Key{Char(char),Enter,Backspace,Delete,Left,Right,Home,End,Up,Down,Tab,Cancel}
#[derive(Clone,Copy)]pub struct Line{pub chars:[char;96],pub len:usize}
impl Line{pub const fn empty()->Self{Self{chars:['\0';96],len:0}}pub fn utf8(&self,out:&mut[u8;384])->usize{let mut len=0;for &ch in &self.chars[..self.len]{let mut b=[0;4];let text=ch.encode_utf8(&mut b);out[len..len+text.len()].copy_from_slice(text.as_bytes());len+=text.len();}len}}
pub enum Action{Changed,Submit(Line),TooLong,Complete,None}
pub struct Editor{pub line:Line,pub cursor:usize,overflow:bool,history:[Line;8],count:usize,next:usize,back:usize,draft:Line}
impl Editor{
 pub const fn new()->Self{Self{line:Line::empty(),cursor:0,overflow:false,history:[Line::empty();8],count:0,next:0,back:0,draft:Line::empty()}}
 pub fn replace(&mut self,text:&str){self.line=Line::empty();for c in text.chars().take(96){self.line.chars[self.line.len]=c;self.line.len+=1;}self.cursor=self.line.len;self.overflow=false;}
 pub fn key(&mut self,key:Key)->Action{match key{
 Key::Enter=>{let line=self.line;let bad=self.overflow;self.line=Line::empty();self.cursor=0;self.overflow=false;self.back=0;
 if bad{return Action::TooLong;}if line.len>0{let prev=(self.next+7)%8;let duplicate=self.count>0&&self.history[prev].len==line.len&&self.history[prev].chars[..line.len]==line.chars[..line.len];if !duplicate{self.history[self.next]=line;self.next=(self.next+1)%8;self.count=(self.count+1).min(8);}}Action::Submit(line)},
 Key::Char(c) if !c.is_control()=>{if self.line.len==96{self.overflow=true;return Action::None;}self.line.chars.copy_within(self.cursor..self.line.len,self.cursor+1);self.line.chars[self.cursor]=c;self.cursor+=1;self.line.len+=1;self.back=0;Action::Changed},
 Key::Backspace=>{if self.overflow{self.overflow=false;return Action::Changed;}if self.cursor>0{self.line.chars.copy_within(self.cursor..self.line.len,self.cursor-1);self.cursor-=1;self.line.len-=1;self.back=0;}Action::Changed},
 Key::Delete=>{if self.cursor<self.line.len{self.line.chars.copy_within(self.cursor+1..self.line.len,self.cursor);self.line.len-=1;self.back=0;}Action::Changed},
 Key::Left=>{self.cursor=self.cursor.saturating_sub(1);Action::Changed},Key::Right=>{self.cursor=(self.cursor+1).min(self.line.len);Action::Changed},Key::Home=>{self.cursor=0;Action::Changed},Key::End=>{self.cursor=self.line.len;Action::Changed},
 Key::Up=>{if self.back<self.count{if self.back==0{self.draft=self.line;}self.back+=1;self.line=self.history[(self.next+8-self.back)%8];self.cursor=self.line.len;}Action::Changed},
 Key::Down=>{if self.back>0{self.back-=1;self.line=if self.back==0{self.draft}else{self.history[(self.next+8-self.back)%8]};self.cursor=self.line.len;}Action::Changed},
 Key::Tab=>Action::Complete,Key::Cancel=>{self.line=Line::empty();self.cursor=0;self.back=0;self.overflow=false;Action::Changed},_=>Action::None}}
}
/// UTF-8 and common ANSI arrows from COM1; malformed sequences are discarded.
pub struct Decoder{escape:[u8;8],len:usize,utf:[u8;4],used:usize,need:usize,last_cr:bool}
impl Decoder{pub const fn new()->Self{Self{escape:[0;8],len:0,utf:[0;4],used:0,need:0,last_cr:false}}
 pub fn byte(&mut self,b:u8)->Option<Key>{
 if self.len>0{if self.len==8{self.len=0;return None;}self.escape[self.len]=b;self.len+=1;if self.len==2&&b!=b'['{self.len=0;return None;}if self.len>=3&&(b.is_ascii_alphabetic()||b==b'~'){let result=match &self.escape[1..self.len]{b"[A"=>Some(Key::Up),b"[B"=>Some(Key::Down),b"[C"=>Some(Key::Right),b"[D"=>Some(Key::Left),b"[H"|b"[1~"=>Some(Key::Home),b"[F"|b"[4~"=>Some(Key::End),b"[3~"=>Some(Key::Delete),_=>None};self.len=0;return result;}return None;}
 if self.used>0{if b&0xc0!=0x80{self.used=0;return self.byte(b);}self.utf[self.used]=b;self.used+=1;if self.used==self.need{let result=core::str::from_utf8(&self.utf[..self.used]).ok().and_then(|s|s.chars().next()).filter(|c|!c.is_control()).map(Key::Char);self.used=0;return result;}return None;}
 if b==27{self.escape[0]=b;self.len=1;return None;}if b==b'\n'&&self.last_cr{self.last_cr=false;return None;}self.last_cr=b==b'\r';
 match b{b'\r'|b'\n'=>Some(Key::Enter),8|127=>Some(Key::Backspace),9=>Some(Key::Tab),3=>Some(Key::Cancel),32..=126=>Some(Key::Char(b as char)),0xc2..=0xf4=>{self.utf[0]=b;self.used=1;self.need=if b<0xe0{2}else if b<0xf0{3}else{4};None},_=>None}
 }}
#[cfg(test)]mod tests{use super::*;
#[test]fn unicode_edit_history(){let mut e=Editor::new();for c in "żółć".chars(){e.key(Key::Char(c));}e.key(Key::Home);e.key(Key::Char('ą'));e.key(Key::Right);e.key(Key::Delete);let mut b=[0;384];let n=e.line.utf8(&mut b);assert_eq!(core::str::from_utf8(&b[..n]).unwrap(),"ążłć");e.key(Key::Enter);e.key(Key::Char('x'));e.key(Key::Up);assert_eq!(e.line.len,4);e.key(Key::Down);assert_eq!(e.line.chars[0],'x');}
#[test]fn bounds_and_decoder(){let mut e=Editor::new();for _ in 0..97{e.key(Key::Char('a'));}assert!(matches!(e.key(Key::Enter),Action::TooLong));let mut d=Decoder::new();assert_eq!(d.byte(27),None);assert_eq!(d.byte(b'['),None);assert_eq!(d.byte(b'D'),Some(Key::Left));assert_eq!(d.byte(0xc5),None);assert_eq!(d.byte(0xbc),Some(Key::Char('ż')));assert_eq!(d.byte(b'\r'),Some(Key::Enter));assert_eq!(d.byte(b'\n'),None);assert_eq!(d.byte(0xff),None);}
}
