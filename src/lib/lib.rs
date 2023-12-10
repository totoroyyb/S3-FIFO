// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }


static DEFAULT_RINGBUF_SIZE: usize = 100;

struct RingBuffer<T> {
    buffer: Box<[T]>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> RingBuffer<T> 
    where T: Default + Clone 
    {
        assert!(capacity != 0);

        RingBuffer { 
            buffer: vec![T::default(); capacity].into_boxed_slice(), 
            capacity, 
            head: 0, 
            tail: 0, 
            size: 0
        }
    }

    fn push_front(&mut self, value: T) {
        self.head = self.index_backword(self.head);
        self.buffer[self.head] = value;

        if self.size == self.capacity {
            // TODO: overwrite happens (special handling)
            // TAIL will be overwritten.
            // No increment on size
            self.tail = self.index_backword(self.tail);
        } else {
            self.size += 1;
        }
    }

    fn push_back(&mut self, value: T) {
        self.buffer[self.tail] = value;
        self.tail = self.index_forward(self.tail);

        if self.size == self.capacity {
            // TODO: overwrite happens (special handling)
            // HEAD is overwritten.
            // No increment on size.
            self.head = self.index_forward(self.head);
        } else {
            self.size += 1;
        }
    }

    fn pop_front(&mut self) -> Option<T>
    where T: Clone
    {
        if self.size == 0 {
            None
        } else {
            let old_head = self.head;
            self.head = self.index_forward(self.head);
            self.size -= 1;
            Some(self.buffer[old_head].clone())
        }
    }

    fn pop_back(&mut self) -> Option<T>
    where T: Clone
    {
        if self.size == 0 {
            None
        } else {
            self.tail = self.index_backword(self.tail);
            self.size -= 1;
            Some(self.buffer[self.tail].clone())
        }
    }

}

///
/// Some helper functions
/// 
impl<T> RingBuffer<T> {
    fn get_index(&self, mut orig_index: usize, offset: isize) -> usize {
        orig_index %= self.capacity;
        let offset = offset.rem_euclid(self.capacity as isize) as usize;
        (orig_index + offset) % self.capacity
    }

    // Get the index next to the `orig_index`, wrapped by the `self.capacity`.
    fn index_forward(&self, orig_index: usize) -> usize {
        self.get_index(orig_index, 1)
    }

    // Get the index before the `orig_index`, wrapped by the `self.capacity`.
    fn index_backword(&self, orig_index: usize) -> usize {
        self.get_index(orig_index, -1)
    }
}

impl<T> Default for RingBuffer<T>
where T: Default + Clone
{
    fn default() -> Self
    {
        RingBuffer::new(DEFAULT_RINGBUF_SIZE)
    } 
}
