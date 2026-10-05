//! Two checkpoint commit protocol, with static scratch and explicit dirty state.
use super::checkpoint as wire;
use crate::block::{self,Error};
static mut BUFFER:[u8;wire::PAYLOAD]=[0;wire::PAYLOAD];
static mut READY:bool=false;static mut DIRTY:bool=false;static mut GENERATION:u64=0;static mut SLOT:usize=0;
fn buffer()->&'static mut[u8;wire::PAYLOAD]{unsafe{&mut *core::ptr::addr_of_mut!(BUFFER)}}
fn read_slot(slot:usize)->Result<Option<u64>,Error>{let mut header=[0;512];block::transfer(wire::slot_lba(slot),&mut header,false)?;let Some(sequence)=wire::generation(&header)else{return Ok(None);};
 for(i,chunk)in buffer().chunks_exact_mut(512).enumerate(){block::transfer(wire::slot_lba(slot)+1+i as u64,chunk.try_into().unwrap(),false)?;}
 if !wire::payload_matches(&header,buffer()){return Ok(None);}
 // Capacity excludes all protected program nodes and root. Import performs final validation.
 if wire::validate(buffer(),super::files().durable_capacity()).is_err(){return Ok(None);}Ok(Some(sequence))
}
pub fn mount()->Result<(),Error>{unsafe{READY=false;DIRTY=false;GENERATION=0;SLOT=0;}let mut header=[0;512];block::transfer(0,&mut header,false)?;if !wire::superblock(&header){return Err(Error::Protocol);}
 let first=read_slot(0)?;let second=read_slot(1)?;let(slot,sequence)=match(first,second){(None,None)=>return Err(Error::Protocol),(Some(a),Some(b))if a==b=>return Err(Error::Protocol),(Some(a),Some(b))if b>a=>(1,b),(Some(a),_)=>(0,a),(None,Some(b))=>(1,b)};
 if read_slot(slot)?!=Some(sequence){return Err(Error::Protocol);}super::files().durable_import(buffer()).map_err(|_|Error::Protocol)?;
 unsafe{READY=true;DIRTY=false;GENERATION=sequence;SLOT=slot;}Ok(())
}
pub fn changed(){unsafe{DIRTY=true;}}
pub fn dirty()->bool{unsafe{DIRTY}}
pub fn sync()->Result<(),Error>{unsafe{
 if !READY{return Err(Error::Absent);}if !DIRTY{return Ok(());}if block::readonly(){return Err(Error::ReadOnly);}
 let sequence=GENERATION.checked_add(1).ok_or(Error::Protocol)?;let slot=1-SLOT;
 super::files().durable_export(buffer()).map_err(|_|Error::Protocol)?;wire::validate(buffer(),super::files().durable_capacity()).map_err(|_|Error::Protocol)?;
 let mut zero=[0;512];block::transfer(wire::slot_lba(slot),&mut zero,true)?;block::flush()?;marker(b"HAIOS:DISK:INVALIDATED\n");test_pause(1);
 for(i,chunk)in buffer().chunks_exact_mut(512).enumerate(){block::transfer(wire::slot_lba(slot)+1+i as u64,chunk.try_into().unwrap(),true)?;}
 block::flush()?;marker(b"HAIOS:DISK:PAYLOAD\n");test_pause(2);let mut header=wire::header(sequence,buffer());block::transfer(wire::slot_lba(slot),&mut header,true)?;block::flush()?;test_pause(3);
 GENERATION=sequence;SLOT=slot;DIRTY=false;marker(b"HAIOS:DISK:COMMIT\n");Ok(())
}}
fn marker(s:&[u8]){crate::serial::write(s);}
#[cfg(haios_test_storage_hooks)]static mut PAUSE:u64=0;
#[cfg(haios_test_storage_hooks)]pub fn set_test_pause(phase:u64){unsafe{PAUSE=phase;}}
fn test_pause(phase:u64){
 #[cfg(haios_test_storage_hooks)]unsafe{if PAUSE==phase{crate::serial::write(b"HAIOS:TEST:PAUSED\n");loop{core::arch::asm!("cli; hlt",options(nomem,nostack));}}}
 #[cfg(not(haios_test_storage_hooks))]let _=phase;
}
pub fn status(){crate::serial::write(b"DISK ready=");crate::serial::number(unsafe{READY}as u64);crate::serial::write(b" readonly=");crate::serial::number(block::readonly()as u64);crate::serial::write(b" dirty=");crate::serial::number(dirty()as u64);crate::serial::write(b" generation=");crate::serial::number(unsafe{GENERATION});crate::serial::write(b"\n");}
