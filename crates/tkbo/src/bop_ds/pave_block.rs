// Rust translation of BOPDS_PaveBlock.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! An edge interval between adjacent paves, plus inserted vertices and shrunk bounds.
use super::Pave;
use rustycad_tkmath::bnd::BoundingBox3;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Shared mutable ownership mirrors an OCCT `occ::handle<BOPDS_PaveBlock>`.
pub type PaveBlockHandle = Arc<Mutex<PaveBlock>>;

/// `BOPDS_PaveBlock`; two endpoint paves define one interval on an original edge.
#[derive(Clone, Debug)]
pub struct PaveBlock {
    edge: i32,
    original_edge: i32,
    pave1: Pave,
    pave2: Pave,
    extra_paves: Vec<Pave>,
    extra_indices: HashSet<i32>,
    shrunk_first: f64,
    shrunk_last: f64,
    shrunk_box: BoundingBox3,
    splittable: bool,
}

impl Default for PaveBlock {
    fn default() -> Self {
        Self {
            edge: -1,
            original_edge: -1,
            pave1: Pave::default(),
            pave2: Pave::default(),
            extra_paves: Vec::new(),
            extra_indices: HashSet::new(),
            shrunk_first: -99.0,
            shrunk_last: -99.0,
            shrunk_box: BoundingBox3::default(),
            splittable: false,
        }
    }
}

impl PaveBlock {
    /// Sets the result edge index; `block.set_edge(8)`.
    pub fn set_edge(&mut self, edge: i32) {
        self.edge = edge;
    }

    /// Reads the result edge index; `PaveBlock::default().edge() == -1`.
    pub fn edge(&self) -> i32 {
        self.edge
    }

    /// Tests whether this block has a nonnegative result edge; `block.has_edge()`.
    pub fn has_edge(&self) -> bool {
        self.edge >= 0
    }

    /// Writes the edge index and reports whether it is nonnegative; `block.has_edge(&mut i)`.
    pub fn has_edge_into(&self, edge: &mut i32) -> bool {
        *edge = self.edge;
        self.has_edge()
    }

    /// Sets the unsplit source edge index; `block.set_original_edge(4)`.
    pub fn set_original_edge(&mut self, edge: i32) {
        self.original_edge = edge;
    }

    /// Reads the unsplit source edge index; `PaveBlock::default().original_edge() == -1`.
    pub fn original_edge(&self) -> i32 {
        self.original_edge
    }

    /// Reports whether result and source edge indices differ; `block.is_split_edge()`.
    pub fn is_split_edge(&self) -> bool {
        self.edge != self.original_edge
    }

    /// Sets the first interval endpoint; `block.set_pave1(Pave::new(1, 0.0))`.
    pub fn set_pave1(&mut self, pave: Pave) {
        self.pave1 = pave;
    }

    /// Reads the first interval endpoint; `block.pave1().index() == 1`.
    pub fn pave1(&self) -> Pave {
        self.pave1
    }

    /// Sets the second interval endpoint; `block.set_pave2(Pave::new(2, 1.0))`.
    pub fn set_pave2(&mut self, pave: Pave) {
        self.pave2 = pave;
    }

    /// Reads the second interval endpoint; `block.pave2().index() == 2`.
    pub fn pave2(&self) -> Pave {
        self.pave2
    }

    /// Reads endpoint parameters in stored order; `block.range() == (0.0, 1.0)`.
    pub fn range(&self) -> (f64, f64) {
        (self.pave1.parameter(), self.pave2.parameter())
    }

    /// Reads endpoint vertex indices in stored order; `block.indices() == (1, 2)`.
    pub fn indices(&self) -> (i32, i32) {
        (self.pave1.index(), self.pave2.index())
    }

    /// Tests endpoint indices in either direction; `block.has_same_bounds(&other)`.
    pub fn has_same_bounds(&self, other: &Self) -> bool {
        let (first, second) = self.indices();
        let (other_first, other_second) = other.indices();
        (first, second) == (other_first, other_second)
            || (first, second) == (other_second, other_first)
    }

    /// Adds an extra pave once per vertex index; `block.append_ext_pave(Pave::new(3, 0.5))`.
    pub fn append_ext_pave(&mut self, pave: Pave) {
        if self.extra_indices.insert(pave.index()) {
            self.extra_paves.push(pave);
        }
    }

    /// Appends an extra pave without deduplication; `block.append_ext_pave1(pave)`.
    pub fn append_ext_pave1(&mut self, pave: Pave) {
        self.extra_paves.push(pave);
    }

    /// Removes every extra pave with the vertex index if the native fence contains it.
    pub fn remove_ext_pave(&mut self, vertex: i32) {
        if !self.extra_indices.remove(&vertex) {
            return;
        }
        self.extra_paves.retain(|pave| pave.index() != vertex);
    }

    /// Borrows extra paves in insertion order; `block.ext_paves()`.
    pub fn ext_paves(&self) -> &[Pave] {
        &self.extra_paves
    }

    /// Mutably borrows the native extra-pave sequence; `block.change_ext_paves().push(pave)`.
    pub fn change_ext_paves(&mut self) -> &mut Vec<Pave> {
        &mut self.extra_paves
    }

    /// Reports whether inserted paves require interval rebuilding; `block.is_to_update()`.
    pub fn is_to_update(&self) -> bool {
        !self.extra_paves.is_empty()
    }

    /// Finds the first extra pave within strict parameter tolerance, preserving `index` on miss.
    pub fn contains_parameter(&self, parameter: f64, tolerance: f64, index: &mut i32) -> bool {
        let Some(found) = self
            .extra_paves
            .iter()
            .find(|pave| (pave.parameter() - parameter).abs() < tolerance)
        else {
            return false;
        };
        *index = found.index();
        true
    }

    /// Splits this interval around all extra paves, appending the new blocks to `output`.
    pub fn update(&mut self, output: &mut Vec<Self>, include_endpoints: bool) {
        let mut paves = self.update_paves(include_endpoints);
        self.extra_paves.clear();
        self.extra_indices.clear();
        if paves.len() < 2 {
            return;
        }
        paves.sort_by(pave_order);
        self.append_intervals(output, &paves);
    }

    /// Reads paves to split from the requested endpoints and the extra list.
    fn update_paves(&self, include_endpoints: bool) -> Vec<Pave> {
        let endpoint_count = if include_endpoints { 2 } else { 0 };
        let mut paves = Vec::with_capacity(endpoint_count + self.extra_paves.len());
        if include_endpoints {
            paves.extend([self.pave1, self.pave2]);
        }
        paves.extend(self.extra_paves.iter().copied());
        paves
    }

    /// Adds one output interval between each neighboring sorted pave.
    fn append_intervals(&self, output: &mut Vec<Self>, paves: &[Pave]) {
        for pair in paves.windows(2) {
            let mut block = Self::default();
            block.set_original_edge(self.original_edge);
            block.set_pave1(pair[0]);
            block.set_pave2(pair[1]);
            output.push(block);
        }
    }

    /// Stores shrunk parameter bounds, box and splitability; `block.set_shrunk_data(...)`.
    pub fn set_shrunk_data(
        &mut self,
        first: f64,
        last: f64,
        bounds: &BoundingBox3,
        splittable: bool,
    ) {
        self.shrunk_first = first;
        self.shrunk_last = last;
        self.shrunk_box = bounds.clone();
        self.splittable = splittable;
    }

    /// Reads stored shrunk bounds and splitability; `block.shrunk_data()`.
    pub fn shrunk_data(&self) -> (f64, f64, &BoundingBox3, bool) {
        (
            self.shrunk_first,
            self.shrunk_last,
            &self.shrunk_box,
            self.splittable,
        )
    }

    /// Reports whether the shrunk box is nonvoid; `block.has_shrunk_data()`.
    pub fn has_shrunk_data(&self) -> bool {
        !self.shrunk_box.is_void()
    }

    /// Reports whether the shrunk range can be split; `block.is_splittable()`.
    pub fn is_splittable(&self) -> bool {
        self.splittable
    }

    /// Writes the native compact dump; `block.dump(&mut String::new())?`.
    pub fn dump(&self, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(
            output,
            " PB:{{ E:{} orE:{} Pave1:",
            self.edge, self.original_edge
        )?;
        self.pave1.dump(output)?;
        output.write_str(" Pave2:")?;
        self.pave2.dump(output)?;
        output.write_str(" }")
    }
}

/// Matches `BOPDS_Pave::IsLess`, where only the parameter participates in ordering.
fn pave_order(first: &Pave, second: &Pave) -> Ordering {
    if first.parameter() < second.parameter() {
        return Ordering::Less;
    }
    if second.parameter() < first.parameter() {
        return Ordering::Greater;
    }
    Ordering::Equal
}
