//! Power-off for the supported QEMU q35 VM only; no general ACPI interpreter.
use core::arch::asm;
unsafe fn out16(port:u16,value:u16){unsafe{asm!("out dx, ax",in("dx")port,in("ax")value,options(nomem,nostack,preserves_flags));}}
unsafe fn read8(port:u16)->u8{let value;unsafe{asm!("in al, dx",in("dx")port,out("al")value,options(nomem,nostack,preserves_flags));}value}
unsafe fn pci(offset:u32)->u32{let value;unsafe{
 // One CPU, interrupt gate: PCI address/data transaction cannot be interleaved.
 asm!("out dx, eax",in("dx")0xcf8u16,in("eax")(0x8000f800u32|offset),options(nomem,nostack,preserves_flags));
 asm!("in eax, dx",in("dx")0xcfcu16,out("eax")value,options(nomem,nostack,preserves_flags));}value}
pub fn shutdown()->bool{unsafe{
 out16(0x510,0);for expected in *b"QEMU"{if read8(0x511)!=expected{return false;}}
 // ICH9 LPC 00:1f.0, active I/O PMBASE and ACPI decode, all read from PCI.
 if pci(0)!=0x29188086||pci(0x44)&0x80==0{return false;}
 let raw=pci(0x40);let base=(raw&0xff80)as u16;
 if raw&1==0||base==0{return false;}
 crate::serial::write("Zamykanie HAIOS. Dane w RAM zostaną usunięte.\n".as_bytes());
 crate::framebuffer::flush();crate::serial::write(b"HAIOS:SHUTDOWN\n");
 // QEMU PM1_CNT: sleep type zero, SLP_EN bit 13; preserves SCI_EN.
 let port=base+4;let previous:u16;asm!("in ax, dx",in("dx")port,out("ax")previous,options(nomem,nostack,preserves_flags));
 out16(port,(previous&!0x3c00)|0x2000);
 // If a launcher keeps QEMU alive on shutdown, stop all guest execution.
 crate::halt()
}}
