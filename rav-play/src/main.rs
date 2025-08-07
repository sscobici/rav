// use rav::{data::Packet, format::FormatContext, io::IoContext};
// use std::io::Result;

// fn main() -> Result<()> {
//     let io_context = IoContext::from_path("D:\\Media\\Torrent\\TV Demo\\LG Tech 4K Demo.ts")?;
//     let format = FormatContext::new(&io_context.data);
//     let mut input_packet = Packet::default();

//     // loop
//     format.next_packet(&mut input_packet);

//     Ok(())
// }

use std::{fmt::Error, fs, io::{self, Cursor}};


pub fn main() -> io::Result<()> {
    let data = fs::read("myfile.mkv")?;
    let buf = data.as_slice();
    let pos: usize = 0;
    let it = buf.iter();

    if buf.len() < 20 {
        return Err(io::Error::other("file is too short"));
    }
    if buf[pos..pos+4] != u32::to_le_bytes(0x1a45dfa3) {
        return Err(io::Error::other("doesn't start with EBML Header"));
    }
    let size = buf[5]; 
    Ok(())
}