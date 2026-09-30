// Rust translation of TestArretCadre of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TestArretCadre`: the stop test of a line that reached the domain border, on the
//! processed start points, along the whole current line (which is cut at the stop)
//! and then on the last step.
use super::perform::Multiples;
use super::points::evaluate_at;
use super::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine};
use crate::int_walk::IntWalkWalkingData;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkmath::math::Vector;

/// `CutVectorByTolerances`: the components under the parameter resolutions become 0;
/// returns the scalar product of the two cut vectors.
fn cut_dot((x1, y1): (f64, f64), (x2, y2): (f64, f64), tolerance: &Vector<'_>) -> f64 {
    let cut = |value: f64, tol: f64| if value.abs() < tol { 0. } else { value };
    let (x1, y1) = (cut(x1, tolerance[1]), cut(y1, tolerance[2]));
    let (x2, y2) = (cut(x2, tolerance[1]), cut(y2, tolerance[2]));
    x1 * x2 + y1 * y2
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// Whether the step from `(up, vp)` to `(uc, vc)` passes by (or ends on) the point
    /// `(us, vs)`, with the components under the resolutions cut.
    fn cadre_passes(
        &self,
        (up, vp): (f64, f64),
        (uc, vc): (f64, f64),
        (us, vs): (f64, f64),
    ) -> bool {
        let scal: f64 = cut_dot((up - us, vp - vs), (uc - us, vc - vs), &self.tolerance);
        scal < 0.0 || ((uc - us).abs() < self.tolerance[1] && (vc - vs).abs() < self.tolerance[2])
    }
    /// Whether the step passes by start point `i` or one of its multiplicity points
    /// (the first `first` multiplicity points belong to the earlier start points).
    fn cadre_stops(
        &self,
        mult: &Multiples,
        (i, first): (usize, i32),
        previous: (f64, f64),
        current: (f64, f64),
    ) -> Result<bool, Failure> {
        let data: IntWalkWalkingData = self.wd1[i];
        if self.cadre_passes(previous, current, (data.ustart, data.vstart)) {
            return Ok(true);
        }
        if self.nb_multiplicities[i] <= 0 {
            return Ok(false);
        }
        for k in first + 1..=first + self.nb_multiplicities[i] {
            if self.cadre_passes(previous, current, mult.at(k)?) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// `TestArretCadre`: for each processed start point, looks along `line` for a step
    /// passing by it, cuts the line there and stops (`irang` its index); otherwise
    /// tests the step from the last line point to `uv` (`irang` then negative: the
    /// point must be added to the line). On a stop `uv` becomes the start point.
    pub(super) fn test_arret_cadre(
        &mut self,
        mult: &Multiples,
        line: &T::Line,
        sp: &mut T::Function,
        uv: &mut Vector<'_>,
        irang: &mut i32,
    ) -> Result<(), Failure> {
        *irang = 0;
        for i in 1..self.wd1.len() {
            if self.wd1[i].etat >= 0 {
                continue;
            }
            let first: i32 = if self.nb_multiplicities[i] > 0 {
                self.nb_multiplicities[1..i].iter().sum()
            } else {
                0
            };
            if self.cadre_along_line(mult, (i, first), line, sp, (uv, irang))? {
                return Ok(());
            }
        }
        Ok(())
    }
    /// The scan of one start point along the line, then on the last step; returns
    /// whether it stopped.
    fn cadre_along_line(
        &mut self,
        mult: &Multiples,
        (i, first): (usize, i32),
        line: &T::Line,
        sp: &mut T::Function,
        (uv, irang): (&mut Vector<'_>, &mut i32),
    ) -> Result<bool, Failure> {
        let mut previous: (f64, f64) = self.surface_parameters(&line.value(1)?);
        for j in 2..=line.nb_points() {
            let current: (f64, f64) = self.surface_parameters(&line.value(j)?);
            if self.cadre_stops(mult, (i, first), previous, current)? {
                line.cut(j)?;
                self.stop_on_start(i, uv, irang);
                evaluate_at(sp, uv)?;
                self.fix_cut_tangent(line, sp, j)?;
                return Ok(true);
            }
            previous = current;
        }
        if !self.cadre_stops(mult, (i, first), previous, (uv[1], uv[2]))? {
            return Ok(false);
        }
        self.stop_on_start(i, uv, irang);
        *irang = -*irang;
        evaluate_at(sp, uv)?;
        Ok(true)
    }
    /// A stop on start point `i`: `irang` is `i` and `uv` its parameters.
    fn stop_on_start(&self, i: usize, uv: &mut Vector<'_>, irang: &mut i32) {
        *irang = i as i32;
        (uv[1], uv[2]) = (self.wd1[i].ustart, self.wd1[i].vstart);
    }
    /// After a cut at `j`: a tangent vector stored past the new end moves to the new
    /// last point (`previousd3d`); the section tangent is read (it can raise) and its
    /// reversal against the last chord is computed and unused, as natively.
    fn fix_cut_tangent(&self, line: &T::Line, sp: &mut T::Function, j: i32) -> Result<(), Failure> {
        let nbp: i32 = line.nb_points();
        let (_, indextg) = line.tangent_vector();
        if indextg <= nbp || !(j > 3 && j <= nbp + 1) {
            return Ok(());
        }
        let mut dir3d: Vector3 = sp.direction3d()?;
        let chord: Vector3 =
            Vector3::between(line.value(j - 2)?.value(), line.value(j - 1)?.value());
        if dir3d.dot(chord) < 0.0 {
            dir3d.reverse();
        }
        line.set_tangent_vector(self.previousd3d, j - 1);
        Ok(())
    }
}
