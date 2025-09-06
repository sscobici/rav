
use std::sync::Arc;

use crate::data::{IoBuf, IoRef, Packet};
use crate::error::{invalid_input_error, retry_later_error, Result, Error};

pub struct Stream {
    pub id: usize,
    pub codec_params: Vec<u8>,
}

pub trait IoBufRing {
    fn add_iobuf(&mut self, iobuf: IoBuf) -> Result<()>;
    fn remove_iobuf(&mut self) -> Result<IoBuf>;
}

pub trait IoBufSupply {
    fn open_input(&mut self, uri: &str) -> Result<()>;
    fn supply_iobufs(&mut self, len_bytes: usize, parsed_iobufs: &[IoBuf], new_iobufs: &mut [IoBuf]) -> Result<usize>;
}


#[derive(Debug)]
pub struct IoBufSupplierIoUring {
//    src: std::fs::File,
}

impl IoBufSupply for IoBufSupplierIoUring {
    fn open_input(&mut self, uri: &str) -> Result<()> {
//        self.src = std::fs::File::open(uri)?;
        Ok(())
    }

    fn supply_iobufs(&mut self, len_bytes: usize, parsed_iobufs: &[IoBuf], new_iobufs: &mut [IoBuf]) -> Result<usize> {
        // iobuf_ring.remove_iobuf()?;
        // iobuf_ring.add_iobuf(iobuf);
        Ok(0)
    }
}

pub trait MediaIoBufRead {
    fn get_u8(&mut self) -> Result<u8>;
    fn get_ioref(&mut self, ioref: &mut IoRef, len: usize) -> Result<()>;
}

pub trait Demux {
    fn read_packet(&mut self, packet: &mut Packet) -> Result<()>;
}

pub struct FormatContext {
    demuxer: Box<dyn Demux>,
}

impl FormatContext {
    pub fn open_input(uri: &str) -> Self {
        let mut iobuf_supplier = IoBufSupplierIoUring {};
        iobuf_supplier.open_input(uri);
        let stream = MediaSourceStream::new(iobuf_supplier);
        let demuxer = DemuxerMkv { iobuf_reader: stream };
        Self { demuxer: Box::new(demuxer) }
    }

    pub fn read_packet(&mut self, packet: &mut Packet) -> Result<()> {
        self.demuxer.read_packet(packet)
    }
}

pub struct IoBuf2 {
    // buff allocated space
    data: Arc<[u8]>,
    // how much data is actually there   
    len: u64
}

pub struct DemuxerMkv<S: MediaIoBufRead> {
    iobuf_reader: S
}

impl<S: MediaIoBufRead> Demux for DemuxerMkv<S> {
    fn read_packet(&mut self, packet: &mut Packet) -> Result<()> {
        let b = self.iobuf_reader.get_u8()?;
        Ok(())
    }
}

/// The fixed size of the internal ring buffer, preferably to be equal to 2^n
const RING_SIZE: usize = 4;
const RING_MASK: usize = RING_SIZE - 1;
/// A stream that consumes IoBufs from a fixed-size ring buffer.
/// It allows for zero-copy reading of data into Packets.
#[derive(Debug)]
pub struct MediaSourceStream<S: IoBufSupply> {
    iobuf_supplier: S,
    // ring of buffers, 
	ring: [IoBuf; RING_SIZE],
	/// ring mask
    ring_mask: usize,
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

impl<S: IoBufSupply> MediaSourceStream<S> {
    pub fn new(iobuf_supplier: S) -> Self {
        Self {
            iobuf_supplier,
            ring: Default::default(),
            ring_mask: RING_MASK,
            ring_remove_idx: 0,
            ring_add_idx: 0,
            ring_cur_idx: 0,
            ring_cur_pos: 0,
            stream_pos: 0,
            stream_len: 0 }
    }
    
    fn supply_iobufs(&mut self, len_bytes: usize) -> Result<()> {
        // get all parsed iobufs into array, can be 0
        let mut parsed_iobufs: [IoBuf; RING_SIZE] = Default::default();
        let mut count_parsed_iobufs = 0;
        while count_parsed_iobufs < parsed_iobufs.len() {
            match self.remove_iobuf() {
                Ok(iobuf) => {
                    parsed_iobufs[count_parsed_iobufs] = iobuf;
                    count_parsed_iobufs += 1;
                },
                Err(_) => break,
            }
        }

        // if we don't have space in ring - it means that non of the buffers were parsed
        let max_new_iobufs = RING_SIZE - (self.ring_add_idx - self.ring_remove_idx + RING_SIZE) % RING_SIZE;
        if max_new_iobufs == 0 {
            return retry_later_error();  // all existing buffers are still referenced and cannot be released and there is no more space in ring buf
        }

        // supply parsed buffers and ask for new buffers to cover at least len_bytes
        let mut new_iobufs: [IoBuf; RING_SIZE] = Default::default();
        let count_new_iobufs = self.iobuf_supplier.supply_iobufs(
            len_bytes,
            &parsed_iobufs[0..count_parsed_iobufs],
            &mut new_iobufs[0..max_new_iobufs])?;

        let mut new_bytes = 0;
        for iobuf in new_iobufs.iter_mut().take(count_new_iobufs) {
            new_bytes += iobuf.len;
            self.add_iobuf(std::mem::take(iobuf))?;  // TODO check that it doesn't fail - otherwise iobuf will be dealocated from heap.
        }

        if new_bytes < len_bytes {
            return retry_later_error();
        }
        
        Ok(())
    }
}

impl<S: IoBufSupply> IoBufRing for MediaSourceStream<S> {

    /// Adds a new IoBuf to the stream's ring buffer. IoBuf.len should be greater than zero and less or equal than IoBuf.buf.len()
    fn add_iobuf(&mut self, iobuf: IoBuf) -> Result<()> {
        if iobuf.len == 0 || iobuf.len > iobuf.buf.len() {
            return invalid_input_error();
        }

        let next_add_idx = (self.ring_add_idx + 1) & self.ring_mask;
        // The buffer is full if the next add index would be the same as the remove index.
        // This means we can store up to RING_SIZE - 1 items.
        if next_add_idx == self.ring_remove_idx {
            return retry_later_error();
        }

        self.ring[self.ring_add_idx] = iobuf;
        self.ring_add_idx = next_add_idx;

        Ok(())
    }

    /// Removes a parsed IoBuf from the stream's ring buffer. IoBuf should not have Packets referencing it
    fn remove_iobuf(&mut self) -> Result<IoBuf> {
        if self.ring_remove_idx == self.ring_add_idx {
            return retry_later_error();
        }

        if Arc::strong_count(&self.ring[self.ring_remove_idx].buf) == 1 {
            let iobuf = std::mem::take(&mut self.ring[self.ring_remove_idx]);
            self.ring_remove_idx = (self.ring_remove_idx + 1) % RING_SIZE;
            return Ok(iobuf);
        }

        retry_later_error()
    }
}

impl<S: IoBufSupply> MediaIoBufRead for MediaSourceStream<S> {

    /// Reads a single byte, advancing position.
    fn get_u8(&mut self) -> Result<u8> {
        if self.ring_cur_idx == self.ring_add_idx {
            self.supply_iobufs(1)?;  // we need 1 byte
        }
        let result = self.ring[self.ring_cur_idx].buf[self.ring_cur_pos];

        // Advance position
        self.ring_cur_pos += 1;
        self.stream_pos += 1;
        if self.ring_cur_pos == self.ring[self.ring_cur_idx].len {
            self.ring_cur_idx = (self.ring_cur_idx + 1) & self.ring_mask;
            self.ring_cur_pos = 0;
        }

        Ok(result)
    }

    fn get_ioref(&mut self, ioref: &mut IoRef, len: usize) -> Result<()> {
        if len == 0 {
            return invalid_input_error();
        }

        if self.ring_cur_idx == self.ring_add_idx {
            return retry_later_error();
        }

        let cur_buf_remaining = self.ring[self.ring_cur_idx].len - self.ring_cur_pos;

        if cur_buf_remaining >= len {
            // Can serve from current IoBuf without copying - fast track
            ioref.shared_buf = Some(self.ring[self.ring_cur_idx].buf.clone());
            ioref.buf = None;
            ioref.offset = self.ring_cur_pos;
            ioref.len = len;

            // Advance position
            self.ring_cur_pos += len;
            self.stream_pos += len;
            if self.ring_cur_pos == self.ring[self.ring_cur_idx].len {
                self.ring_cur_idx = (self.ring_cur_idx + 1) & self.ring_mask;
                self.ring_cur_pos = 0;
            }

            return Ok(());
        }

        // Check if total available data is enough
        let mut total_available = cur_buf_remaining;
        let mut idx = (self.ring_cur_idx + 1) & self.ring_mask;
        while idx != self.ring_add_idx && total_available < len {
            total_available += self.ring[idx].len;
            idx = (idx + 1) & self.ring_mask;
        }

        if total_available < len {
            // TODO need to read more data
            return retry_later_error();
        }

        // Allocate and copy data from multiple IoBufs
        let mut new_buf  = Vec::with_capacity(len);
        let mut remaining = len;
        let mut cur_idx = self.ring_cur_idx;
        let mut cur_pos = self.ring_cur_pos;

        loop {
            let buf_rem = self.ring[cur_idx].len - cur_pos;
            let to_copy = buf_rem.min(remaining);

            new_buf.extend_from_slice(&self.ring[cur_idx].buf[cur_pos..cur_pos + to_copy]);

            remaining -= to_copy;
            cur_pos += to_copy;
            if cur_pos == self.ring[cur_idx].len {
                cur_idx = (cur_idx + 1) & self.ring_mask;
                cur_pos = 0;
            }            

            if remaining == 0 {
                break;
            }
        }

        // Advance position
        self.ring_cur_idx = cur_idx;
        self.ring_cur_pos = cur_pos;
        self.stream_pos += len;

        // Set IoRef to owned buffer
        ioref.shared_buf = None;
        ioref.buf = Some(new_buf.into_boxed_slice());
        ioref.offset = 0;
        ioref.len = len;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn new_iobuf(data: &[u8]) -> IoBuf {
        IoBuf {
            buf: Arc::from(data),
            len: data.len(),
        }
    }

    #[test]
    fn add_single_buf() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});

        let mut err_buf = new_iobuf(b"123");
        err_buf.len = 0;
        assert_eq!(stream.add_iobuf(err_buf), Err(Error::InvalidInput));

        let mut err_buf = new_iobuf(b"123");
        err_buf.len = 4;
        assert_eq!(stream.add_iobuf(err_buf), Err(Error::InvalidInput));

        let mut err_buf = new_iobuf(b"");
        err_buf.len = 1;
        assert_eq!(stream.add_iobuf(err_buf), Err(Error::InvalidInput));

        let buf = new_iobuf(b"hello");
        assert!(stream.add_iobuf(buf).is_ok());
        assert_eq!(stream.ring_add_idx, 1);
        assert_eq!(stream.ring_remove_idx, 0);
        assert_eq!(stream.ring_cur_idx, 0);
        assert_eq!(stream.ring_cur_pos, 0);
    }

    #[test]
    fn add_three_bufs_and_check_full() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        for i in 0..RING_SIZE - 1 {
            let buf = new_iobuf(&[i as u8]);
            assert!(stream.add_iobuf(buf).is_ok());
        }
        let full_buf = new_iobuf(b"full");
        assert_eq!(stream.add_iobuf(full_buf), Err(Error::RetryLater));
    }

    #[test]
    fn cannot_remove_referenced_buf() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"a")).unwrap();

        {
            let mut ioref = IoRef::default();
            let ioref = stream.get_ioref(&mut ioref, 1);
            assert!(ioref.is_ok());
            let result = stream.remove_iobuf();
            assert!(result.is_err());
            assert_eq!(result.err().unwrap(), Error::RetryLater);
        }
        assert!(stream.remove_iobuf().is_ok());
    }

    #[test]
    fn remove_one_and_add_one() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        // Add max possible bufs to leave one slot empty
        for i in 0..RING_SIZE - 1 {
            let buf = new_iobuf(&[i as u8]);
            assert!(stream.add_iobuf(buf).is_ok());
        }
        // Read 2 bytes to "remove" 2 bufs logically
        for _ in 0..2 {
            assert!(stream.get_u8().is_ok());
        }

        assert_eq!(stream.ring_cur_idx, 2);
        assert!(stream.remove_iobuf().is_ok());

        // Add a new buf, should succeed
        let new_buf = new_iobuf(b"test");
        assert!(stream.add_iobuf(new_buf).is_ok());
        // Add index should have wrapped around to 0
        assert_eq!(stream.ring_add_idx, 0);
    }
    
    // --- read_ioref tests ---
    #[test]
    fn no_data() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        let mut ioref = IoRef::default();

        // Requesting 0 bytes, but no data is available
        assert_eq!(stream.get_ioref(&mut ioref, 0), Err(Error::InvalidInput));

        // Requesting 1 bytes, but no data is available
        assert_eq!(stream.get_ioref(&mut ioref, 1), Err(Error::RetryLater));
    }

    #[test]
    fn not_enough_data_in_current_buf() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"a")).unwrap();
        let mut ioref = IoRef::default();
        // Requesting 2 bytes, but only 1 are available
        assert_eq!(stream.get_ioref(&mut ioref, 2), Err(Error::RetryLater));
    }

    #[test]
    fn not_enough_data_in_all_bufs() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"a")).unwrap();
        stream.add_iobuf(new_iobuf(b"b")).unwrap();
        let mut ioref = IoRef::default();
        // Requesting 3 bytes, but only 2 are available
        assert_eq!(stream.get_ioref(&mut ioref, 3), Err(Error::RetryLater));
    }

    #[test]
    fn data_found_in_single_buf_more_data_remain() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"abcdef")).unwrap();
        let mut ioref = IoRef::default();
        // Read 3 bytes
        assert!(stream.get_ioref(&mut ioref, 3).is_ok());
        assert_eq!(ioref.len, 3);
        assert_eq!(ioref.offset, 0);
        // The shared buf should be the same
        assert!(ioref.shared_buf.is_some());
        assert_eq!(stream.ring_cur_idx, 0);
        assert_eq!(stream.ring_cur_pos, 3);
    }

    #[test]
    fn data_found_in_single_buf_no_more_data_remain() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"abcd")).unwrap();
        let mut ioref = IoRef::default();
        // Read all 4 bytes
        assert!(stream.get_ioref(&mut ioref, 4).is_ok());
        assert_eq!(ioref.len, 4);
        assert!(ioref.shared_buf.is_some());
        // The stream should advance to the next buffer
        assert_eq!(stream.ring_cur_idx, 1);
        assert_eq!(stream.ring_cur_pos, 0);
    }

    #[test]
    fn data_found_in_two_bufs_more_data_remain() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"abc")).unwrap();
        stream.add_iobuf(new_iobuf(b"def")).unwrap();
        let mut ioref = IoRef::default();
        // Read 5 bytes, spanning two bufs
        assert!(stream.get_ioref(&mut ioref, 5).is_ok());
        assert_eq!(ioref.len, 5);
        // This read should result in a copy
        assert!(ioref.buf.is_some());
        assert!(ioref.shared_buf.is_none());
        assert_eq!(&ioref.buf.unwrap()[..], b"abcde");
        // Stream state should be updated correctly
        assert_eq!(stream.ring_cur_idx, 1);
        assert_eq!(stream.ring_cur_pos, 2);
    }

    #[test]
    fn data_found_in_two_bufs_no_more_data_remain() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"abc")).unwrap();
        stream.add_iobuf(new_iobuf(b"de")).unwrap();
        let mut ioref = IoRef::default();
        // Read all 5 bytes
        assert!(stream.get_ioref(&mut ioref, 5).is_ok());
        assert_eq!(ioref.len, 5);
        assert!(ioref.buf.is_some());
        assert_eq!(&ioref.buf.unwrap()[..], b"abcde");
        // Stream should be at the add index, indicating no more data
        assert_eq!(stream.ring_cur_idx, 2);
        assert_eq!(stream.ring_cur_pos, 0);
    }
    
    #[test]
    fn data_found_in_three_bufs_no_more_data_remain() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"a")).unwrap();
        stream.add_iobuf(new_iobuf(b"b")).unwrap();
        stream.add_iobuf(new_iobuf(b"c")).unwrap();
        let mut ioref = IoRef::default();
        // Read all 3 bytes, spanning all three bufs
        assert!(stream.get_ioref(&mut ioref, 3).is_ok());
        assert_eq!(ioref.len, 3);
        assert!(ioref.buf.is_some());
        assert_eq!(&ioref.buf.unwrap()[..], b"abc");
        // Stream should be at the add index, which has wrapped around
        assert_eq!(stream.ring_cur_idx, 3);
        assert_eq!(stream.ring_cur_pos, 0);
    }

    #[test]
    fn read_past_current_and_wrap_around() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        // Add max possible bufs to leave one slot empty
        for i in 0..RING_SIZE - 1 {
            let buf = new_iobuf(&[i as u8]);
            assert!(stream.add_iobuf(buf).is_ok());
        }
    
        // Read first 2 bufs to advance to the next buf
        let mut ioref_initial = IoRef::default();
        assert!(stream.get_ioref(&mut ioref_initial, 2).is_ok());
        assert_eq!(stream.ring_cur_idx, 2);
        assert_eq!(stream.ring_cur_pos, 0);

        // removing two bufs
        assert!(stream.remove_iobuf().is_ok());
        assert!(stream.remove_iobuf().is_ok());

        // add other 2 bufs
        stream.add_iobuf(new_iobuf(b"a")).unwrap();
        stream.add_iobuf(new_iobuf(b"b")).unwrap();
    
        // Read RING_SIZE - 1 bytes which will span from the last buffer and wrap around
        let mut ioref = IoRef::default();
        assert!(stream.get_ioref(&mut ioref, RING_SIZE - 1).is_ok());
        assert_eq!(ioref.len, RING_SIZE - 1);
        assert!(ioref.buf.is_some());
    
        // The stream state should be updated to point to the correct position after the read
        assert_eq!(stream.ring_cur_idx, 1);
        assert_eq!(stream.ring_cur_pos, 0);
    }

    #[test]
    fn get_u8_no_data() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        assert_eq!(stream.get_u8(), Err(Error::RetryLater));
    }

    #[test]
    fn get_u8_test() {
        let mut stream = MediaSourceStream::new(IoBufSupplierIoUring {});
        stream.add_iobuf(new_iobuf(b"abc")).unwrap();
        stream.add_iobuf(new_iobuf(b"de")).unwrap();
        
        // Read first byte
        assert_eq!(stream.get_u8().unwrap(), b'a');
        assert_eq!(stream.ring_cur_idx, 0);
        assert_eq!(stream.ring_cur_pos, 1);
        
        // Read last byte of first buffer
        assert_eq!(stream.get_u8().unwrap(), b'b');
        assert_eq!(stream.get_u8().unwrap(), b'c');
        assert_eq!(stream.ring_cur_idx, 1);
        assert_eq!(stream.ring_cur_pos, 0);
        
        // Read first byte of second buffer
        assert_eq!(stream.get_u8().unwrap(), b'd');
        assert_eq!(stream.ring_cur_idx, 1);
        assert_eq!(stream.ring_cur_pos, 1);
    }
}

