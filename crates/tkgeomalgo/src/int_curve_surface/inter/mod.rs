// Rust translation of IntCurveSurface_Inter.pxx (instantiated by IntCurveSurface_HInter.hxx/.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_InterImpl`: the curve-surface intersection algorithm, generic
//! over the curve and surface tools, the polygon and the polyhedron
//! (docs/geomalgo-design.md section 11). The public entry points are here, the
//! steps by polygon and polyhedron in `internal`, the analytic paths of the conics
//! in `conics`.
mod conics;
mod internal;

use super::Intersection;
use super::inter_utils::{SectionPolygon, SectionPolyhedron, decompose_surface_intervals};
use super::tools::{InterCurveTool, InterSurfaceTool};
use crate::int_imp::{TheCurveTool, ThePSurfaceTool};
use crate::intf::{Polygon3dTool, PolyhedronTool};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// The polygon constructors the algorithm calls (`PolygonType`).
pub trait SamplePolygon<C: InterCurveTool>: SectionPolygon + Sized {
    /// Constructor (Curve, NbPnt).
    fn with_samples(curve: &C::Curve, nb_pnt: i32) -> Result<Self, Failure>;
    /// Constructor (Curve, U1, U2, NbPnt).
    fn with_range(curve: &C::Curve, u1: f64, u2: f64, nb_pnt: i32) -> Result<Self, Failure>;
    /// Constructor (Curve, Upars).
    fn with_parameters(curve: &C::Curve, upars: &Array1<'_, f64>) -> Result<Self, Failure>;
    /// `InfParameter()`.
    fn inf_parameter(&self) -> f64;
    /// `SupParameter()`.
    fn sup_parameter(&self) -> f64;
}

/// The polyhedron constructors the algorithm calls (`PolyhedronType`).
pub trait SamplePolyhedron<S: InterSurfaceTool>: SectionPolyhedron + Sized {
    /// Constructor (Surface, nbdU, nbdV, U1, V1, U2, V2).
    fn uniform(
        surface: &S::Surface,
        nbd_u: i32,
        nbd_v: i32,
        bounds: [f64; 4],
    ) -> Result<Self, Failure>;
    /// Constructor (Surface, Upars, Vpars).
    fn with_parameters(
        surface: &S::Surface,
        upars: &Array1<'_, f64>,
        vpars: &Array1<'_, f64>,
    ) -> Result<Self, Failure>;
    /// `Bounding()`.
    fn bounding(&self) -> &BoundingBox3;
}

/// The types an instance of the algorithm names: `CurveTool`, `SurfaceTool`,
/// `PolygonType`, `PolyhedronType` and the Intf tools of the interference. The CS
/// function, the exact intersector and the quadric-curve intersector follow from
/// the tools, as in both native instances.
pub trait InterTypes {
    /// `CurveTool`.
    type CurveTool: InterCurveTool;
    /// `SurfaceTool`.
    type SurfaceTool: InterSurfaceTool;
    /// `PolygonType`.
    type Polygon: SamplePolygon<Self::CurveTool>;
    /// `PolyhedronType`.
    type Polyhedron: SamplePolyhedron<Self::SurfaceTool>;
    /// The `ToolPolygon3d` of the interference.
    type PolygonTool: Polygon3dTool<Polygon = Self::Polygon>;
    /// The `ToolPolyh` of the interference.
    type PolyhedronTool: PolyhedronTool<Polyhedron = Self::Polyhedron>;
}

/// The curve of an instance.
pub(crate) type CurveOf<T> = <<T as InterTypes>::CurveTool as crate::int_imp::TheCurveTool>::Curve;
/// The surface of an instance.
pub(crate) type SurfaceOf<T> =
    <<T as InterTypes>::SurfaceTool as crate::int_imp::ThePSurfaceTool>::Surface;

/// The intersection of a curve and a surface: points, and whether the curve is
/// parallel to the surface, read through [`Intersection`] (`Deref`).
/// `IntCurveSurface_HInter` is [`HInter`](super::HInter).
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::HInter;
/// let inter = HInter::new();
/// assert!(!inter.is_done());
/// ```
pub struct InterGen<T: InterTypes> {
    base: Intersection,
    types: PhantomData<fn() -> T>,
}

impl<T: InterTypes> Default for InterGen<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: InterTypes> Deref for InterGen<T> {
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: InterTypes> DerefMut for InterGen<T> {
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

impl<T: InterTypes> InterGen<T> {
    /// The empty constructor: not done; `HInter::new()`.
    pub fn new() -> Self {
        Self {
            base: Intersection::new(),
            types: PhantomData,
        }
    }
    /// Clears the results and marks the computation done before it runs (an
    /// exception leaves the points found so far, as natively).
    fn start(&mut self) {
        self.base.reset_fields();
        self.base.done = true;
    }
    /// `Perform(Curve, Surface)`: intersects the curve with each C2 patch of the
    /// surface; `inter.perform(&curve, &surface)?`.
    pub fn perform(&mut self, curve: &CurveOf<T>, surface: &SurfaceOf<T>) -> Result<(), Failure> {
        self.start();
        for bounds in decompose_surface_intervals::<T::SurfaceTool>(surface)? {
            self.perform_bounds(curve, surface, bounds.u0, bounds.v0, bounds.u1, bounds.v1)?;
        }
        Ok(())
    }
    /// `Perform(Curve, Polygon, Surface)`: with the given polygon of the curve and a
    /// polyhedron of the whole surface (at most 40 x 40 samples);
    /// `inter.perform_polygon(&curve, &polygon, &surface)?`.
    pub fn perform_polygon(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
    ) -> Result<(), Failure> {
        type S<T> = <T as InterTypes>::SurfaceTool;
        self.start();
        let (u1, v1) = (
            S::<T>::first_u_parameter(surface)?,
            S::<T>::first_v_parameter(surface)?,
        );
        let (u2, v2) = (
            S::<T>::last_u_parameter(surface)?,
            S::<T>::last_v_parameter(surface)?,
        );
        let nbsu: i32 = S::<T>::nb_samples_u(surface, u1, u2)?.min(40);
        let nbsv: i32 = S::<T>::nb_samples_v(surface, v1, v2)?.min(40);
        let polyhedron = T::Polyhedron::uniform(surface, nbsu, nbsv, [u1, v1, u2, v2])?;
        self.perform_polygon_polyhedron(curve, polygon, surface, &polyhedron)
    }
    /// `Perform(Curve, Surface, Polyhedron)`: with the given polyhedron and the
    /// polygon of the curve at its sample count;
    /// `inter.perform_polyhedron(&curve, &surface, &polyhedron)?`.
    pub fn perform_polyhedron(
        &mut self,
        curve: &CurveOf<T>,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
    ) -> Result<(), Failure> {
        type C<T> = <T as InterTypes>::CurveTool;
        self.start();
        let (u1, u2) = (
            C::<T>::first_parameter(curve)?,
            C::<T>::last_parameter(curve)?,
        );
        let polygon = T::Polygon::with_samples(curve, C::<T>::nb_samples(curve, u1, u2)?)?;
        self.perform_polygon_polyhedron(curve, &polygon, surface, polyhedron)
    }
    /// The whole surface range `[u1, v1, u2, v2]`.
    fn surface_range(surface: &SurfaceOf<T>) -> Result<[f64; 4], Failure> {
        type S<T> = <T as InterTypes>::SurfaceTool;
        Ok([
            S::<T>::first_u_parameter(surface)?,
            S::<T>::first_v_parameter(surface)?,
            S::<T>::last_u_parameter(surface)?,
            S::<T>::last_v_parameter(surface)?,
        ])
    }
    /// `Perform(Curve, Polygon, Surface, Polyhedron)`: with both given, over the
    /// whole surface range; `inter.perform_polygon_polyhedron(&c, &pg, &s, &ph)?`.
    pub fn perform_polygon_polyhedron(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
    ) -> Result<(), Failure> {
        self.start();
        let [u1, v1, u2, v2] = Self::surface_range(surface)?;
        self.internal_perform_polyhedron(curve, polygon, surface, polyhedron, u1, v1, u2, v2)
    }
    /// `Perform(Curve, Polygon, Surface, Polyhedron, BndBSB)`: the same with a
    /// prepared grid of the triangles; `inter.perform_polygon_polyhedron_grid(&c, &pg, &s, &ph, &mut bsb)?`.
    pub fn perform_polygon_polyhedron_grid(
        &mut self,
        curve: &CurveOf<T>,
        polygon: &T::Polygon,
        surface: &SurfaceOf<T>,
        polyhedron: &T::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.start();
        let [u1, v1, u2, v2] = Self::surface_range(surface)?;
        self.internal_perform_polyhedron_grid(
            curve,
            polygon,
            surface,
            polyhedron,
            [u1, v1, u2, v2],
            grid,
        )
    }
}
