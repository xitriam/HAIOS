//! Minimal local ABI subset of pinned Limine 0BSD protocol; no external crate.
use core::{mem::{align_of, size_of}, ptr};
#[repr(C)]
struct Request<T> { id: [u64; 4], revision: u64, response: *const T }
#[repr(C)]
struct Pair { revision: u64, value: u64 }
#[repr(C)]
struct Image { revision: u64, physical: u64, virtual_base: u64 }
#[repr(C)]
struct Map { revision: u64, count: u64, entries: *const *const Entry }
#[repr(C)]
struct Entry { base: u64, length: u64, kind: u64 }
#[repr(C)]
struct Paging { request: Request<Pair>, mode: u64, max_mode: u64, min_mode: u64 }
#[repr(C)]
struct Framebuffers{revision:u64,count:u64,entries:*const *const Framebuffer}
#[repr(C)]
struct Framebuffer{address:*mut u8,width:u64,height:u64,pitch:u64,bpp:u16,model:u8,red_size:u8,red_shift:u8,green_size:u8,green_shift:u8,blue_size:u8,blue_shift:u8,unused:[u8;7],edid_size:u64,edid:*const u8,mode_count:u64,modes:*const u8}
#[used] #[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER:Request<Framebuffers>=request(0x9d5827dcd881dd75,0xa3148604f6fab11b);
pub fn framebuffer()->Result<crate::framebuffer::Config,&'static [u8]>{unsafe{
 let r=response(ptr::addr_of!(FRAMEBUFFER.response).read_volatile())?;if r.count==0||r.count>16{return Err(b"framebuffer-count");}
 let address=r.entries as usize;if address<0xffff800000000000||address%8!=0||address.checked_add(r.count as usize*8).is_none(){return Err(b"framebuffer-pointer");}
 let f=response(r.entries.read())?;if f.bpp!=32||f.model!=1||f.width<1248||f.width>4096||f.height<400||f.height>4096||f.pitch<f.width*4||f.pitch%4!=0||f.pitch>65536{return Err(b"framebuffer-geometry");}
 if [f.red_size,f.green_size,f.blue_size]!=[8;3]||[f.red_shift,f.green_shift,f.blue_shift].iter().any(|&s|s>24)||f.red_shift.abs_diff(f.green_shift)<8||f.red_shift.abs_diff(f.blue_shift)<8||f.green_shift.abs_diff(f.blue_shift)<8{return Err(b"framebuffer-format");}
 let bytes=f.pitch.checked_mul(f.height).filter(|&x|x<=64*1024*1024).ok_or(b"framebuffer-size" as &[u8])?;let base=f.address as usize;if base<0xffff800000000000||base%4!=0||base.checked_add(bytes as usize).is_none(){return Err(b"framebuffer-address");}
 Ok(crate::framebuffer::Config{address:base,width:f.width as usize,height:f.height as usize,pitch:f.pitch as usize,red:f.red_shift,green:f.green_shift,blue:f.blue_shift})
}}
const COMMON: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];
const fn request<T>(a: u64, b: u64) -> Request<T> {
    Request { id: [COMMON[0], COMMON[1], a, b], revision: 0, response: ptr::null() }
}
#[used] #[unsafe(link_section = ".limine_requests_start")]
static START: [u64; 4] = [0xf6b8f4b39de7d1ae, 0xfab91a6940fcb9cf, 0x785c6ed015d3e316, 0x181e920a7852b9d9];
#[used] #[unsafe(link_section = ".limine_requests")]
static mut BASE: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 6];
#[used] #[unsafe(link_section = ".limine_requests")]
static mut HHDM: Request<Pair> = request(0x48dcf1cb8ad2b852, 0x63984e959a98244b);
#[used] #[unsafe(link_section = ".limine_requests")]
static mut MEMMAP: Request<Map> = request(0x67cf3d9d378a806f, 0xe304acdfc50c3c62);
#[used] #[unsafe(link_section = ".limine_requests")]
static mut IMAGE: Request<Image> = request(0x71ba76863cc55f63, 0xb2644a48c516a487);
#[used] #[unsafe(link_section = ".limine_requests")]
static mut PAGING: Paging = Paging { request: request(0x95c1a0edab0944cb, 0xa4e5cb3842f7488a), mode: 0, max_mode: 0, min_mode: 0 };
#[used] #[unsafe(link_section = ".limine_requests_end")]
static END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];
const _: () = {
    assert!(size_of::<Request<Pair>>() == 48 && align_of::<Request<Pair>>() == 8);
    assert!(size_of::<Paging>() == 72 && size_of::<Map>() == 24);
    assert!(size_of::<Entry>() == 24 && size_of::<Image>() == 24);
};
pub struct Summary { pub entries: u64, pub usable_bytes: u64, pub hhdm_offset: u64, map: &'static Map }
impl Summary {
    pub fn usable_regions(&self, mut visit: impl FnMut(u64,u64)->Result<(), &'static [u8]>) -> Result<(), &'static [u8]> {
        for i in 0..self.map.count as usize {
            let entry = unsafe { response(self.map.entries.add(i).read())? };
            if entry.kind == 0 { visit(entry.base,entry.length)?; }
        }
        Ok(())
    }
}
unsafe fn response<T>(p: *const T) -> Result<&'static T, &'static [u8]> {
    let address = p as usize;
    if address < 0xffff800000000000 || address % align_of::<T>() != 0 || address.checked_add(size_of::<T>()).is_none() {
        return Err(b"response-pointer");
    }
    // Safety: trusted, pinned bootloader supplies mapped response storage. Checks
    // above reject null/noncanonical/unaligned/overflow; they cannot prove mapping.
    // No allocator, reclamation or page-table changes occur during this borrow.
    Ok(unsafe { &*p })
}
pub fn check() -> Result<Summary, &'static [u8]> {
    unsafe {
        let base = ptr::addr_of!(BASE).read_volatile();
        if base[2] != 0 || base[1] != 6 { return Err(b"base-revision"); }
        let hhdm_pointer = ptr::addr_of!(HHDM.response).read_volatile();
        #[cfg(haios_test_missing_hhdm)]
        let hhdm_pointer = { let _ = hhdm_pointer; ptr::null::<Pair>() };
        let hhdm = response(hhdm_pointer)?;
        let paging = response(ptr::addr_of!(PAGING.request.response).read_volatile())?;
        let image = response(ptr::addr_of!(IMAGE.response).read_volatile())?;
        let map = response(ptr::addr_of!(MEMMAP.response).read_volatile())?;
        if hhdm.revision != 0 || paging.revision != 0 || image.revision != 0 || map.revision != 0 { return Err(b"response-revision"); }
        if paging.value != 0 || image.virtual_base < 0xffffffff80000000 || hhdm.value < 0xffff800000000000 { return Err(b"address-mode"); }
        if map.count == 0 || map.count > 4096 { return Err(b"map-count"); }
        let bytes = (map.count as usize).checked_mul(size_of::<*const Entry>()).ok_or(b"map-overflow" as &[u8])?;
        let address = map.entries as usize;
        if address < 0xffff800000000000 || address % 8 != 0 || address.checked_add(bytes).is_none() { return Err(b"map-pointer"); }
        let mut usable = 0u64;
        let mut previous_base = 0;
        for i in 0..map.count as usize {
            let entry = response(map.entries.add(i).read())?;
            if entry.length == 0 || entry.kind > 8 || entry.base.checked_add(entry.length).is_none() { return Err(b"map-entry"); }
            if entry.base < previous_base { return Err(b"map-order"); }
            previous_base = entry.base;
            if entry.kind == 0 {
                if hhdm.value.checked_add(entry.base).is_none() || hhdm.value.checked_add(entry.base+entry.length).is_none() { return Err(b"hhdm-range"); }
                // Non-usable entries may overlap each other. Usable must never overlap anything.
                for j in 0..map.count as usize {
                    if i == j { continue; }
                    let other = response(map.entries.add(j).read())?;
                    let end = other.base.checked_add(other.length).ok_or(b"map-entry" as &[u8])?;
                    if entry.base < end && other.base < entry.base+entry.length { return Err(b"map-overlap"); }
                }
            }
            if entry.kind == 0 { usable = usable.checked_add(entry.length).ok_or(b"usable-overflow" as &[u8])?; }
        }
        Ok(Summary { entries: map.count, usable_bytes: usable, hhdm_offset: hhdm.value, map })
    }
}

const _:()={assert!(core::mem::size_of::<Framebuffer>()==80);assert!(core::mem::offset_of!(Framebuffer,edid_size)==48);};
