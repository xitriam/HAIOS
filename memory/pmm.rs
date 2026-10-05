//! Single-CPU physical frame allocator. No heap, paging, or bootloader reclamation.
pub const PAGE: u64 = 4096;
const MAX_REGIONS: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Overflow, Overlap, Capacity, Empty, Metadata, InvalidFree, DoubleFree }
#[derive(Clone, Copy)]
struct Region { start: u64, end: u64, first: usize }
const EMPTY: Region = Region { start: 0, end: 0, first: 0 };
pub struct Regions { regions: [Region; MAX_REGIONS], count: usize, frames: usize }
impl Regions {
    pub fn new() -> Self { Self { regions: [EMPTY; MAX_REGIONS], count: 0, frames: 0 } }
    pub fn add(&mut self, base: u64, length: u64) -> Result<(), Error> {
        let raw_end = base.checked_add(length).ok_or(Error::Overflow)?;
        let start = base.checked_add(PAGE-1).ok_or(Error::Overflow)? / PAGE * PAGE;
        let start = start.max(PAGE); // Never allocate physical page zero.
        let end = raw_end / PAGE * PAGE;
        if start >= end { return Ok(()); }
        if self.regions[..self.count].iter().any(|r| start < r.end && r.start < end) { return Err(Error::Overlap); }
        if self.count == MAX_REGIONS { return Err(Error::Capacity); }
        let pages = usize::try_from((end-start)/PAGE).map_err(|_| Error::Overflow)?;
        let total = self.frames.checked_add(pages).ok_or(Error::Overflow)?;
        self.regions[self.count] = Region { start, end, first: self.frames };
        self.count += 1; self.frames = total;
        Ok(())
    }
    pub fn bitmap_bytes(&self) -> Result<usize, Error> {
        if self.frames == 0 { return Err(Error::Empty); }
        self.frames.checked_add(7).map(|n| n/8).ok_or(Error::Overflow)
    }
    pub fn metadata(&self) -> Result<(u64,u64), Error> {
        let bytes = self.bitmap_bytes()? as u64;
        let length = bytes.checked_add(PAGE-1).ok_or(Error::Overflow)? / PAGE * PAGE;
        self.regions[..self.count].iter().find(|r| r.end-r.start >= length)
            .map(|r| (r.start,length)).ok_or(Error::Metadata)
    }
    fn address(&self, bit: usize) -> u64 {
        for r in &self.regions[..self.count] {
            let n = ((r.end-r.start)/PAGE) as usize;
            if bit >= r.first && bit-r.first < n { return r.start+(bit-r.first) as u64*PAGE; }
        }
        unreachable!()
    }
    fn index(&self, address: u64) -> Option<usize> {
        if address % PAGE != 0 { return None; }
        self.regions[..self.count].iter().find(|r| address >= r.start && address < r.end)
            .map(|r| r.first+((address-r.start)/PAGE) as usize)
    }
}
pub struct Allocator<'a> { regions: Regions, bitmap: &'a mut [u8], metadata: (u64,u64), free: usize, cursor: usize }
impl<'a> Allocator<'a> {
    pub fn new(regions: Regions, bitmap: &'a mut [u8], metadata: (u64,u64)) -> Result<Self, Error> {
        if bitmap.len() < regions.bitmap_bytes()? || regions.metadata()? != metadata { return Err(Error::Metadata); }
        bitmap.fill(0xff); // Padding bits remain unavailable.
        let mut allocator = Self { regions, bitmap, metadata, free: 0, cursor: 0 };
        for bit in 0..allocator.regions.frames {
            if !allocator.reserved(allocator.regions.address(bit)) { allocator.set(bit,false); allocator.free += 1; }
        }
        Ok(allocator)
    }
    fn reserved(&self, address: u64) -> bool { address >= self.metadata.0 && address-self.metadata.0 < self.metadata.1 }
    fn used(&self, bit: usize) -> bool { self.bitmap[bit/8] & (1 << (bit%8)) != 0 }
    fn set(&mut self, bit: usize, used: bool) {
        let mask = 1 << (bit%8);
        if used { self.bitmap[bit/8] |= mask; } else { self.bitmap[bit/8] &= !mask; }
    }
    pub fn free_frames(&self) -> usize { self.free }
    pub fn alloc(&mut self) -> Option<u64> {
        if self.free == 0 { return None; }
        for _ in 0..self.regions.frames {
            let bit = self.cursor; self.cursor = (bit+1) % self.regions.frames;
            if !self.used(bit) { self.set(bit,true); self.free -= 1; return Some(self.regions.address(bit)); }
        }
        unreachable!()
    }
    /// Caller must own the frame and stop all uses before releasing it. No generation handles yet.
    pub fn release(&mut self, address: u64) -> Result<(), Error> {
        if self.reserved(address) { return Err(Error::InvalidFree); }
        let bit = self.regions.index(address).ok_or(Error::InvalidFree)?;
        if !self.used(bit) { return Err(Error::DoubleFree); }
        self.set(bit,false); self.free += 1; Ok(())
    }
    /// Boot-only diagnostic: exhaust a fresh allocator, then restore every frame.
    pub fn exhaustion_test(&mut self) -> Result<(), Error> {
        let initial = self.free;
        if initial != self.regions.frames-(self.metadata.1/PAGE) as usize { return Err(Error::Metadata); }
        let mut count=0;
        while let Some(address)=self.alloc() {
            if self.reserved(address) || address == 0 { return Err(Error::Metadata); }
            count+=1;
        }
        if count != initial || self.free != 0 || self.alloc().is_some() { return Err(Error::Metadata); }
        for bit in 0..self.regions.frames {
            let address=self.regions.address(bit);
            if !self.reserved(address) { self.release(address)?; }
        }
        if self.free != initial { return Err(Error::Metadata); }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test] fn allocation_reclamation_oom_and_reserved() {
        let mut regions=Regions::new();regions.add(0,PAGE*12).unwrap();regions.add(PAGE*100,PAGE*5).unwrap();
        let metadata=regions.metadata().unwrap();let mut bitmap=vec![0;regions.bitmap_bytes().unwrap()];
        let mut a=Allocator::new(regions,&mut bitmap,metadata).unwrap();
        assert_eq!(a.free_frames(),15);a.exhaustion_test().unwrap();
        let mut owned=BTreeSet::new();while let Some(p)=a.alloc(){assert!(p!=0 && p!=metadata.0);assert!(owned.insert(p));}
        assert_eq!(owned.len(),15);assert_eq!(a.alloc(),None);
        assert_eq!(a.release(metadata.0),Err(Error::InvalidFree));assert_eq!(a.release(0),Err(Error::InvalidFree));
        assert_eq!(a.release(PAGE*99),Err(Error::InvalidFree));assert_eq!(a.release(PAGE+1),Err(Error::InvalidFree));
        for p in owned {a.release(p).unwrap();assert_eq!(a.release(p),Err(Error::DoubleFree));}
        assert_eq!(a.free_frames(),15);
    }
    #[test] fn fragmented_high_addresses_and_inward_rounding() {
        let mut r=Regions::new();r.add(1,PAGE*4).unwrap();r.add(1<<45,PAGE*2).unwrap();
        assert_eq!(r.frames,5);assert_eq!(r.bitmap_bytes(),Ok(1));
        let meta=r.metadata().unwrap();let mut bits=[0;1];let mut a=Allocator::new(r,&mut bits,meta).unwrap();
        let mut owned=vec![];while let Some(p)=a.alloc(){owned.push(p);}
        assert_eq!(owned,vec![PAGE*2,PAGE*3,1<<45,(1<<45)+PAGE]);
    }
    #[test] fn bad_ranges_and_capacity() {
        let mut r=Regions::new();assert_eq!(r.bitmap_bytes(),Err(Error::Empty));
        assert_eq!(r.add(u64::MAX-1,8),Err(Error::Overflow));r.add(PAGE,PAGE*3).unwrap();
        assert_eq!(r.add(PAGE*2,PAGE),Err(Error::Overlap));
        let mut r=Regions::new();for i in 0..MAX_REGIONS {r.add((i as u64*2+1)*PAGE,PAGE).unwrap();}
        assert_eq!(r.add(PAGE*1000,PAGE),Err(Error::Capacity));
    }
    #[test] fn insufficient_or_wrong_metadata() {
        let mut r=Regions::new();r.add(PAGE,PAGE*16).unwrap();let m=r.metadata().unwrap();
        assert!(matches!(Allocator::new(r,&mut [0;1],m),Err(Error::Metadata)));
        let mut r=Regions::new();r.add(PAGE,PAGE*16).unwrap();assert!(matches!(Allocator::new(r,&mut [0;2],(PAGE*2,PAGE)),Err(Error::Metadata)));
    }
    #[test] fn multi_page_metadata_never_released() {
        let mut r=Regions::new();r.add(PAGE,PAGE*40000).unwrap();let m=r.metadata().unwrap();assert_eq!(m.1,PAGE*2);
        let mut bits=vec![0;r.bitmap_bytes().unwrap()];let mut a=Allocator::new(r,&mut bits,m).unwrap();
        assert_eq!(a.release(m.0+PAGE),Err(Error::InvalidFree));a.exhaustion_test().unwrap();assert_eq!(a.free_frames(),39998);
    }
    #[test] fn repeated_mixed_cycles() {
        let mut r=Regions::new();r.add(PAGE,PAGE*67).unwrap();let m=r.metadata().unwrap();let mut bits=vec![0;r.bitmap_bytes().unwrap()];
        let mut a=Allocator::new(r,&mut bits,m).unwrap();let mut owned=BTreeSet::new();let mut seed=123u64;
        for _ in 0..20000 {seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if seed%3!=0 {if let Some(p)=a.alloc(){assert!(owned.insert(p));}}
            else if let Some(p)=owned.iter().nth((seed as usize >> 8) % owned.len().max(1)).copied(){owned.remove(&p);a.release(p).unwrap();}
            assert_eq!(a.free_frames()+owned.len(),66);
        }
        for p in owned {a.release(p).unwrap();}assert_eq!(a.free_frames(),66);
    }
}
