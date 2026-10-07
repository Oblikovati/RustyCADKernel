//! `BOPDS_IteratorSI`: the broad phase of the self-interference check, which pairs the
//! sub-shapes of one argument with each other.
use super::{BOPDSIterator, DataStructure, PreparationOptions, source_types};
use crate::bop_ds::ShapeInfo;
use crate::int_tools::Context;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// The candidate pairs of the self-interference check, read through the `BOPDSIterator` API
/// (`Initialize`, `More`, `Next`, `Value`).
///
/// ```
/// use rustycad_tkbo::bop_ds::BOPDSIteratorSI;
/// let mut iterator = BOPDSIteratorSI::new();
/// iterator.update_by_level_of_check(4);
/// assert_eq!(iterator.expected_length(), 0);
/// ```
#[derive(Default)]
pub struct BOPDSIteratorSI {
    iterator: BOPDSIterator,
}

impl Deref for BOPDSIteratorSI {
    type Target = BOPDSIterator;
    fn deref(&self) -> &BOPDSIterator {
        &self.iterator
    }
}

impl DerefMut for BOPDSIteratorSI {
    fn deref_mut(&mut self) -> &mut BOPDSIterator {
        &mut self.iterator
    }
}

impl BOPDSIteratorSI {
    /// An iterator with no candidate pair (`BOPDS_IteratorSI()`; the allocator overload maps
    /// here too); `BOPDSIteratorSI::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects the interfering pairs among all sub-shapes of the argument, the same range
    /// included (`Prepare` with the overridden `Intersect`); a pair of a shape and one of its
    /// own sub-shapes is skipped, and with `check_obb` a pair whose oriented boxes are apart;
    /// `iterator.prepare_with_context(&ds, Some(&context), false, 0.0)?`.
    pub fn prepare_with_context(
        &mut self,
        structure: &DataStructure,
        context: Option<&Context>,
        check_obb: bool,
        fuzzy: f64,
    ) -> Result<(), Failure> {
        let options = PreparationOptions {
            context,
            check_obb,
            fuzzy,
        };
        let iterator = &mut self.iterator;
        iterator.clear_lists();
        iterator.shape_types = source_types(structure)?;
        for pair in iterator.candidate_pairs(structure, ShapeInfo::is_interfering)? {
            let (first, second) = pair.indices();
            if iterator.is_contained_pair(structure, first, second)?
                || iterator.is_out_obb(structure, first, second, options)?
            {
                continue;
            }
            iterator.add_typed_pair(structure, pair)?;
        }
        Ok(())
    }

    /// Clears the candidate lists of the interference kinds above `level` (0 VV .. 9 ZZ)
    /// (`UpdateByLevelOfCheck`); `iterator.update_by_level_of_check(4)`.
    pub fn update_by_level_of_check(&mut self, level: i32) {
        let first_cleared = (level + 1).max(0) as usize;
        for list in self.iterator.lists.iter_mut().skip(first_cleared) {
            list.clear();
        }
    }

    /// The prepared base iterator, which the pave filler traverses;
    /// `filler_iterator = iterator.into_iterator()`.
    pub fn into_iterator(self) -> BOPDSIterator {
        self.iterator
    }
}
