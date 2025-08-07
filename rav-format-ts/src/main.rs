use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use log::{error, info, warn};

// Dummy structures to represent different components
struct IOBuffer;

struct Packet;

struct DecodedFrame;


pub fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Channels for inter-thread communication
    let (iobuf_tx, iobuf_rx) = mpsc::channel::<IOBuffer>();
    let (video_tx, video_rx) = mpsc::channel::<Packet>();
    let (audio_tx, audio_rx) = mpsc::channel::<Packet>();

    info!("Creating threads");
    // IO thread
    let io_handle = {
        thread::spawn(move || {
            for iobuf in iobuf_rx {
                // Simulate reading data into iobuf
                info!("IOThread: New empty IOBuffer available, fill it syncronously");
                thread::sleep(Duration::from_millis(1000));
                // demuxer.add(iobuf)?;
                info!("IOThread: IOBuffer was filled, add it to demuxer");
            }
        })
    };

    // Demuxer thread
    let demux_handle = {
        thread::spawn(move || {
            let mut i = 0;
            while true {
                // Simulate demuxing
                info!("DemuxerThread: demuxing next packet");
                thread::sleep(Duration::from_millis(700));
                info!("DemuxerThread: sending packet for decoding");
                // demux next packet - will block if no IOBuffer data is awailable
                if i % 2 == 0 {
                    video_tx.send(Packet);
                }
                else {
                    audio_tx.send(Packet);
                }
                i = i + 1;
            }
        })
    };

    // Video Decoding thread
    let video_handle = {
        thread::spawn(move || {
            for packet in video_rx {
                // Simulate decoding
                info!("VideoDecodingThread: New packet, decode it");
                thread::sleep(Duration::from_millis(1000));
                // demuxer.add(iobuf)?;
                info!("VideoDecodingThread: packet was decoded");
            }
        })
    };

    // Audio Decoding thread
    let audio_handle = {
        thread::spawn(move || {
            for packet in audio_rx {
                // Simulate decoding
                info!("AudioDecodingThread: New packet, decode it");
                thread::sleep(Duration::from_millis(300));
                // demuxer.add(iobuf)?;
                info!("AudioDecodingThread: packet was decoded");
            }
        })
    };

    io_handle.join().unwrap();


    // 1KB buffer filled with zeroes
//    let buffer = [0u8; 1024*1024];

    // Use it as input to your demuxer
  //  process_demux(&buffer);
}

// fn process_demux(buffer: &[u8]) {
//     // Example: Read MPEG-TS packets (188 bytes)
//     let packet_size = 188;
//     for chunk in buffer.chunks_exact(packet_size) {
//         if chunk.len() == packet_size {
//             demux_packet(chunk);
//         }
//     }
// }

// fn demux_packet(packet: &[u8]) {
//     // Interpret the packet — no copy involved
//     let pid = ((packet[1] & 0x1F) as u16) << 8 | packet[2] as u16;
//     println!("PID: {:#X}", pid);
// }
