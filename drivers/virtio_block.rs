//! Single outstanding request, legacy PCI VirtIO-blk, polling; QEMU-only contract.
use core::{arch::asm,ptr,sync::atomic::{fence,Ordering}};
#[derive(Clone,Copy,Debug,PartialEq)]pub enum Error{Absent,Unsupported,Memory,ReadOnly,Bounds,Timeout,Io,Protocol}
unsafe fn in8(p:u16)->u8{let v;unsafe{asm!("in al,dx",in("dx")p,out("al")v,options(nomem,nostack,preserves_flags));}v}
unsafe fn in16(p:u16)->u16{let v;unsafe{asm!("in ax,dx",in("dx")p,out("ax")v,options(nomem,nostack,preserves_flags));}v}
unsafe fn in32(p:u16)->u32{let v;unsafe{asm!("in eax,dx",in("dx")p,out("eax")v,options(nomem,nostack,preserves_flags));}v}
unsafe fn out8(p:u16,v:u8){unsafe{asm!("out dx,al",in("dx")p,in("al")v,options(nomem,nostack,preserves_flags));}}
unsafe fn out16(p:u16,v:u16){unsafe{asm!("out dx,ax",in("dx")p,in("ax")v,options(nomem,nostack,preserves_flags));}}
unsafe fn out32(p:u16,v:u32){unsafe{asm!("out dx,eax",in("dx")p,in("eax")v,options(nomem,nostack,preserves_flags));}}
unsafe fn pci(address:u32,offset:u32)->u32{unsafe{out32(0xcf8,address|offset);in32(0xcfc)}}
struct Device{port:u16,queue:u64,buffer:u64,offset:u64,size:usize,used_offset:usize,index:u16,sectors:u64,readonly:bool,failed:bool}
static mut DEVICE:Option<Device>=None;
fn device()->Result<&'static mut Device,Error>{unsafe{(&mut *ptr::addr_of_mut!(DEVICE)).as_mut().ok_or(Error::Absent)}}
pub fn init(offset:u64)->Result<(),Error>{unsafe{
 let mut found=None;
 for slot in 0..32u32{for function in 0..8u32{let address=0x80000000|(slot<<11)|(function<<8);
  if pci(address,0)==0x10011af4{if found.is_some(){return Err(Error::Unsupported);}found=Some(address);}
 }}
 let address=found.ok_or(Error::Absent)?;
 if pci(address,8)&255!=0||pci(address,0x2c)>>16!=2{return Err(Error::Unsupported);}
 let bar=pci(address,0x10);if bar&1==0||bar&0xffff0000!=0||bar&0xfffc==0||bar&0xfffc>0xffd0{return Err(Error::Unsupported);}let port=(bar&0xfffc)as u16;
 out32(0xcf8,address|4);let command=in16(0xcfc);out16(0xcfc,command|5);
 out8(port+18,0);let mut reset=false;for _ in 0..10000{if in8(port+18)==0{reset=true;break;}}if !reset{return Err(Error::Timeout);}
 out8(port+18,1);out8(port+18,3);let features=in32(port);crate::serial::write(b"DISK:features=");crate::serial::number(features as u64);crate::serial::write(b"\n");
 if features&(1<<9)==0{out8(port+18,128);return Err(Error::Unsupported);}
 out32(port+4,features&((1<<5)|(1<<9)));out16(port+14,0);let size=in16(port+12)as usize;crate::serial::write(b"DISK:queue=");crate::serial::number(size as u64);crate::serial::write(b"\n");
 if size<4||size>1024||!size.is_power_of_two()||in32(port+8)!=0{out8(port+18,128);return Err(Error::Unsupported);}
 let mut frames=[0u64;9];for i in 0..9{match crate::memory::alloc(){Some(f)=>frames[i]=f,None=>{for f in frames{if f!=0{let _=crate::memory::release(f);}}return Err(Error::Memory);}}}
 if (1..8).any(|i|frames[i]!=frames[0]+i as u64*4096)||frames[0]>>44!=0{for f in frames{let _=crate::memory::release(f);}return Err(Error::Memory);}
 let used_offset=(16*size+6+2*size+4095)&!4095;
 ptr::write_bytes((offset+frames[0])as *mut u8,0,32768);ptr::write_bytes((offset+frames[8])as *mut u8,0,4096);
 // Suppress interrupts; indices are still read with volatile and full fences.
 ((offset+frames[0]+(size*16)as u64)as *mut u16).write_volatile(1);
 out32(port+8,(frames[0]>>12)as u32);out8(port+18,7);
 let low=in32(port+20);let high=in32(port+24);let sectors=((high as u64)<<32)|low as u64;
 crate::serial::write(b"DISK:sectors=");crate::serial::number(sectors);crate::serial::write(b"\n");
 if sectors!=131072{out8(port+18,128);return Err(Error::Unsupported);}
 *ptr::addr_of_mut!(DEVICE)=Some(Device{port,queue:frames[0],buffer:frames[8],offset,size,used_offset,index:0,sectors,readonly:features&(1<<5)!=0,failed:false});Ok(())
}}
pub fn readonly()->bool{device().map_or(true,|d|d.readonly||d.failed)}
pub fn transfer(sector:u64,data:&mut[u8;512],write:bool)->Result<(),Error>{request(if write{1}else{0},sector,Some(data))}
pub fn flush()->Result<(),Error>{request(4,0,None)}
fn request(kind:u32,sector:u64,mut data:Option<&mut[u8;512]>)->Result<(),Error>{let d=device()?;
 if d.failed{return Err(Error::Io);}if sector>=d.sectors{return Err(Error::Bounds);}if kind!=0&&d.readonly{return Err(Error::ReadOnly);}
 unsafe{
  let q=(d.offset+d.queue)as *mut u8;let b=(d.offset+d.buffer)as *mut u8;
  b.cast::<u32>().write_volatile(kind);b.add(4).cast::<u32>().write_volatile(0);b.add(8).cast::<u64>().write_volatile(sector);b.add(1024).write_volatile(255);
  if kind==1{ptr::copy_nonoverlapping(data.as_ref().unwrap().as_ptr(),b.add(64),512);}
  let descriptor=|i:usize,physical:u64,length:u32,flags:u16,next:u16|{let p=q.add(i*16);p.cast::<u64>().write_volatile(physical);p.add(8).cast::<u32>().write_volatile(length);p.add(12).cast::<u16>().write_volatile(flags);p.add(14).cast::<u16>().write_volatile(next);};
  descriptor(0,d.buffer,16,1,if kind==4{2}else{1});
  if kind!=4{descriptor(1,d.buffer+64,512,1|if kind==0{2}else{0},2);}
  descriptor(2,d.buffer+1024,1,2,0);
  let avail=q.add(16*d.size);avail.add(4+2*(d.index as usize%d.size)).cast::<u16>().write_volatile(0);fence(Ordering::SeqCst);
  avail.add(2).cast::<u16>().write_volatile(d.index.wrapping_add(1));fence(Ordering::SeqCst);out16(d.port+16,0);
  let used=q.add(d.used_offset);let mut completed=false;
  for _ in 0..20_000_000{if used.add(2).cast::<u16>().read_volatile()!=d.index{completed=true;break;}core::hint::spin_loop();}
  if !completed{d.failed=true;out8(d.port+18,128);return Err(Error::Timeout);}fence(Ordering::SeqCst);
  let index=used.add(2).cast::<u16>().read_volatile();let entry=used.add(4+8*(d.index as usize%d.size));let id=entry.cast::<u32>().read_volatile();let length=entry.add(4).cast::<u32>().read_volatile();
  if index!=d.index.wrapping_add(1)||id!=0||length<1||length>if kind==0{513}else{1}{d.failed=true;out8(d.port+18,128);return Err(Error::Protocol);}
  d.index=index;let _=in8(d.port+19);match b.add(1024).read_volatile(){0=>{},1=>return Err(Error::Io),2=>return Err(Error::Unsupported),_=>{d.failed=true;return Err(Error::Protocol);}}
  if kind==0{ptr::copy_nonoverlapping(b.add(64),data.as_mut().unwrap().as_mut_ptr(),512);}Ok(())
 }
}
