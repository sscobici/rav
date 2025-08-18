use std::{ffi::c_void, sync::Arc};

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
// --- Error Type ---
/// Defines errors that can occur while operating the MediaSourceStream.
#[derive(Debug, PartialEq)]
pub enum MediaError {
    /// Not enough data in the stream to complete the operation.
    NotEnoughData,
    /// The ring buffer is full and cannot accept a new IoBuf.
    RingBufferFull,
    /// The requested slice is too large to fit into a single Packet (spans more than 4 IoBufs).
    PacketTooLarge,
}

// --- Data Structures ---

/// A reference to a segment of a shared buffer.
/// This is the core of the zero-copy mechanism, as it allows passing
/// around references to data without copying the data itself.
#[derive(Debug, Clone, Default)]
pub struct IoRef {
    /// A shared, immutable reference to the underlying byte buffer.
    buf: Option<Arc<[u8]>>,
    /// The starting position of this reference within the buffer.
    offset: usize,
    /// The length of the data segment this reference points to.
    len: usize,
}

/// A packet of data that can be composed of up to 4 non-contiguous buffer segments.
/// This allows a single logical data packet to be read even if it spans multiple
/// IoBufs in the stream's ring buffer.
#[derive(Debug, Default)]
pub struct Packet2 {
    /// An array of buffer references that constitute the packet's data.
    pub bufs: [IoRef; 4],
}

impl Packet2 {
    /// Resets the packet to its default, empty state.
    pub fn clear(&mut self) {
        // Replace each IoRef with a default, effectively dropping any Arcs.
        self.bufs = Default::default();
    }

    /// Returns the total length of the data contained in the packet.
    pub fn len(&self) -> usize {
        self.bufs.iter().map(|b| b.len).sum()
    }

    /// Returns the number of valid IoRefs in the packet.
    pub fn bufs_len(&self) -> usize {
        self.bufs.iter().filter(|ioref| ioref.buf.is_some()).count()
    }
}

/// A contiguous block of memory, owned and shared via an Arc.
#[derive(Debug, Default)]
pub struct IoBuf {
    /// The shared buffer. `None` if the IoBuf is empty/uninitialized.
    pub(crate) buf: Arc<[u8]>,
    /// The length of the actual content in the buffer.
    pub(crate) len: usize,
}

impl IoBuf {
    /// Creates a new IoBuf from a Vec<u8>.
    pub fn from_vec(data: Vec<u8>) -> Self {
        let len = data.len();
        IoBuf {
            buf: Arc::from(data),
            len,
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