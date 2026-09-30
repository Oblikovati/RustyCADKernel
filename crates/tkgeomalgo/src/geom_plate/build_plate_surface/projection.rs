// Rust translation of ProjectCurve, ProjectedCurve and ProjectPoint of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projections of curves and points on the initial surface.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::curve_constraint::GeomPlateCurveConstraint;
use crate::geom_plate::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, errors};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

impl GeomPlateBuildPlateSurface {
    /// `new GeomAdaptor_Surface(mySurfInit)`.
    pub(super) fn init_adaptor(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Ok(Handle::new(GeomAdaptorSurface::with_surface(&self.surf_init)?).cast())
    }

    /// Resets the parametric tolerances to the resolutions of `tol3d` on the initial
    /// surface and initializes the projector on its bounds (the native sequence
    /// `Bounds`, `GeomAdaptor_Surface`, `UResolution`, `VResolution`, `Initialize`).
    pub(super) fn init_projector(&mut self) -> Result<(), Failure> {
        let tolerances: (f64, f64) = self.projector_on_init(|surface, tol3d| {
            Ok((surface.u_resolution(tol3d)?, surface.v_resolution(tol3d)?))
        })?;
        (self.tol_u, self.tol_v) = tolerances;
        Ok(())
    }

    /// Initializes the projector on the bounds of the initial surface with the
    /// parametric tolerances that `tolerances` gives for its adaptor, and returns them.
    pub(super) fn projector_on_init(
        &mut self,
        tolerances: impl FnOnce(&dyn Adaptor3dSurface, f64) -> Result<(f64, f64), Failure>,
    ) -> Result<(f64, f64), Failure> {
        let (u1, v1, u2, v2) = self.surf_init_ref()?.bounds();
        let adaptor: Handle<dyn Adaptor3dSurface> = self.init_adaptor()?;
        let surface = live(&adaptor, "the initial surface adaptor")?;
        let (tol_u, tol_v) = tolerances(surface, self.tol3d)?;
        // The native call passes Bounds(u1, v1, u2, v2) as (Uinf, Usup, Vinf, Vsup).
        self.proj
            .initialize_shared(&adaptor, (u1, v1), (u2, v2), (tol_u, tol_v))?;
        Ok((tol_u, tol_v))
    }

    /// The parameter on the initial surface of the nearest projection of `p3d`, the
    /// first of equal ones (`ProjectPoint`); StdFail_NotDone or Standard_OutOfRange
    /// when the projection fails.
    pub(super) fn project_point(&mut self, p3d: Point3) -> Result<Point2, Failure> {
        self.proj.perform(p3d)?;
        let mut nearest: i32 = 1;
        if self.proj.nb_ext()? > 1 {
            let mut dist2mini: f64 = self.proj.square_distance(1)?;
            for i in 2..=self.proj.nb_ext()? {
                if self.proj.square_distance(i)? < dist2mini {
                    dist2mini = self.proj.square_distance(i)?;
                    nearest = i;
                }
            }
        }
        let (u, v) = self.proj.point(nearest)?.parameter();
        Ok(Point2::new(u, v))
    }

    /// The point on the initial surface of the parameter `u` of `lin`: on its
    /// projected curve, else on its 2d curve, else the projection of `p3d()` (the
    /// native sequence of `LoadCurve`, `Disc2dContour` and `EcartContraintesMil`).
    pub(super) fn uv_on_init(
        &mut self,
        lin: &GeomPlateCurveConstraint,
        u: f64,
        p3d: impl FnOnce() -> Result<Point3, Failure>,
    ) -> Result<Point2, Failure> {
        let projected = lin.projected_curve();
        if let Some(curve) = projected.get() {
            return curve.value(u);
        }
        let curve2d = lin.curve2d_on_surf()?;
        if let Some(curve) = curve2d.get() {
            return curve.value(u);
        }
        let point: Point3 = p3d()?;
        self.project_point(point)
    }

    /// The 2d curve of `curv` on the initial surface (`ProjectCurve`): its projection
    /// approximated in C1 when it is one curve on the whole range, a degree 1 Bezier
    /// curve on the point when it projects to one point, null otherwise.
    pub(super) fn project_curve(
        &self,
        curv: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let hsur: Handle<dyn Adaptor3dSurface> = self.init_adaptor()?;
        let tol: f64 = self.tol3d / 10.0;
        let projector = Handle::new(CompProjectedCurve::with_tolerances(&hsur, curv, tol, tol)?);
        let curve = live(curv, "the curve to project")?;
        let (udeb_check, ufin_check) = (curve.first_parameter()?, curve.last_parameter()?);
        let proj = live(&projector, "the projection")?;
        let (proj_udeb, proj_ufin) = proj.bounds(1)?;
        let partial: bool = proj.nb_curves() != 1
            || (udeb_check - proj_udeb).abs() > precision::p_confusion()
            || (ufin_check - proj_ufin).abs() > precision::p_confusion();
        if partial {
            return single_point_curve(proj);
        }
        let (udeb, ufin) = proj.bounds(1)?;
        let max_seg: i32 = 20 + proj.nb_intervals(Shape::C3)?;
        let c2d: Handle<dyn Adaptor2dCurve2d> = projector.cast();
        let mut appr = CurveOnSurface::without_perform(&c2d, &hsur, udeb, ufin, self.tol3d);
        let options = CurveOnSurfaceOptions {
            max_segments: max_seg,
            max_degree: 10,
            continuity: Shape::C1,
            only_3d: false,
            only_2d: true,
        };
        appr.perform(options)?;
        Ok(appr.curve_2d().cast())
    }

    /// The trimmed projection of `curv` on the initial surface within the parametric
    /// tolerances (`ProjectedCurve`): null unless it is one curve on the whole range.
    /// The native member is private and nothing calls it.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn projected_curve(
        &self,
        curv: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let hsur: Handle<dyn Adaptor3dSurface> = self.init_adaptor()?;
        let (tol_u, tol_v) = (self.tol_u / 10.0, self.tol_v / 10.0);
        let projector = CompProjectedCurve::with_tolerances(&hsur, curv, tol_u, tol_v)?;
        if projector.nb_curves() != 1 {
            return Ok(Handle::default());
        }
        let curve = live(curv, "the curve to project")?;
        let (first1, last1) = (curve.first_parameter()?, curve.last_parameter()?);
        let (first2, last2) = projector.bounds(1)?;
        let tol: f64 = self.tol_u.max(self.tol_v);
        if (first1 - first2).abs() <= tol && (last1 - last2).abs() <= tol {
            let trimmed = projector.trim(first2, last2, precision::p_confusion())?;
            return Ok(trimmed.cast::<CompProjectedCurve>().cast());
        }
        Ok(Handle::default())
    }
}

/// The projection reduced to one point as a degree 1 Bezier curve, or null.
fn single_point_curve(proj: &CompProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (single, p2d) = proj.is_single_pnt(1)?;
    if !single {
        return Ok(Handle::default());
    }
    let mut poles: Array1<'static, Point2> =
        Array1::with_bounds(1, 2).map_err(errors::array_bounds)?;
    poles.init(p2d);
    Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
    use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
    use rustycad_tkmath::gp::{Axis3, Direction3};

    fn builder_on_plane() -> GeomPlateBuildPlateSurface {
        let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let mut builder = GeomPlateBuildPlateSurface::with_surface(
            &plane, 3, 10, 3, 1e-5, 1e-4, 0.01, 0.1, false,
        )
        .unwrap();
        builder.init_projector().unwrap();
        builder
    }

    fn segment(z: f64) -> Handle<dyn Adaptor3dCurve> {
        let dir = Direction3::new(1.0, 1.0, 0.0).unwrap();
        let line: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, z), dir)).cast();
        Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 2.0).unwrap()).cast()
    }

    #[test]
    fn projected_curve_trims_a_whole_projection() {
        let builder = builder_on_plane();
        let projected = builder.projected_curve(&segment(1.0)).unwrap();
        let curve = projected.get().expect("one continuous projection");
        assert!((curve.first_parameter().unwrap() - 0.0).abs() < 1e-9);
        assert!((curve.last_parameter().unwrap() - 2.0).abs() < 1e-9);
        let uv: Point2 = curve.value(1.0).unwrap();
        let s = std::f64::consts::FRAC_1_SQRT_2;
        assert!((uv.x() - s).abs() < 1e-7 && (uv.y() - s).abs() < 1e-7);
    }

    #[test]
    fn project_point_and_curve_on_the_plane() {
        let mut builder = builder_on_plane();
        let uv = builder.project_point(Point3::new(1.0, 2.0, 3.0)).unwrap();
        assert_eq!((uv.x(), uv.y()), (1.0, 2.0));
        let curve2d = builder.project_curve(&segment(-1.0)).unwrap();
        let p = curve2d.get().unwrap().value(2.0).unwrap();
        let s = std::f64::consts::SQRT_2;
        assert!((p.x() - s).abs() < 1e-6 && (p.y() - s).abs() < 1e-6);
    }
}
