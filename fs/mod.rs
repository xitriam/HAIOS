//! Single-CPU IRQ-disabled ownership, same lifecycle as the console.
pub mod ram;
static mut FILES:ram::Fs=ram::Fs::new();
pub fn files()->&'static mut ram::Fs{unsafe{&mut *core::ptr::addr_of_mut!(FILES)}}
pub fn init(){let f=files();f.mkdir("/bin").unwrap();f.mkdir("/home").unwrap();f.write("/about","HAIOS 0.2.0 — pliki w RAM. Dane znikają po zamknięciu VM. Bez dysków, sieci i AI.\n".as_bytes(),false).unwrap();f.write("/home/czytaj.txt","Witaj w HAIOS! Użyj help, mkdir, write, cat i edit.\n".as_bytes(),false).unwrap();
for name in crate::image::NAMES{let mut path=[0u8;64];path[..5].copy_from_slice(b"/bin/");path[5..5+name.len()].copy_from_slice(name.as_bytes());let path=core::str::from_utf8(&path[..5+name.len()]).unwrap();f.write(path,crate::image::program(name).unwrap(),false).unwrap();f.protect(path).unwrap();}f.protect("/bin").unwrap();}
