use std::ffi::c_void;

/// Represents a compressed data packet. C-friendly layout.
#[repr(C)]
#[derive(Debug, Default)]
pub struct Packet<'a> {     
    // slices to actual data, can be many, probably needs to be converted to fixes size array if we don't expect packet data to span too many slices
    slices: Vec<&'a [u8]>,             // 24 bytes + 4*16 bytes in heap for slices or more

    // cache for the merged memory
    merged: Option<Box<[u8]>>,       // 16 bytes
}

impl<'a> Packet<'a> {
    pub fn clear(&mut self) {
        self.slices.clear();
        self.merged = None;
    }

    pub fn push(&mut self, data: &'a[u8]) {
        self.slices.push(data);
    }

    // extract data slice, if there are more than 1 slices, merge them in memory and return a single slice to that memory,
    // cache the merged memory in the merged field.
    fn data(&self) -> &[u8] {
        match self.slices.len() {
            0 => &[],
            _ => self.slices[0],
        }
    }
}

/// Represents a decoded, raw media frame, which always resides in hardware.
#[repr(C)]
#[derive(Debug)]
pub struct Frame {
    /// Opaque handle to the hardware resource (e.g., a GPU texture).
    pub handle: *mut c_void,
    pub width: u32,
    pub height: u32,
    // ... other metadata like pixel format
}