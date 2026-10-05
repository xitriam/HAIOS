//! Fixed two-page user address spaces; inherited supervisor kernel mappings.
use core::{arch::asm,ptr};
use crate::memory;
pub const CODE:u64=0x400000;pub const STACK:u64=0x800000;const PAGE:u64=4096;
static mut OFFSET:u64=0;static mut KERNEL_ROOT:u64=0;
#[derive(Clone,Copy)] pub struct Space{pub root:u64,owned:[u64;7]}
impl Space{pub const fn empty()->Self{Self{root:0,owned:[0;7]}}}
fn pointer(frame:u64)->*mut u64{unsafe{(OFFSET+frame)as *mut u64}}
pub fn init(offset:u64){unsafe{
    assert!(core::arch::x86_64::__cpuid(0x80000001).edx & (1<<20) != 0, "NX required");
    OFFSET=offset;let cr3:u64;asm!("mov {},cr3",out(reg)cr3,options(nomem,nostack,preserves_flags));KERNEL_ROOT=cr3&0x000ffffffffff000;
    // NXE and supervisor write protection; kernel upper mapping remains Limine's.
    let mut low:u32;let mut high:u32;asm!("rdmsr",in("ecx")0xc0000080u32,out("eax")low,out("edx")high,options(nostack));low|=1<<11;
    asm!("wrmsr",in("ecx")0xc0000080u32,in("eax")low,in("edx")high,options(nostack));
    let cr0:u64;asm!("mov {},cr0",out(reg)cr0,options(nomem,nostack));asm!("mov cr0,{}",in(reg)(cr0|(1<<16)|(1<<2)|(1<<3)),options(nostack));
}}
pub fn kernel(){unsafe{asm!("mov cr3,{}",in(reg)KERNEL_ROOT,options(nostack));}}
pub fn activate(space:&Space){unsafe{asm!("mov cr3,{}",in(reg)space.root,options(nostack));}}
pub fn create(code:&[u8])->Option<Space>{
    if code.is_empty()||code.len()>4096{return None;}
    let mut s=Space::empty();
    for i in 0..7{match memory::alloc(){Some(p)=>{s.owned[i]=p;unsafe{ptr::write_bytes(pointer(p).cast::<u8>(),0,4096);}},None=>{destroy(s);return None;}}}
    let [root,pdpt,pd,pt_code,pt_stack,text,stack]=s.owned;s.root=root;
    unsafe{
        for i in 256..512{pointer(root).add(i).write(pointer(KERNEL_ROOT).add(i).read()&!4);}
        pointer(root).write(pdpt|7);pointer(pdpt).write(pd|7);
        pointer(pd).add(2).write(pt_code|7);pointer(pd).add(4).write(pt_stack|7);
        pointer(pt_code).write(text|5);pointer(pt_stack).write(stack|7|(1<<63));
        ptr::copy_nonoverlapping(code.as_ptr(),pointer(text).cast::<u8>(),code.len());
    }
    Some(s)
}
pub fn destroy(space:Space){for frame in space.owned {if frame!=0{assert!(memory::release(frame).is_ok());}}}
pub fn read_user(space:&Space,address:u64,length:usize)->Option<&'static [u8]>{
    if length>256{return None;}
    let end=address.checked_add(length as u64)?;
    let (base,physical)=if address>=CODE&&end<=CODE+PAGE{(CODE,space.owned[5])}else if address>=STACK&&end<=STACK+PAGE{(STACK,space.owned[6])}else{return None;};
    // Safety: validated own mapped page, copied via supervisor HHDM, handler excludes teardown.
    Some(unsafe{core::slice::from_raw_parts((pointer(physical)as usize+(address-base)as usize)as *const u8,length)})
}
