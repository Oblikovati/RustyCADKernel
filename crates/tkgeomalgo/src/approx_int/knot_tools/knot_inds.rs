//! The draft knot indices: the extrema of the discrete curvature, knots in its
//! monotone intervals and around the sharp feature points.
use super::{ApproxIntKnotTools, seq_at};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkmath::math::Vector;

/// `aSinCoeff^2 = (3 - sqrt(5)) / 8`.
pub(super) const SIN_COEFF2: f64 = 0.09549150281252627;

/// The curvature and the coordinates of the point set (`theCurv`, `theCoords`,
/// `theDim`).
pub(super) struct CurvatureData<'a> {
    pub(super) curv: &'a Array1<'a, f64>,
    pub(super) coords: &'a [f64],
    pub(super) dim: usize,
}

impl CurvatureData<'_> {
    fn curv(&self, i: i32) -> Result<f64, Failure> {
        arrays::at(self.curv, i)
    }

    /// Whether the angle at `mid` between the chords from `prev` and to `next` has a
    /// sine above `aSinCoeff` (`mp > aSinCoeff2 * m1 * m2`, the outer product of the
    /// chords).
    pub(super) fn sharp(&self, prev: i32, mid: i32, next: i32) -> bool {
        let lower: i32 = self.curv.lower();
        let dim: usize = self.dim;
        let ici: usize = (prev - lower) as usize * dim;
        let ici1: usize = (next - lower) as usize * dim;
        let icm: usize = (mid - lower) as usize * dim;
        let c = self.coords;
        let mut v1: Vec<f64> = vec![0.0; dim];
        let mut v2: Vec<f64> = vec![0.0; dim];
        let (mut mp, mut m1, mut m2): (f64, f64, f64) = (0.0, 0.0, 0.0);
        for k in 0..dim {
            v1[k] = c[icm + k] - c[ici + k];
            m1 += v1[k] * v1[k];
            v2[k] = c[ici1 + k] - c[icm + k];
            m2 += v2[k] * v2[k];
        }
        for k in 1..dim {
            for l in 0..k {
                let p: f64 = v1[k] * v2[l] - v1[l] * v2[k];
                mp += p * p;
            }
        }
        mp > SIN_COEFF2 * m1 * m2
    }
}

impl ApproxIntKnotTools {
    /// `InsKnotBefI(theI, theCurv, theCoords, theDim, theInds, ChkCurv)`: inserts a knot
    /// before the index `the_i` of `inds` where the curvature changes more than 3 times
    /// on the interval, else at the point nearest to the mean curvature (checked for a
    /// sharp angle when `chk_curv`); false when nothing is inserted.
    pub(super) fn ins_knot_bef_i(
        the_i: i32,
        data: &CurvatureData<'_>,
        inds: &mut Vec<i32>,
        chk_curv: bool,
    ) -> Result<bool, Failure> {
        let an_ind1: i32 = seq_at(inds, the_i)?;
        let an_ind: i32 = seq_at(inds, the_i - 1)?;
        if an_ind1 - an_ind == 1 {
            return Ok(false);
        }
        let curv: f64 = 0.5 * (data.curv(an_ind)? + data.curv(an_ind1)?);
        let limit_curvature_change: f64 = 3.0;
        let conf: f64 = precision::confusion();
        for j in an_ind + 1..an_ind1 {
            let (cj, ca) = (data.curv(j)?, data.curv(an_ind)?);
            // I: curvature change criterion (non-null curvature).
            if cj > conf
                && ca > conf
                && (cj / ca > limit_curvature_change || cj / ca < 1.0 / limit_curvature_change)
            {
                insert_before(inds, the_i, j)?;
                return Ok(true);
            }
            // II: angular criterion.
            let mid: i32 = Self::mean_point(data, (an_ind, an_ind1), j, curv)?;
            if mid > 0 && (!chk_curv || data.sharp(an_ind, mid, an_ind1)) {
                insert_before(inds, the_i, mid)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The point `j - 1` or `j` nearest to the mean curvature when it lies between their
    /// curvatures, kept off the ends of the interval; 0 otherwise.
    fn mean_point(
        data: &CurvatureData<'_>,
        (an_ind, an_ind1): (i32, i32),
        j: i32,
        curv: f64,
    ) -> Result<i32, Failure> {
        let mut mid: i32 = 0;
        let (ac, ac1) = (data.curv(j - 1)?, data.curv(j)?);
        if (curv >= ac && curv <= ac1) || (curv >= ac1 && curv <= ac) {
            mid = if (curv - ac).abs() < (curv - ac1).abs() {
                j - 1
            } else {
                j
            };
        }
        if mid == an_ind {
            mid += 1;
        }
        if mid == an_ind1 {
            mid -= 1;
        }
        Ok(mid)
    }

    /// `ComputeKnotInds(theCoords, theDim, thePars, theInds)`: the draft knot indices
    /// of the point set; `ApproxIntKnotTools::compute_knot_inds(&coords, 3, &pars, &mut inds)?`.
    pub fn compute_knot_inds(
        coords: &[f64],
        dim: i32,
        pars: &Vector<'_>,
        inds: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        // I: the discrete curvature.
        let mut curv: Array1<'static, f64> = Array1::with_bounds(pars.lower(), pars.upper())
            .map_err(|e| e.into_failure("ApproxInt_KnotTools::ComputeKnotInds"))?;
        let max_curv: f64 = Self::build_curvature(coords, dim, pars, &mut curv)?;
        inds.push(curv.lower());
        if max_curv <= precision::confusion() {
            // Linear case.
            inds.push(curv.upper());
            return Ok(());
        }
        let data = CurvatureData {
            curv: &curv,
            coords,
            dim: dim as usize,
        };
        // II: the extrema of the curvature.
        let feature_inds: Vec<i32> = Self::curvature_extrema(&data, inds)?;
        // III: knots in the monotone intervals of the curvature.
        let mut i: i32 = 1;
        loop {
            i += 1;
            if Self::ins_knot_bef_i(i, &data, inds, true)? {
                i -= 1;
            }
            if i >= inds.len() as i32 {
                break;
            }
        }
        // IV: the feature points.
        Self::check_feature_points(&data, &feature_inds, inds)
    }

    /// Step II of `ComputeKnotInds`: appends the extrema of the curvature (and the last
    /// index) to `inds`; the extrema are the feature points.
    fn curvature_extrema(
        data: &CurvatureData<'_>,
        inds: &mut Vec<i32>,
    ) -> Result<Vec<i32>, Failure> {
        // Not Precision::PConfusion: the eps is not of the parameter space kind.
        let eps: f64 = 1.0e-9;
        let eps1: f64 = 1.0e3 * eps;
        let (lower, upper) = (data.curv.lower(), data.curv.upper());
        let mut feature_inds: Vec<i32> = Vec::new();
        for i in lower + 1..upper {
            let d1: f64 = data.curv(i)? - data.curv(i - 1)?;
            let d2: f64 = data.curv(i)? - data.curv(i + 1)?;
            let (ad1, ad2) = (d1.abs(), d2.abs());
            let extremum: bool = (d1 * d2 > 0.0 && ad1 > eps && ad2 > eps)
                || (ad1 < eps && ad2 > eps1)
                || (ad1 > eps1 && ad2 < eps);
            if extremum && i != inds[inds.len() - 1] {
                inds.push(i);
                feature_inds.push(i);
            }
        }
        if upper != inds[inds.len() - 1] {
            inds.push(upper);
        }
        Ok(feature_inds)
    }

    /// Step IV of `ComputeKnotInds`: around each sharp feature point, a knot is inserted
    /// on the side of the larger curvature jump.
    fn check_feature_points(
        data: &CurvatureData<'_>,
        feature_inds: &[i32],
        inds: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let mut j: i32 = 2;
        for &an_ind in feature_inds {
            while j < inds.len() as i32 {
                if seq_at(inds, j)? != an_ind {
                    j += 1;
                    continue;
                }
                let prev: i32 = seq_at(inds, j - 1)?;
                let next: i32 = seq_at(inds, j + 1)?;
                if !data.sharp(prev, an_ind, next) {
                    j += 1;
                    break;
                }
                // Insert new knots.
                let d1: f64 = (data.curv(an_ind)? - data.curv(prev)?).abs();
                let d2: f64 = (data.curv(an_ind)? - data.curv(next)?).abs();
                if d1 > d2 {
                    if !Self::ins_knot_bef_i(j, data, inds, false)? {
                        break;
                    }
                    j += 1;
                } else if !Self::ins_knot_bef_i(j + 1, data, inds, false)? {
                    break;
                }
            }
        }
        Ok(())
    }
}

/// `NCollection_Sequence::InsertBefore(Index, Item)` on a 1-based sequence.
fn insert_before(inds: &mut Vec<i32>, index: i32, item: i32) -> Result<(), Failure> {
    seq_at(inds, index)?;
    inds.insert((index - 1) as usize, item);
    Ok(())
}
