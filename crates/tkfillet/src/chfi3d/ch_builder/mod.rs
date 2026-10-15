// Rust translation of ChFi3d_ChBuilder.hxx and the members of ChFi3d_ChBuilder.cxx,
// ChFi3d_ChBuilder_C2.cxx and ChFi3d_ChBuilder_C3.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi3d_ChBuilder`: the chamfer builder. It owns the fields of the root builder
//! ([`ChFi3dBuilder`]) and the mode of the new chamfers, and implements the virtual
//! members of the root ([`ChFi3dBuilderOps`]) with the chamfer functions of BlendFunc:
//! one distance, two distances (classic or constant throat with penetration), or a
//! distance and an angle. The chamfers on obstacles (restrictions) are not implemented
//! natively: those members keep the Standard_Failure of the root.
mod contours;
mod extent;
mod functions;
mod perform;
mod simul;
mod three_corner;
mod three_corner_plane;
mod three_corner_write;

use super::builder::{ChFi3dBuilder, ChFi3dBuilderOps, SurfCall, SurfOut};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{
    ChamfMode, ChamfSpine, ElSpine, SpineHandle, SpineObject, StripeHandle, SurfDataHandle,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::Vector;
use std::sync::MutexGuard;

/// The chamfer builder (`ChFi3d_ChBuilder`): the root builder and the mode of the
/// chamfers added.
///
/// ```
/// use rustycad_tkfillet::chfi3d::ChFi3dChBuilder;
/// use rustycad_tkfillet::chfi_ds::ChamfMode;
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone();
/// let b = ChFi3dChBuilder::new(&solid, 1.0e-2)?;
/// assert_eq!(b.mode(), ChamfMode::ClassicChamfer);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ChFi3dChBuilder {
    pub(crate) base: ChFi3dBuilder,
    pub(crate) my_mode: ChamfMode,
}

/// The chamfer spine of `spine` (`occ::down_cast<ChFiDS_ChamfSpine>`), locked;
/// Standard_ConstructionError with `message` when it is not a chamfer spine.
pub(crate) fn chamf_spine<'a>(
    guard: &'a MutexGuard<'_, dyn SpineObject>,
    message: &str,
) -> Result<&'a ChamfSpine, Failure> {
    guard
        .as_chamf_spine()
        .ok_or_else(|| Failure::with_message("Standard_ConstructionError", message))
}

impl ChFi3dChBuilder {
    /// `ChFi3d_ChBuilder(S, Ta)`: the chamfer builder of `s` with the angular tolerance
    /// `ta`, classic chamfers; `ChFi3dChBuilder::new(&solid, 1.0e-2)?`.
    pub fn new(s: &Shape, ta: f64) -> Result<Self, Failure> {
        Ok(Self {
            base: ChFi3dBuilder::new(s, ta)?,
            my_mode: ChamfMode::ClassicChamfer,
        })
    }

    /// The fields of the root builder; `b.builder_base().is_done()`.
    pub fn builder_base(&self) -> &ChFi3dBuilder {
        &self.base
    }

    /// The fields of the root builder, mutable.
    pub fn builder_base_mut(&mut self) -> &mut ChFi3dBuilder {
        &mut self.base
    }

    /// `Compute()`: the chamfers of all the contours, the corners and the result;
    /// `b.compute()?`.
    pub fn compute(&mut self) -> Result<(), Failure> {
        (self as &mut dyn ChFi3dBuilderOps).compute()
    }
}

impl ChFi3dBuilderOps for ChFi3dChBuilder {
    fn base(&self) -> &ChFi3dBuilder {
        &self.base
    }
    fn base_mut(&mut self) -> &mut ChFi3dBuilder {
        &mut self.base
    }
    fn simul_kpart(&self, sd: &SurfDataHandle) -> Result<(), Failure> {
        self.simul_kpart_impl(sd)
    }
    fn simul_surf(
        &mut self,
        data: &mut SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        self.simul_surf_impl(data, call, out)
    }
    fn perform_surf(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        self.perform_surf_impl(data, call, out)
    }
    /// `PerformTwoCorner(Index)`: the corner of two chamfers by intersection.
    fn perform_two_corner(&mut self, index: i32) -> Result<(), Failure> {
        self.base.perform_two_cornerby_inter(index)?;
        Ok(())
    }
    fn perform_three_corner(&mut self, index: i32) -> Result<(), Failure> {
        self.perform_three_corner_impl(index)
    }
    fn extent_one_corner(&mut self, v: &Vertex, s: &StripeHandle) -> Result<(), Failure> {
        self.extent_one_corner_impl(v, s)
    }
    fn extent_two_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure> {
        self.extent_two_corner_impl(v, ls)
    }
    fn extent_three_corner(&mut self, v: &Vertex, ls: &[StripeHandle]) -> Result<(), Failure> {
        self.extent_three_corner_impl(v, ls)
    }
    fn perform_first_section(
        &self,
        s: &SpineHandle,
        hguide: &Handle<ElSpine>,
        choix: i32,
        surfaces: (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
        domains: (&WalkingDomain, &WalkingDomain),
        par: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<(bool, State, State), Failure> {
        self.perform_first_section_impl(s, hguide, choix, surfaces, domains, par, soldep)
    }
    fn set_regul(&mut self) -> Result<(), Failure> {
        self.set_regul_impl()
    }
}
