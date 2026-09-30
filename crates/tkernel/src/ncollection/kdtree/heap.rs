//! Distance-only max heap: equal distances never bubble upward or replace the root.
pub(super) struct NearestHeap {
    entries: Vec<(f64, usize)>,
    capacity: usize,
}
impl NearestHeap {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }
    pub(super) fn threshold(&self) -> Option<f64> {
        if self.entries.len() == self.capacity {
            Some(self.entries[0].0)
        } else {
            None
        }
    }
    pub(super) fn offer(&mut self, distance: f64, index: usize) {
        if self.entries.len() < self.capacity {
            self.entries.push((distance, index));
            self.sift_up();
        } else if distance < self.entries[0].0 {
            self.entries[0] = (distance, index);
            self.sift_down();
        }
    }
    fn sift_up(&mut self) {
        let mut index = self.entries.len() - 1;
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.entries[index].0 <= self.entries[parent].0 {
                break;
            }
            self.entries.swap(index, parent);
            index = parent;
        }
    }
    fn sift_down(&mut self) {
        let mut index = 0;
        loop {
            let mut largest = index;
            for child in [2 * index + 1, 2 * index + 2] {
                if child < self.entries.len() && self.entries[child].0 > self.entries[largest].0 {
                    largest = child;
                }
            }
            if largest == index {
                return;
            }
            self.entries.swap(index, largest);
            index = largest;
        }
    }
    pub(super) fn sorted(mut self) -> Vec<(f64, usize)> {
        let mut result = vec![(0.0, 0); self.entries.len()];
        for position in (0..result.len()).rev() {
            result[position] = self.entries[0];
            let last = self.entries.pop().unwrap();
            if !self.entries.is_empty() {
                self.entries[0] = last;
                self.sift_down();
            }
        }
        result
    }
}
