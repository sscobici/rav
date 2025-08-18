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

use std::{fmt::Error, fs, io::{self, Cursor, Read}};

#[derive(Debug)]
pub struct Packet<'a> {
    pub data: &'a [u8],
}

pub fn main() {
    // Get the first command line argument.
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("file path not provided");

    // Open the media source.
    let mut src = std::fs::File::open(path).expect("failed to open media");

    const BUFFER_COUNT: usize = 10;
    const BUFFER_SIZE: usize = 1_048_576; // 1MB
    let mut buf_pool: Vec<Vec<u8>> = (0..BUFFER_COUNT)
        .map(|_| Vec::with_capacity(BUFFER_SIZE))
        .collect();

    for _ in 0..2 {
        if let Some(mut buffer) = buf_pool.pop() {
            src.read_exact(&mut buffer).unwrap();
            //demuxer.add_buffer(id, buffer); // Ownership transferred to demuxer
        }
    }
}