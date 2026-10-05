//! Bounded i8042 polling. This backend is for the approved QEMU q35 only.
mod decode;
use core::arch::asm;use crate::{editor::Key,interrupts::out};
static mut DECODER:decode::Decoder=decode::Decoder::new();
unsafe fn input(port:u16)->u8{let b:u8;unsafe{asm!("in al,dx",in("dx")port,out("al")b,options(nomem,nostack,preserves_flags));}b}
fn output(port:u16,b:u8)->bool{for _ in 0..100000{if unsafe{input(0x64)}&2==0{unsafe{out(port,b);}return true;}core::hint::spin_loop();}false}
fn read()->Option<u8>{for _ in 0..100000{let status=unsafe{input(0x64)};if status&1!=0{let b=unsafe{input(0x60)};if status&0xe0==0{return Some(b);}}core::hint::spin_loop();}None}
fn command(b:u8)->bool{output(0x60,b)&&read()==Some(0xfa)}
pub fn init()->bool{if !output(0x64,0xad)||!output(0x64,0xa7){return false;}for _ in 0..32{if unsafe{input(0x64)}&1==0{break;}unsafe{input(0x60);}}
 if !output(0x64,0x20){return false;}let Some(config)=read()else{return false;};if !output(0x64,0x60)||!output(0x60,(config|0x40)&!0x03)||!output(0x64,0xae){return false;}
 command(0xf5)&&command(0xf0)&&command(2)&&command(0xf4)
}
pub fn poll(mut receive:impl FnMut(Key)){for _ in 0..32{let status=unsafe{input(0x64)};if status&1==0{break;}let b=unsafe{input(0x60)};if status&0xe0!=0{continue;}let decoder=unsafe{&mut *core::ptr::addr_of_mut!(DECODER)};if let Some(key)=decoder.byte(b){receive(key);}}}
