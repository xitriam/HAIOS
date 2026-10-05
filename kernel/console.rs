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
 if split==0 || line.starts_with("help "){crate::help::names(&mut candidate);}
 else if line.starts_with("run "){for name in crate::image::NAMES{candidate(name);}candidate("ipc");}
 else{let (directory,stem)=match prefix.rfind('/'){Some(i)=>(&prefix[..i+1],&prefix[i+1..]),None=>("",prefix)};
 if let Ok(dir)=path(if directory.is_empty(){"."}else{directory}){let _=fs::files().list(dir.as_str(),|name,_,_,_|{if name.starts_with(stem){let mut full=[0u8;128];let size=directory.len()+name.len();if size<128{full[..directory.len()].copy_from_slice(directory.as_bytes());full[directory.len()..size].copy_from_slice(name.as_bytes());candidate(core::str::from_utf8(&full[..size]).unwrap());}}});}}
 if matches>0{while length>0&&core::str::from_utf8(&common[..length]).is_err(){length-=1;}let mut out=[0u8;384];out[..split].copy_from_slice(&bytes[..split]);out[split..split+length].copy_from_slice(&common[..length]);let end=split+length;if let Ok(s)=core::str::from_utf8(&out[..end]){if s.chars().count()<=96{input().replace(s);}}if matches>1{text("\nDopasowania: ");serial::number(matches);text(" (uzupełniono wspólny początek)\n");}redraw();}}
fn command(line:&str){let line=line.trim();let (cmd,rest)=line.split_once(char::is_whitespace).unwrap_or((line,""));let rest=rest.trim_start();let mut words=rest.split_whitespace();let arg=words.next();let extra=words.next().is_some();
 match (cmd,arg,extra){
 #[cfg(haios_test_storage_hooks)]("dtest",Some(n),false)=>{if let Ok(n)=n.parse::<u64>(){fs::durable::set_test_pause(n);}},
 ("",_,_)=>{},("help",name,false)=>crate::help::show(name),
 ("sync",None,false)=>{match fs::durable::sync(){Ok(())=>text("OK zsynchronizowano z dyskiem\n"),Err(_)=>text("ERROR dysk: zapis nie powiódł się; zmiany pozostają w RAM\n")}},
 ("disk",None,false)=>fs::durable::status(),
 ("shutdown",None,false)=>{if fs::durable::dirty()&&fs::durable::sync().is_err(){text("ERROR shutdown: zapis nie powiódł się; system pozostaje uruchomiony\n");return;}if !crate::power::shutdown(){text("ERROR shutdown: wyłączenie niedostępne na tej platformie\n");}},
 ("reboot",None,false)=>{if fs::durable::dirty()&&fs::durable::sync().is_err(){text("ERROR reboot: zapis nie powiódł się; system pozostaje uruchomiony\n");return;}if !crate::power::reboot(){text("ERROR reboot: restart niedostępny na tej platformie\n");}},
 ("credits",None,false)=>text("Twórcy HAIOS — Human × Artificial Intelligence Operating System\nMateusz (xitriam): pomysł, kierunek projektu, decyzje i testy.\nChatGPT / Codex (OpenAI): narzędzia AI współtworzące architekturę, kod, testy i dokumentację.\nSkładniki zewnętrzne: Limine, Rust, DejaVu; ich odrębne licencje i notices są zachowane.\n"),
 ("version",None,false)=>text("HAIOS 0.3.0-dev console, x86_64/Rust/Limine; eksperymentalny, niewydany\n"),
 ("mem",None,false)=>process::info(),("ps",None,false)=>process::ps(),
 ("run",Some("ipc"),false)=>{if !process::pair(){text("ERROR process capacity or memory: brak zasobów\n");}},
 ("run",Some(name),_)=>{let args=rest[name.len()..].trim_start();if process::spawn_with(name,args).is_none(){text("ERROR unknown program, capacity or memory: program lub zasoby\n");}},
 ("kill",Some(pid),false)=>{if pid.parse::<u64>().ok().is_none_or(|p|!process::kill(p)){text("ERROR unknown pid\n");}},
 ("echo"|"calc"|"wc"|"uptime",_,_)=>{let normalized=if cmd=="wc"{match path(rest){Ok(p)=>Some(p),Err(e)=>{error(e);return;}}}else{None};let args=normalized.as_ref().map_or(rest,|p|p.as_str());if process::spawn_with(cmd,args).is_none(){text("ERROR program: za długie argumenty lub brak zasobów\n");}},
 ("pwd",None,false)=>{text(cwd());text("\n");},("clear",None,false)=>text("\x1b[2J\x1b[H"),
 ("stat",None,false)=>{let (nodes,bytes)=fs::files().usage();text("RAMFS nodes=");serial::number(nodes as u64);text("/32 bytes=");serial::number(bytes as u64);text("\n");},
 ("write"|"append",Some(p),_)=>{match path(p){Ok(p)=>{let content=rest[arg.unwrap().len()..].trim_start();
 if let Err(e)=fs::files().write(p.as_str(),content.as_bytes(),cmd=="append"){error(e);}else{fs::durable::changed();text("OK zapisano w RAM; sync utrwala na dysku\n");}},Err(e)=>error(e)}},
 ("ls",a,false)=>{if a.is_none(){text("RAM: hello count sender receiver fault writefault badptr spin fpu about\n");}match path(a.unwrap_or(".")){Ok(p)=>{if let Err(e)=fs::files().list(p.as_str(),|name,dir,len,ro|{text(name);if dir{text("/");}text(" ");serial::number(len as u64);if ro{text(" [ro]");}text("\n");}){error(e);}},Err(e)=>error(e)}},
 ("cat",Some("about"),false)=>text("HAIOS 0.3: własne programy, pliki w RAM i dedykowanym dysku VM. Bez sieci i AI.\n"),
 ("cat"|"cd"|"mkdir"|"touch"|"rm"|"edit",Some(p),false)=>{match path(p){Ok(p)=>{let result=match cmd{
 "cd"=>fs::files().is_dir(p.as_str()).map(|_|unsafe{CWD=Some(p)}),"mkdir"=>fs::files().mkdir(p.as_str()),"touch"=>fs::files().write(p.as_str(),b"",true),"rm"=>fs::files().remove(p.as_str()),
 "cat"=>fs::files().read(p.as_str()).map(|data|{if let Ok(s)=core::str::from_utf8(data){text(s);text("\n");}else{text("ERROR plik binarny\n");}}),
 _=>{let valid=fs::files().read(p.as_str());if matches!(valid,Err(ram::Error::IsDirectory)){Err(ram::Error::IsDirectory)}else{let e=edit();e.len=0;if let Ok(data)=valid{if core::str::from_utf8(data).is_err(){text("ERROR plik binarny\n");return;}e.bytes[..data.len()].copy_from_slice(data);e.len=data.len();}e.active=true;e.path=Some(p);text("Nowa treść lub edycja (4096 bajtów); .show .clear .line N TEKST .save .cancel\n");if e.len>0{serial::write(&e.bytes[..e.len]);text("\n");}Ok(())}}
 };if let Err(e)=result{error(e);}else if matches!(cmd,"mkdir"|"touch"|"rm"){fs::durable::changed();}},Err(e)=>error(e)}},
 _=>text("ERROR command: błędne polecenie; wpisz help\n")}}
fn edit_line(e:&mut Edit,rest:&str)->bool{let Some((number,replacement))=rest.split_once(' ')else{return false;};let Ok(number)=number.parse::<usize>()else{return false;};if number==0{return false;}let mut start=0;let mut current=1;while current<number{let Some(i)=e.bytes[start..e.len].iter().position(|&b|b==b'\n')else{return false;};start+=i+1;current+=1;}if start>=e.len{return false;}let end=e.bytes[start..e.len].iter().position(|&b|b==b'\n').map_or(e.len,|i|start+i);let old=end-start;let new=replacement.len();let Some(length)=e.len.checked_sub(old).and_then(|n|n.checked_add(new)).filter(|&n|n<=4096)else{return false;};e.bytes.copy_within(end..e.len,start+new);e.bytes[start..start+new].copy_from_slice(replacement.as_bytes());e.len=length;true}
fn key(k:Key){let action=input().key(k);match action{Action::Changed=>redraw(),Action::Complete=>complete(),Action::TooLong=>{text("\nERROR line too long: limit 96 znaków\n");prompt();},Action::Submit(line)=>{let mut bytes=[0;384];let n=line.utf8(&mut bytes);let s=core::str::from_utf8(&bytes[..n]).unwrap();text("\n");if edit().active{if s==".cancel"{edit().active=false;text("Anulowano\n");}else if s==".save"{let result={let e=edit();fs::files().write(e.path.as_ref().unwrap().as_str(),&e.bytes[..e.len],false)};match result{Ok(())=>{edit().active=false;fs::durable::changed();text("OK zapisano w RAM; sync utrwala na dysku\n");},Err(e)=>error(e)}}else if s==".show"{let e=edit();serial::write(&e.bytes[..e.len]);text("\n");}else if s==".clear"{edit().len=0;}else if let Some(rest)=s.strip_prefix(".line "){let e=edit();let result=edit_line(e,rest);if !result{text("ERROR .line: numer istniejącego wiersza i tekst; limit 4096 bajtów\n");}}else{let e=edit();let separator=usize::from(e.len>0&&e.bytes[e.len-1]!=b'\n');if e.len+separator+n+1<=4096{if separator!=0{e.bytes[e.len]=b'\n';e.len+=1;}e.bytes[e.len..e.len+n].copy_from_slice(&bytes[..n]);e.len+=n;e.bytes[e.len]=b'\n';e.len+=1;}else{text("ERROR limit edytora: 4096 bajtów\n");}}}else{command(s);}prompt();},Action::None=>{}}}
pub fn poll(){for _ in 0..32{let Some(b)=serial::read()else{break;};let k=unsafe{(&mut *core::ptr::addr_of_mut!(DECODER)).byte(b)};if let Some(k)=k{key(k);}}crate::keyboard::poll(key);crate::framebuffer::flush();}

/// Restore the framebuffer prompt after asynchronous process output. COM1 retains diagnostics.
pub fn prepare_output(){crate::framebuffer::write(b"\r\x1b[2K");}
pub fn restore_display(){let mut bytes=[0;384];let (n,left)={let e=input();(e.line.utf8(&mut bytes),e.line.len-e.cursor)};crate::framebuffer::write(b"\r\x1b[2K");crate::framebuffer::write(if edit().active{b"edit> "}else{b"haios> "});crate::framebuffer::write(&bytes[..n]);for _ in 0..left{crate::framebuffer::write(b"\x1b[D");}}
