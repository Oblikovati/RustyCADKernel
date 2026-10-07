// Rust translation of ChFi2d_FilletAlgo::PerformNewton and ProcessPoint
// (ChFi2d_FilletAlgo.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The recursive Newton search of the roots of the fillet function on an interval.
use super::{ChFi2dFilletAlgo, FilletPoint};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

impl ChFi2dFilletAlgo {
    /// `PerformNewton(theLeft, theRight)`: a root at the left point (or at the right one
    /// on a tiny interval) is stored; else for each value of the left point, the
    /// parabola through both points gives the parameter where the interval is split.
    pub(super) fn perform_newton(
        &mut self,
        left: &FilletPoint,
        right: &FilletPoint,
    ) -> Result<(), Failure> {
        // check the left: if this is solution store it
        let a = left.has_solution(self.radius);
        if a != 0 {
            if left.is_valid(a)? {
                self.store_root(left.get_param());
            }
            return Ok(());
        }
        let dx = right.get_param() - left.get_param();
        if dx < 1.0e-6 * precision::confusion() {
            let a = right.has_solution(self.radius);
            if a != 0 && right.is_valid(a)? {
                self.store_root(right.get_param());
            }
            return Ok(());
        }
        for a in 1..=left.get_nb_values() {
            self.newton_on_value(left, right, a, dx)?;
        }
        Ok(())
    }

    /// The root found at `param` on the current side.
    fn store_root(&mut self, param: f64) {
        self.result_params.push(param);
        self.result_orientation.push(i32::from(self.start_side));
    }

    /// The split of the interval for the value `a` of the left point: at the root (or
    /// the extremum) of the parabola, else in the middle.
    fn newton_on_value(
        &mut self,
        left: &FilletPoint,
        right: &FilletPoint,
        a: i32,
        dx: f64,
    ) -> Result<(), Failure> {
        let (aa, b, c) = parabola(left, right, a, dx)?;
        let lp = left.get_param();
        let det = b * b - 2.0 * aa * c;
        let middle = lp + dx / 2.0; // linear division otherwise
        if aa.abs() < precision::confusion() {
            return self.linear_split(left, right, (b, c), middle);
        }
        if b.abs() > (det * 1000000.0).abs() {
            // possible floating point operations accuracy errors
            self.process_point(left, right, middle)?;
            return Ok(());
        }
        if det > 0.0 {
            // two solutions
            let det = det.sqrt();
            let done = self.process_point(left, right, (-b + det) / aa)?
                || self.process_point(left, right, (-b - det) / aa)?;
            if !done {
                self.process_point(left, right, middle)?;
            }
            return Ok(());
        }
        let x0 = -b / aa; // use extremum
        let target = if x0 > lp && x0 < right.get_param() {
            x0
        } else {
            middle
        };
        self.process_point(left, right, target)?;
        Ok(())
    }

    /// The linear case of the parabola `b x + c`: the split at its root when it lies
    /// inside the interval, in the middle when `b` vanishes.
    fn linear_split(
        &mut self,
        left: &FilletPoint,
        right: &FilletPoint,
        (b, c): (f64, f64),
        middle: f64,
    ) -> Result<(), Failure> {
        if b.abs() > 10e-20 {
            let x0 = -c / b; // use extremum
            if x0 > left.get_param() && x0 < right.get_param() {
                self.process_point(left, right, x0)?;
            }
            return Ok(());
        }
        self.process_point(left, right, middle)?;
        Ok(())
    }

    /// `ProcessPoint(theLeft, theRight, theParameter)`: the interval split at the
    /// parameter (kept a hundredth of the interval away from its ends) and the Newton
    /// method on both halves; false when the parameter is outside. The recursion is
    /// stopped after 3000 levels, the derivative step grown after 100 and 1000.
    fn process_point(
        &mut self,
        left: &FilletPoint,
        right: &FilletPoint,
        parameter: f64,
    ) -> Result<bool, Failure> {
        if !(parameter >= left.get_param() && parameter < right.get_param()) {
            return Ok(false);
        }
        let dx = right.get_param() - left.get_param();
        let mut parameter = parameter;
        if parameter - left.get_param() < dx / 100.0 {
            parameter = left.get_param() + dx / 100.0;
        }
        if right.get_param() - parameter < dx / 100.0 {
            parameter = right.get_param() - dx / 100.0;
        }
        // Protection on infinite loops.
        self.degree_of_recursion += 1;
        let Some(diffx) = self.recursion_step(dx) else {
            return Ok(true);
        };
        let mut point1 = left.copy();
        let mut point2 = FilletPoint::new(parameter);
        self.fill_point(&mut point2, point1.get_param())?;
        self.fill_diff(&mut point2, diffx, true)?;
        point1.filter_points(&point2)?;
        self.perform_newton(&point1, &point2)?;
        point2.filter_points(right)?;
        self.perform_newton(&point2, right)?;
        Ok(true)
    }

    /// The derivative step of a split by the recursion depth, `None` beyond 3000.
    fn recursion_step(&self, dx: f64) -> Option<f64> {
        let mut diffx = 0.001 * dx;
        if self.degree_of_recursion > 100 {
            diffx *= 10.0;
            if self.degree_of_recursion > 1000 {
                diffx *= 10.0;
                if self.degree_of_recursion > 3000 {
                    return None;
                }
            }
        }
        Some(diffx)
    }
}

/// The coefficients `(a, b, c)` of the parabola `a x^2 / 2 + b x + c` whose derivative
/// joins the differences of the value `index` of the left point and of its near value
/// on the right point, through the left value.
fn parabola(
    left: &FilletPoint,
    right: &FilletPoint,
    index: i32,
    dx: f64,
) -> Result<(f64, f64, f64), Failure> {
    let near = left.get_near(index)?;
    let (lp, ld, lv) = (
        left.get_param(),
        left.get_diff(index)?,
        left.get_value(index)?,
    );
    let aa = (right.get_diff(near)? - ld) / dx;
    let b = ld - aa * lp;
    let c = lv - ld * lp + aa * lp * lp / 2.0;
    Ok((aa, b, c))
}
