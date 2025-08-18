use std::sync::Arc;

use bytes::{Buf, Bytes};

use crate::data::{IoBuf, MediaError, Packet, Packet2};
use crate::io::IoContext;

pub struct Stream {
    pub id: usize,
    pub codec_params: Vec<u8>,
}

pub struct FormatContext<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> FormatContext<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let pos: usize = 0;
        if data[pos..pos+4] == b"EBML"[..] {
 
        }
        Self { data, pos: 0 }
    }

    pub fn next_packet(&self, packet: &mut Packet<'a>) {
        packet.clear();
        packet.push(&self.data[..100]);
    }
}


type Error = Box<dyn std::error::Error>;
pub struct IoBuf2 {
    // buff allocated space
    data: Arc<[u8]>,
    // how much data is actually there   
    len: u64
}

pub struct Demuxer {
    /// The ring buffer of IoBufs which are supplied to demuxer
    ring: [IoBuf2; 4],
    /// The ring buffer's wrap-around mask.
    ring_mask: usize,
    /// The read index in ring.
    read_idx: usize,
    /// The write index in ring
    write_idx: usize,
    /// Absolute position of the stream.
    abs_pos: u64,
    /// index in the ring where parsing currently is
    cur_idx: usize,
    /// 
    cur_pos: u64,
}

impl Demuxer {
//     pub fn read_packet(&self, packet: &mut Packet2) -> Result<(), Error> {
//         packet.bufs[0] = self.bufs[0].clone();
//         packet.offset = 0;
//         packet.len = 5;
//         Ok(())
//     }
}

/// A stream that consumes IoBufs from a fixed-size ring buffer.
/// It allows for zero-copy reading of data into Packets.
#[derive(Debug, Default)]
pub struct MediaSourceStream {
	ring: [IoBuf; Self::RING_SIZE],
	/// The index where the IoBuf will be removed once the buffer is read.
	ring_remove_idx: usize,
	/// The index where the next IoBuf will be added.
	ring_add_idx: usize,
	/// The index of the IoBuf currently being read from.
	ring_cur_idx: usize,
	/// The position in the current IoBuf to read from.
	ring_cur_pos: usize,
	/// absolute stream position
	stream_pos: usize,
	/// total stream length
	stream_len: usize,
}

impl MediaSourceStream {
    /// The fixed size of the internal ring buffer.
    const RING_SIZE: usize = 4;

    /// Adds a new IoBuf to the stream's ring buffer.
    pub fn add_iobuf(&mut self, buf: IoBuf) -> Result<(), MediaError> {
        // TODO check buf

        let next_add_idx = (self.ring_add_idx + 1) % Self::RING_SIZE;
        // The buffer is full if the next add index would be the same as the remove index.
        // This means we can store up to RING_SIZE items.
        if next_add_idx == self.ring_remove_idx {
            return Err(MediaError::RingBufferFull);
        }

        self.ring[self.ring_add_idx] = buf;
        self.ring_add_idx = next_add_idx;

        Ok(())
    }

    /// Reads a single byte, advancing position.
    pub fn get_u8(&mut self) -> Result<u8, MediaError> {
        if self.ring_cur_idx == self.ring_add_idx {
            return Err(MediaError::NotEnoughData);
        }
        let result = self.ring[self.ring_cur_idx].buf[self.ring_cur_pos];

        // advance
        self.ring_cur_pos += 1;
        self.stream_pos += 1;

        // switch to next if reached the end of current IoBuf
        if self.ring_cur_pos == self.ring[self.ring_cur_idx].len {
            self.ring_cur_idx = (self.ring_cur_idx + 1) % Self::RING_SIZE;
            self.ring_cur_pos = 0;
        }
        
        Ok(result)
    }
}