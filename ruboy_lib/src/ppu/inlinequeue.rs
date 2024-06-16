use std::fmt::Debug;

pub struct InlineQueue<T, const N: usize> {
    data: [Option<T>; N],
    cur_elements: usize,
}

impl<T, const N: usize> Debug for InlineQueue<T, N>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fifo")
            .field("data", &self.data)
            .field("cur_elements", &self.cur_elements)
            .finish()
    }
}

impl<T, const N: usize> Clone for InlineQueue<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            cur_elements: self.cur_elements,
        }
    }
}

impl<T, const N: usize> Copy for InlineQueue<T, N> where T: Copy {}

impl<T, const N: usize> InlineQueue<T, N> {
    pub fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| None),
            cur_elements: 0,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..N {
            self.data[i] = None;
        }

        self.cur_elements = 0;
    }

    pub const fn len(&self) -> usize {
        self.cur_elements
    }

    pub const fn space_remaining(&self) -> usize {
        N - self.len()
    }

    pub const fn empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn push(&mut self, elem: T) -> Result<(), ()> {
        self.push_n([elem])
    }

    pub fn push_n<const I: usize>(&mut self, elems: [T; I]) -> Result<(), ()> {
        if self.len() + I > N {
            return Err(());
        }

        let init_len = self.len();

        for (i, elem) in elems.into_iter().enumerate() {
            self.data[init_len + i] = Some(elem);
        }

        self.cur_elements += I;

        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Result<T, ()> {
        let result: [T; 1] = self.pop_n()?;

        Ok(result.into_iter().nth(0).unwrap())
    }

    pub fn pop_n<const I: usize>(&mut self) -> Result<[T; I], ()> {
        if self.len() < I {
            return Err(());
        }

        let buf: [T; I] = std::array::from_fn(|i| self.data[i].take().unwrap());

        for i in I..self.len() {
            self.data[i - I] = self.data[i].take();
            debug_assert!(self.data[i - I].is_some());
        }

        self.cur_elements -= I;

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::InlineQueue;

    #[test]
    fn len() {
        let mut x = InlineQueue::<u8, 16>::new();

        assert_eq!(0, x.len());

        x.push(1).unwrap();
        assert_eq!(1, x.len());

        x.push_n([2, 3]).unwrap();

        assert_eq!(3, x.len());

        x.pop().unwrap();

        assert_eq!(2, x.len());

        x.pop_n::<2>().unwrap();

        assert_eq!(0, x.len());
    }

    #[test]
    fn empty() {
        let mut x = InlineQueue::<u8, 16>::new();

        assert!(x.empty());

        x.push(1).unwrap();

        assert!(!x.empty());

        x.pop().unwrap();

        assert!(x.empty());
    }

    #[test]
    fn filled() {
        let mut x = InlineQueue::<u8, 16>::new();

        for i in 0..16 {
            x.push(i).unwrap();
        }

        assert!(x.push(16).is_err());
    }

    #[test]
    fn pop_empty() {
        let mut x = InlineQueue::<u8, 16>::new();

        for i in 0..16 {
            x.push(i).unwrap();
        }

        for _ in 0..16 {
            x.pop().unwrap();
        }

        assert!(x.pop().is_err());

        x.push(1).unwrap();

        assert!(x.pop_n::<2>().is_err());
    }

    #[test]
    fn pop_equal() {
        let mut x = InlineQueue::<u8, 16>::new();

        for i in 0..16 {
            x.push(i).unwrap();
        }

        assert_eq!(0, x.pop().unwrap());

        let rest = x.pop_n::<15>().unwrap();

        (0..rest.len()).for_each(|i| {
            assert_eq!(i + 1, rest[i] as usize);
        });
    }

    #[test]
    fn space_remaining_ok() {
        let mut x = InlineQueue::<u8, 16>::new();

        assert_eq!(16, x.space_remaining());

        x.push(1).unwrap();

        assert_eq!(15, x.space_remaining());

        x.push_n([2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
            .unwrap();

        assert_eq!(1, x.space_remaining());

        x.push(16).unwrap();

        assert_eq!(0, x.space_remaining());
    }
}
