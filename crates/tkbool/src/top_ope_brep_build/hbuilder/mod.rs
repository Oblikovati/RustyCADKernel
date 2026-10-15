// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_HBuilder.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_HBuilder`: the builder (a `TopOpeBRepBuild_Builder1`) shared
//! through handles, with the section iteration and the ancestors of the section edges.
//! Its fields are locked (a transient changed through any handle): the members
//! returning references natively return copies, and the builder is reached through
//! [`TopOpeBRepBuildHBuilder::change_builder`].
mod ancestors;

use super::TopOpeBRepBuildBuilder1;
use crate::top_ope_brep_ds::{TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::State;
use std::cell::RefCell;
use std::sync::RwLockWriteGuard;

thread_local! {
    /// `PLE`, `PITLE` (thread-local statics): the section list and the index of its
    /// iterator (None while the native pointers are null).
    static SECTION_ITERATION: RefCell<Option<(List<Shape>, usize)>> = const { RefCell::new(None) };
}

/// The fields of the shared builder.
pub struct HBuilderState {
    /// `myBuilder`.
    pub builder: TopOpeBRepBuildBuilder1,
    pub(crate) my_sect_edge_ds_edges1: DataMap<Shape, i32, ShapeMapHasher>,
    pub(crate) my_sect_edge_ds_edges2: DataMap<Shape, i32, ShapeMapHasher>,
    pub(crate) my_ds_edges_ds_faces1: DataMap<i32, List<i32>>,
    pub(crate) my_ds_edges_ds_faces2: DataMap<i32, List<i32>>,
    pub(crate) my_make_edge_ancestor_is_done: bool,
    pub(crate) my_sect_edge_ds_curve: DataMap<Shape, i32, ShapeMapHasher>,
    pub(crate) my_make_curve_ancestor_is_done: bool,
    pub(crate) my_new_vertex_ds_point: DataMap<Shape, i32, ShapeMapHasher>,
    pub(crate) my_make_point_ancestor_is_done: bool,
    pub(crate) my_empty_integer_list: List<i32>,
}

/// The shared builder of the topological operations (`TopOpeBRepBuild_HBuilder`);
/// `let hb = Handle::new(TopOpeBRepBuildHBuilder::new(&TopOpeBRepDSBuildTool::new()));`.
pub struct TopOpeBRepBuildHBuilder {
    parent: StandardTransient,
    my_state: SharedState<HBuilderState>,
}

impl_transient!(
    TopOpeBRepBuildHBuilder,
    "TopOpeBRepBuild_HBuilder",
    StandardTransient,
    parent
);

impl TopOpeBRepBuildHBuilder {
    /// `TopOpeBRepBuild_HBuilder(BT)`: a Builder1 on a copy of the build tool, no
    /// ancestor map done; `TopOpeBRepBuildHBuilder::new(&TopOpeBRepDSBuildTool::new())`.
    pub fn new(bt: &TopOpeBRepDSBuildTool) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_state: SharedState::new(HBuilderState {
                builder: TopOpeBRepBuildBuilder1::new(bt),
                my_sect_edge_ds_edges1: DataMap::default(),
                my_sect_edge_ds_edges2: DataMap::default(),
                my_ds_edges_ds_faces1: DataMap::default(),
                my_ds_edges_ds_faces2: DataMap::default(),
                my_make_edge_ancestor_is_done: false,
                my_sect_edge_ds_curve: DataMap::default(),
                my_make_curve_ancestor_is_done: false,
                my_new_vertex_ds_point: DataMap::default(),
                my_make_point_ancestor_is_done: false,
                my_empty_integer_list: List::default(),
            }),
        }
    }

    /// `ChangeBuilder()`: the builder, locked for change; `hb.change_builder().builder.is_k_part()`.
    pub fn change_builder(&self) -> RwLockWriteGuard<'_, HBuilderState> {
        self.my_state.write()
    }

    /// `BuildTool()`: a copy of the build tool; `hb.build_tool()`.
    pub fn build_tool(&self) -> TopOpeBRepDSBuildTool {
        *self.my_state.read().builder.build_tool()
    }

    /// `ChangeBuildTool()`: the build tool changed by `f`;
    /// `hb.change_build_tool(|bt| bt.set_overwrite(true))`.
    pub fn change_build_tool<R>(&self, f: impl FnOnce(&mut TopOpeBRepDSBuildTool) -> R) -> R {
        f(self.my_state.write().builder.change_build_tool())
    }

    /// `Perform(HDS)`; `hb.perform(&hds)?`.
    pub fn perform(&self, hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
        self.my_state.write().builder.perform(hds)
    }

    /// `Perform(HDS, S1, S2)`; `hb.perform_shapes(&hds, &s1, &s2)?`.
    pub fn perform_shapes(
        &self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        s1: &Shape,
        s2: &Shape,
    ) -> Result<(), Failure> {
        self.my_state.write().builder.perform_shapes(hds, s1, s2)
    }

    /// `Clear()`: the builder cleared, the ancestor maps to redo; `hb.clear()?`.
    pub fn clear(&self) -> Result<(), Failure> {
        let mut st = self.my_state.write();
        st.builder.clear()?;
        st.init_extended_section_ds(3);
        Ok(())
    }

    /// `DataStructure()`; `hb.data_structure()`.
    pub fn data_structure(&self) -> Handle<TopOpeBRepDSHDataStructure> {
        self.my_state.read().builder.data_structure()
    }

    /// `MergeShapes(S1, ToBuild1, S2, ToBuild2)`; `hb.merge_shapes(&s1, State::Out, &s2, State::In)?`.
    pub fn merge_shapes(
        &self,
        s1: &Shape,
        tb1: State,
        s2: &Shape,
        tb2: State,
    ) -> Result<(), Failure> {
        self.my_state.write().builder.merge_shapes(s1, tb1, s2, tb2)
    }

    /// `MergeSolids(S1, ToBuild1, S2, ToBuild2)`; `hb.merge_solids(&s1, State::Out, &s2, State::In)?`.
    pub fn merge_solids(
        &self,
        s1: &Shape,
        tb1: State,
        s2: &Shape,
        tb2: State,
    ) -> Result<(), Failure> {
        self.my_state.write().builder.merge_solids(s1, tb1, s2, tb2)
    }

    /// `MergeSolid(S, ToBuild)`; `hb.merge_solid(&s, State::Out)?`.
    pub fn merge_solid(&self, s: &Shape, tb: State) -> Result<(), Failure> {
        self.my_state.write().builder.merge_solid(s, tb)
    }

    /// `IsSplit(S, ToBuild)`; `hb.is_split(&e, State::In)`.
    pub fn is_split(&self, s: &Shape, tb: State) -> bool {
        self.my_state.read().builder.is_split(s, tb)
    }

    /// `Splits(S, ToBuild)`, copied; `hb.splits(&e, State::In)`.
    pub fn splits(&self, s: &Shape, tb: State) -> List<Shape> {
        self.my_state.read().builder.splits(s, tb).clone()
    }

    /// `IsMerged(S, ToBuild)`; `hb.is_merged(&s, State::In)`.
    pub fn is_merged(&self, s: &Shape, tb: State) -> bool {
        self.my_state.read().builder.is_merged(s, tb)
    }

    /// `Merged(S, ToBuild)`, copied; `hb.merged(&s, State::In)`.
    pub fn merged(&self, s: &Shape, tb: State) -> List<Shape> {
        self.my_state.read().builder.merged(s, tb).clone()
    }

    /// `NewVertex(I)`, copied; `hb.new_vertex(1)?`.
    pub fn new_vertex(&self, i: i32) -> Result<Shape, Failure> {
        self.my_state.read().builder.new_vertex(i).cloned()
    }

    /// `NewEdges(I)`, copied; `hb.new_edges(1)`.
    pub fn new_edges(&self, i: i32) -> List<Shape> {
        self.my_state.read().builder.new_edges(i).clone()
    }

    /// `ChangeNewEdges(I)`: the new edges of the curve changed by `f`;
    /// `hb.change_new_edges(1, |l| l.clear())`.
    pub fn change_new_edges<R>(&self, i: i32, f: impl FnOnce(&mut List<Shape>) -> R) -> R {
        f(self.my_state.write().builder.change_new_edges(i))
    }

    /// `NewFaces(I)`, copied; `hb.new_faces(1)?`.
    pub fn new_faces(&self, i: i32) -> Result<List<Shape>, Failure> {
        self.my_state.read().builder.new_faces(i).cloned()
    }

    /// `Section()`, copied; `hb.section()?`.
    pub fn section(&self) -> Result<List<Shape>, Failure> {
        self.my_state.write().builder.section().cloned()
    }

    /// `InitExtendedSectionDS(k)`: the curve (1), edge (2) or both (3) ancestor maps to
    /// redo; `hb.init_extended_section_ds(3)`.
    pub fn init_extended_section_ds(&self, k: i32) {
        self.my_state.write().init_extended_section_ds(k);
    }

    /// `InitSection(k)`: the section iteration started on the section curves (1),
    /// edges (2) or both (3), empty for another `k`; `hb.init_section(3)?`.
    pub fn init_section(&self, k: i32) -> Result<(), Failure> {
        SECTION_ITERATION.with_borrow_mut(|s| *s = Some((List::default(), 0)));
        let mut st = self.my_state.write();
        st.init_extended_section_ds(k);
        let mut ple = List::default();
        match k {
            1 => st.builder.section_curves(&mut ple)?,
            2 => st.builder.section_edges(&mut ple)?,
            3 => st.builder.section_into(&mut ple)?,
            _ => return Ok(()),
        }
        SECTION_ITERATION.with_borrow_mut(|s| *s = Some((ple, 0)));
        Ok(())
    }

    /// `MoreSection()`; `hb.more_section()`.
    pub fn more_section(&self) -> bool {
        SECTION_ITERATION.with_borrow(|s| s.as_ref().is_some_and(|(l, i)| *i < l.len()))
    }

    /// `NextSection()`; `hb.next_section()`.
    pub fn next_section(&self) {
        SECTION_ITERATION.with_borrow_mut(|s| {
            if let Some((l, i)) = s.as_mut()
                && *i < l.len()
            {
                *i += 1;
            }
        });
    }

    /// `CurrentSection()` (Standard_ProgramError past the end); `hb.current_section()?`.
    pub fn current_section(&self) -> Result<Shape, Failure> {
        SECTION_ITERATION
            .with_borrow(|s| s.as_ref().and_then(|(l, i)| l.iter().nth(*i).cloned()))
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_ProgramError",
                    "no more CurrentSection",
                    "TopOpeBRepBuild_HBuilder",
                )
            })
    }

    /// `IsKPart()`; `hb.is_k_part()`.
    pub fn is_k_part(&self) -> i32 {
        self.my_state.read().builder.is_k_part()
    }

    /// `MergeKPart(TB1, TB2)`: the particular case merged when there is one;
    /// `hb.merge_k_part(State::Out, State::In)?`.
    pub fn merge_k_part(&self, tb1: State, tb2: State) -> Result<(), Failure> {
        if self.is_k_part() != 0 {
            self.my_state
                .write()
                .builder
                .merge_k_part_states(tb1, tb2)?;
        }
        Ok(())
    }
}

impl HBuilderState {
    /// `InitExtendedSectionDS(k)` on the locked fields.
    fn init_extended_section_ds(&mut self, k: i32) {
        match k {
            1 => self.my_make_curve_ancestor_is_done = false,
            2 => self.my_make_edge_ancestor_is_done = false,
            3 => {
                self.my_make_edge_ancestor_is_done = false;
                self.my_make_curve_ancestor_is_done = false;
            }
            _ => {}
        }
    }
}
