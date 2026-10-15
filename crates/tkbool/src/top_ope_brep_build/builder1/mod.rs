// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Builder1.hxx/.cxx,
// TopOpeBRepBuild_Builder1_1.cxx and TopOpeBRepBuild_Builder1_2.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_Builder1`: the builder with the states of the shapes of the
//! operation (`GLOBAL_USE_NEW_BUILDER`, never set natively) and the 2d corrections of
//! the faces it makes. Its virtual members are reached from the base builder through
//! its data (`b1`), its own members are methods of the builder.
mod correct;
mod not_same_dom;
mod on_parts;
mod pieces_on;
mod same2d;
mod same_dom;
mod same_dom_edge;
mod split_edge;
mod states;

pub use pieces_on::TwoPiecesSeq;

use super::builder::DataMapOfShapeListOfShape;
use super::globals::GLOBAL_USE_NEW_BUILDER;
use super::{
    MapOfTrans2dInfo, TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildTools, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_ds::{TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

thread_local! {
    /// `mySDEdgeMap` (static of Builder1.cxx): the edges of the same domain faces.
    static SD_EDGE_MAP: RefCell<IndexedMapOfShape> = RefCell::new(IndexedMapOfShape::default());
    /// `processedEdges` (static of Builder1_1.cxx).
    static PROCESSED_EDGES: RefCell<IndexedMapOfShape> = RefCell::new(IndexedMapOfShape::default());
    /// `theUsedVertexMap` (static of Builder1_1.cxx).
    static USED_VERTEX_MAP: RefCell<IndexedMapOfShape> = RefCell::new(IndexedMapOfShape::default());
    /// `theUnkStateVer` (static of Builder1_1.cxx).
    static UNK_STATE_VER: RefCell<MapOfShape> = RefCell::new(MapOfShape::default());
}

/// The members of `TopOpeBRepBuild_Builder1`.
#[derive(Default)]
pub(crate) struct Builder1Data {
    /// `myFSplits` (declared, unused natively).
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_f_splits: DataMapOfShapeListOfShape,
    /// `myESplits` (declared, unused natively).
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_e_splits: DataMapOfShapeListOfShape,
    pub(crate) my_same_dom_map: IndexedMapOfShape,
    pub(crate) my_sd_face_to_fill: Shape,
    pub(crate) my_base_face_to_fill: Shape,
    pub(crate) my_map_of_edge_faces: IndexedDataMapOfShapeListOfShape,
    /// `myMapOfEdgeWithFaceState` (oriented keys).
    pub(crate) my_map_of_edge_with_face_state: DataMap<Shape, bool>,
    pub(crate) my_processed_parts_out2d: IndexedMapOfShape,
    pub(crate) my_processed_parts_on2d: IndexedMapOfShape,
    pub(crate) my_splits_on_to_keep: IndexedMapOfShape,
    /// `mySourceShapes` (oriented keys).
    pub(crate) my_source_shapes: IndexedMap<Shape>,
    pub(crate) my_map_of_correct2d_edges: MapOfTrans2dInfo,
}

impl Drop for Builder1Data {
    /// `~TopOpeBRepBuild_Builder1()`: the static maps of Builder1_1.cxx cleared.
    fn drop(&mut self) {
        PROCESSED_EDGES.with_borrow_mut(|m| m.clear(true));
        USED_VERTEX_MAP.with_borrow_mut(|m| m.clear(true));
        UNK_STATE_VER.with_borrow_mut(|m| m.clear(true));
    }
}

/// The builder with the states of the shapes (`TopOpeBRepBuild_Builder1`), a builder
/// whose virtual members are those of Builder1;
/// `let mut b = TopOpeBRepBuildBuilder1::new(&TopOpeBRepDSBuildTool::new());`.
pub struct TopOpeBRepBuildBuilder1(TopOpeBRepBuildBuilder);

impl TopOpeBRepBuildBuilder1 {
    /// `TopOpeBRepBuild_Builder1(BT)`: the builder with empty Builder1 maps;
    /// `TopOpeBRepBuildBuilder1::new(&TopOpeBRepDSBuildTool::new())`.
    pub fn new(bt: &TopOpeBRepDSBuildTool) -> Self {
        let mut b = TopOpeBRepBuildBuilder::new(bt);
        b.b1 = Some(Box::default());
        Self(b)
    }
}

impl Deref for TopOpeBRepBuildBuilder1 {
    type Target = TopOpeBRepBuildBuilder;
    fn deref(&self) -> &TopOpeBRepBuildBuilder {
        &self.0
    }
}

impl DerefMut for TopOpeBRepBuildBuilder1 {
    fn deref_mut(&mut self) -> &mut TopOpeBRepBuildBuilder {
        &mut self.0
    }
}

/// Standard_NullObject for a Builder1 member called on a plain builder.
fn not_builder1() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "not a TopOpeBRepBuild_Builder1",
        "TopOpeBRepBuild_Builder1",
    )
}

/// The first shape of a list (Standard_NoSuchObject when empty, as the native
/// iterator value).
pub(crate) fn first(l: &List<Shape>) -> Result<Shape, Failure> {
    l.first().cloned().ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "empty list",
            "TopOpeBRepBuild_Builder1",
        )
    })
}

impl TopOpeBRepBuildBuilder {
    /// The Builder1 members (Standard_NullObject for a plain builder).
    pub(crate) fn b1d(&self) -> Result<&Builder1Data, Failure> {
        self.b1.as_deref().ok_or_else(not_builder1)
    }

    /// The Builder1 members, mutable.
    pub(crate) fn b1d_mut(&mut self) -> Result<&mut Builder1Data, Failure> {
        self.b1.as_deref_mut().ok_or_else(not_builder1)
    }

    /// The Builder1 maps cleared at the start of Perform.
    fn b1_clear_maps(&mut self) -> Result<(), Failure> {
        let d = self.b1d_mut()?;
        d.my_same_dom_map.clear(true);
        d.my_map_of_edge_faces.clear(true);
        d.my_splits_on_to_keep.clear(true);
        d.my_processed_parts_out2d.clear(true);
        d.my_processed_parts_on2d.clear(true);
        Ok(())
    }

    /// `TopOpeBRepBuild_Builder1::Clear()`: the base Clear.
    pub(crate) fn b1_clear(&mut self) -> Result<(), Failure> {
        self.clear_base()
    }

    /// `TopOpeBRepBuild_Builder1::Perform(HDS)`: the base Perform.
    pub(crate) fn b1_perform(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        self.perform_base(hds)
    }

    /// `TopOpeBRepBuild_Builder1::Perform(HDS, S1, S2)`: the base one unless the new
    /// builder is on; then the shapes set, the particular case found (kole and
    /// koletge as soso) and, for soso, the states of the shapes computed.
    pub(crate) fn b1_perform_shapes(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        s1: &Shape,
        s2: &Shape,
    ) -> Result<(), Failure> {
        if !GLOBAL_USE_NEW_BUILDER.get() {
            return self.perform_shapes_base(hds, s1, s2);
        }
        self.b1_clear_maps()?;
        self.my_shape1 = s1.clone();
        self.my_shape2 = s2.clone();
        self.perform(hds)?;
        self.my_is_k_part = self.find_is_k_part()?;
        if self.my_is_k_part == 1 || self.my_is_k_part == 5 {
            self.my_is_k_part = 4;
        }
        if self.my_is_k_part == 4 {
            self.perform_shape_with_states()?;
        }
        Ok(())
    }

    /// `TopOpeBRepBuild_Builder1::MergeKPart()`: the merge of the particular case; for
    /// soso the merged solids of the first shape corrected in 2d; then End.
    pub(crate) fn b1_merge_k_part(&mut self) -> Result<(), Failure> {
        match self.my_is_k_part {
            1 => self.merge_k_part_iskole()?,
            5 => self.merge_k_part_iskoletge()?,
            2 => self.merge_k_part_isdisj()?,
            3 => self.merge_k_part_isfafa()?,
            4 => {
                self.merge_k_part_issoso()?;
                let (s1, st1) = (self.my_shape1.clone(), self.my_state1);
                if self.merged(&s1, st1).is_empty() {
                    return self.end();
                }
                let mut merged = std::mem::take(self.change_merged(&s1, st1));
                let r = merged
                    .iter_mut()
                    .try_for_each(|s| self.correct_result2d(s).map(|_| ()));
                *self.change_merged(&s1, st1) = merged;
                r?;
            }
            _ => {}
        }
        self.end()
    }

    /// `TopOpeBRepBuild_Builder1::GFillSolidSFS(SO1, LSO2, G1, SFS)`: the base one unless
    /// the new builder is on; then the shells out of the structure added by their
    /// state, the others filled by their faces.
    pub(crate) fn b1_g_fill_solid_sfs(
        &mut self,
        so1: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        if !GLOBAL_USE_NEW_BUILDER.get() {
            return self.g_fill_solid_sfs_base(so1, lso2, g1, sfs);
        }
        let mut efm = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.my_shape1,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut efm,
        )?;
        topexp::map_shapes_and_ancestors(
            &self.my_shape2,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut efm,
        )?;
        self.b1d_mut()?.my_map_of_edge_faces = efm;
        let (tb1, _) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        self.my_solid_to_fill = topods::solid(so1)?;
        let mut ex_shell =
            TopOpeBRepToolShapeExplorer::new(so1, ShapeEnum::Shell, ShapeEnum::Shape)?;
        while ex_shell.more() {
            let sh = ex_shell.current().clone();
            if !self.hds()?.has_shape(&sh, true) {
                if self.hds()?.ds().get_shape_with_state(&sh).state() == tb1 {
                    sfs.add_shape(&sh.oriented(Self::orient(sh.orientation(), rev_ori1)))?;
                }
            } else {
                self.g_fill_shell_sfs(&sh, lso2, g1, sfs)?;
            }
            ex_shell.next()?;
        }
        Ok(())
    }

    /// `TopOpeBRepBuild_Builder1::GFillShellSFS(SH, LSO2, G1, SFS)`: the base one unless
    /// the new builder is on; then the faces out of the structure added by their state,
    /// the same domain faces (not yet done) filled, then the other faces.
    pub(crate) fn b1_g_fill_shell_sfs(
        &mut self,
        sh: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        if !GLOBAL_USE_NEW_BUILDER.get() {
            return self.g_fill_shell_sfs_base(sh, lso2, g1, sfs);
        }
        let (tb1, _) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        let mut ex_face = TopOpeBRepToolShapeExplorer::new(sh, ShapeEnum::Face, ShapeEnum::Shape)?;
        while ex_face.more() {
            let f = ex_face.current().clone();
            if !self.hds()?.has_shape(&f, true) {
                if self.hds()?.ds().get_shape_with_state(&f).state() == tb1 {
                    sfs.add_element(&f.oriented(Self::orient(f.orientation(), rev_ori1)))?;
                }
            } else if self.hds()?.has_same_domain(&f, true)
                && !self.b1d()?.my_same_dom_map.contains(&f)
            {
                self.g_fill_face_same_dom_sfs(&f, lso2, g1, sfs)?;
            }
            ex_face.next()?;
        }
        let mut ex_face = TopOpeBRepToolShapeExplorer::new(sh, ShapeEnum::Face, ShapeEnum::Shape)?;
        while ex_face.more() {
            let f = ex_face.current().clone();
            if self.hds()?.has_shape(&f, true) && !self.hds()?.has_same_domain(&f, true) {
                self.g_fill_face_not_same_dom_sfs(&f, lso2, g1, sfs)?;
            }
            ex_face.next()?;
        }
        Ok(())
    }

    /// `TopOpeBRepBuild_Builder1::GWESMakeFaces(FF, WES, LOF)`: the base faces, normalized
    /// (no INTERNAL or EXTERNAL edges) for soso, then corrected in 2d.
    pub(crate) fn b1_gwes_make_faces(
        &mut self,
        ff: &Shape,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        lof: &mut List<Shape>,
    ) -> Result<(), Failure> {
        self.gwes_make_faces_base(ff, wes, lof)?;
        let mut corr = List::default();
        for f in lof.iter() {
            corr.append(if self.my_is_k_part == 4 {
                TopOpeBRepBuildTools::normalize_face(f)?
            } else {
                f.clone()
            });
        }
        let mut corr1 = List::default();
        let d = self.b1d_mut()?;
        for f in corr.iter() {
            corr1.append(TopOpeBRepBuildTools::correct_face2d(
                f,
                &d.my_source_shapes,
                &mut d.my_map_of_correct2d_edges,
            )?);
        }
        *lof = corr1;
        Ok(())
    }

    /// The state, the reversal and the GTopo (permuted for a reversed tool) of the
    /// shapes of a rank: the object's, else the tool's.
    pub(crate) fn rank_states(
        g1: &TopOpeBRepBuildGTopo,
        iref: i32,
    ) -> Result<(rustycad_tkg3d::top_abs::State, bool), Failure> {
        let (tb1, tb2) = g1.states_on()?;
        Ok(if iref == 1 {
            (tb1, g1.is_to_reverse1()?)
        } else {
            (tb2, g1.is_to_reverse2()?)
        })
    }

    /// `Orient(ori, rev)` of a shape.
    pub(crate) fn oriented_by(s: &Shape, rev: bool) -> Shape {
        s.oriented(Self::orient(s.orientation(), rev))
    }
}
