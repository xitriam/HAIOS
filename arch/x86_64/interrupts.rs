//! GDT/TSS/IDT and PIC/PIT for single-CPU QEMU. Interrupt gates prevent reentrancy.
use core::{arch::{asm,global_asm},ptr};
global_asm!(include_str!("interrupts.S"));
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Frame {pub r15:u64,pub r14:u64,pub r13:u64,pub r12:u64,pub r11:u64,pub r10:u64,pub r9:u64,pub r8:u64,pub rsi:u64,pub rdi:u64,pub rbp:u64,pub rdx:u64,pub rcx:u64,pub rbx:u64,pub rax:u64,pub vector:u64,pub error:u64,pub rip:u64,pub cs:u64,pub flags:u64,pub rsp:u64,pub ss:u64}
impl Frame {pub const fn zero()->Self{Self{r15:0,r14:0,r13:0,r12:0,r11:0,r10:0,r9:0,r8:0,rsi:0,rdi:0,rbp:0,rdx:0,rcx:0,rbx:0,rax:0,vector:0,error:0,rip:0,cs:0,flags:0,rsp:0,ss:0}}}
#[repr(C,packed)] struct Pointer{limit:u16,base:u64}
#[repr(C,packed)] struct Tss{reserved:u32,rsp:[u64;3],reserved2:u64,ist:[u64;7],reserved3:u64,reserved4:u16,iomap:u16}
#[repr(C,align(16))] struct Stack([u8;65536]);
static mut DF_STACK:Stack=Stack([0;65536]);
static mut IRQ_STACK:Stack=Stack([0;65536]);static mut IDLE_STACK:Stack=Stack([0;65536]);
static mut TSS:Tss=Tss{reserved:0,rsp:[0;3],reserved2:0,ist:[0;7],reserved3:0,reserved4:0,iomap:104};
static mut GDT:[u64;7]=[0,0x00af9a000000ffff,0x00cf92000000ffff,0x00cff2000000ffff,0x00affa000000ffff,0,0];
#[repr(C,packed)] #[derive(Clone,Copy)] struct Gate{low:u16,selector:u16,ist:u8,flags:u8,middle:u16,high:u32,reserved:u32}
const EMPTY:Gate=Gate{low:0,selector:0,ist:0,flags:0,middle:0,high:0,reserved:0};
static mut IDT:[Gate;256]=[EMPTY;256];
unsafe extern "C"{static isr_table:[u64;48];fn isr_128();fn reload_segments();pub fn kernel_idle()->!;}
fn gate(address:u64,dpl3:bool)->Gate{Gate{low:address as u16,selector:8,ist:0,flags:if dpl3{0xee}else{0x8e},middle:(address>>16)as u16,high:(address>>32)as u32,reserved:0}}
pub unsafe fn out(port:u16,value:u8){unsafe{asm!("out dx, al",in("dx")port,in("al")value,options(nomem,nostack,preserves_flags));}}
pub fn idle_frame()->Frame{let mut f=Frame::zero();f.rip=kernel_idle as *const () as u64;f.cs=8;f.ss=16;f.flags=0x202;f.rsp=ptr::addr_of!(IDLE_STACK)as u64+65536;f}
pub fn init(){unsafe{
    // Safety: boot only, static mapped storage, one CPU with interrupts disabled.
    ptr::addr_of_mut!(TSS.rsp).cast::<u64>().write_unaligned(ptr::addr_of!(IRQ_STACK)as u64+65536);
    ptr::addr_of_mut!(TSS.ist).cast::<u64>().write_unaligned(ptr::addr_of!(DF_STACK)as u64+65536);
    let base=ptr::addr_of!(TSS)as u64;let gdt=ptr::addr_of_mut!(GDT).cast::<u64>();
    gdt.add(5).write(103|((base&0xffffff)<<16)|(0x89<<40)|(((base>>24)&255)<<56));gdt.add(6).write(base>>32);
    let gdtr=Pointer{limit:55,base:gdt as u64};asm!("lgdt [{}]",in(reg)&gdtr,options(readonly,nostack,preserves_flags));reload_segments();
    asm!("ltr ax",in("ax")40u16,options(nostack,preserves_flags));
    let idt=ptr::addr_of_mut!(IDT).cast::<Gate>();for i in 0..48{idt.add(i).write(gate(isr_table[i],false));}let mut df=gate(isr_table[8],false);df.ist=1;idt.add(8).write(df);idt.add(128).write(gate(isr_128 as *const () as u64,true));
    let idtr=Pointer{limit:4095,base:idt as u64};asm!("lidt [{}]",in(reg)&idtr,options(readonly,nostack,preserves_flags));
    // Limine leaves LAPIC enabled with LINT0 masked. This single-CPU PIC backend
    // disables guest xAPIC via IA32_APIC_BASE so legacy IRQ0 reaches the CPU.
    let mut low:u32;let high:u32;
    asm!("rdmsr",in("ecx")0x1bu32,out("eax")low,out("edx")high,options(nostack));assert!(low & 0x400 == 0, "x2APIC not supported by PIC backend");low &= !0x800;
    asm!("wrmsr",in("ecx")0x1bu32,in("eax")low,in("edx")high,options(nostack));
    // Legacy PIC remapped to 32/40; only timer IRQ0 enabled.
    out(0x20,0x11);out(0xa0,0x11);out(0x21,32);out(0xa1,40);out(0x21,4);out(0xa1,2);out(0x21,1);out(0xa1,1);out(0x21,0xfe);out(0xa1,0xff);
    let divisor=11932u16;out(0x43,0x36);out(0x40,divisor as u8);out(0x40,(divisor>>8)as u8);
}}

const _: () = { assert!(core::mem::size_of::<Frame>()==176); assert!(core::mem::offset_of!(Frame,rip)==136); assert!(core::mem::size_of::<Tss>()==104); assert!(core::mem::size_of::<Gate>()==16); };
