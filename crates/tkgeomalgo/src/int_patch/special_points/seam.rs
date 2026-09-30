// Rust translation of the FuncPreciseSeam class and the IsPointOnSurface function of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_SpecialPoints.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The local helpers of `IntPatch_SpecialPoints.cxx`: the seam function and the point
//! projection.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS, GenLocateExtPS};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `FuncPreciseSeam`: the distance `P(x1, x2) - Q(iso, x3)` (or `Q(x3, iso)`) between
/// the parametric surface and the seam iso of the quadric. The native evaluations catch
/// every Standard_Failure and answer false.
pub(super) struct FuncPreciseSeam<'a> {
    q_surf: &'a dyn Adaptor3dSurface,
    p_surf: &'a dyn Adaptor3dSurface,
    // 1 for the U seam (the iso is in U, x3 is V), 0 for the V one.
    seam_coord_ind: usize,
    iso_parameter: f64,
}

impl<'a> FuncPreciseSeam<'a> {
    /// `FuncPreciseSeam(theQSurf, thePSurf, isTheUSeam, theIsoParameter)`.
    pub(super) fn new(
        q_surf: &'a dyn Adaptor3dSurface,
        p_surf: &'a dyn Adaptor3dSurface,
        is_u_seam: bool,
        iso_parameter: f64,
    ) -> Self {
        Self {
            q_surf,
            p_surf,
            seam_coord_ind: usize::from(is_u_seam),
            iso_parameter,
        }
    }

    fn quadric_uv(&self, x3: f64) -> [f64; 2] {
        let mut uv: [f64; 2] = [self.iso_parameter, self.iso_parameter];
        uv[self.seam_coord_ind] = x3;
        uv
    }

    fn try_value(&self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<(), Failure> {
        let (ix, i_f) = (x.lower(), f.lower());
        let uv: [f64; 2] = self.quadric_uv(x[ix + 2]);
        let p1: Point3 = self.p_surf.value(x[ix], x[ix + 1])?;
        let p2: Point3 = self.q_surf.value(uv[0], uv[1])?;
        let d = p1.xyz() - p2.xyz();
        (f[i_f], f[i_f + 1], f[i_f + 2]) = (d.x(), d.y(), d.z());
        Ok(())
    }

    fn try_derivatives(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<(), Failure> {
        let (ix, r, c) = (x.lower(), d.lower_row(), d.lower_col());
        let uv: [f64; 2] = self.quadric_uv(x[ix + 2]);
        let d1 = self.p_surf.d1(x[ix], x[ix + 1])?;
        let d2 = self.q_surf.d1(uv[0], uv[1])?;
        let seam: Vector3 = [d2.d1u, d2.d1v][self.seam_coord_ind].reversed();
        for (col, v) in [d1.d1u, d1.d1v, seam].into_iter().enumerate() {
            let col: i32 = c + col as i32;
            (d[(r, col)], d[(r + 1, col)], d[(r + 2, col)]) = (v.x(), v.y(), v.z());
        }
        Ok(())
    }
}

impl FunctionSet for FuncPreciseSeam<'_> {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        self.try_value(x, f).is_ok()
    }
}

impl FunctionSetWithDerivatives for FuncPreciseSeam<'_> {
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
        self.try_derivatives(x, d).is_ok()
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
        self.try_value(x, f).is_ok() && self.try_derivatives(x, d).is_ok()
    }
}

/// `IsPointOnSurface(theSurf, thePt, theTol, theProjPt, theUpar, theVpar)`: the nearest
/// projection of `pt` on the surface when it is nearer than `tol`, as
/// `(point, u, v)`. The elementary and swept surfaces use Extrema_ExtPS (minima); the
/// other ones Extrema_GenLocateExtPS from `(u, v)`.
pub(super) fn is_point_on_surface(
    surf: &dyn Adaptor3dSurface,
    pt: Point3,
    tol: f64,
    (u, v): (f64, f64),
) -> Result<Option<(Point3, f64, f64)>, Failure> {
    match surf.get_type()? {
        SurfaceType::Plane
        | SurfaceType::Cylinder
        | SurfaceType::Cone
        | SurfaceType::Sphere
        | SurfaceType::Torus
        | SurfaceType::SurfaceOfExtrusion
        | SurfaceType::SurfaceOfRevolution => nearest_extremum(surf, pt, tol),
        _ => located_extremum(surf, pt, tol, (u, v)),
    }
}

fn nearest_extremum(
    surf: &dyn Adaptor3dSurface,
    pt: Point3,
    tol: f64,
) -> Result<Option<(Point3, f64, f64)>, Failure> {
    let (tol_u, tol_v) = (surf.u_resolution(tol)?, surf.v_resolution(tol)?);
    let extr = ExtPS::with_surface(pt, surf, tol_u, tol_v, ExtFlag::Min, ExtAlgo::Grad)?;
    if !extr.is_done() || extr.nb_ext()? < 1 {
        return Ok(None);
    }
    let mut index: i32 = 1;
    let mut sq_dist_min: f64 = extr.square_distance(index)?;
    for i in index + 1..=extr.nb_ext()? {
        let sq_d: f64 = extr.square_distance(i)?;
        if sq_d < sq_dist_min {
            sq_dist_min = sq_d;
            index = i;
        }
    }
    if sq_dist_min > tol * tol {
        return Ok(None);
    }
    let p = extr.point(index)?;
    let (u, v) = p.parameter();
    Ok(Some((*p.value(), u, v)))
}

fn located_extremum(
    surf: &dyn Adaptor3dSurface,
    pt: Point3,
    tol: f64,
    (u, v): (f64, f64),
) -> Result<Option<(Point3, f64, f64)>, Failure> {
    let conf: f64 = precision::p_confusion();
    let mut extr = GenLocateExtPS::new(surf, conf, conf);
    extr.perform(pt, u, v, false)?;
    if !extr.is_done() || extr.square_distance()? > tol * tol {
        return Ok(None);
    }
    let p = extr.point()?;
    let (u, v) = p.parameter();
    Ok(Some((*p.value(), u, v)))
}
