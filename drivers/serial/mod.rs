//! QEMU COM1, 115200 8N1; bounded polling, no interrupts.
use core::arch::asm;
const BASE: u16 = 0x3f8;
unsafe fn out(port: u16, value: u8) {
    // Safety: approved x86-64 ring-0 QEMU port, never a host device.
    unsafe { asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags)); }
}
unsafe fn input(port: u16) -> u8 {
    let value: u8;
    unsafe { asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags)); }
    value
}
pub fn init() {
    unsafe {
        out(BASE + 1, 0); out(BASE + 3, 0x80);
        out(BASE, 1); out(BASE + 1, 0); out(BASE + 3, 3);
        out(BASE + 2, 0xc7); out(BASE + 4, 3);
    }
}
pub fn write(bytes: &[u8]) -> bool {
    crate::framebuffer::write(bytes);
    log(bytes)
}
/// Diagnostics retained on COM1 without cluttering the user screen.
pub fn log(bytes: &[u8]) -> bool {
    for &byte in bytes {
        let mut ready = false;
        for _ in 0..1_000_000 {
            if unsafe { input(BASE + 5) } & 0x20 != 0 { ready = true; break; }
            core::hint::spin_loop();
        }
        if !ready { return false; }
        unsafe { out(BASE, byte); }
    }
    true
}
pub fn number(mut value: u64) -> bool {
    let mut buf = [0u8; 20];
    let mut index = buf.len();
    loop {
        index -= 1; buf[index] = b'0' + (value % 10) as u8; value /= 10;
        if value == 0 { break; }
    }
    write(&buf[index..])
}

pub fn read() -> Option<u8> {
    if unsafe { input(BASE+5) } & 1 == 0 {None} else {Some(unsafe {input(BASE)})}
}

pub fn log_number(mut value:u64)->bool{let mut b=[0;20];let mut i=20;loop{i-=1;b[i]=b'0'+(value%10)as u8;value/=10;if value==0{break;}}log(&b[i..])}
