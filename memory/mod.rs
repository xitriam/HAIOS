//! Boot integration for the PMM; the allocator is not yet a persistent kernel service.
pub mod pmm;
pub mod vmm;
use crate::limine;
use pmm::{Allocator,Regions,Error,PAGE};
fn mapped(offset:u64,physical:u64,length:u64)->Result<usize,&'static [u8]> {
    let start=offset.checked_add(physical).ok_or(b"pmm-address" as &[u8])?;
    let end=start.checked_add(length).ok_or(b"pmm-address" as &[u8])?;
    if start<0xffff800000000000 || end<=start {return Err(b"pmm-address");}
    Ok(start as usize)
}
pub fn initialize(summary:&limine::Summary)->Result<u64,&'static [u8]> {
    let mut regions=Regions::new();
    summary.usable_regions(|base,length| regions.add(base,length).map_err(|_|b"pmm-regions" as &[u8]))?;
    let meta=regions.metadata().map_err(|_|b"pmm-metadata" as &[u8])?;
    let bytes=regions.bitmap_bytes().map_err(|_|b"pmm-metadata" as &[u8])?;
    let address=mapped(summary.hhdm_offset,meta.0,meta.1)?;
    // Safety: metadata is within a validated Usable range mapped RW by pinned Limine.
    // Single CPU, IRQs disabled; no other allocator or alias accesses this storage.
    // Reserve the entire containing frame range before handing out any frame.
    let bitmap=unsafe{core::slice::from_raw_parts_mut(address as *mut u8,bytes)};
    let mut a=Allocator::new(regions,bitmap,meta).map_err(|_|b"pmm-init" as &[u8])?;
    a.exhaustion_test().map_err(|_|b"pmm-oom-test" as &[u8])?;
    let initial=a.free_frames();
    let first=a.alloc().ok_or(b"pmm-capacity" as &[u8])?;
    let second=a.alloc().ok_or(b"pmm-capacity" as &[u8])?;
    if first==second{return Err(b"pmm-alias");}
    let p=mapped(summary.hhdm_offset,first,PAGE)? as *mut u64;
    let q=mapped(summary.hhdm_offset,second,PAGE)? as *mut u64;
    // Safety: both complete Usable frames are exclusively owned by this test.
    unsafe{p.write_volatile(0x1122334455667788);q.write_volatile(0xaabbccddeeff0099);
        if p.read_volatile()!=0x1122334455667788 || q.read_volatile()!=0xaabbccddeeff0099{return Err(b"pmm-write");}
        p.write_volatile(0);q.write_volatile(0);
    }
    if a.release(meta.0)!=Err(Error::InvalidFree) || a.release(first+1)!=Err(Error::InvalidFree){return Err(b"pmm-invalid-free");}
    a.release(first).map_err(|_|b"pmm-free" as &[u8])?;
    if a.release(first)!=Err(Error::DoubleFree){return Err(b"pmm-double-free");}
    a.release(second).map_err(|_|b"pmm-free" as &[u8])?;
    if a.free_frames()!=initial{return Err(b"pmm-balance");}
    unsafe { *core::ptr::addr_of_mut!(PMM) = Some(a); }
    Ok(initial as u64)
}

static mut PMM: Option<Allocator<'static>> = None;
// Called only during boot or an interrupt gate with IRQ disabled, on the single CPU.
pub fn alloc() -> Option<u64> { unsafe { (&mut *core::ptr::addr_of_mut!(PMM)).as_mut()?.alloc() } }
pub fn release(frame:u64) -> Result<(),Error> { unsafe { (&mut *core::ptr::addr_of_mut!(PMM)).as_mut().ok_or(Error::Empty)?.release(frame) } }
pub fn free_frames() -> usize { unsafe { (&*core::ptr::addr_of!(PMM)).as_ref().map_or(0,|a|a.free_frames()) } }
