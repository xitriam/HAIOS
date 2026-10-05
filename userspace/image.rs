//! Read-only built-in RAM program catalog. Flat one-page programs, not an ELF loader.
use core::{arch::global_asm,slice};
global_asm!(include_str!("demo.S"));
unsafe extern "C"{
    static user_fpu_start:u8;static user_fpu_end:u8;
    static user_hello_start:u8;static user_hello_end:u8;static user_count_start:u8;static user_count_end:u8;
    static user_fault_start:u8;static user_fault_end:u8;static user_writefault_start:u8;static user_writefault_end:u8;
    static user_badptr_start:u8;static user_badptr_end:u8;static user_sender_start:u8;static user_sender_end:u8;
    static user_receiver_start:u8;static user_receiver_end:u8;static user_spin_start:u8;static user_spin_end:u8;
}
pub fn program(name:&str)->Option<&'static [u8]>{unsafe{
    let (start,end)=match name{
        "fpu"=>(&raw const user_fpu_start,&raw const user_fpu_end),
        "hello"=>(&raw const user_hello_start,&raw const user_hello_end),"count"=>(&raw const user_count_start,&raw const user_count_end),
        "fault"=>(&raw const user_fault_start,&raw const user_fault_end),"writefault"=>(&raw const user_writefault_start,&raw const user_writefault_end),
        "badptr"=>(&raw const user_badptr_start,&raw const user_badptr_end),"sender"=>(&raw const user_sender_start,&raw const user_sender_end),
        "receiver"=>(&raw const user_receiver_start,&raw const user_receiver_end),"spin"=>(&raw const user_spin_start,&raw const user_spin_end),_=>return None,
    };
    Some(slice::from_raw_parts(start,end as usize-start as usize))
}}
pub fn label(name:&str)->Option<&'static str>{match name{"fpu"=>Some("fpu"),"hello"=>Some("hello"),"count"=>Some("count"),"fault"=>Some("fault"),"writefault"=>Some("writefault"),"badptr"=>Some("badptr"),"sender"=>Some("sender"),"receiver"=>Some("receiver"),"spin"=>Some("spin"),_=>None}}
