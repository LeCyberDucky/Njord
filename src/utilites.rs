pub struct Memory<T> {
    capacity: usize,
    count: usize,
    pub data: std::collections::VecDeque<T>,
}

impl<T> Memory<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            count: 0,
            data: std::collections::VecDeque::new(),
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let mut result = None;
        if self.data.len() == self.capacity {
            result = self.data.pop_front();
        }

        self.data.push_back(value);
        self.count += 1;
        result
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<T> std::ops::Index<usize> for Memory<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T> std::ops::IndexMut<usize> for Memory<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}
