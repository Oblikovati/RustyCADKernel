//! A wrapped zero stride has usable retained storage, but later native slots form sparse diagonals.
use super::DynamicArray;
use crate::standard::Failure;
impl<T> DynamicArray<T> {
    pub(super) fn prepare_append_location(&mut self) -> (usize, usize) {
        if self.increment == 0 {
            self.require_retained_first_append();
            return (0, 0);
        }
        let location = self.element_coordinates(self.used);
        if location.0 == self.blocks.len() {
            self.add_block();
        }
        location
    }
    fn require_retained_first_append(&self) {
        let layout = self
            .blocks
            .as_slice()
            .first()
            .map(|block| (block.len(), block.capacity()));
        let valid = self.used == 0
            && size_of::<T>() != 0
            && layout.is_some_and(|(initialized, capacity)| initialized == 0 && capacity > 0);
        assert!(
            valid,
            "zero-stride append index {}, blocks {}, first layout {layout:?}: expected index0 with an empty retained nonzero-sized slot; later sparse slots require a separate ownership policy",
            self.used,
            self.blocks.len()
        );
    }
    pub(super) fn require_retained_erasure(&self) {
        if self.increment != 0 {
            return;
        }
        self.validate_retained_erasure()
            .unwrap_or_else(|failure| panic!("zero-stride EraseLast: {failure}"));
    }
    pub(super) fn validate_retained_erasure(&self) -> Result<(), Failure> {
        let first = self.blocks.as_slice().first().map(|block| block.len());
        let valid = self.used == 1
            && first == Some(1)
            && self.blocks.iter().skip(1).all(|block| block.is_empty());
        if valid {
            return Ok(());
        }
        let reason = format!(
            "zero-stride used {}, first length {first:?}: expected one registered first slot and empty later blocks",
            self.used
        );
        Err(Failure::with_type("Standard_DomainError", &reason, ""))
    }
    pub(super) fn clear_payload_prefixes(&mut self) {
        for block in &mut self.blocks {
            if self.increment == 0 {
                // OCCT 3d097a0 Clear computes zero destruction counts; preserve bytes without moving T.
                unsafe { block.set_len(0) };
            } else {
                block.clear();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_stride_publication_preserves_the_conservative_tail_scan() {
        let mut array = DynamicArray::with_increment(2);
        array.append(1);
        array.clear(false);
        array.set_increment(usize::MAX);
        array.append(7);
        assert!(array.has_initialized_tail());
        array.erase_last();
        assert!(!array.has_initialized_tail());
    }
    #[test]
    fn invalid_sparse_erasure_metadata_is_rejected_before_state_change() {
        let mut array = DynamicArray::with_increment(2);
        array.append(1);
        array.clear(false);
        array.set_increment(usize::MAX);
        array.append(7);
        array.used = 2;
        assert!(array.validate_retained_erasure().is_err());
        assert_eq!(array.used, 2);
        assert_eq!(array.blocks[0].len(), 1);
    }
}
