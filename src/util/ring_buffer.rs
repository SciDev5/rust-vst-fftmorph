use std::ops;

pub struct RingBuffer<T> {
    data: Vec<T>,
    zero_offset: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(len: usize, initial: T) -> Self {
        Self {
            data: vec![initial; len],
            zero_offset: 0,
        }
    }
    
    /// Clone the contents of the slice into the ring buffer,
    /// shifting forward, overwriting the old content.
    pub fn push_clone_from_slice(&mut self, slice: &[T]) {
        assert!(slice.len() <= self.len(), "RingBuffer: Attemped to push chunk that's longer than target buffer.");
        let (lower, upper) = self.slice_raw_mut(0, slice.len() as isize);

        lower.clone_from_slice(&slice[..lower.len()]);
        upper.clone_from_slice(&slice[lower.len()..]);

        self.shift(slice.len() as isize);
    }
}
#[allow(unused)]
impl<T> RingBuffer<T> {
    pub fn from_vec(initial: Vec<T>) -> Self {
        Self {
            data: initial,
            zero_offset: 0,
        }
    }
    /// Shift the ring buffer, incrementing `zero_offset`.
    ///
    /// This means `rb[j]` becomes `rb[j-i]`
    pub fn shift(&mut self, i: isize) {
        self.zero_offset += i.rem_euclid(self.len() as isize) as usize
    }
    fn resolve_index(&self, i: isize) -> usize {
        isize::rem_euclid(i+self.zero_offset as isize, self.len() as isize) as usize
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Swap the first element out for the new one, shift the buffer
    /// forward by one, and return the old first element.
    pub fn push_pop(&mut self, new_elt: T) -> T {
        let mut elt = new_elt;
        std::mem::swap(&mut self[0], &mut elt);
        self.shift(1);
        elt
    }
    /// Swap a slice of equal length out for the new one, shift the buffer
    /// forward by the length, and return the old content.
    pub fn push_pop_slice(&mut self, slice: &mut [T]) {
        assert!(slice.len() <= self.len(), "RingBuffer: Attemped to push chunk that's longer than target buffer.");
        let (lower, upper) = self.slice_raw_mut(0, slice.len() as isize);

        lower.swap_with_slice(&mut slice[..lower.len()]);
        upper.swap_with_slice(&mut slice[lower.len()..]);

        self.shift(slice.len() as isize);
    }
    pub fn slice_raw(&self, from_i: isize, to_i: isize) -> (&[T], &[T]) {
        let from_raw_i = self.resolve_index(from_i);
        let to_raw_i = self.resolve_index(to_i);

        if from_raw_i >= to_raw_i {
            // loop around the separated part of the ring
            (&self.data[from_raw_i..], &self.data[..to_raw_i])
        } else {
            (&self.data[from_raw_i..to_raw_i], &[])
        }
    }
    pub fn slice_raw_mut(&mut self, from_i: isize, to_i: isize) -> (&mut [T], &mut [T]) {
        let from_raw_i = self.resolve_index(from_i);
        let to_raw_i = self.resolve_index(to_i);

        if from_raw_i >= to_raw_i {
            // loop around the separated part of the ring
            let whole = &mut self.data[..];
            let (upper, not_upper) = whole.split_at_mut(to_raw_i);
            let (_, lower) = not_upper.split_at_mut(from_raw_i - to_raw_i);
            (
                lower,
                // &mut self.data[from_raw_i..],
                upper,
                // &mut self.data[..to_raw_i],
            )
        } else {
            (&mut self.data[from_raw_i..to_raw_i], &mut [])
        }
    }
}
impl<T> ops::Index<isize> for RingBuffer<T> {
    type Output = T;
    fn index(&self, index: isize) -> &Self::Output {
        &self.data[self.resolve_index(index)]
    }
}
impl<T> ops::IndexMut<isize> for RingBuffer<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let index_raw = self.resolve_index(index);
        &mut self.data[index_raw]
    }
}

#[cfg(test)]
mod test {
    use super::RingBuffer;

    #[test]
    fn ring_buffer_slice_and_shift () {
        let mut rb = RingBuffer::new(10, 0);
        rb[1] = 1;
        rb[2] = 2;
        assert_eq!(rb.slice_raw(0, 10), (&[0,1,2,0,0,0,0,0,0,0]as &[i32],&[]as &[i32]), "rb[0..10]");
        rb.shift(2);
        assert_eq!(rb.slice_raw(0, 10), (&[2,0,0,0,0,0,0,0]as &[i32],&[0,1]as &[i32]), "rb.shift(2)[0..10]");
        assert_eq!(rb.slice_raw(-2, 8), (&[0,1,2,0,0,0,0,0,0,0]as &[i32],&[]as &[i32]), "rb.shift(2)[-2..8]");
    }
    #[test]
    fn ring_buffer_push () {
        let mut rb = RingBuffer::new(4, 9);
        assert_eq!(rb.slice_raw(0, 4), (&[9,9,9,9] as &[i32],&[]as &[i32]), "rb[0..4]");
        rb.push_clone_from_slice(&[2,3]);
        assert_eq!(rb.slice_raw(-2, 2), (&[2,3,9,9] as &[i32],&[]as &[i32]), "rb.push(&[2,3]); rb[-2..2]");
    }
}