#![no_std]
#![no_main]
use core::arch::global_asm;
#[path = "../boot/limine.rs"] mod limine;
#[path = "../drivers/serial/mod.rs"] mod serial;
#[path = "../memory/mod.rs"] mod memory;
#[path = "../arch/x86_64/interrupts.rs"] mod interrupts;
#[path = "../userspace/image.rs"] mod image;
#[path = "../ipc/mailbox.rs"] mod mailbox;
#[path = "../scheduler/process.rs"] mod process;
mod console;
global_asm!(include_str!("../arch/x86_64/entry.S"));
unsafe extern "C" { fn haios_halt() -> !; }
fn halt() -> ! { unsafe { haios_halt() } }
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    serial::init();
    if !serial::write(b"HAIOS:BOOT:BEGIN\n") { halt(); }
    match limine::check() {
        Ok(summary) => {
            let frames = match memory::initialize(&summary) {
                Ok(frames) => frames,
                Err(reason) => { serial::write(b"HAIOS:BOOT:ERROR:");serial::write(reason);serial::write(b"\n");halt(); }
            };
            let ok = serial::write(b"HAIOS:PMM:OK free_frames=") && serial::number(frames) && serial::write(b"\n") && serial::write(b"HAIOS:MAP:entries=") && serial::number(summary.entries)
                && serial::write(b" usable_bytes=") && serial::number(summary.usable_bytes)
                && serial::write(b"\nHAIOS:BOOT:OK\n");
            if !ok { halt(); }
            interrupts::init();
            memory::vmm::init(summary.hhdm_offset);
            console::ready();
            unsafe { interrupts::kernel_idle() }
        }
        Err(reason) => { serial::write(b"HAIOS:BOOT:ERROR:"); serial::write(reason); serial::write(b"\n"); }
    }
    halt()
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    serial::write(b"HAIOS:PANIC\n"); halt()
}
