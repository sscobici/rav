
use std::io::Result;

use bytes::{Buf, Bytes};

use crate::data::Packet;
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


