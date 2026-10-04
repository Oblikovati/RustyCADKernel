//! AABB candidate-pair preparation for Boolean interference algorithms.
use super::{DataStructure, IndexRange, Pair};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BOPDS_Iterator` candidate pairs grouped by native interference type.
/// `let mut iterator = BOPDSIterator::default(); iterator.prepare(&ds)?;`.
#[derive(Default)]
pub struct BOPDSIterator {
    lists: [Vec<Pair>; 10],
    active_pairs: Vec<Pair>,
    position: usize,
    length: i32,
    run_parallel: bool,
}

impl BOPDSIterator {
    /// Builds source-range-filtered AABB candidates from an initialized data structure.
    /// `iterator.prepare(&ds)?`.
    pub fn prepare(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        self.clear_lists();
        let candidates = self.candidate_pairs(structure)?;
        for range_index in 0..structure.nb_ranges() {
            let range = structure.range(range_index)?;
            self.add_range_pairs(structure, range, &candidates)?;
        }
        Ok(())
    }

    /// Selects one shape-type pair and resets traversal; `iterator.initialize(Vertex, Edge)`.
    pub fn initialize(&mut self, first: ShapeEnum, second: ShapeEnum) {
        let Some(list_index) = interference_index(first, second) else {
            self.length = 0;
            return;
        };
        self.active_pairs.clone_from(&self.lists[list_index]);
        self.active_pairs.sort_by_key(Pair::indices);
        self.position = 0;
        self.length = self.active_pairs.len() as i32;
    }

    /// Reports whether `value` can read another pair; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.position < self.active_pairs.len()
    }

    /// Advances to the next selected pair; `iterator.next()`.
    pub fn next(&mut self) {
        self.position += usize::from(self.more());
    }

    /// Reads the current source indices or reports iterator exhaustion.
    /// `let (first, second) = iterator.value()?`.
    pub fn value(&self, structure: &DataStructure) -> Result<(i32, i32), Failure> {
        let pair = self
            .active_pairs
            .get(self.position)
            .ok_or_else(|| Failure::new("BOPDS_Iterator::Value called without a current pair"))?;
        orient_pair(structure, *pair)
    }

    /// Reads the selected pair count; `iterator.expected_length()`.
    pub fn expected_length(&self) -> i32 {
        self.length
    }

    /// Matches OCCT's half-length task estimate with a minimum of one.
    /// `iterator.block_length()`.
    pub fn block_length(&self) -> i32 {
        if self.length <= 1 {
            return 1;
        }
        (0.5 * f64::from(self.length)) as i32
    }

    /// Sets the parallel-processing option, retained for later intersection stages.
    /// `iterator.set_run_parallel(true)`.
    pub fn set_run_parallel(&mut self, enabled: bool) {
        self.run_parallel = enabled;
    }

    /// Reads the parallel-processing option; `iterator.run_parallel()`.
    pub fn run_parallel(&self) -> bool {
        self.run_parallel
    }

    fn clear_lists(&mut self) {
        for list in &mut self.lists {
            list.clear();
        }
        self.active_pairs.clear();
        self.position = 0;
        self.length = 0;
    }

    fn candidate_pairs(&self, structure: &DataStructure) -> Result<Vec<Pair>, Failure> {
        let mut pairs = Vec::new();
        for first in 0..structure.nb_source_shapes() {
            self.append_intersecting_pairs(structure, first, &mut pairs)?;
        }
        pairs.sort_by_key(Pair::indices);
        Ok(pairs)
    }

    fn append_intersecting_pairs(
        &self,
        structure: &DataStructure,
        first: i32,
        pairs: &mut Vec<Pair>,
    ) -> Result<(), Failure> {
        let first_info = structure.shape_info(first)?;
        if !first_info.has_brep() {
            return Ok(());
        }
        for second in first + 1..structure.nb_source_shapes() {
            self.append_intersecting_pair(structure, first, second, pairs)?;
        }
        Ok(())
    }

    fn append_intersecting_pair(
        &self,
        structure: &DataStructure,
        first: i32,
        second: i32,
        pairs: &mut Vec<Pair>,
    ) -> Result<(), Failure> {
        let first_info = structure.shape_info(first)?;
        let second_info = structure.shape_info(second)?;
        if second_info.has_brep() && first_info.box_bounds().intersects(second_info.box_bounds()) {
            pairs.push(Pair::new(first, second));
        }
        Ok(())
    }

    fn add_range_pairs(
        &mut self,
        structure: &DataStructure,
        range: &IndexRange,
        candidates: &[Pair],
    ) -> Result<(), Failure> {
        for pair in candidates {
            let (first, second) = pair.indices();
            if !range.contains(first) {
                break;
            }
            if range.contains(second) || self.is_contained_pair(structure, first, second)? {
                continue;
            }
            self.add_typed_pair(structure, *pair)?;
        }
        Ok(())
    }

    fn is_contained_pair(
        &self,
        structure: &DataStructure,
        first: i32,
        second: i32,
    ) -> Result<bool, Failure> {
        let first_info = structure.shape_info(first)?;
        let second_info = structure.shape_info(second)?;
        let first_rank = shape_rank(first_info.shape_type());
        let second_rank = shape_rank(second_info.shape_type());
        Ok(
            (first_rank < second_rank && first_info.has_sub_shape(second))
                || (first_rank > second_rank && second_info.has_sub_shape(first)),
        )
    }

    fn add_typed_pair(&mut self, structure: &DataStructure, pair: Pair) -> Result<(), Failure> {
        let (first, second) = pair.indices();
        let first_type = structure.shape_info(first)?.shape_type();
        let second_type = structure.shape_info(second)?.shape_type();
        if let Some(index) = interference_index(first_type, second_type) {
            self.lists[index].push(pair);
        }
        Ok(())
    }
}

fn orient_pair(structure: &DataStructure, pair: Pair) -> Result<(i32, i32), Failure> {
    let (first, second) = pair.indices();
    let first_type = structure.shape_info(first)?.shape_type();
    let second_type = structure.shape_info(second)?.shape_type();
    if shape_rank(first_type) < shape_rank(second_type) {
        return Ok((second, first));
    }
    Ok((first, second))
}

fn interference_index(first: ShapeEnum, second: ShapeEnum) -> Option<usize> {
    let first = interference_rank(first);
    let second = interference_rank(second);
    let code = second * 10 + first;
    match code {
        77 => Some(0),
        76 | 67 => Some(1),
        66 => Some(2),
        74 | 47 => Some(3),
        64 | 46 => Some(4),
        44 => Some(5),
        72 | 27 => Some(6),
        62 | 26 => Some(7),
        42 | 24 => Some(8),
        22 => Some(9),
        _ => None,
    }
}

fn interference_rank(shape_type: ShapeEnum) -> i32 {
    shape_rank(shape_type)
}

fn shape_rank(shape_type: ShapeEnum) -> i32 {
    match shape_type {
        ShapeEnum::Compound => 0,
        ShapeEnum::CompSolid => 1,
        ShapeEnum::Solid => 2,
        ShapeEnum::Shell => 3,
        ShapeEnum::Face => 4,
        ShapeEnum::Wire => 5,
        ShapeEnum::Edge => 6,
        ShapeEnum::Vertex => 7,
        ShapeEnum::Shape => 8,
    }
}
