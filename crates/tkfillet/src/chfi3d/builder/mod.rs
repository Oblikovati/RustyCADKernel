// Rust translation of ChFi3d_Builder.hxx and the members of ChFi3d_Builder.cxx,
// ChFi3d_Builder_1.cxx, ChFi3d_Builder_2.cxx, ChFi3d_Builder_6.cxx, ChFi3d_Builder_C1.cxx,
// ChFi3d_Builder_C2.cxx, ChFi3d_Builder_CnCrn.cxx, ChFi3d_Builder_SpKP.cxx and
// ChFi3d_Builder_NotImp.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi3d_Builder`: the root of the fillet and chamfer builders. The fields are
//! [`ChFi3dBuilder`]; the virtual members are the [`ChFi3dBuilderOps`] trait, implemented
//! by the fillet and the chamfer builders, and the members that call them are methods of
//! `dyn ChFi3dBuilderOps`.
//!
//! The data structure is owned by the builder (`ds`) while the fillets are computed, and
//! moved into the `TopOpeBRepDS_HDataStructure` handle (`my_ds`) when the solid is rebuilt
//! by `TopOpeBRepBuild_HBuilder`.
mod accessors;
mod args;
mod at_end;
mod at_end_chain;
mod at_end_cut;
mod at_end_cut_last;
mod at_end_finish;
mod at_end_write;
mod call_perform;
mod compute;
mod compute_vertex;
mod corner_tools;
mod data_complete;
mod data_compute;
mod data_compute_ends;
mod data_rst;
mod data_store;
mod element;
mod element_extend;
mod ends;
mod extremity;
mod faces_search;
mod find_face;
mod kgen;
mod kpart_set;
mod on_el_spine;
mod on_el_spine_run;
mod on_el_spine_walk;
mod one_corner;
mod one_corner_cork;
mod one_corner_prol;
mod one_corner_write;
mod pending_corners;
mod seq_edit;
mod set_of_surf;
mod spkp;
mod spkp_fill;
mod spkp_place;
mod start_obstacle;
mod start_obstacle_end;
mod start_sol;
mod trunc;
mod two_corner_inter;
mod two_corner_inter_big;

pub use args::{BlendSide, RstCall, RstRef, SurfCall, SurfOut, WalkSettings};
pub(crate) use compute::spine_of;
pub(crate) use data_complete::LineFunction;
pub(crate) use data_compute::ComputeCall;
pub(crate) use data_rst::{RstSide, RstWalk};
pub(crate) use two_corner_inter::{CornerEnd, two_stripes_at};

use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::ElSpine;
use crate::chfi_ds::{Map, Regul, SpineHandle, StripeHandle, StripeMap, SurfDataHandle};
use rustycad_tkbool::top_ope_brep_build::TopOpeBRepBuildHBuilder;
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::math::Vector;

/// The fields of `ChFi3d_Builder`: the tolerances, the ancestor maps of the shape, the
/// data structure and its builder, the contours (stripes) and the results.
///
/// ```
/// use rustycad_tkfillet::chfi3d::ChFi3dBuilder;
/// assert!(std::mem::size_of::<ChFi3dBuilder>() > 0);
/// ```
pub struct ChFi3dBuilder {
    pub(crate) tolappangle: f64,
    pub(crate) tolesp: f64,
    pub(crate) tol2d: f64,
    pub(crate) tolapp3d: f64,
    pub(crate) tolapp2d: f64,
    pub(crate) fleche: f64,
    pub(crate) my_conti: Continuity,
    pub(crate) my_ef_map: Map,
    pub(crate) my_eso_map: Map,
    pub(crate) my_esh_map: Map,
    pub(crate) my_vf_map: Map,
    pub(crate) my_ve_map: Map,
    /// The data structure being filled (moved into `my_ds` when the solid is rebuilt).
    pub(crate) ds: TopOpeBRepDSDataStructure,
    pub(crate) my_ds: Handle<TopOpeBRepDSHDataStructure>,
    /// The data structure was moved into `my_ds`.
    pub(crate) ds_moved: bool,
    pub(crate) my_coup: Handle<TopOpeBRepBuildHBuilder>,
    pub(crate) my_list_stripe: Vec<StripeHandle>,
    pub(crate) my_vdata_map: StripeMap,
    pub(crate) my_regul: Vec<Regul>,
    pub(crate) badstripes: Vec<StripeHandle>,
    pub(crate) badvertices: Vec<Shape>,
    pub(crate) my_evi_map: DataMap<Shape, Vec<i32>, ShapeMapHasher>,
    pub(crate) my_edge_first_face: DataMap<Shape, Shape, ShapeMapHasher>,
    pub(crate) done: bool,
    pub(crate) hasresult: bool,
    pub(crate) my_shape: Shape,
    pub(crate) angular: f64,
    pub(crate) my_generated: List<Shape>,
    pub(crate) my_shape_result: Shape,
    pub(crate) bad_shape: Shape,
}

/// The virtual members of `ChFi3d_Builder`, implemented by the fillet and the chamfer
/// builders (`base` and `base_mut` give the fields). The overloads of `SimulSurf` and
/// `PerformSurf` on restrictions raise `Standard_Failure` unless overridden, as
/// ChFi3d_Builder_NotImp.cxx.
pub trait ChFi3dBuilderOps {
    /// The fields of the builder.
    fn base(&self) -> &ChFi3dBuilder;
    /// The fields of the builder, mutable.
    fn base_mut(&mut self) -> &mut ChFi3dBuilder;
    /// `SimulKPart(SD)`: the sections of a particular case for the simulation.
    fn simul_kpart(&self, sd: &SurfDataHandle) -> Result<(), Failure>;
    /// `SimulSurf(Data, Guide, Spine, Choix, S1, I1, S2, I2, TolGuide, First, Last, Inside,
    /// Appro, Forward, RecOnS1, RecOnS2, Soldep, Intf, Intl)`.
    fn simul_surf(
        &mut self,
        data: &mut SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure>;
    /// `SimulSurf` with the first side on a restriction (`PC1`, `Sref1`, `PCref1`,
    /// `Decroch1`) and the second on a face of orientation `Or2`.
    fn simul_surf_rst1(
        &mut self,
        _data: &mut SurfDataHandle,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("SimulSurf Not Implemented"))
    }
    /// `SimulSurf` with the second side on a restriction.
    fn simul_surf_rst2(
        &mut self,
        _data: &mut SurfDataHandle,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("SimulSurf Not Implemented"))
    }
    /// `SimulSurf` with both sides on restrictions.
    fn simul_surf_rst_rst(
        &mut self,
        _data: &mut SurfDataHandle,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("SimulSurf Not Implemented"))
    }
    /// `PerformSurf(Data, Guide, Spine, Choix, S1, I1, S2, I2, MaxStep, Fleche, TolGuide,
    /// First, Last, Inside, Appro, Forward, RecOnS1, RecOnS2, Soldep, Intf, Intl)`.
    fn perform_surf(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure>;
    /// `PerformSurf` with the first side on a restriction.
    fn perform_surf_rst1(
        &mut self,
        _data: &mut Vec<SurfDataHandle>,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("PerformSurf Not Implemented"))
    }
    /// `PerformSurf` with the second side on a restriction.
    fn perform_surf_rst2(
        &mut self,
        _data: &mut Vec<SurfDataHandle>,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("PerformSurf Not Implemented"))
    }
    /// `PerformSurf` with both sides on restrictions.
    fn perform_surf_rst_rst(
        &mut self,
        _data: &mut Vec<SurfDataHandle>,
        _call: &RstCall<'_>,
        _out: &mut SurfOut,
        _decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        Err(Failure::new("PerformSurf Not Implemented"))
    }
    /// `PerformTwoCorner(Index)`: the corner of two contours at a vertex.
    fn perform_two_corner(&mut self, index: i32) -> Result<(), Failure>;
    /// `PerformThreeCorner(Index)`: the corner of three contours at a vertex.
    fn perform_three_corner(&mut self, index: i32) -> Result<(), Failure>;
    /// `ExtentOneCorner(V, S)`: the extension of a contour ending alone at a vertex.
    fn extent_one_corner(&mut self, v: &Vertex, s: &StripeHandle) -> Result<(), Failure>;
    /// `ExtentTwoCorner(V, LS)`: the extensions of two contours meeting at a vertex.
    fn extent_two_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure>;
    /// `ExtentThreeCorner(V, LS)`: the extensions of three contours meeting at a vertex.
    fn extent_three_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure>;
    /// `PerformFirstSection(S, HGuide, Choix, S1, S2, I1, I2, Par, SolDep, Pos1, Pos2)`:
    /// the first section at `par`, its solution in `soldep` and the states of its ends.
    #[allow(clippy::too_many_arguments)]
    fn perform_first_section(
        &self,
        s: &SpineHandle,
        hguide: &Handle<ElSpine>,
        choix: i32,
        surfaces: (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        domains: (&WalkingDomain, &WalkingDomain),
        par: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<(bool, State, State), Failure>;
    /// `SetRegul()`: the continuities of the edges between the fillets and the faces.
    fn set_regul(&mut self) -> Result<(), Failure>;
}
