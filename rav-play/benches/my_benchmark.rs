use std::{hint::black_box, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use rav::data::{IoBuf, IoRef};
use rav::format::{IoBufRing, IoBufSupplierIoUring, MediaIoBufRead, MediaSourceStream};

fn new_iobuf(data: &[u8]) -> IoBuf {
    IoBuf {
        buf: Arc::from(data),
        len: data.len(),
    }
}

fn read_ioref_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("read_ioref");
    g.bench_function("read_ioref", |b| {
        b.iter_with_setup(|| {
            let mut stream = MediaSourceStream::new(IoBufSupplierIoUring{});
            let iobuf = new_iobuf(b"abcdef");
            stream.add_iobuf(iobuf).unwrap();
            let ioref = IoRef::default();
            (stream, ioref)
        }, |(mut stream, mut ioref)| {
            stream.get_ioref(black_box(&mut ioref), 3);
        });
    });

    g.bench_function("read_ioref_alloc", |b| {
        b.iter_with_setup(|| {
            let mut stream = MediaSourceStream::new(IoBufSupplierIoUring{});
            stream.add_iobuf(new_iobuf(b"abcdefg")).unwrap();
            stream.add_iobuf(new_iobuf(b"hijklmnop")).unwrap();
            let ioref = IoRef::default();
            (stream, ioref)
        }, |(mut stream, mut ioref)| {
            stream.get_ioref(black_box(&mut ioref), 8);
        });
    });
}

criterion_group!(benches, read_ioref_benchmark);
criterion_main!(benches);