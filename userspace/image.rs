//! Read-only built-in RAM program catalog. Flat one-page programs, not an ELF loader.
use core::{arch::global_asm,slice};
global_asm!(include_str!("demo.S"));
global_asm!(include_str!("utilities.S"));
pub const NAMES:&[&str]=&["hello","count","sender","receiver","fault","writefault","badptr","spin","fpu","echo","calc","uptime","wc","fsbadptr"];
unsafe extern "C"{
 static user_fsbadptr_start:u8;static user_fsbadptr_end:u8;
 static user_echo_start:u8;static user_echo_end:u8;static user_calc_start:u8;static user_calc_end:u8;static user_uptime_start:u8;static user_uptime_end:u8;static user_wc_start:u8;static user_wc_end:u8;
    static user_fpu_start:u8;static user_fpu_end:u8;
    static user_hello_start:u8;static user_hello_end:u8;static user_count_start:u8;static user_count_end:u8;
    static user_fault_start:u8;static user_fault_end:u8;static user_writefault_start:u8;static user_writefault_end:u8;
    static user_badptr_start:u8;static user_badptr_end:u8;static user_sender_start:u8;static user_sender_end:u8;
    static user_receiver_start:u8;static user_receiver_end:u8;static user_spin_start:u8;static user_spin_end:u8;
}
pub fn program(name:&str)->Option<&'static [u8]>{unsafe{
    let (start,end)=match name{
 "fsbadptr"=>(&raw const user_fsbadptr_start,&raw const user_fsbadptr_end),
 "echo"=>(&raw const user_echo_start,&raw const user_echo_end),"calc"=>(&raw const user_calc_start,&raw const user_calc_end),"uptime"=>(&raw const user_uptime_start,&raw const user_uptime_end),"wc"=>(&raw const user_wc_start,&raw const user_wc_end),
        "fpu"=>(&raw const user_fpu_start,&raw const user_fpu_end),
        "hello"=>(&raw const user_hello_start,&raw const user_hello_end),"count"=>(&raw const user_count_start,&raw const user_count_end),
        "fault"=>(&raw const user_fault_start,&raw const user_fault_end),"writefault"=>(&raw const user_writefault_start,&raw const user_writefault_end),
        "badptr"=>(&raw const user_badptr_start,&raw const user_badptr_end),"sender"=>(&raw const user_sender_start,&raw const user_sender_end),
        "receiver"=>(&raw const user_receiver_start,&raw const user_receiver_end),"spin"=>(&raw const user_spin_start,&raw const user_spin_end),_=>return None,
    };
    Some(slice::from_raw_parts(start,end as usize-start as usize))
}}
pub fn label(name:&str)->Option<&'static str>{NAMES.iter().copied().find(|n|*n==name)}
