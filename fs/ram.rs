//! Bounded, allocation-free RAM filesystem. Pure logic; no devices or user pointers.
pub const NODES:usize=32;pub const CONTENT:usize=4096;pub const PATH:usize=128;
#[derive(Clone,Copy,Debug,PartialEq)]pub enum Error{Invalid,NotFound,NotDirectory,IsDirectory,Exists,Capacity,TooLarge,ReadOnly,NotEmpty}
#[derive(Clone,Copy)]pub struct Name{bytes:[u8;PATH],len:usize}
impl Name{pub fn as_str(&self)->&str{core::str::from_utf8(&self.bytes[..self.len]).unwrap()}}
pub fn normalize(cwd:&str,path:&str)->Result<Name,Error>{
    if path.is_empty()||path.chars().any(|c|c.is_control()||c.is_whitespace()){return Err(Error::Invalid);}
    let mut out=Name{bytes:[0;PATH],len:1};out.bytes[0]=b'/';
    for source in [if path.starts_with('/') {""}else{cwd},path]{for part in source.split('/'){
        if part.is_empty()||part=="."{continue;}if part==".."{if out.len>1{while out.len>1&&out.bytes[out.len-1]!=b'/'{out.len-=1;}if out.len>1{out.len-=1;}}continue;}
        if part.len()>24{return Err(Error::TooLarge);}let sep=usize::from(out.len>1);
        if out.len+sep+part.len()>=PATH{return Err(Error::TooLarge);}if sep==1{out.bytes[out.len]=b'/';out.len+=1;}
        out.bytes[out.len..out.len+part.len()].copy_from_slice(part.as_bytes());out.len+=part.len();
    }}Ok(out)
}
#[derive(Clone,Copy)]struct Node{used:bool,directory:bool,readonly:bool,name:Name,data:[u8;CONTENT],len:usize}
const EMPTY:Node=Node{used:false,directory:false,readonly:false,name:Name{bytes:[0;PATH],len:0},data:[0;CONTENT],len:0};
pub struct Fs{nodes:[Node;NODES]}
impl Fs{
    pub const fn new()->Self{let mut nodes=[EMPTY;NODES];nodes[0].used=true;nodes[0].directory=true;nodes[0].name.bytes[0]=b'/';nodes[0].name.len=1;Self{nodes}}
    fn find(&self,path:&str)->Result<usize,Error>{self.nodes.iter().position(|n|n.used&&n.name.as_str()==path).ok_or(Error::NotFound)}
    pub fn is_dir(&self,path:&str)->Result<(),Error>{let n=&self.nodes[self.find(path)?];if n.directory{Ok(())}else{Err(Error::NotDirectory)}}
    fn parent(path:&str)->&str{match path.rfind('/'){Some(0)=>"/",Some(i)=>&path[..i],None=>""}}
    fn create(&mut self,path:&str,directory:bool)->Result<usize,Error>{
        let name=normalize("/",path)?;if name.as_str()!=path||path=="/"{return Err(Error::Invalid);}if self.find(path).is_ok(){return Err(Error::Exists);}
        let parent=self.find(Self::parent(path))?;if !self.nodes[parent].directory{return Err(Error::NotDirectory);}if self.nodes[parent].readonly{return Err(Error::ReadOnly);}
        let i=self.nodes.iter().position(|n|!n.used).ok_or(Error::Capacity)?;self.nodes[i]=EMPTY;self.nodes[i].used=true;self.nodes[i].directory=directory;self.nodes[i].name=name;Ok(i)
    }
    pub fn mkdir(&mut self,path:&str)->Result<(),Error>{self.create(path,true).map(|_|())}
    pub fn write(&mut self,path:&str,data:&[u8],append:bool)->Result<(),Error>{
        if data.len()>CONTENT{return Err(Error::TooLarge);}let i=match self.find(path){Ok(i)=>i,Err(Error::NotFound)=>self.create(path,false)?,Err(e)=>return Err(e)};
        let n=&mut self.nodes[i];if n.directory{return Err(Error::IsDirectory);}if n.readonly{return Err(Error::ReadOnly);}let start=if append{n.len}else{0};
        let end=start.checked_add(data.len()).filter(|&x|x<=CONTENT).ok_or(Error::TooLarge)?;n.data[start..end].copy_from_slice(data);
        if end<n.len{n.data[end..n.len].fill(0);}n.len=end;Ok(())
    }
    pub fn read(&self,path:&str)->Result<&[u8],Error>{let n=&self.nodes[self.find(path)?];if n.directory{Err(Error::IsDirectory)}else{Ok(&n.data[..n.len])}}
    pub fn remove(&mut self,path:&str)->Result<(),Error>{
        let i=self.find(path)?;if i==0||self.nodes[i].readonly{return Err(Error::ReadOnly);}if self.nodes[i].directory&&self.nodes.iter().any(|n|n.used&&Self::parent(n.name.as_str())==path){return Err(Error::NotEmpty);}self.nodes[i]=EMPTY;Ok(())
    }
    pub fn protect(&mut self,path:&str)->Result<(),Error>{let i=self.find(path)?;self.nodes[i].readonly=true;Ok(())}
    pub fn list(&self,path:&str,mut visit:impl FnMut(&str,bool,usize,bool))->Result<(),Error>{self.is_dir(path)?;for n in &self.nodes{if n.used&&n.name.as_str()!=path&&Self::parent(n.name.as_str())==path{let name=n.name.as_str().rsplit('/').next().unwrap();visit(name,n.directory,n.len,n.readonly);}}Ok(())}
    pub fn durable_capacity(&self)->usize{NODES-self.nodes.iter().filter(|n|n.used&&(n.readonly||n.name.as_str()=="/")).count()}
    pub fn durable_export(&self,out:&mut[u8])->Result<(),Error>{if out.len()!=super::checkpoint::PAYLOAD{return Err(Error::Invalid);}out.fill(0);let mut index=0;
        for n in &self.nodes{if !n.used||n.readonly||n.name.as_str()=="/"{continue;}let b=&mut out[index*super::checkpoint::RECORD..(index+1)*super::checkpoint::RECORD];b[0]=if n.directory{2}else{1};b[1]=n.name.len as u8;b[2..4].copy_from_slice(&(n.len as u16).to_le_bytes());b[4..4+n.name.len].copy_from_slice(&n.name.bytes[..n.name.len]);b[132..132+n.len].copy_from_slice(&n.data[..n.len]);index+=1;}Ok(())
    }
    pub fn durable_import(&mut self,bytes:&[u8])->Result<(),Error>{let keep=self.nodes.iter().filter(|n|n.used&&(n.readonly||n.name.as_str()=="/")).count();super::checkpoint::validate(bytes,NODES-keep).map_err(|_|Error::Invalid)?;
        for n in &mut self.nodes{if n.used&&!n.readonly&&n.name.as_str()!="/"{*n=EMPTY;}}
        // All relationships already validated; fill nodes directly, independent of record order.
        for b in bytes.chunks_exact(super::checkpoint::RECORD){if let Some((name,dir,data))=super::checkpoint::record(b).map_err(|_|Error::Invalid)?{let i=self.nodes.iter().position(|n|!n.used).ok_or(Error::Capacity)?;let n=&mut self.nodes[i];*n=EMPTY;n.used=true;n.directory=dir;n.name=normalize("/",name)?;n.len=data.len();n.data[..data.len()].copy_from_slice(data);}}Ok(())
    }
    pub fn usage(&self)->(usize,usize){(self.nodes.iter().filter(|n|n.used).count(),self.nodes.iter().filter(|n|n.used&&!n.directory).map(|n|n.len).sum())}
}
#[cfg(test)]mod tests{use super::*;
#[test]fn paths(){assert_eq!(normalize("/a/b","../c//./d").unwrap().as_str(),"/a/c/d");assert_eq!(normalize("/","../../").unwrap().as_str(),"/");assert_eq!(normalize("/a","/żółć").unwrap().as_str(),"/żółć");assert!(normalize("/","has space").is_err());assert!(normalize("/","abcdefghijklmnopqrstuvwxyz").is_err());}
#[test]fn storage_and_limits(){let mut f=Fs::new();f.mkdir("/home").unwrap();f.write("/home/a","Zażółć".as_bytes(),false).unwrap();f.write("/home/a",b"!",true).unwrap();assert_eq!(f.read("/home/a").unwrap(),"Zażółć!".as_bytes());f.write("/home/a",b"x",false).unwrap();assert_eq!(f.read("/home/a").unwrap(),b"x");assert_eq!(f.remove("/home"),Err(Error::NotEmpty));f.write("/home/a",&[7;CONTENT],false).unwrap();assert_eq!(f.write("/home/a",b"x",true),Err(Error::TooLarge));assert_eq!(f.read("/home/a").unwrap().len(),CONTENT);f.remove("/home/a").unwrap();f.remove("/home").unwrap();assert_eq!(f.usage(),(1,0));}
#[test]fn readonly_capacity_and_churn(){let mut f=Fs::new();f.mkdir("/bin").unwrap();f.write("/bin/app",b"code",false).unwrap();f.protect("/bin/app").unwrap();f.protect("/bin").unwrap();assert_eq!(f.write("/bin/app",b"bad",false),Err(Error::ReadOnly));assert_eq!(f.write("/bin/new",b"bad",false),Err(Error::ReadOnly));for i in 0..NODES-3{let name=std::format!("/f{i}");f.write(&name,b"data",false).unwrap();}assert_eq!(f.write("/full",b"",false),Err(Error::Capacity));for _ in 0..1000{f.remove("/f0").unwrap();f.write("/f0",b"again",false).unwrap();}assert_eq!(f.usage().0,NODES);assert_eq!(f.remove("/"),Err(Error::ReadOnly));}
}
