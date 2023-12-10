use std::fmt::{self, Debug};

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

    fn len(&self) -> usize {
        self.size
    }

}

impl<T> RingBuffer<T>
where T: Clone
{
    fn pop_front(&mut self) -> Option<T>
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
    {
        if self.size == 0 {
            None
        } else {
            self.tail = self.index_backword(self.tail);
            self.size -= 1;
            Some(self.buffer[self.tail].clone())
        }
    }

    fn peak_front(&self) -> Option<T>
    where T: Clone 
    {
        if self.size == 0 {
            None
        } else {
            Some(self.buffer[self.head].clone())
        }
    }

    fn peak_back(&self) -> Option<T>
    where T: Clone
    {
        if self.size == 0 {
            None
        } else {
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

    fn get_values(&self) -> Vec<T>
    where T: Clone 
    {
        let mut values = Vec::<T>::new();
        let mut i = self.head;

        // Avoid the case where the buffer is full and head == tail in this case.
        if self.size == 0 {
            return values;
        }

        values.push(self.buffer[i].clone());
        i = self.index_forward(i);
        while i != self.tail {
            values.push(self.buffer[i].clone());
            i = self.index_forward(i);
        }
        values
    }
}

impl<T> fmt::Debug for RingBuffer<T>
where T: Clone + Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let organized_arr = self.get_values();
        // TODO: implement a custom iterator

        f.debug_struct("RingBuffer")
            .field("elements", &organized_arr)
            .field("capacity", &self.capacity)
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("size", &self.size).finish()
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

#[cfg(test)]
mod ring_buffer_tests {
    use super::*;

    #[test]
    fn init() {
        let rb = RingBuffer::<usize>::new(5);
        assert_eq!(rb.capacity, 5);
        assert_eq!(rb.head, 0);
        assert_eq!(rb.tail, 0);
        assert_eq!(rb.size, 0);
    }

    #[test]
    fn default_init() {
        let rb = RingBuffer::<usize>::default();
        assert_eq!(rb.capacity, DEFAULT_RINGBUF_SIZE);
    }

    #[test]
    fn push_back() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE / 2) {
            rb.push_back(i);
            expected.push(i);
        }
        let result = rb.get_values();
        assert_eq!(result, expected);
    }

    #[test]
    fn push_front() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE / 2) {
            rb.push_front(i);
            expected.push(i);
        }
        expected.reverse();
        let result = rb.get_values();
        assert_eq!(result, expected);
    }

    #[test]
    fn push_pop_back() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE / 2) {
            rb.push_back(i);
            expected.push(i);
        }
        for _ in 0..(DEFAULT_RINGBUF_SIZE / 4) {
            rb.pop_back();
            expected.pop();
        }

        let result = rb.get_values();
        assert_eq!(result, expected);
    }

    #[test]
    fn empty_pop() {
        let mut rb = RingBuffer::<usize>::default();
        let val = rb.pop_front();
        assert_eq!(val, None);
        let val = rb.pop_back();
        assert_eq!(val, None);
    }

    #[test]
    fn over_pop() {
        let mut rb = RingBuffer::<usize>::default();
        rb.push_back(0);
        rb.push_back(1);

        let val = rb.pop_back();
        assert_eq!(val, Some(1));
        let val = rb.pop_back();
        assert_eq!(val, Some(0));

        let should_be_none = rb.pop_back();
        assert_eq!(should_be_none, None);
        let should_be_none = rb.pop_front();
        assert_eq!(should_be_none, None);

        rb.push_back(10);
        let final_val = rb.pop_back();
        assert_eq!(final_val, Some(10));
    }

    #[test]
    fn over_push_back() {
        let mut rb = RingBuffer::<usize>::new(3);
        rb.push_back(0);
        rb.push_back(1);
        rb.push_back(2);

        rb.push_back(3);
        rb.push_back(4);
        rb.push_back(5);
        let result = rb.get_values();
        assert_eq!(rb.capacity, 3);
        assert_eq!(rb.size, 3);

        assert_eq!(rb.head, 0);
        assert_eq!(rb.tail, 0);
        assert_eq!(result, vec![3, 4, 5]);
    }

    #[test]
    fn head_overwritten() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE * 2) {
            rb.push_back(i);
            if i >= DEFAULT_RINGBUF_SIZE {
                expected.push(i);
            }
        }

        let result = rb.get_values();
        assert_eq!(result, expected);
    }

    #[test]
    fn stack_sim() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE / 2) {
            rb.push_back(i);
            expected.push(i);
        }

        for i in 0..(DEFAULT_RINGBUF_SIZE / 4) {
            let val = rb.pop_front();
            assert_eq!(val, Some(i));
            assert_eq!(rb.get_values(), expected[i+1..]);
        }
    }

    #[test]
    fn overflow_stack_sim() {
        let mut rb = RingBuffer::<usize>::default();
        let mut expected = Vec::<usize>::new();
        for i in 0..(DEFAULT_RINGBUF_SIZE * 2) {
            rb.push_back(i);
            expected.push(i);
        }

        let expected = &expected[DEFAULT_RINGBUF_SIZE..];

        for i in 0..DEFAULT_RINGBUF_SIZE {
            let val = rb.pop_front();
            assert_eq!(val, Some(DEFAULT_RINGBUF_SIZE + i));
            assert_eq!(rb.get_values(), expected[i+1..]);
        }
    }
}
