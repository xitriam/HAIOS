//! Single-CPU preemptive processes with private VMM spaces and bounded IPC.
use crate::{interrupts::{self,Frame},memory::{self,vmm},serial,image,mailbox};
#[derive(Clone,Copy)]struct Process{alive:bool,pid:u64,name:&'static str,frame:Frame,space:vmm::Space,mail:mailbox::Mailbox,switches:u64}
const EMPTY:Process=Process{alive:false,pid:0,name:"",frame:Frame::zero(),space:vmm::Space::empty(),mail:mailbox::Mailbox::new(),switches:0};
struct State{processes:[Process;4],current:Option<usize>,next_pid:u64,ticks:u64,last:usize}
static mut STATE:State=State{processes:[EMPTY;4],current:None,next_pid:1,ticks:0,last:3};
// Unique access: only IRQ-disabled boot and non-reentrant interrupt dispatcher on one CPU.
fn state()->&'static mut State{unsafe{&mut *core::ptr::addr_of_mut!(STATE)}}
pub fn ticks()->u64{state().ticks}
pub fn spawn(name:&str)->Option<u64>{
    let name=image::label(name)?;let code=image::program(name)?;
    let s=state();let slot=s.processes.iter().position(|p|!p.alive)?;
    let pid=s.next_pid;let next=pid.checked_add(1)?;let space=vmm::create(code)?;
    let mut f=Frame::zero();f.rip=vmm::CODE;f.cs=35;f.ss=27;f.flags=0x202;f.rsp=vmm::STACK+4096-8;
    s.processes[slot]=Process{alive:true,pid,name,frame:f,space,mail:mailbox::Mailbox::new(),switches:0};s.next_pid=next;
    serial::write(b"\nPROC:START pid=");serial::number(pid);serial::write(b" name=");serial::write(name.as_bytes());serial::write(b"\n");Some(pid)
}
pub fn pair()->bool{
    let Some(first)=spawn("receiver")else{return false;};let Some(second)=spawn("sender")else{kill(first);return false;};
    let s=state();for p in &mut s.processes{if p.pid==second&&p.alive{p.frame.r12=first;}}
    true
}
pub fn ps(){let s=state();serial::write(b"\nPID NAME SWITCHES\n");for p in &s.processes{if p.alive{serial::number(p.pid);serial::write(b" ");serial::write(p.name.as_bytes());serial::write(b" ");serial::number(p.switches);serial::write(b"\n");}}}
pub fn kill(pid:u64)->bool{
    let s=state();let Some(i)=s.processes.iter().position(|p|p.alive&&p.pid==pid)else{return false;};
    vmm::kernel();let space=s.processes[i].space;s.processes[i]=EMPTY;if s.current==Some(i){s.current=None;}vmm::destroy(space);
    serial::write(b"\nPROC:EXIT pid=");serial::number(pid);serial::write(b"\n");true
}
fn schedule(frame:&mut Frame){
    let s=state();if frame.cs&3==3{if let Some(i)=s.current{if s.processes[i].alive{s.processes[i].frame=*frame;}}}
    for step in 1..=4{let i=(s.last+step)%4;if s.processes[i].alive{
        s.current=Some(i);s.last=i;s.processes[i].switches+=1;*frame=s.processes[i].frame;vmm::activate(&s.processes[i].space);return;
    }}
    s.current=None;vmm::kernel();*frame=interrupts::idle_frame();
}
fn errno(number:u64)->u64{0u64.wrapping_sub(number)}
fn syscall(frame:&mut Frame){
    let Some(i)=state().current else{frame.rax=errno(38);return;};
    frame.rdx=0;
    match frame.rax{
        1=>{let s=state();if let Some(bytes)=vmm::read_user(&s.processes[i].space,frame.rdi,frame.rsi as usize){
            serial::write(b"\nUSER pid=");serial::number(s.processes[i].pid);serial::write(b" ");
            frame.rax=if serial::write(bytes){frame.rsi}else{errno(5)};
        }else{frame.rax=errno(14);}},
        2=>{let pid=state().processes[i].pid;kill(pid);schedule(frame);},
        3=>{frame.rax=0;schedule(frame);},
        4=>{frame.rax=state().ticks;},
        5=>{let s=state();frame.rax=match s.processes.iter_mut().find(|p|p.alive&&p.pid==frame.rdi){Some(p)=>if p.mail.send(frame.rsi){0}else{errno(11)},None=>errno(3)};},
        6=>{match state().processes[i].mail.receive(){Some(value)=>frame.rax=value,None=>{frame.rax=errno(11);frame.rdx=11;}}},
        _=>frame.rax=errno(38),
    }
}
#[unsafe(no_mangle)]pub extern "C" fn interrupt_dispatch(frame:&mut Frame){
    match frame.vector{
        32=>{state().ticks=state().ticks.wrapping_add(1);crate::console::poll();unsafe{interrupts::out(0x20,0x20);}schedule(frame);},
        128 if frame.cs&3==3=>syscall(frame),
        0..=31=>{serial::write(b"\nEXCEPTION vector=");serial::number(frame.vector);serial::write(b" error=");serial::number(frame.error);
            if frame.cs&3==3{if let Some(i)=state().current{let pid=state().processes[i].pid;serial::write(b" pid=");serial::number(pid);serial::write(b"\n");kill(pid);}schedule(frame);}
            else{serial::write(b" KERNEL:FAULT\n");crate::halt();}
        },
        33..=47=>unsafe{if frame.vector>=40{interrupts::out(0xa0,0x20);}interrupts::out(0x20,0x20);},
        _=>{serial::write(b"KERNEL:BAD-INTERRUPT\n");crate::halt();}
    }
}
pub fn info(){serial::write(b"\nMEM free_frames=");serial::number(memory::free_frames()as u64);serial::write(b" page_bytes=4096 ticks=");serial::number(ticks());serial::write(b"\n");}
