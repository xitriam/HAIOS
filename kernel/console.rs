//! Single CPU, IRQ-disabled console frontend with bounded text storage.
use crate::{editor::{Editor,Decoder,Key,Action},serial,process,fs::{self,ram}};
static mut INPUT:Editor=Editor::new();static mut DECODER:Decoder=Decoder::new();
static mut CWD:Option<ram::Name>=None;
struct Edit{active:bool,path:Option<ram::Name>,bytes:[u8;4096],len:usize}
static mut EDIT:Edit=Edit{active:false,path:None,bytes:[0;4096],len:0};
fn input()->&'static mut Editor{unsafe{&mut *core::ptr::addr_of_mut!(INPUT)}}
fn edit()->&'static mut Edit{unsafe{&mut *core::ptr::addr_of_mut!(EDIT)}}
fn cwd()->&'static str{unsafe{(&*core::ptr::addr_of!(CWD)).as_ref().map_or("/",|p|p.as_str())}}
fn text(s:&str){serial::write(s.as_bytes());}
pub fn prompt(){text(if edit().active{"edit> "}else{"haios> "});}
pub fn ready(){text("\nHAIOS:CONSOLE:READY\nWpisz help, aby zobaczyć polecenia.\n");prompt();crate::framebuffer::flush();}
fn redraw(){let mut bytes=[0;384];let (n,left)={let e=input();(e.line.utf8(&mut bytes),e.line.len-e.cursor)};text("\r\x1b[2K");prompt();serial::write(&bytes[..n]);if left>0{text("\x1b[");serial::number(left as u64);text("D");}}
fn error(e:ram::Error){text("ERROR RAMFS: ");text(match e{ram::Error::Invalid=>"błędna ścieżka",ram::Error::NotFound=>"nie znaleziono",ram::Error::NotDirectory=>"to nie katalog",ram::Error::IsDirectory=>"to katalog",ram::Error::Exists=>"już istnieje",ram::Error::Capacity=>"brak wolnych plików",ram::Error::TooLarge=>"przekroczony rozmiar",ram::Error::ReadOnly=>"tylko do odczytu",ram::Error::NotEmpty=>"katalog nie jest pusty"});text("\n");}
fn path(s:&str)->Result<ram::Name,ram::Error>{ram::normalize(cwd(),s)}
fn complete(){let mut bytes=[0;384];let n=input().line.utf8(&mut bytes);let line=core::str::from_utf8(&bytes[..n]).unwrap();if input().cursor!=input().line.len{return;}
 let split=line.rfind(' ').map_or(0,|i|i+1);let prefix=&line[split..];let mut common=[0u8;128];let mut length=0;let mut matches=0;
 let mut candidate=|name:&str|{if !name.starts_with(prefix){return;}if matches==0{length=name.len();common[..length].copy_from_slice(name.as_bytes());}else{length=common[..length].iter().zip(name.bytes()).take_while(|(a,b)|**a==*b).count();}matches+=1;};
 if split==0{for name in ["help","version","mem","ps","run","kill","ls","cat","pwd","cd","mkdir","touch","write","append","rm","stat","edit","clear","shutdown","echo","calc","uptime","wc"]{candidate(name);}}
 else if line.starts_with("run "){for name in crate::image::NAMES{candidate(name);}candidate("ipc");}
 else{let (directory,stem)=match prefix.rfind('/'){Some(i)=>(&prefix[..i+1],&prefix[i+1..]),None=>("",prefix)};
 if let Ok(dir)=path(if directory.is_empty(){"."}else{directory}){let _=fs::files().list(dir.as_str(),|name,_,_,_|{if name.starts_with(stem){let mut full=[0u8;128];let size=directory.len()+name.len();if size<128{full[..directory.len()].copy_from_slice(directory.as_bytes());full[directory.len()..size].copy_from_slice(name.as_bytes());candidate(core::str::from_utf8(&full[..size]).unwrap());}}});}}
 if matches>0{while length>0&&core::str::from_utf8(&common[..length]).is_err(){length-=1;}let mut out=[0u8;384];out[..split].copy_from_slice(&bytes[..split]);out[split..split+length].copy_from_slice(&common[..length]);let end=split+length;if let Ok(s)=core::str::from_utf8(&out[..end]){if s.chars().count()<=96{input().replace(s);}}if matches>1{text("\nDopasowania: ");serial::number(matches);text(" (uzupełniono wspólny początek)\n");}redraw();}}
fn command(line:&str){let line=line.trim();let (cmd,rest)=line.split_once(char::is_whitespace).unwrap_or((line,""));let rest=rest.trim_start();let mut words=rest.split_whitespace();let arg=words.next();let extra=words.next().is_some();
 match (cmd,arg,extra){
 ("",_,_)=>{},("help",None,false)=>text("help version mem ps run NAME kill PID ls cat about\nPrograms: hello count ipc fault writefault badptr spin fpu\npwd cd mkdir touch write append rm stat edit clear shutdown echo calc uptime wc\nPliki w RAM: write ŚCIEŻKA TEKST; append dopisuje tekst.\nedit ŚCIEŻKA: nowa treść, .save zapisuje, .cancel anuluje.\nStrzałki: edycja i historia; Tab: uzupełnienie. AltGr: polskie znaki.\nDane znikają po zamknięciu VM. /bin jest tylko do odczytu.\n"),
 ("shutdown",None,false)=>{if !crate::power::shutdown(){text("ERROR shutdown: wyłączenie niedostępne na tej platformie\n");}},
 ("version",None,false)=>text("HAIOS 0.2.0-dev console, x86_64/Rust/Limine; eksperymentalny, niewydany\n"),
 ("mem",None,false)=>process::info(),("ps",None,false)=>process::ps(),
 ("run",Some("ipc"),false)=>{if !process::pair(){text("ERROR process capacity or memory: brak zasobów\n");}},
 ("run",Some(name),_)=>{let args=rest[name.len()..].trim_start();if process::spawn_with(name,args).is_none(){text("ERROR unknown program, capacity or memory: program lub zasoby\n");}},
 ("kill",Some(pid),false)=>{if pid.parse::<u64>().ok().is_none_or(|p|!process::kill(p)){text("ERROR unknown pid\n");}},
 ("echo"|"calc"|"wc"|"uptime",_,_)=>{let normalized=if cmd=="wc"{match path(rest){Ok(p)=>Some(p),Err(e)=>{error(e);return;}}}else{None};let args=normalized.as_ref().map_or(rest,|p|p.as_str());if process::spawn_with(cmd,args).is_none(){text("ERROR program: za długie argumenty lub brak zasobów\n");}},
 ("pwd",None,false)=>{text(cwd());text("\n");},("clear",None,false)=>text("\x1b[2J\x1b[H"),
 ("stat",None,false)=>{let (nodes,bytes)=fs::files().usage();text("RAMFS nodes=");serial::number(nodes as u64);text("/32 bytes=");serial::number(bytes as u64);text("\n");},
 ("write"|"append",Some(p),_)=>{match path(p){Ok(p)=>{let content=rest[arg.unwrap().len()..].trim_start();
 if let Err(e)=fs::files().write(p.as_str(),content.as_bytes(),cmd=="append"){error(e);}else{text("OK zapisano\n");}},Err(e)=>error(e)}},
 ("ls",a,false)=>{if a.is_none(){text("RAM: hello count sender receiver fault writefault badptr spin fpu about\n");}match path(a.unwrap_or(".")){Ok(p)=>{if let Err(e)=fs::files().list(p.as_str(),|name,dir,len,ro|{text(name);if dir{text("/");}text(" ");serial::number(len as u64);if ro{text(" [ro]");}text("\n");}){error(e);}},Err(e)=>error(e)}},
 ("cat",Some("about"),false)=>text("HAIOS RAM image: własne programy, pliki w RAM. Bez dysków, sieci i AI.\n"),
 ("cat"|"cd"|"mkdir"|"touch"|"rm"|"edit",Some(p),false)=>{match path(p){Ok(p)=>{let result=match cmd{
 "cd"=>fs::files().is_dir(p.as_str()).map(|_|unsafe{CWD=Some(p)}),"mkdir"=>fs::files().mkdir(p.as_str()),"touch"=>fs::files().write(p.as_str(),b"",true),"rm"=>fs::files().remove(p.as_str()),
 "cat"=>fs::files().read(p.as_str()).map(|data|{if let Ok(s)=core::str::from_utf8(data){text(s);text("\n");}else{text("ERROR plik binarny\n");}}),
 _=>{let valid=fs::files().read(p.as_str());if matches!(valid,Err(ram::Error::IsDirectory)){Err(ram::Error::IsDirectory)}else{let e=edit();e.active=true;e.path=Some(p);e.len=0;text("Nowa treść (4096 bajtów); .save / .cancel\n");Ok(())}}
 };if let Err(e)=result{error(e);}},Err(e)=>error(e)}},
 _=>text("ERROR command: błędne polecenie; wpisz help\n")}}
fn key(k:Key){let action=input().key(k);match action{Action::Changed=>redraw(),Action::Complete=>complete(),Action::TooLong=>{text("\nERROR line too long: limit 96 znaków\n");prompt();},Action::Submit(line)=>{let mut bytes=[0;384];let n=line.utf8(&mut bytes);let s=core::str::from_utf8(&bytes[..n]).unwrap();text("\n");if edit().active{if s==".cancel"{edit().active=false;text("Anulowano\n");}else if s==".save"{let result={let e=edit();fs::files().write(e.path.as_ref().unwrap().as_str(),&e.bytes[..e.len],false)};match result{Ok(())=>{edit().active=false;text("OK zapisano\n");},Err(e)=>error(e)}}else{let e=edit();if e.len+n+1<=4096{e.bytes[e.len..e.len+n].copy_from_slice(&bytes[..n]);e.len+=n;e.bytes[e.len]=b'\n';e.len+=1;}else{text("ERROR limit edytora: 4096 bajtów\n");}}}else{command(s);}prompt();},Action::None=>{}}}
pub fn poll(){for _ in 0..32{let Some(b)=serial::read()else{break;};let k=unsafe{(&mut *core::ptr::addr_of_mut!(DECODER)).byte(b)};if let Some(k)=k{key(k);}}crate::keyboard::poll(key);crate::framebuffer::flush();}

/// Restore the framebuffer prompt after asynchronous process output. COM1 retains diagnostics.
pub fn prepare_output(){crate::framebuffer::write(b"\r\x1b[2K");}
pub fn restore_display(){let mut bytes=[0;384];let (n,left)={let e=input();(e.line.utf8(&mut bytes),e.line.len-e.cursor)};crate::framebuffer::write(b"\r\x1b[2K");crate::framebuffer::write(if edit().active{b"edit> "}else{b"haios> "});crate::framebuffer::write(&bytes[..n]);for _ in 0..left{crate::framebuffer::write(b"\x1b[D");}}
