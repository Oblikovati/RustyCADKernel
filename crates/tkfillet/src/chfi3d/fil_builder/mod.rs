// Rust translation of ChFi3d_FilBuilder.hxx and the members of ChFi3d_FilBuilder.cxx,
// ChFi3d_FilBuilder_C2.cxx and ChFi3d_FilBuilder_C3.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi3d_FilBuilder`: the fillet builder. It owns the fields of the root builder
//! ([`ChFi3dBuilder`]) and the shape of the sections, and implements the virtual members
//! of the root ([`ChFi3dBuilderOps`]) with the fillet functions of BlendFunc and
//! BRepBlend: constant radius, or a law of evolution of the radius.
mod contours;
mod extent;
mod functions;
mod perform;
mod perform_rst;
mod simul;
mod simul_rst;
mod split;
mod three_corner;
mod two_corner;

use super::builder::{ChFi3dBuilder, ChFi3dBuilderOps, RstCall, SurfCall, SurfOut};
use super::fillet_shape::FilletShape;
use crate::blend_func::SectionShape;
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ElSpine, FilSpine, SpineHandle, StripeHandle, SurfDataHandle, lock};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::Vector;
use std::sync::MutexGuard;

/// The fillet builder (`ChFi3d_FilBuilder`): the root builder and the shape of the
/// sections of the fillets.
///
/// ```
/// use rustycad_tkfillet::chfi3d::{ChFi3dFilBuilder, FilletShape};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone();
/// let b = ChFi3dFilBuilder::new(&solid, FilletShape::Rational, 1.0e-2)?;
/// assert_eq!(b.get_fillet_shape(), FilletShape::Rational);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ChFi3dFilBuilder {
    pub(crate) base: ChFi3dBuilder,
    /// `myShape`: the shape of the sections (BlendFunc_SectionShape).
    pub(crate) section_shape: SectionShape,
}

/// Standard_ConstructionError: the spine of a contour is not a fillet spine.
pub(crate) fn not_fillet_spine(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

/// The fillet spine of `spine` (`occ::down_cast<ChFiDS_FilSpine>`), locked;
/// Standard_ConstructionError with `message` when it is not a fillet spine.
pub(crate) fn fil_spine<'a>(
    guard: &'a MutexGuard<'_, dyn crate::chfi_ds::SpineObject>,
    message: &str,
) -> Result<&'a FilSpine, Failure> {
    guard
        .as_fil_spine()
        .ok_or_else(|| not_fillet_spine(message))
}

/// Whether the spine of the contour is a fillet spine of constant radius, and that
/// radius (`fsp->IsConstant()`, `fsp->Radius()`; the radius is 0 when not constant).
pub(crate) fn constant_radius(spine: &SpineHandle, message: &str) -> Result<Option<f64>, Failure> {
    let g = lock(spine);
    let fsp = fil_spine(&g, message)?;
    if fsp.is_constant() {
        return Ok(Some(fsp.radius()?));
    }
    Ok(None)
}

impl ChFi3dFilBuilder {
    /// `ChFi3d_FilBuilder(S, FShape, Ta)`: the fillet builder of `s` with the shape of
    /// its sections and the angular tolerance `ta`;
    /// `ChFi3dFilBuilder::new(&solid, FilletShape::Rational, 1.0e-2)?`.
    pub fn new(s: &Shape, fshape: FilletShape, ta: f64) -> Result<Self, Failure> {
        let mut b = Self {
            base: ChFi3dBuilder::new(s, ta)?,
            section_shape: SectionShape::Rational,
        };
        b.set_fillet_shape(fshape);
        Ok(b)
    }

    /// The fields of the root builder; `b.builder_base().is_done()`.
    pub fn builder_base(&self) -> &ChFi3dBuilder {
        &self.base
    }

    /// The fields of the root builder, mutable; `b.builder_base_mut().set_params(...)`.
    pub fn builder_base_mut(&mut self) -> &mut ChFi3dBuilder {
        &mut self.base
    }

    /// `SetFilletShape(FShape)`: the shape of the sections;
    /// `b.set_fillet_shape(FilletShape::Polynomial)`.
    pub fn set_fillet_shape(&mut self, fshape: FilletShape) {
        self.section_shape = match fshape {
            FilletShape::Rational => SectionShape::Rational,
            FilletShape::QuasiAngular => SectionShape::QuasiAngular,
            FilletShape::Polynomial => SectionShape::Polynomial,
        };
    }

    /// `GetFilletShape()`: the shape of the sections (rational for a linear one);
    /// `b.get_fillet_shape()`.
    pub fn get_fillet_shape(&self) -> FilletShape {
        match self.section_shape {
            SectionShape::QuasiAngular => FilletShape::QuasiAngular,
            SectionShape::Polynomial => FilletShape::Polynomial,
            _ => FilletShape::Rational,
        }
    }

    /// `Compute()`: the fillets of all the contours, the corners and the result;
    /// `b.compute()?`.
    pub fn compute(&mut self) -> Result<(), Failure> {
        (self as &mut dyn ChFi3dBuilderOps).compute()
    }
}

impl ChFi3dBuilderOps for ChFi3dFilBuilder {
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
    fn simul_surf_rst1(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.simul_surf_on_rst(data, call, out, decroch, true)
    }
    fn simul_surf_rst2(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.simul_surf_on_rst(data, call, out, decroch, false)
    }
    fn simul_surf_rst_rst(
        &mut self,
        data: &mut SurfDataHandle,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.simul_surf_rst_rst_impl(data, call, out, decroch)
    }
    fn perform_surf(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        self.perform_surf_impl(data, call, out)
    }
    fn perform_surf_rst1(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.perform_surf_on_rst(data, call, out, decroch, true)
    }
    fn perform_surf_rst2(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.perform_surf_on_rst(data, call, out, decroch, false)
    }
    fn perform_surf_rst_rst(
        &mut self,
        data: &mut Vec<SurfDataHandle>,
        call: &RstCall<'_>,
        out: &mut SurfOut,
        decroch: &mut [bool; 2],
    ) -> Result<(), Failure> {
        self.perform_surf_rst_rst_impl(data, call, out, decroch)
    }
    fn perform_two_corner(&mut self, index: i32) -> Result<(), Failure> {
        self.perform_two_corner_impl(index)
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
