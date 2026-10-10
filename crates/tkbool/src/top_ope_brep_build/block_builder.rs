// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_BlockBuilder.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_BlockBuilder`: the connected blocks of the elements of a shape set
//! (the elements reached from each new start element through their neighbours), with
//! a validity flag per element and a regularity flag per block.
use super::loops::TopOpeBRepBuildBlockIterator;
use super::shape_set::TopOpeBRepBuildShapeSet;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap};
use rustycad_tkernel::standard::Failure;

/// The blocks of a shape set (`TopOpeBRepBuild_BlockBuilder`); natively the block
/// index is uninitialized until InitBlock (0 here);
/// `let bb = TopOpeBRepBuildBlockBuilder::with_shape_set(&mut wes)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildBlockBuilder {
    my_oriented_shape_map_is_valid: DataMap<i32, i32>,
    my_oriented_shape_map: IndexedMap<Shape>,
    my_blocks: Vec<i32>,
    my_block_index: i32,
    my_is_done: bool,
    my_blocks_is_regular: Vec<i32>,
}

/// Standard_Failure "OutOfRange" of the element accessors.
fn out_of_range(what: &str) -> Failure {
    Failure::new(&format!(
        "OutOfRange: {what}; expected an element of the block builder"
    ))
}

impl TopOpeBRepBuildBlockBuilder {
    /// `TopOpeBRepBuild_BlockBuilder()`; `TopOpeBRepBuildBlockBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_BlockBuilder(SS)`: `MakeBlock(SS)`;
    /// `TopOpeBRepBuildBlockBuilder::with_shape_set(&mut wes)?`.
    pub fn with_shape_set(ss: &mut dyn TopOpeBRepBuildShapeSet) -> Result<Self, Failure> {
        let mut bb = Self::new();
        bb.make_block(ss)?;
        Ok(bb)
    }

    /// The elements reached from the element of index `eindex` through their
    /// neighbours, added in breadth order; whether every element has at most two
    /// neighbours per sub-shape (regular).
    fn grow_block(
        &mut self,
        ss: &mut dyn TopOpeBRepBuildShapeSet,
        mut eindex: i32,
    ) -> Result<bool, Failure> {
        let mut is_regular = true;
        while eindex <= self.my_oriented_shape_map.len() as i32 {
            let e1 = self
                .my_oriented_shape_map
                .find_key(eindex as usize)?
                .clone();
            let cur_nei = ss.max_number_sub_shape(&e1)?;
            is_regular = is_regular && cur_nei <= 2;
            ss.init_neighbours(&e1)?;
            while ss.more_neighbours() {
                let n = ss.neighbour()?.clone();
                self.add_element(&n);
                ss.next_neighbour()?;
            }
            eindex += 1;
        }
        Ok(is_regular)
    }

    /// `MakeBlock(SS)`: a block per start element not yet reached;
    /// `bb.make_block(&mut wes)?`.
    pub fn make_block(&mut self, ss: &mut dyn TopOpeBRepBuildShapeSet) -> Result<(), Failure> {
        self.my_oriented_shape_map.clear(false);
        self.my_oriented_shape_map_is_valid.clear(false);
        self.my_blocks.clear();
        self.my_blocks_is_regular.clear();
        ss.init_start_elements();
        while ss.more_start_elements() {
            let e = ss.start_element()?.clone();
            let mextent = self.my_oriented_shape_map.len() as i32;
            let eindex = self.add_element(&e);
            if eindex > mextent {
                self.my_blocks.push(eindex);
                let is_regular = self.grow_block(ss, eindex)?;
                self.my_blocks_is_regular
                    .push(if is_regular { 1 } else { 0 });
            }
            ss.next_start_element();
        }
        self.my_blocks
            .push(self.my_oriented_shape_map.len() as i32 + 1);
        self.my_is_done = true;
        Ok(())
    }

    /// `InitBlock()`; `bb.init_block()`.
    pub fn init_block(&mut self) {
        self.my_block_index = 1;
    }

    /// `MoreBlock()`; `bb.more_block()`.
    pub fn more_block(&self) -> bool {
        self.my_block_index < self.my_blocks.len() as i32
    }

    /// `NextBlock()`; `bb.next_block()`.
    pub fn next_block(&mut self) {
        self.my_block_index += 1;
    }

    /// The bound of index `i` (from 1) of the blocks; Standard_OutOfRange outside.
    fn block_bound(&self, i: i32) -> Result<i32, Failure> {
        usize::try_from(i - 1)
            .ok()
            .and_then(|k| self.my_blocks.get(k))
            .copied()
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "TopOpeBRepBuild_BlockBuilder: block {i}; expected 1..={}",
                    self.my_blocks.len()
                ))
            })
    }

    /// `BlockIterator()`: the indices of the current block; `bb.block_iterator()?`.
    pub fn block_iterator(&self) -> Result<TopOpeBRepBuildBlockIterator, Failure> {
        let lower = self.block_bound(self.my_block_index)?;
        let upper = self.block_bound(self.my_block_index + 1)? - 1;
        Ok(TopOpeBRepBuildBlockIterator::new(lower, upper))
    }

    /// `Element(BI)`: the element at the iterator; Standard_Failure past its end;
    /// `bb.element_at(&bi)?`.
    pub fn element_at(&self, bi: &TopOpeBRepBuildBlockIterator) -> Result<&Shape, Failure> {
        if !bi.more() {
            return Err(out_of_range("the block iterator is past its end"));
        }
        self.my_oriented_shape_map.find_key(bi.value() as usize)
    }

    /// `Element(I)`: the element of index `i`; Standard_Failure for an unknown index;
    /// `bb.element(1)?`.
    pub fn element(&self, index: i32) -> Result<&Shape, Failure> {
        if !self.my_oriented_shape_map_is_valid.is_bound(&index) {
            return Err(out_of_range(&format!("index {index}")));
        }
        self.my_oriented_shape_map.find_key(index as usize)
    }

    /// `Element(S)`: the index of the element (with its orientation);
    /// Standard_Failure for an unknown element; `bb.element_index(&edge)?`.
    pub fn element_index(&self, e: &Shape) -> Result<i32, Failure> {
        if !self.my_oriented_shape_map.contains(e) {
            return Err(out_of_range("unknown element"));
        }
        Ok(self.my_oriented_shape_map.find_index(e) as i32)
    }

    /// `ElementIsValid(BI)`: false past the end; Standard_NoSuchObject for an unknown
    /// index; `bb.element_is_valid_at(&bi)?`.
    pub fn element_is_valid_at(&self, bi: &TopOpeBRepBuildBlockIterator) -> Result<bool, Failure> {
        if !bi.more() {
            return Ok(false);
        }
        Ok(*self.my_oriented_shape_map_is_valid.find(&bi.value())? == 1)
    }

    /// `ElementIsValid(I)`: false for an unknown index; `bb.element_is_valid(1)`.
    pub fn element_is_valid(&self, sindex: i32) -> bool {
        self.my_oriented_shape_map_is_valid
            .seek(&sindex)
            .is_some_and(|&v| v == 1)
    }

    /// `AddElement(S)`: the index of the element, (re)marked valid;
    /// `bb.add_element(&edge)`.
    pub fn add_element(&mut self, s: &Shape) -> i32 {
        let sindex = self.my_oriented_shape_map.add(s.clone()) as i32;
        self.my_oriented_shape_map_is_valid.bind(sindex, 1);
        sindex
    }

    /// `SetValid(BI, isvalid)`: nothing past the end; `bb.set_valid_at(&bi, false)`.
    pub fn set_valid_at(&mut self, bi: &TopOpeBRepBuildBlockIterator, isvalid: bool) {
        if bi.more() {
            self.my_oriented_shape_map_is_valid
                .bind(bi.value(), if isvalid { 1 } else { 0 });
        }
    }

    /// `SetValid(I, isvalid)`: nothing for an unknown index; `bb.set_valid(1, false)`.
    pub fn set_valid(&mut self, sindex: i32, isvalid: bool) {
        if self.my_oriented_shape_map_is_valid.is_bound(&sindex) {
            self.my_oriented_shape_map_is_valid
                .bind(sindex, if isvalid { 1 } else { 0 });
        }
    }

    /// `CurrentBlockIsRegular()`; Standard_OutOfRange past the blocks;
    /// `bb.current_block_is_regular()?`.
    pub fn current_block_is_regular(&self) -> Result<bool, Failure> {
        let i = usize::try_from(self.my_block_index - 1)
            .ok()
            .and_then(|k| self.my_blocks_is_regular.get(k))
            .copied();
        Ok(i.ok_or_else(|| {
            Failure::out_of_range(&format!(
                "TopOpeBRepBuild_BlockBuilder::CurrentBlockIsRegular: block {}",
                self.my_block_index
            ))
        })? == 1)
    }
}
