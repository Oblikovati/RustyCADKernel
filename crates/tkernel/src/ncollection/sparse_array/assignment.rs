//! Preserve OCCT's ascending callback order and partially copied extent on unwinding.
use super::SparseArray;

impl<T> SparseArray<T> {
    fn discard_sparse_block(&mut self, number: usize) {
        if let Some(block) = self.blocks.remove(&number) {
            self.count -= block.iter().filter(|slot| slot.is_some()).count();
            drop(block);
        }
    }

    /// Preserves native self-assignment without conflicting Rust borrows; `array.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl<T: Clone> SparseArray<T> {
    /// Copies in ascending index order, preserving completed copies on panic; `target.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        if self.increment != source.increment {
            self.clear();
            self.increment = source.increment;
        }
        for number in 0..source.block_slots {
            self.assign_sparse_block(source, number);
        }
        let trailing: Vec<usize> = self
            .blocks
            .range(source.block_slots..)
            .map(|(&n, _)| n)
            .collect();
        for number in trailing {
            self.discard_sparse_block(number);
        }
        self
    }

    fn assign_sparse_block(&mut self, source: &Self, number: usize) {
        let Some(source_block) = source.blocks.get(&number) else {
            self.discard_sparse_block(number);
            return;
        };
        self.ensure_sparse_block(number);
        let block = self.blocks.get_mut(&number).unwrap();
        for (destination, original) in block.iter_mut().zip(source_block.iter()) {
            assign_sparse_slot(destination, original, &mut self.count);
        }
    }
}

fn assign_sparse_slot<T: Clone>(
    destination: &mut Option<T>,
    original: &Option<T>,
    count: &mut usize,
) {
    match (destination.as_mut(), original.as_ref()) {
        (Some(current), Some(value)) => current.clone_from(value),
        (None, Some(value)) => {
            *destination = Some(value.clone());
            *count += 1;
        }
        (Some(_), None) => {
            *count -= 1;
            *destination = None;
        }
        (None, None) => {}
    }
}
