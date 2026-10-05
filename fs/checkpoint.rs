//! Pure wire-format validation. No struct transmute, pointer or device access.
pub const RECORD:usize=4608;pub const RECORDS:usize=32;pub const PAYLOAD:usize=RECORD*RECORDS;
pub const SECTORS:u64=131072;
pub fn slot_lba(slot:usize)->u64{8+slot as u64*512}
pub fn crc(bytes:&[u8])->u32{let mut value=!0u32;for &byte in bytes{value^=byte as u32;for _ in 0..8{value=(value>>1)^if value&1!=0{0xedb88320}else{0};}}!value}
fn u32_at(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
pub fn superblock(b:&[u8;512])->bool{&b[..8]==b"HAIOSD03"&&u32_at(b,8)==1&&u32_at(b,12)==512&&u64::from_le_bytes(b[16..24].try_into().unwrap())==SECTORS&&u32_at(b,24)==RECORD as u32&&u32_at(b,28)==32&&u32_at(b,32)==crc(&b[..32])&&b[36..].iter().all(|&x|x==0)}
pub fn header(generation:u64,payload:&[u8])->[u8;512]{let mut b=[0u8;512];b[..8].copy_from_slice(b"HAIOSC03");b[8..16].copy_from_slice(&generation.to_le_bytes());b[16..20].copy_from_slice(&crc(payload).to_le_bytes());b[20..24].copy_from_slice(&(PAYLOAD as u32).to_le_bytes());let checksum=crc(&b[..24]);b[24..28].copy_from_slice(&checksum.to_le_bytes());b}
pub fn generation(b:&[u8;512])->Option<u64>{if &b[..8]!=b"HAIOSC03"||u32_at(b,20)!=PAYLOAD as u32||u32_at(b,24)!=crc(&b[..24])||b[28..].iter().any(|&x|x!=0){return None;}let sequence=u64::from_le_bytes(b[8..16].try_into().unwrap());(sequence!=0).then_some(sequence)}
pub fn payload_matches(header:&[u8;512],bytes:&[u8])->bool{bytes.len()==PAYLOAD&&crc(bytes)==u32_at(header,16)}
pub fn record(bytes:&[u8])->Result<Option<(&str,bool,&[u8])>,()>{
 if bytes.len()!=RECORD{return Err(());}if bytes[0]==0{return if bytes.iter().all(|&x|x==0){Ok(None)}else{Err(())};}
 if bytes[0]!=1&&bytes[0]!=2{return Err(());}let n=bytes[1]as usize;let len=u16::from_le_bytes([bytes[2],bytes[3]])as usize;
 if n==0||n>=128||len>4096||(bytes[0]==2&&len!=0){return Err(());}
 let name=core::str::from_utf8(&bytes[4..4+n]).map_err(|_|())?;let canonical=super::ram::normalize("/",name).map_err(|_|())?;
 if name=="/"||name=="/bin"||name.starts_with("/bin/")||canonical.as_str()!=name{return Err(());}
 if bytes[4+n..132].iter().any(|&x|x!=0)||bytes[132+len..].iter().any(|&x|x!=0){return Err(());}
 let data=&bytes[132..132+len];core::str::from_utf8(data).map_err(|_|())?;Ok(Some((name,bytes[0]==2,data)))
}
pub fn validate(bytes:&[u8],capacity:usize)->Result<(),()>{if bytes.len()!=PAYLOAD{return Err(());}let mut count=0;
 for (i,raw)in bytes.chunks_exact(RECORD).enumerate(){if let Some((name,_,_))=record(raw)?{count+=1;if count>capacity{return Err(());}
  let parent=match name.rfind('/'){Some(0)=>"/",Some(j)=>&name[..j],None=>return Err(())};let mut parent_found=parent=="/";
  for(j,other)in bytes.chunks_exact(RECORD).enumerate(){if let Some((other_name,dir,_))=record(other)?{if i!=j&&other_name==name{return Err(());}if other_name==parent&&dir{parent_found=true;}}}
  if !parent_found{return Err(());}
 }}Ok(())}
#[cfg(test)]mod tests{use super::*;
 fn entries()->Vec<u8>{let mut b=vec![0;PAYLOAD];b[0]=2;b[1]=5;b[4..9].copy_from_slice(b"/home");let i=RECORD;b[i]=1;b[i+1]=7;b[i+2]=1;b[i+4..i+11].copy_from_slice(b"/home/a");b[i+132]=b'x';b}
 #[test]fn checksums_structure_and_mutations(){assert_eq!(crc(b"123456789"),0xcbf43926);let b=entries();assert_eq!(validate(&b,2),Ok(()));assert_eq!(validate(&b,1),Err(()));let h=header(1,&b);assert_eq!(generation(&h),Some(1));assert!(payload_matches(&h,&b));
  for i in (0..b.len()).step_by(512){let mut broken=b.clone();broken[i]^=1;assert!(!payload_matches(&h,&broken));}
  for i in 0..512{let mut broken=h;broken[i]^=1;assert_eq!(generation(&broken),None);}
 }
 #[test]fn reject_bad_paths_lengths_parent_and_duplicate(){for offset in [0,1,2,3,4,132,RECORD+1,RECORD+3,RECORD+4,RECORD+133]{let mut b=entries();b[offset]=255;assert!(validate(&b,32).is_err(),"invalid byte at {offset}");}
  let mut b=entries();let file=b[RECORD..RECORD*2].to_vec();b[RECORD*2..RECORD*3].copy_from_slice(&file);assert!(validate(&b,32).is_err());
  let mut b=entries();b[..RECORD].fill(0);assert!(validate(&b,32).is_err());
 }
}
