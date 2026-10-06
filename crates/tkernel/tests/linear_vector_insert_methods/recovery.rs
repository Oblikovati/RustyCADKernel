use super::*;

impl Drop for InsertFixture {
    fn drop(&mut self) {
        self.trace.active.set(false);
        self.repair_registered_slots();
        self.destroy_unpublished_tail();
        // SAFETY: every registered slot is initialized and the extra tail no longer owns a value.
        unsafe { ManuallyDrop::drop(&mut self.vector) };
    }
}

impl InsertFixture {
    fn repair_registered_slots(&mut self) {
        for index in 0..self.vector.len() {
            if self.trace.live.borrow()[index] {
                continue;
            }
            // SAFETY: this known dead slot is exclusively owned by the suspended vector.
            unsafe {
                self.vector.data_mut_ptr().add(index).write(InsertItem::new(
                    90 + i32::try_from(index).unwrap(),
                    &self.trace,
                ));
            }
            self.trace.live.borrow_mut()[index] = true;
        }
    }

    fn destroy_unpublished_tail(&mut self) {
        let tail = self.trace.count.get();
        if self.vector.len() != tail || !self.trace.live.borrow()[tail] {
            return;
        }
        // SAFETY: a completed backward shift constructed the unregistered tail exactly once.
        unsafe { self.vector.data_mut_ptr().add(tail).drop_in_place() };
        self.trace.live.borrow_mut()[tail] = false;
    }
}
