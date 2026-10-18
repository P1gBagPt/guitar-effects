use rtrb::{Consumer, Producer, RingBuffer};

pub fn create_audio_ringbuffer<T: Copy + Default>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    RingBuffer::new(capacity)
}
