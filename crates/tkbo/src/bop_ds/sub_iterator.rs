//! Borrowed subset broad-phase traversal with native sorting and per-intersection fencing.
use super::{DataStructure, Pair};
use crate::bop_tools::{BoxSet, PairSelector};
use rustycad_tkernel::{
    ncollection::{BaseAllocator, DynamicArray, List, Map},
    standard::Failure,
};
use rustycad_tkmath::{bnd::box3_to_bvh, bvh::BvhVec3d};

/// Selects interfering subsets while borrowing their source storage; `BOPDSSubIterator::default()`.
pub struct BOPDSSubIterator<'a> {
    structure: Option<&'a DataStructure>,
    first: Option<&'a List<i32>>,
    second: Option<&'a List<i32>>,
    pairs: DynamicArray<Pair>,
    position: usize,
    end: usize,
}

impl Default for BOPDSSubIterator<'_> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}

impl<'a> BOPDSSubIterator<'a> {
    /// Creates empty subset traversal; `let iterator = BOPDSSubIterator::new();`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Uses the supplied resource for pair blocks; `BOPDSSubIterator::with_allocator(allocator)`.
    pub fn with_allocator(allocator: BaseAllocator) -> Self {
        Self {
            structure: None,
            first: None,
            second: None,
            pairs: DynamicArray::with_allocator(1, Some(allocator)),
            position: 0,
            end: 0,
        }
    }

    /// Borrows or clears the source DS; `iterator.set_ds(Some(&ds))`.
    pub fn set_ds(&mut self, structure: Option<&'a DataStructure>) {
        self.structure = structure;
    }

    /// Returns the selected DS; `iterator.ds()?`.
    pub fn ds(&self) -> Result<&'a DataStructure, Failure> {
        self.structure.ok_or_else(|| missing_subset_input("DS"))
    }

    /// Borrows the first subset; `iterator.set_subset1(&indices)`.
    pub fn set_subset1(&mut self, indices: &'a List<i32>) {
        self.first = Some(indices);
    }

    /// Reads the first borrowed subset; `iterator.subset1()?`.
    pub fn subset1(&self) -> Result<&'a List<i32>, Failure> {
        self.first.ok_or_else(|| missing_subset_input("SubSet1"))
    }

    /// Borrows the second subset; `iterator.set_subset2(&indices)`.
    pub fn set_subset2(&mut self, indices: &'a List<i32>) {
        self.second = Some(indices);
    }

    /// Reads the second borrowed subset; `iterator.subset2()?`.
    pub fn subset2(&self) -> Result<&'a List<i32>, Failure> {
        self.second.ok_or_else(|| missing_subset_input("SubSet2"))
    }

    /// Clears and prepares pairs without initializing the cursor; `iterator.prepare()?`.
    pub fn prepare(&mut self) -> Result<(), Failure> {
        self.pairs.clear(false);
        if self.structure.is_none() {
            return Ok(());
        }
        let count = self.subset1()?.len() + self.subset2()?.len();
        if self.subset1()?.is_empty() || self.subset2()?.is_empty() {
            return Ok(());
        }
        self.pairs.set_increment(2 * count);
        self.intersect()
    }

    /// Appends one intersection pass, fencing duplicate inputs within it; `iterator.intersect()?`.
    pub fn intersect(&mut self) -> Result<(), Failure> {
        if self.subset1()?.is_empty() || self.subset2()?.is_empty() {
            return Ok(());
        }
        let ds = self.ds()?;
        let pairs = subset_candidates(ds, self.subset1()?, self.subset2()?)?;
        let mut fence = Map::<Pair>::default();
        for (first, second) in pairs {
            let pair = Pair::new(first.min(second), first.max(second));
            if first != second && fence.add(pair) && !subset_is_contained(ds, first, second)? {
                self.pairs.append(pair);
            }
        }
        Ok(())
    }

    /// Sorts pairs and starts a fresh traversal; `iterator.initialize()`.
    pub fn initialize(&mut self) {
        let mut sorted = self.pairs.iter().copied().collect::<Vec<_>>();
        sorted.sort_by_key(Pair::indices);
        for (index, pair) in sorted.into_iter().enumerate() {
            *self
                .pairs
                .change_value(index)
                .expect("existing subset pair") = pair;
        }
        self.position = 0;
        self.end = self.pairs.len();
    }

    /// Reports whether the initialized cursor is valid; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.position < self.end
    }

    /// Advances the initialized cursor; `iterator.next()`.
    pub fn next(&mut self) {
        self.position = self.position.saturating_add(1);
    }

    /// Reads indices with the more specific shape first; `iterator.value()?`.
    pub fn value(&self) -> Result<(i32, i32), Failure> {
        let pair = self.pairs.value(self.position).map_err(|_| {
            Failure::new(&format!(
                "BOPDS_SubIterator::Value received cursor {}; expected initialized pair below {}",
                self.position, self.end
            ))
        })?;
        let (first, second) = pair.indices();
        let ds = self.ds()?;
        if (ds.shape_info(first)?.shape_type() as i32)
            < (ds.shape_info(second)?.shape_type() as i32)
        {
            return Ok((second, first));
        }
        Ok((first, second))
    }

    /// Returns the prepared pair count; `iterator.expected_length()`.
    pub fn expected_length(&self) -> i32 {
        self.pairs.length()
    }
}

fn missing_subset_input(name: &str) -> Failure {
    Failure::new(&format!(
        "BOPDS_SubIterator received absent {name}; expected borrowed input"
    ))
}

fn subset_candidates(
    ds: &DataStructure,
    first: &List<i32>,
    second: &List<i32>,
) -> Result<Vec<(i32, i32)>, Failure> {
    if first.is_empty() || second.is_empty() {
        return Ok(Vec::new());
    }
    let mut first_tree = subset_tree(ds, first)?;
    let mut second_tree = subset_tree(ds, second)?;
    let mut selector = PairSelector::default();
    selector.set_bvh_sets(&mut first_tree, &mut second_tree);
    selector.select()?;
    selector.sort();
    Ok(selector
        .pairs()
        .iter()
        .map(|pair| (pair.id1, pair.id2))
        .collect())
}

fn subset_tree(ds: &DataStructure, indices: &List<i32>) -> Result<BoxSet<BvhVec3d>, Failure> {
    let mut tree = BoxSet::default();
    tree.set_size(indices.len());
    for index in indices.iter() {
        tree.add(*index, box3_to_bvh(ds.shape_info(*index)?.box_bounds())?);
    }
    tree.build()?;
    Ok(tree)
}

fn subset_is_contained(ds: &DataStructure, first: i32, second: i32) -> Result<bool, Failure> {
    let first_info = ds.shape_info(first)?;
    let second_info = ds.shape_info(second)?;
    let first_rank = first_info.shape_type() as i32;
    let second_rank = second_info.shape_type() as i32;
    Ok(
        (first_rank < second_rank && first_info.has_sub_shape(second))
            || (first_rank > second_rank && second_info.has_sub_shape(first)),
    )
}
