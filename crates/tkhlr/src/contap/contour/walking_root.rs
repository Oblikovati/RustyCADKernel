//! Shifted bisection and the exact native FunctionSetRoot callback sequence.
use crate::contap::{ContapSurfFunction, ContapTFunction};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::{self, Point3, Vector3},
    math::{FunctionSetRoot, Matrix, Vector},
};
pub(super) type Uv = [f64; 2];
pub(super) struct WalkingRoot {
    solver: FunctionSetRoot,
    tolerances: Vector<'static>,
    lower: Vector<'static>,
    upper: Vector<'static>,
    residual: Vector<'static>,
    gradient: Matrix<'static>,
}
pub(super) fn parameter_vector(uv: Uv) -> Result<Vector<'static>, Failure> {
    let mut result = Vector::with_bounds(1, 2)?;
    result[1] = uv[0];
    result[2] = uv[1];
    Ok(result)
}
impl WalkingRoot {
    pub fn new(
        function: &ContapSurfFunction,
        u_tolerance: f64,
        v_tolerance: f64,
    ) -> Result<Self, Failure> {
        let tolerances = parameter_vector([u_tolerance, v_tolerance])?;
        let surface = function.surface();
        let lower = parameter_vector([surface.first_u_parameter()?, surface.first_v_parameter()?])?;
        let upper = parameter_vector([surface.last_u_parameter()?, surface.last_v_parameter()?])?;
        Ok(Self {
            solver: FunctionSetRoot::new(function, &tolerances, 30)?,
            tolerances,
            lower,
            upper,
            residual: Vector::with_bounds(1, 1)?,
            gradient: Matrix::with_bounds(1, 1, 1, 2).expect("fixed contour derivative shape"),
        })
    }
    pub fn near(&self, a: Uv, b: Uv) -> bool {
        (a[0] - b[0]).abs() <= self.tolerances[1] && (a[1] - b[1]).abs() <= self.tolerances[2]
    }
    pub fn evaluate(&mut self, function: &mut ContapSurfFunction, uv: Uv) -> Result<(), Failure> {
        function.values(
            &parameter_vector(uv)?,
            &mut self.residual,
            &mut self.gradient,
        )?;
        Ok(())
    }
    pub fn solve(
        &mut self,
        function: &mut ContapSurfFunction,
        mut low: Uv,
        mut high: Uv,
        reference: Vector3,
        diagnostics: &mut dyn std::io::Write,
    ) -> Result<Option<Uv>, Failure> {
        loop {
            let candidate = [
                low[0] + (2. / 3.) * (high[0] - low[0]),
                low[1] + (2. / 3.) * (high[1] - low[1]),
            ];
            let Some(uv) = self.project(function, candidate, diagnostics)? else {
                return Ok(None);
            };
            let cross = view_cross(function, function.point())?;
            if cross.magnitude() <= gp::resolution() || self.near(uv, low) || self.near(uv, high) {
                return Ok(Some(uv));
            }
            if cross.dot(reference) < 0. {
                high = uv;
            } else {
                low = uv;
            }
        }
    }
    fn project(
        &mut self,
        function: &mut ContapSurfFunction,
        start: Uv,
        diagnostics: &mut dyn std::io::Write,
    ) -> Result<Option<Uv>, Failure> {
        self.solver.perform_bounded(
            function,
            &parameter_vector(start)?,
            &self.lower,
            &self.upper,
            false,
        )?;
        function.take_failure()?;
        if !self.solver.is_done() {
            return Ok(failed_projection(diagnostics));
        }
        let root = self.solver.root()?;
        let uv = [root[1], root[2]];
        self.evaluate(function, uv)?;
        if self.residual[1].abs() <= function.tolerance() {
            return Ok(Some(uv));
        }
        Ok(failed_projection(diagnostics))
    }
}
fn failed_projection(diagnostics: &mut dyn std::io::Write) -> Option<Uv> {
    // OCCT 3d097a0 emits this user-visible diagnostic on both failed convergence paths.
    let _ = writeln!(diagnostics, "Echec recherche internal points");
    None
}
pub(super) fn view_cross(
    function: &mut ContapSurfFunction,
    point: Point3,
) -> Result<Vector3, Failure> {
    if function.is_tangent()? {
        return Ok(Vector3::default());
    }
    let tangent = function.direction3d()?;
    let direction = match function.function_type() {
        ContapTFunction::ContourPrs | ContapTFunction::DraftPrs => {
            Vector3::from(point.xyz() - function.eye().xyz())
        }
        _ => Vector3::from(function.direction()),
    };
    Ok(direction.crossed(tangent))
}
