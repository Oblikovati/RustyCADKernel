// Rust translation of GeomBndLib_Sphere and GeomBndLib_Torus at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Sphere` and `GeomBndLib_Torus`: sphere patches from their axis extremes and
//! boundary circles; the torus delegates to BndLib (and to OtherSurface for BoxOptimal).
use super::circle::Circle;
use super::live;
use super::other_surface::OtherSurface;
use super::patch::{Patch, add_axis_extrema, add_iso_circles};
use crate::bnd_lib;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomSphericalSurface, GeomSurface, GeomToroidalSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Circle3, Point3, Sphere3};
use std::f64::consts::PI;

/// The sphere's axis extents `[xmin, xmax, ymin, ymax, zmin, zmax]`.
pub fn sphere_extents(sphere: &Sphere3) -> [f64; 6] {
    let (centre, radius): (Point3, f64) = (sphere.location(), sphere.radius());
    [
        centre.x() - radius,
        centre.x() + radius,
        centre.y() - radius,
        centre.y() + radius,
        centre.z() - radius,
        centre.z() + radius,
    ]
}

/// `computeSphere` (GeomBndLib) / `ComputeSphere` (BndLib): the whole-sphere box when
/// the patch spans both periods, otherwise the axis extremes in the patch and the
/// boundary iso circles added by `add_arc`; `sphere_patch(&mut bounds, &sphere, patch, &arc)`.
pub fn sphere_patch(
    bounds: &mut BoundingBox3,
    sphere: &Sphere3,
    patch: Patch,
    add_arc: &super::patch::ArcAdder<'_>,
) -> Result<(), Failure> {
    let e: [f64; 6] = sphere_extents(sphere);
    let u_period: f64 = 2.0 * PI - precision::p_confusion();
    let v_period: f64 = PI - precision::p_confusion();
    if patch.u_max - patch.u_min >= u_period && patch.v_max - patch.v_min >= v_period {
        bounds.update_bounds([e[0], e[2], e[4]], [e[1], e[3], e[5]]);
        return Ok(());
    }
    let (position, radius) = (sphere.position(), sphere.radius());
    let parameters = |p: Point3| elslib::sphere_parameters(position, radius, p);
    add_axis_extrema(bounds, sphere.location(), e, &parameters, patch);
    let u_iso = |u: f64| elslib::sphere_u_iso(position, radius, u);
    let v_iso = |v: f64| elslib::sphere_v_iso(position, radius, v);
    add_iso_circles(bounds, &u_iso, &v_iso, patch, add_arc)
}

/// `GeomBndLib_Sphere`: the box of a `Geom_SphericalSurface` patch.
pub struct Sphere {
    geom: Handle<GeomSphericalSurface>,
}

impl Sphere {
    /// The evaluator of `sphere`; `Sphere::new(handle)`.
    pub fn new(sphere: Handle<GeomSphericalSurface>) -> Self {
        Self { geom: sphere }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomSphericalSurface> {
        &self.geom
    }
    fn sphere(&self) -> Result<Sphere3, Failure> {
        live(&self.geom, "GeomBndLib_Sphere")?.sphere()
    }
    /// `Box(Tol)`: the cube `centre +/- R`; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let e: [f64; 6] = sphere_extents(&self.sphere()?);
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        bounds.update_bounds([e[0], e[2], e[4]], [e[1], e[3], e[5]]);
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: `Box(Tol)` for the natural bounds (within
    /// Precision::Angular()), otherwise the patch analysis with GeomBndLib_Circle arcs;
    /// `evaluator.bounding_box_range(0.0, 6.3, 0.0, 1.5, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let sphere: Sphere3 = self.sphere()?;
        let angular: f64 = precision::angular();
        if u_min.abs() < angular
            && (u_max - 2.0 * PI).abs() < angular
            && (v_min + PI / 2.0).abs() < angular
            && (v_max - PI / 2.0).abs() < angular
        {
            return self.bounding_box(tolerance);
        }
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let add_arc = |b: &mut BoundingBox3, circle: &Circle3, first: f64, last: f64| {
            b.add_box(&Circle::circ_arc_box(circle, first, last, 0.0));
            Ok(())
        };
        sphere_patch(
            &mut bounds,
            &sphere,
            Patch {
                u_min,
                u_max,
                v_min,
                v_max,
            },
            &add_arc,
        )?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 6.3, 0.0, 1.5, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u_min, u_max, v_min, v_max, tolerance)
    }
    /// `BoxOptimal(Tol)`: `Box(Tol)`; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        self.bounding_box(tolerance)
    }
}

/// `GeomBndLib_Torus`: the box of a `Geom_ToroidalSurface` patch through BndLib.
pub struct Torus {
    geom: Handle<GeomToroidalSurface>,
}

impl Torus {
    /// The evaluator of `torus`; `Torus::new(handle)`.
    pub fn new(torus: Handle<GeomToroidalSurface>) -> Self {
        Self { geom: torus }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomToroidalSurface> {
        &self.geom
    }
    /// `Box(Tol)`: `BndLib::Add(gp_Torus, Tol, B)`; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        bnd_lib::add_torus(
            &live(&self.geom, "GeomBndLib_Torus")?.torus()?,
            tolerance,
            &mut bounds,
        );
        Ok(bounds)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: `BndLib::Add(gp_Torus, patch, Tol, B)`;
    /// `evaluator.bounding_box_range(0.0, 1.5, 0.0, 3.1, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let torus = live(&self.geom, "GeomBndLib_Torus")?.torus()?;
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        bnd_lib::add_torus_patch(&torus, [u_min, u_max, v_min, v_max], tolerance, &mut bounds)?;
        Ok(bounds)
    }
    fn other(&self) -> Result<GeomAdaptorSurface, Failure> {
        GeomAdaptorSurface::with_surface(&self.geom.cast::<dyn GeomSurface>())
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: GeomBndLib_OtherSurface on a
    /// GeomAdaptor_Surface; `evaluator.bounding_box_optimal_range(0.0, 1.5, 0.0, 3.1, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let adaptor: GeomAdaptorSurface = self.other()?;
        let adaptor: &dyn Adaptor3dSurface = &adaptor;
        OtherSurface::new(adaptor).bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance)
    }
    /// `BoxOptimal(Tol)`: GeomBndLib_OtherSurface over the adaptor bounds;
    /// `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let adaptor: GeomAdaptorSurface = self.other()?;
        let adaptor: &dyn Adaptor3dSurface = &adaptor;
        OtherSurface::new(adaptor).bounding_box_optimal(tolerance)
    }
}
