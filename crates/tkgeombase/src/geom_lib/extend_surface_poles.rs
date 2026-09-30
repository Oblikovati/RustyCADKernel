// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (the pole
// computations of ExtendSurfByLength) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The flat pole rows, boundary tangents and extended poles of
//! `GeomLib::ExtendSurfByLength`.
use super::extend_surface::{BoundarySide, extended_knots, weight_attempts};
use crate::array_access::{grid, put};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Vector3};

const THE_SURFACE_TANGENT_TOLERANCE: f64 = 1.0e-12;
const THE_TANGENT_REVERSAL_ANGLE: f64 = 2.0;
const THE_SURFACE_WEIGHT_TOLERANCE: f64 = 10.0 * precision::p_confusion();

/// The surface as one B-spline curve along the extension direction: its degree, pole
/// count, dimension (3 or 4 per pole across), flat knots and flat poles.
pub(super) struct SurfaceRows {
    pub cdeg: i32,
    pub nb_p: i32,
    pub cdim: i32,
    pub gap: usize,
    pub flat_knots: Array1<'static, f64>,
    pub poles: Vec<f64>,
}

impl SurfaceRows {
    pub fn of(bs: &GeomBSplineSurface, in_u: bool, rational: bool) -> Result<Self, Failure> {
        let gap: usize = if rational { 4 } else { 3 };
        let (cdeg, nb_p, across): (i32, i32, i32) = if in_u {
            (bs.u_degree(), bs.nb_u_poles(), bs.nb_v_poles())
        } else {
            (bs.v_degree(), bs.nb_v_poles(), bs.nb_u_poles())
        };
        let flat_knots: Array1<'static, f64> = if in_u {
            bs.u_knot_sequence()
        } else {
            bs.v_knot_sequence()
        };
        let surface_poles: Array2<'static, Point3> = bs.poles();
        let weights: Option<Array2<'static, f64>> = bs.weights();
        let mut poles: Vec<f64> = Vec::with_capacity(across as usize * gap * nb_p as usize);
        let (outer, inner): (i32, i32) = (nb_p, across);
        for o in 1..=outer {
            for i in 1..=inner {
                let (u, v) = if in_u { (o, i) } else { (i, o) };
                let pole: Point3 = crate::array_access::at(&surface_poles, u, v, "Poles")?;
                poles.extend([pole.x(), pole.y(), pole.z()]);
                if rational {
                    let w: f64 = weights
                        .as_ref()
                        .map_or(Ok(1.0), |w| crate::array_access::at(w, u, v, "Weights"))?;
                    poles.push(w);
                }
            }
        }
        Ok(Self {
            cdeg,
            nb_p,
            cdim: across * gap as i32,
            gap,
            flat_knots,
            poles,
        })
    }
}

/// The point and tangents of the boundary row, and the scale of each tangent that
/// makes the extension chord `Length` (`Lambda`, `lambmin`); `ok` is false when two
/// adjacent tangents make an angle above 2 rad.
pub(super) struct BoundaryTangents {
    pub point: Vec<f64>,
    pub tgte: Vec<f64>,
    pub lambda: Vec<f64>,
    pub lambmin: f64,
    pub ok: bool,
}

impl BoundaryTangents {
    pub fn at(rows: &SurfaceRows, side: BoundarySide) -> Result<Self, Failure> {
        let fk: &Array1<'static, f64> = &rows.flat_knots;
        // The parameter of the connection knot.
        let tbord: f64 = if side.after {
            *fk.value(fk.upper() - rows.cdeg).map_err(knots_error)?
        } else {
            *fk.value(fk.lower() + rows.cdeg).map_err(knots_error)?
        };
        let dimension: usize = rows.cdim as usize;
        let request: i32 = side.continuity.max(1);
        let mut result: Vec<f64> = vec![0.0; dimension * (request as usize + 1)];
        let extrap: [i32; 2] = [rows.cdeg, rows.cdeg];
        bsplclib::eval_extrapolated(
            tbord,
            false,
            request,
            &extrap,
            rows.cdeg,
            fk,
            rows.cdim,
            &rows.poles,
            &mut result,
        )?;
        let point: Vec<f64> = result[..dimension].to_vec();
        let mut tgte: Vec<f64> = result[dimension..2 * dimension].to_vec();
        let (lambda, lambmin, ok) = constraint_scales(&mut tgte, rows.gap, side)?;
        Ok(Self {
            point,
            tgte,
            lambda,
            lambmin,
            ok,
        })
    }
}

// Calculation of the constraint to reach: Length / |T| per pole across (0 for a null
// tangent), their minimum, and the reversal test; the weight tangents and scales of
// a rational surface are zeroed.
fn constraint_scales(
    tgte: &mut [f64],
    gap: usize,
    side: BoundarySide,
) -> Result<(Vec<f64>, f64, bool), Failure> {
    let mut lambda: Vec<f64> = vec![0.0; tgte.len()];
    let (mut lambmin, mut ok): (f64, bool) = (side.length, true);
    let (mut old_t, mut old_n): (Vector3, f64) = (Vector3::default(), 0.0);
    for coord in (0..tgte.len()).step_by(gap) {
        let cur_t = Vector3::new(tgte[coord], tgte[coord + 1], tgte[coord + 2]);
        let n_tgte: f64 = cur_t.magnitude();
        let mut val: f64 = 0.0;
        if n_tgte > THE_SURFACE_TANGENT_TOLERANCE {
            val = side.length / n_tgte;
            // A reversal between adjacent boundary tangents: raising the transverse
            // degree moves the surface poles closer to the boundary curve.
            if old_n > THE_SURFACE_TANGENT_TOLERANCE
                && cur_t.angle(old_t)? > THE_TANGENT_REVERSAL_ANGLE
            {
                ok = false;
            }
            lambmin = std_min(lambmin, val);
        }
        lambda[coord..coord + 3].fill(val);
        if side.rational {
            tgte[coord + 3] = 0.0;
            lambda[coord + 3] = 0.0;
        }
        (old_t, old_n) = (cur_t, n_tgte);
    }
    Ok((lambda, lambmin, ok))
}

fn knots_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    crate::native_failure::array_failure(error, "ExtendSurfByLength flat knots")
}

/// The extended poles and weights and the flat knots of the extended direction.
pub(super) struct ExtensionPoles {
    new_poles: Array2<'static, Point3>,
    new_weights: Array2<'static, f64>,
    flat_knots: Vec<f64>,
    cdeg: i32,
}

impl ExtensionPoles {
    /// BSplCLib::TangExtendToConstraint towards `Point + dir Lambda Tgte`, retried
    /// with a third of `lambmin` while a rational weight vanishes (five attempts).
    pub fn extend(
        rows: &SurfaceRows,
        tangents: &BoundaryTangents,
        (continuity, after): (i32, bool),
        in_u: bool,
        bs: &GeomBSplineSurface,
    ) -> Result<Self, Failure> {
        let rational: bool = rows.gap == 4;
        let (poles, constraint_point) = homogeneous(rows, tangents, after, rational)?;
        let mut fk: Vec<f64> = vec![0.0; (rows.nb_p + rows.cdeg + 1 + rows.cdeg) as usize];
        let mut p_res: Vec<f64> = vec![0.0; poles.len() + (rows.cdeg * rows.cdim) as usize];
        let mut lambmin: f64 = tangents.lambmin;
        let mut grids = ResultGrids::default();
        let mut attempt: i32 = 1;
        loop {
            let (nb_poles, nb_knots) = bsplclib::tang_extend_to_constraint(
                &rows.flat_knots,
                lambmin,
                rows.nb_p,
                &poles,
                rows.cdim,
                rows.cdeg,
                &constraint_point,
                continuity,
                after,
                &mut fk,
                &mut p_res,
            )?;
            let (nu, nv) = if in_u {
                (nb_poles, bs.nb_v_poles())
            } else {
                (bs.nb_u_poles(), nb_poles)
            };
            grids.resize(nu, nv)?;
            let null_weight: bool = grids.copy_result(&p_res, rows, in_u)?;
            if !null_weight || attempt == weight_attempts() {
                let flat_knots: Vec<f64> = fk[..nb_knots as usize].to_vec();
                return Ok(Self {
                    new_poles: grids.poles,
                    new_weights: grids.weights,
                    flat_knots,
                    cdeg: rows.cdeg,
                });
            }
            // Echec de l'Extension rationnelle.
            lambmin /= 3.0;
            attempt += 1;
        }
    }

    /// The resulting B-spline surface: the extended direction's knots from the flat
    /// knots, the other direction's knots and degree from `bs`.
    pub fn surface(
        &self,
        bs: &GeomBSplineSurface,
        in_u: bool,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let (u, v, degrees) = if in_u {
            let (knots, mults) = extended_knots(&self.flat_knots, bs.nb_u_knots() + 1, self.cdeg)?;
            (
                (knots, mults),
                (bs.v_knots(), bs.v_multiplicities()),
                (self.cdeg, bs.v_degree()),
            )
        } else {
            let (knots, mults) = extended_knots(&self.flat_knots, bs.nb_v_knots() + 1, self.cdeg)?;
            (
                (bs.u_knots(), bs.u_multiplicities()),
                (knots, mults),
                (bs.u_degree(), self.cdeg),
            )
        };
        Ok(Handle::new(GeomBSplineSurface::with_weights(
            &self.new_poles,
            &self.new_weights,
            (&u.0, &u.1),
            (&v.0, &v.1),
            degrees,
            (bs.is_u_periodic(), bs.is_v_periodic()),
        )?))
    }
}

// The constraint point `Point + dir Lambda Tgte`, and for a rational surface the
// poles and the constraint in homogeneous coordinates (x w, y w, z w, w).
fn homogeneous(
    rows: &SurfaceRows,
    tangents: &BoundaryTangents,
    after: bool,
    rational: bool,
) -> Result<(Vec<f64>, Array1<'static, f64>), Failure> {
    let direction: f64 = if after { 1.0 } else { -1.0 };
    let mut constraint_point: Array1<'static, f64> = one_based(rows.cdim)?;
    for (index, value) in constraint_point.as_mut_slice().iter_mut().enumerate() {
        *value = tangents.point[index] + direction * tangents.lambda[index] * tangents.tgte[index];
    }
    let mut poles: Vec<f64> = rows.poles.clone();
    if rational {
        for chunk in poles.chunks_mut(rows.gap) {
            let weight: f64 = chunk[3];
            chunk[0] *= weight;
            chunk[1] *= weight;
            chunk[2] *= weight;
        }
        for chunk in constraint_point.as_mut_slice().chunks_mut(rows.gap) {
            let weight: f64 = chunk[3];
            chunk[0] *= weight;
            chunk[1] *= weight;
            chunk[2] *= weight;
        }
    }
    Ok((poles, constraint_point))
}

// NewPoles and NewWeights: NCollection_Array2::Resize(NU, NV, false) keeps the cells
// when the size does not change (so a retry keeps the previous values where it does
// not write), and a new allocation starts cleared (the OCCT allocator clears it).
#[derive(Default)]
struct ResultGrids {
    poles: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
}

impl ResultGrids {
    fn resize(&mut self, nu: i32, nv: i32) -> Result<(), Failure> {
        if self.poles.col_length() != nu || self.poles.row_length() != nv {
            (self.poles, self.weights) = (grid(nu, nv)?, grid(nu, nv)?);
        }
        Ok(())
    }

    // Copy the result poles as 3D points and weights; the weights near 1 snap to 1,
    // and the copy stops at the first null weight (returning true), that pole left in
    // homogeneous coordinates.
    fn copy_result(
        &mut self,
        p_res: &[f64],
        rows: &SurfaceRows,
        in_u: bool,
    ) -> Result<bool, Failure> {
        let rational: bool = rows.gap == 4;
        if !rational {
            self.weights.init(1.0);
        }
        let cdim: usize = rows.cdim as usize;
        let (nu, nv) = (self.poles.col_length(), self.poles.row_length());
        for u in 0..nu {
            for v in 0..nv {
                let (iu, iv) = (u as usize, v as usize);
                let index: usize = if in_u {
                    iu * cdim + iv * rows.gap
                } else {
                    iu * rows.gap + iv * cdim
                };
                let pole = Point3::new(p_res[index], p_res[index + 1], p_res[index + 2]);
                put(&mut self.poles, u + 1, v + 1, pole, "NewPoles")?;
                if rational && self.divide_by_weight(p_res[index + 3], pole, (u + 1, v + 1))? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // The weight snapped to 1 when near it; true for a null weight (nothing
    // written), else the weight stored and the pole divided by it.
    fn divide_by_weight(
        &mut self,
        mut weight: f64,
        pole: Point3,
        (u, v): (i32, i32),
    ) -> Result<bool, Failure> {
        if (weight - 1.0).abs() < THE_SURFACE_WEIGHT_TOLERANCE {
            weight = 1.0;
        }
        if weight < THE_SURFACE_WEIGHT_TOLERANCE {
            return Ok(true);
        }
        put(&mut self.weights, u, v, weight, "NewWeights")?;
        let mut divided: Point3 = pole;
        divided.set_xyz(pole.xyz() / weight);
        put(&mut self.poles, u, v, divided, "NewPoles")?;
        Ok(false)
    }
}
