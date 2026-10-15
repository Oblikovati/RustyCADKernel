//! The accessors of `TopOpeBRepBuild_Builder`: the build tool, the new geometries, the
//! split and merged maps, the operation predicates and the ranks of the shapes.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_ds::{
    MapOfShapeOn1State, TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSListOfShapeOn1State,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

/// Standard_OutOfRange of an array index.
fn out_of_range(what: &str, i: i32) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("{what}: index {i} out of range"),
        "NCollection_Array1",
    )
}

/// The index of an array of lower bound 0 (`NCollection_HArray1(0, n)`).
fn slot(i: i32, len: usize, what: &str) -> Result<usize, Failure> {
    usize::try_from(i)
        .ok()
        .filter(|k| *k < len)
        .ok_or_else(|| out_of_range(what, i))
}

impl TopOpeBRepBuildBuilder {
    /// `ChangeBuildTool()`; `b.change_build_tool()`.
    pub fn change_build_tool(&mut self) -> &mut TopOpeBRepDSBuildTool {
        &mut self.my_build_tool
    }

    /// `BuildTool()`; `b.build_tool()`.
    pub fn build_tool(&self) -> &TopOpeBRepDSBuildTool {
        &self.my_build_tool
    }

    /// `DataStructure()`: null until Perform; `b.data_structure()`.
    pub fn data_structure(&self) -> Handle<TopOpeBRepDSHDataStructure> {
        self.my_data_structure.clone()
    }

    /// `Classify()`; `b.classify()`.
    pub fn classify(&self) -> bool {
        self.my_classify_val
    }

    /// `ChangeClassify(classify)`: a defined classification flag; `b.change_classify(false)`.
    pub fn change_classify(&mut self, classify: bool) {
        self.my_classify_def = true;
        self.my_classify_val = classify;
    }

    /// `InitSection()` (TopOpeBRepBuild_Section.cxx): no section done and no section
    /// edge; `b.init_section()`.
    pub fn init_section(&mut self) {
        self.my_section_done = false;
        self.my_section.clear();
        self.my_split_section_edges_done = false;
        self.my_split_on.clear(true);
    }

    /// `NewFaces(I)`: the faces of the surface `i` (Standard_OutOfRange outside the
    /// surfaces); `b.new_faces(1)?`.
    pub fn new_faces(&self, i: i32) -> Result<&List<Shape>, Failure> {
        let k = slot(i, self.my_new_faces.len(), "NewFaces")?;
        Ok(&self.my_new_faces[k])
    }

    /// `ChangeNewFaces(I)`; `b.change_new_faces(1)?.append(f)`.
    pub fn change_new_faces(&mut self, i: i32) -> Result<&mut List<Shape>, Failure> {
        let k = slot(i, self.my_new_faces.len(), "ChangeNewFaces")?;
        Ok(&mut self.my_new_faces[k])
    }

    /// `NewEdges(I)`: the edges of the curve `i`, empty when none; `b.new_edges(1)`.
    pub fn new_edges(&self, i: i32) -> &List<Shape> {
        self.my_new_edges
            .find(&i)
            .unwrap_or(&self.my_empty_shape_list)
    }

    /// `ChangeNewEdges(I)`: the edges of the curve `i`, bound empty first;
    /// `b.change_new_edges(1).append(e)`.
    pub fn change_new_edges(&mut self, i: i32) -> &mut List<Shape> {
        if !self.my_new_edges.is_bound(&i) {
            self.my_new_edges.bind(i, List::default());
        }
        self.my_new_edges.change_find(&i).expect("bound above")
    }

    /// `NewVertex(I)`: the vertex of the point `i`; `b.new_vertex(1)?`.
    pub fn new_vertex(&self, i: i32) -> Result<&Shape, Failure> {
        let k = slot(i, self.my_new_vertices.len(), "NewVertex")?;
        Ok(&self.my_new_vertices[k])
    }

    /// `ChangeNewVertex(I)`; `*b.change_new_vertex(1)? = v`.
    pub fn change_new_vertex(&mut self, i: i32) -> Result<&mut Shape, Failure> {
        let k = slot(i, self.my_new_vertices.len(), "ChangeNewVertex")?;
        Ok(&mut self.my_new_vertices[k])
    }

    /// The split map of a state (none for another state than IN, OUT, ON).
    fn split_map(&self, to_build: State) -> Option<&MapOfShapeOn1State> {
        match to_build {
            State::Out => Some(&self.my_split_out),
            State::In => Some(&self.my_split_in),
            State::On => Some(&self.my_split_on),
            _ => None,
        }
    }

    /// The split map of a state, mutable.
    fn split_map_mut(&mut self, to_build: State) -> Option<&mut MapOfShapeOn1State> {
        match to_build {
            State::Out => Some(&mut self.my_split_out),
            State::In => Some(&mut self.my_split_in),
            State::On => Some(&mut self.my_split_on),
            _ => None,
        }
    }

    /// The list on one state of `s` in a map, bound empty first.
    pub(crate) fn bound_losos<'m>(
        m: &'m mut MapOfShapeOn1State,
        s: &Shape,
    ) -> &'m mut TopOpeBRepDSListOfShapeOn1State {
        if !m.is_bound(s) {
            m.bind(s.clone(), TopOpeBRepDSListOfShapeOn1State::default());
        }
        m.change_find(s).expect("bound above")
    }

    /// `ToSplit(S, ToBuild)`: not split yet and with geometry or same domain shapes;
    /// `b.to_split(&e, State::In)?`.
    pub fn to_split(&self, s: &Shape, to_build: State) -> Result<bool, Failure> {
        let issplit = self.is_split(s, to_build);
        let hds = self.hds()?;
        let hasgeom = hds.has_geometry(s);
        let hassame = hds.has_same_domain(s, true);
        Ok(!issplit && (hasgeom || hassame))
    }

    /// `MarkSplit(S, ToBuild, B)` (B natively defaulted to true): the split flag of `s`
    /// on the state; `b.mark_split(&e, State::In, true)`.
    pub fn mark_split(&mut self, s: &Shape, to_build: State, bval: bool) {
        if let Some(p) = self.split_map_mut(to_build) {
            Self::bound_losos(p, s).split(bval);
        }
    }

    /// `IsSplit(S, ToBuild)`; `b.is_split(&e, State::In)`.
    pub fn is_split(&self, s: &Shape, to_build: State) -> bool {
        self.split_map(to_build)
            .and_then(|p| p.find(s).ok())
            .is_some_and(|l| l.is_split())
    }

    /// `Splits(S, ToBuild)`: the split parts of `s` on the state, empty when none;
    /// `b.splits(&e, State::In)`.
    pub fn splits(&self, s: &Shape, to_build: State) -> &List<Shape> {
        match self.split_map(to_build).and_then(|p| p.find(s).ok()) {
            Some(l) => l.list_on_state(),
            None => &self.my_empty_shape_list,
        }
    }

    /// `ChangeSplit(S, ToBuild)`: the split parts of `s` on the state, bound empty
    /// first (the shared empty list for another state); `b.change_split(&e, State::In)`.
    pub fn change_split(&mut self, s: &Shape, to_build: State) -> &mut List<Shape> {
        if self.split_map(to_build).is_none() {
            return &mut self.my_empty_shape_list;
        }
        let p = self.split_map_mut(to_build).expect("checked above");
        Self::bound_losos(p, s).change_list_on_state()
    }

    /// `MSplit(s)`: the split map of the state (the IN map for another state);
    /// `b.msplit(State::On)`.
    pub fn msplit(&self, s: State) -> &MapOfShapeOn1State {
        self.split_map(s).unwrap_or(&self.my_split_in)
    }

    /// `ChangeMSplit(s)`; `b.change_msplit(State::On)`.
    pub fn change_msplit(&mut self, s: State) -> &mut MapOfShapeOn1State {
        match s {
            State::Out => &mut self.my_split_out,
            State::On => &mut self.my_split_on,
            _ => &mut self.my_split_in,
        }
    }

    /// `Reverse(ToBuild1, ToBuild2)`: whether the shapes of a part kept IN are
    /// reversed (not for a common); `TopOpeBRepBuildBuilder::reverse(State::In, State::Out)`.
    pub fn reverse(to_build1: State, to_build2: State) -> bool {
        if to_build1 == State::In && to_build2 == State::In {
            false
        } else {
            to_build1 == State::In
        }
    }

    /// `Orient(Ori, Reverse)`: the complement when reversed;
    /// `TopOpeBRepBuildBuilder::orient(Orientation::Forward, true)`.
    pub fn orient(ori: Orientation, reverse: bool) -> Orientation {
        if reverse {
            top_abs::complement(ori)
        } else {
            ori
        }
    }

    /// `Opec12()`: the operation is the cut of 1 by 2; `b.opec12()`.
    pub fn opec12(&self) -> bool {
        self.my_state1 == State::Out && self.my_state2 == State::In
    }

    /// `Opec21()`: the cut of 2 by 1; `b.opec21()`.
    pub fn opec21(&self) -> bool {
        self.my_state1 == State::In && self.my_state2 == State::Out
    }

    /// `Opecom()`: the common; `b.opecom()`.
    pub fn opecom(&self) -> bool {
        self.my_state1 == State::In && self.my_state2 == State::In
    }

    /// `Opefus()`: the fuse; `b.opefus()`.
    pub fn opefus(&self) -> bool {
        self.my_state1 == State::Out && self.my_state2 == State::Out
    }

    /// `TopType(S)`: the topmost type of a sub-shape of `s` (SHAPE when none);
    /// `TopOpeBRepBuildBuilder::top_type(&s)?`.
    pub fn top_type(s: &Shape) -> Result<ShapeEnum, Failure> {
        use ShapeEnum::*;
        for t in [Compound, CompSolid, Solid, Shell, Face, Wire, Edge, Vertex] {
            if TopOpeBRepToolShapeExplorer::new(s, t, Shape)?.more() {
                return Ok(t);
            }
        }
        Ok(Shape)
    }

    /// `MapShapes(S1, S2)`: the sub-shapes of the two shapes in the maps 1 and 2;
    /// `b.map_shapes(&s1, &s2)?`.
    pub fn map_shapes(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        self.clear_maps();
        if !s1.is_null() {
            topexp::map_shapes(s1, &mut self.my_map1, true, true);
        }
        if !s2.is_null() {
            topexp::map_shapes(s2, &mut self.my_map2, true, true);
        }
        Ok(())
    }

    /// `ClearMaps()`; `b.clear_maps()`.
    pub fn clear_maps(&mut self) {
        self.my_map1.clear(true);
        self.my_map2.clear(true);
    }

    /// `FindSameRank(L1, rank, L2)`: the shapes of `l1` of the rank appended to `l2`
    /// once; `b.find_same_rank(&l1, 1, &mut l2)`.
    pub fn find_same_rank(&self, l1: &List<Shape>, rank: i32, l2: &mut List<Shape>) {
        for s in l1.iter() {
            if self.shape_rank(s) == rank && !Self::contains(s, l2) {
                l2.append(s.clone());
            }
        }
    }

    /// `ShapeRank(s)`: 1 or 2 for a sub-shape of the first or second shape, else 0;
    /// `b.shape_rank(&s)`.
    pub fn shape_rank(&self, s: &Shape) -> i32 {
        if self.is_shape_of(s, 1) {
            1
        } else if self.is_shape_of(s, 2) {
            2
        } else {
            0
        }
    }

    /// `IsShapeOf(s, i)`: `s` is a sub-shape of the shape `i`; `b.is_shape_of(&s, 1)`.
    pub fn is_shape_of(&self, s: &Shape, i: i32) -> bool {
        match i {
            1 => self.my_map1.contains(s),
            2 => self.my_map2.contains(s),
            _ => false,
        }
    }

    /// `Contains(S, L)`: a shape of `l` is the same as `s`;
    /// `TopOpeBRepBuildBuilder::contains(&s, &l)`.
    pub fn contains(s: &Shape, l: &List<Shape>) -> bool {
        l.iter().any(|sl| sl.is_same(s))
    }
}
