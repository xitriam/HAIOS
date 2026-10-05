//! Bounded ASCII COM1 console, polled by timer with interrupts disabled.
use crate::{serial,process};
struct Input{bytes:[u8;96],len:usize,overflow:bool,last_cr:bool}
static mut INPUT:Input=Input{bytes:[0;96],len:0,overflow:false,last_cr:false};
pub fn prompt(){serial::write(b"haios> ");}
pub fn ready(){serial::write(b"\nHAIOS:CONSOLE:READY\nType help for commands.\n");prompt();}
fn command(bytes:&[u8]){
    let Ok(line)=core::str::from_utf8(bytes)else{return;};let mut words=line.split_ascii_whitespace();
    let Some(cmd)=words.next()else{return;};let arg=words.next();let extra=words.next().is_some();
    match (cmd,arg,extra){
        ("help",None,false)=>{serial::write(b"\nhelp version mem ps run NAME kill PID ls cat about\nPrograms: hello count ipc fault writefault badptr spin fpu\n");},
        ("version",None,false)=>{serial::write(b"\nHAIOS 0.1.0-dev console, x86_64/Rust/Limine; experimental, unpublished\n");},
        ("mem",None,false)=>process::info(),("ps",None,false)=>process::ps(),
        ("run",Some("ipc"),false)=>{if !process::pair(){serial::write(b"\nERROR process capacity or memory\n");}},
        ("run",Some(name),false)=>{if process::spawn(name).is_none(){serial::write(b"\nERROR unknown program, capacity or memory\n");}},
        ("kill",Some(pid),false)=>{if pid.parse::<u64>().ok().is_none_or(|p|!process::kill(p)){serial::write(b"\nERROR unknown pid\n");}},
        ("ls",None,false)=>{serial::write(b"\nRAM: hello count sender receiver fault writefault badptr spin fpu about\n");},
        ("cat",Some("about"),false)=>{serial::write(b"\nHAIOS RAM image: read-only built-in files and flat demo programs. No disk writes, networking or AI.\n");},
        _=>{serial::write(b"\nERROR command; type help\n");}
    }
}
pub fn poll(){for _ in 0..32{let Some(byte)=serial::read()else{break;};
    let input=unsafe{&mut *core::ptr::addr_of_mut!(INPUT)};
    if byte==b'\n'&&input.last_cr{input.last_cr=false;continue;}input.last_cr=byte==b'\r';
    match byte{
        b'\r'|b'\n'=>{serial::write(b"\n");let len=input.len;let copy=input.bytes;let overflow=input.overflow;input.len=0;input.overflow=false;
            if overflow{serial::write(b"ERROR line too long\n");}else{command(&copy[..len]);}prompt();},
        8|127=>{if input.len>0{input.len-=1;serial::write(b"\x08 \x08");}},
        32..=126=>{if input.len<96&&!input.overflow{input.bytes[input.len]=byte;input.len+=1;serial::write(&[byte]);}else{input.overflow=true;}},
        _=>{}
    }
}}
