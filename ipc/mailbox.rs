//! Bounded nonblocking FIFO of u64 values, owned by one process.
#[derive(Clone,Copy)] pub struct Mailbox{values:[u64;8],head:usize,count:usize}
impl Mailbox{
    pub const fn new()->Self{Self{values:[0;8],head:0,count:0}}
    pub fn send(&mut self,value:u64)->bool{if self.count==8{return false;}self.values[(self.head+self.count)%8]=value;self.count+=1;true}
    pub fn receive(&mut self)->Option<u64>{if self.count==0{return None;}let value=self.values[self.head];self.head=(self.head+1)%8;self.count-=1;Some(value)}
}
#[cfg(test)] mod tests{use super::*;#[test]fn fifo_full_empty_and_wrap(){let mut m=Mailbox::new();assert_eq!(m.receive(),None);for _ in 0..100 {for i in 0..8{assert!(m.send(i));}assert!(!m.send(999));for i in 0..8{assert_eq!(m.receive(),Some(i));}assert_eq!(m.receive(),None);}}}
