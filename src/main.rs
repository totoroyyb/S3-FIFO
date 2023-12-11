use s3fifo::prelude::*;

fn main() {
    println!("Hello, world!");
    let mut rb: RingBuffer<isize> = RingBuffer::new(10);
    for i in -5..5 {
        rb.push_back(i);
    }
    println!("RingBuffer: {rb:?}");
}
