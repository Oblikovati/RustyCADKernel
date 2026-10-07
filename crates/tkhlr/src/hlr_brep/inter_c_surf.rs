//! The line-surface intersection of HLRBRep: HLRBRep_InterCSurf and its helper
//! classes, the instances of the IntCurveSurface, IntImp and Intf templates on
//! [`LineTool`] (a sight line) and [`SurfaceTool`] (HLRBRep_InterCSurf.cxx and the
//! `*OfInterCSurf*` files), OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! docs/geomalgo-design.md sections 2, 6 and 11 describe the generics.
use super::{LineTool, Surface, SurfaceTool};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::int_curve_surface::{
    InterGen, InterPolygon, InterPolygonTool, InterTypes, QuadCurvExact, QuadCurvFunc,
    SamplePolyhedron, ThePolyhedronOfHInter, ThePolyhedronToolOfHInter,
};
use rustycad_tkgeomalgo::int_imp::{IntImpIntCS, IntImpZerCSParFunc};
use rustycad_tkgeomalgo::intf::InterferencePolygonPolyhedron;
use rustycad_tkmath::bnd::BoundingBox3;
use std::marker::PhantomData;

/// `HLRBRep_ThePolygonOfInterCSurf`: the polygon of a sight line
/// (IntCurveSurface_PolygonUtils on HLRBRep_LineTool).
pub type ThePolygonOfInterCSurf = InterPolygon<LineTool>;
/// `HLRBRep_ThePolygonToolOfInterCSurf`.
pub type ThePolygonToolOfInterCSurf = InterPolygonTool<LineTool>;
/// `HLRBRep_ThePolyhedronOfInterCSurf`: the grid of an HLR surface. Its body is the
/// one of IntCurveSurface_ThePolyhedronOfHInter on the surface's BRepAdaptor_Surface
/// (both classes include IntCurveSurface_PolyhedronUtils.pxx), so the Rust type is
/// shared; the HLR constructors are the [`SamplePolyhedron`] members on
/// [`SurfaceTool`].
pub type ThePolyhedronOfInterCSurf = ThePolyhedronOfHInter;
/// `HLRBRep_ThePolyhedronToolOfInterCSurf`: every member forwards to the polyhedron.
pub type ThePolyhedronToolOfInterCSurf = ThePolyhedronToolOfHInter;
/// `HLRBRep_TheInterferenceOfInterCSurf`: Intf_InterferencePolygonPolyhedron.gxx.
pub type TheInterferenceOfInterCSurf =
    InterferencePolygonPolyhedron<ThePolygonToolOfInterCSurf, ThePolyhedronToolOfInterCSurf>;
/// `HLRBRep_TheCSFunctionOfInterCSurf`: IntImp_ZerCSParFunc.gxx.
pub type TheCSFunctionOfInterCSurf<'a> = IntImpZerCSParFunc<SurfaceTool<'a>, LineTool>;
/// `HLRBRep_TheExactInterCSurf`: IntImp_IntCS.gxx on the CS function.
pub type TheExactInterCSurf<'a> = IntImpIntCS<TheCSFunctionOfInterCSurf<'a>>;
/// `HLRBRep_TheQuadCurvFuncOfTheQuadCurvExactInterCSurf`.
pub type TheQuadCurvFuncOfTheQuadCurvExactInterCSurf = QuadCurvFunc<LineTool>;
/// `HLRBRep_TheQuadCurvExactInterCSurf`.
pub type TheQuadCurvExactInterCSurf<'a> = QuadCurvExact<SurfaceTool<'a>, LineTool>;
/// `HLRBRep_InterCSurf`: the intersection of a sight line with an HLR surface.
pub type InterCSurf<'a> = InterGen<InterCSurfTypes<'a>>;

/// The types of `HLRBRep_InterCSurf`: the line and HLR surface tools, the polygon
/// and polyhedron with their Intf tools.
pub struct InterCSurfTypes<'a>(PhantomData<&'a Surface>);

impl<'a> InterTypes for InterCSurfTypes<'a> {
    type CurveTool = LineTool;
    type SurfaceTool = SurfaceTool<'a>;
    type Polygon = ThePolygonOfInterCSurf;
    type Polyhedron = ThePolyhedronOfInterCSurf;
    type PolygonTool = ThePolygonToolOfInterCSurf;
    type PolyhedronTool = ThePolyhedronToolOfInterCSurf;
}

// The constructors of HLRBRep_ThePolyhedronOfInterCSurf: the shared polyhedron on
// the BRepAdaptor_Surface of the HLR surface.
impl<'a> SamplePolyhedron<SurfaceTool<'a>> for ThePolyhedronOfHInter {
    fn uniform(
        surface: &&'a Surface,
        nbd_u: i32,
        nbd_v: i32,
        bounds: [f64; 4],
    ) -> Result<Self, Failure> {
        let adaptor: &dyn Adaptor3dSurface = surface.surface();
        ThePolyhedronOfHInter::on_adaptor(adaptor, (nbd_u, nbd_v), bounds)
    }
    fn with_parameters(
        surface: &&'a Surface,
        upars: &Array1<'_, f64>,
        vpars: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let adaptor: &dyn Adaptor3dSurface = surface.surface();
        ThePolyhedronOfHInter::on_adaptor_with_parameters(adaptor, upars, vpars)
    }
    fn bounding(&self) -> &BoundingBox3 {
        ThePolyhedronOfHInter::bounding(self)
    }
}
