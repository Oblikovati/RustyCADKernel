// Rust translation of MakeSampling, the FillArrayOfPnt overloads, DegeneratedIndex and
// IsDegenerated of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of a surface triangulation: the surface points at the sampling (moved along
//! the normal by the advanced method), their degenerated flags, and the box around them.
use super::{IntPolyhMaillageAffinage, MY_TOLERANCE};
use crate::int_polyh::array::{IntPolyhArrayOfPointNormal, IntPolyhPointNormal};
use crate::int_polyh::tools::{IntPolyhTools, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The rows and columns of the degenerated points: `(aID1, aID2, aJD1, aJD2)`.
type DegeneratedRanks = [i32; 4];

impl IntPolyhMaillageAffinage {
    /// `MakeSampling(SurfID, theUPars, theVPars)`: the sampling of the surface with its
    /// sample counts and the enlarge flag. `m.make_sampling(1, &mut u, &mut v)?`.
    pub fn make_sampling(
        &self,
        surf_id: i32,
        u_pars: &mut Array1<'static, f64>,
        v_pars: &mut Array1<'static, f64>,
    ) -> Result<(), Failure> {
        let side = &self.sides[Self::rank(surf_id)];
        let counts = (side.nb_samples_u, side.nb_samples_v);
        IntPolyhTools::make_sampling(&side.surface, counts, self.my_enlarge_zone, u_pars, v_pars)
    }

    /// `FillArrayOfPnt(SurfID)`: the points of the default sampling. `m.fill_array_of_pnt(1)?`.
    pub fn fill_array_of_pnt(&mut self, surf_id: i32) -> Result<(), Failure> {
        let (mut u_pars, mut v_pars) = (Array1::default(), Array1::default());
        self.make_sampling(surf_id, &mut u_pars, &mut v_pars)?;
        self.fill_array_of_pnt_params(surf_id, &u_pars, &v_pars, None)
    }

    /// `FillArrayOfPnt(SurfID, isShiftFwd)`: the points of the default sampling moved
    /// along the normal. `m.fill_array_of_pnt_shift(1, true)?`.
    pub fn fill_array_of_pnt_shift(
        &mut self,
        surf_id: i32,
        is_shift_fwd: bool,
    ) -> Result<(), Failure> {
        let (mut u_pars, mut v_pars) = (Array1::default(), Array1::default());
        self.make_sampling(surf_id, &mut u_pars, &mut v_pars)?;
        self.fill_array_of_pnt_shift_params(surf_id, is_shift_fwd, &u_pars, &v_pars, None)
    }

    /// `FillArrayOfPnt(SurfID, Upars, Vpars, theDeflTol)`: the surface points at the
    /// sampling, flagged on degenerated rows, and the box around them enlarged by 1.2 times
    /// the deflection (computed when `None`). `m.fill_array_of_pnt_params(1, &u, &v, None)?`.
    pub fn fill_array_of_pnt_params(
        &mut self,
        surf_id: i32,
        u_pars: &Array1<'_, f64>,
        v_pars: &Array1<'_, f64>,
        defl_tol: Option<f64>,
    ) -> Result<(), Failure> {
        let side = &mut self.sides[Self::rank(surf_id)];
        let (nb_u, nb_v) = (side.nb_samples_u, side.nb_samples_v);
        let deg: DegeneratedRanks =
            degenerated_ranks(&side.surface, (u_pars, nb_u), (v_pars, nb_v))?;
        side.points.init(nb_u * nb_v);
        let mut i_cnt: i32 = 0;
        for i in 1..=nb_u {
            let b_deg_i: bool = deg[0] == i || deg[1] == i;
            let a_u: f64 = array_value(u_pars, i)?;
            for j in 1..=nb_v {
                let a_v: f64 = array_value(v_pars, j)?;
                let a_p: Point3 = side.surface.value(a_u, a_v)?;
                let b_deg: bool = b_deg_i || deg[2] == j || deg[3] == j;
                store_point(&mut side.points[i_cnt], a_p, (a_u, a_v), b_deg);
                i_cnt += 1;
                side.bbox.add_point(a_p);
            }
        }
        side.points.set_nb_items(i_cnt);
        let mut a_tol: f64 = match defl_tol {
            Some(tol) => tol,
            None => IntPolyhTools::compute_deflection(&side.surface, u_pars, v_pars)?,
        };
        a_tol *= 1.2;
        enlarge_box(&mut side.bbox, a_tol)
    }

    /// `FillArrayOfPnt(SurfID, isShiftFwd, thePoints, theUPars, theVPars, theDeflTol)`:
    /// the given surface points moved by 1.5 times the deflection along (or against) their
    /// normal, flagged on degenerated rows, and the box around them.
    /// `m.fill_array_of_pnt_normals(1, true, &normals, &u, &v, 0.1)?`.
    pub fn fill_array_of_pnt_normals(
        &mut self,
        surf_id: i32,
        is_shift_fwd: bool,
        points_norm: &IntPolyhArrayOfPointNormal,
        (u_pars, v_pars): (&Array1<'_, f64>, &Array1<'_, f64>),
        defl_tol: f64,
    ) -> Result<(), Failure> {
        let side = &mut self.sides[Self::rank(surf_id)];
        let (nb_u, nb_v) = (side.nb_samples_u, side.nb_samples_v);
        let deg: DegeneratedRanks =
            degenerated_ranks(&side.surface, (u_pars, nb_u), (v_pars, nb_v))?;
        side.points.init(points_norm.nb_items());
        let mut i_cnt: i32 = 0;
        for i in 1..=nb_u {
            let a_u: f64 = array_value(u_pars, i)?;
            let b_deg_i: bool = deg[0] == i || deg[1] == i;
            for j in 1..=nb_v {
                let a_v: f64 = array_value(v_pars, j)?;
                let a_p: Point3 = shifted(points_norm.value(i_cnt)?, is_shift_fwd, defl_tol);
                let b_deg: bool = b_deg_i || deg[2] == j || deg[3] == j;
                store_point(&mut side.points[i_cnt], a_p, (a_u, a_v), b_deg);
                i_cnt += 1;
                side.bbox.add_point(a_p);
            }
        }
        side.points.set_nb_items(i_cnt);
        enlarge_box(&mut side.bbox, defl_tol * 1.2)
    }

    /// `FillArrayOfPnt(SurfID, isShiftFwd, Upars, Vpars, theDeflTol)`: the shifted points
    /// of the sampling. Natively it always fills the first surface (it passes 1, not
    /// SurfID, to the overload it calls); so does the port.
    /// `m.fill_array_of_pnt_shift_params(2, true, &u, &v, None)?`.
    pub fn fill_array_of_pnt_shift_params(
        &mut self,
        surf_id: i32,
        is_shift_fwd: bool,
        u_pars: &Array1<'_, f64>,
        v_pars: &Array1<'_, f64>,
        defl_tol: Option<f64>,
    ) -> Result<(), Failure> {
        let a_s: Adaptor3dSurfaceHandle = self.sides[Self::rank(surf_id)].surface.clone();
        let a_tol: f64 = match defl_tol {
            Some(tol) => tol,
            None => IntPolyhTools::compute_deflection(&a_s, u_pars, v_pars)?,
        };
        let mut a_points = IntPolyhArrayOfPointNormal::default();
        IntPolyhTools::fill_array_of_point_normal(&a_s, u_pars, v_pars, &mut a_points)?;
        self.fill_array_of_pnt_normals(1, is_shift_fwd, &a_points, (u_pars, v_pars), a_tol)
    }
}

/// `aIP.Set(x, y, z, u, v)` then `SetDegenerated(true)` on a degenerated row: the flag of
/// a point that is not on one keeps its previous value, as natively.
fn store_point(
    target: &mut crate::int_polyh::IntPolyhPoint,
    p: Point3,
    (u, v): (f64, f64),
    deg: bool,
) {
    target.set(p.x(), p.y(), p.z(), u, v, 1);
    if deg {
        target.set_degenerated(deg);
    }
}

/// The point of `pn` moved by `1.5 * defl_tol` along its normal (against it unless
/// `forward`).
fn shifted(pn: &IntPolyhPointNormal, forward: bool, defl_tol: f64) -> Point3 {
    let mut a_norm: Vector3 = pn.normal * (1.5 * defl_tol);
    if !forward {
        a_norm.reverse();
    }
    pn.point.translated(a_norm)
}

/// The box enlarged by `tol` on each side, then its gap set to at least `MyTolerance`.
fn enlarge_box(bbox: &mut BoundingBox3, tol: f64) -> Result<(), Failure> {
    let [a1, a2, a3, b1, b2, b3] = bbox.get()?;
    bbox.update_bounds(
        [a1 - tol, a2 - tol, a3 - tol],
        [b1 + tol, b2 + tol, b3 + tol],
    );
    bbox.enlarge(MY_TOLERANCE);
    Ok(())
}

/// The degenerated rows `(aID1, aID2)` and columns `(aJD1, aJD2)`: the V-isolines are
/// checked first, the U ones only when no V-isoline is degenerated.
fn degenerated_ranks(
    surface: &Adaptor3dSurfaceHandle,
    (u_pars, nb_u): (&Array1<'_, f64>, i32),
    (v_pars, nb_v): (&Array1<'_, f64>, i32),
) -> Result<DegeneratedRanks, Failure> {
    let (jd1, jd2) = degenerated_index(v_pars, nb_v, surface, 1)?;
    if jd1 != 0 || jd2 != 0 {
        return Ok([0, 0, jd1, jd2]);
    }
    let (id1, id2) = degenerated_index(u_pars, nb_u, surface, 2)?;
    Ok([id1, id2, 0, 0])
}

/// `DegeneratedIndex(Xpars, aNbX, aS, aIsoDirection, aI1, aI2)`: the last sample indices
/// within `MyTolerance` of the degenerated first and last isolines (V = const for
/// direction 1, U = const for 2), 0 when none.
fn degenerated_index(
    x_pars: &Array1<'_, f64>,
    nb_x: i32,
    surface: &Adaptor3dSurfaceHandle,
    iso_direction: i32,
) -> Result<(i32, i32), Failure> {
    let tol2: f64 = MY_TOLERANCE * MY_TOLERANCE;
    let first_index: i32 = if iso_direction == 1 { 1 } else { 3 };
    let deg1: Option<f64> = is_degenerated(surface, first_index, tol2)?;
    let deg2: Option<f64> = is_degenerated(surface, first_index + 1, tol2)?;
    if deg1.is_none() && deg2.is_none() {
        return Ok((0, 0));
    }
    let (mut i1, mut i2) = (0, 0);
    for i in 1..=nb_x {
        let a_x: f64 = array_value(x_pars, i)?;
        if deg1.is_some_and(|d| (a_x - d).abs() < MY_TOLERANCE) {
            i1 = i;
        }
        if deg2.is_some_and(|d| (a_x - d).abs() < MY_TOLERANCE) {
            i2 = i;
        }
    }
    Ok((i1, i2))
}

/// `IsDegenerated(aS, aIndex, aTol2, aDegX)`: the parameter of the isoline `index` (1, 2:
/// V first and last; 3, 4: U first and last) when its three sample points are within
/// `sqrt(tol2)` of each other. The samples are `i * step`, not offset by the first bound,
/// as natively.
fn is_degenerated(
    surface: &Adaptor3dSurfaceHandle,
    index: i32,
    tol2: f64,
) -> Result<Option<f64>, Failure> {
    let nb_p: i32 = 3;
    let u = [surface.first_u_parameter()?, surface.last_u_parameter()?];
    let v = [surface.first_v_parameter()?, surface.last_v_parameter()?];
    let v_const: bool = index < 3;
    let (range, fixed) = if v_const {
        (u, if index == 2 { v[1] } else { v[0] })
    } else {
        (v, if index == 4 { u[1] } else { u[0] })
    };
    let at = |t: f64| {
        if v_const {
            surface.value(t, fixed)
        } else {
            surface.value(fixed, t)
        }
    };
    let d: f64 = (range[1] - range[0]) / f64::from(nb_p - 1);
    let mut a_p1: Point3 = at(range[0])?;
    for i in 1..nb_p {
        let mut t: f64 = f64::from(i) * d;
        if i == nb_p - 1 {
            t = range[1];
        }
        let a_p2: Point3 = at(t)?;
        if a_p1.square_distance(&a_p2) > tol2 {
            return Ok(None);
        }
        a_p1 = a_p2;
    }
    Ok(Some(fixed))
}
