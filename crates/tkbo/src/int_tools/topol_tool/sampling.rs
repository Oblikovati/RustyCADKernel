use super::*;
impl TopolTool {
    /// Evaluates the native interior grid (zero based), or explicit parameters (one based); `tool.sample_point(1)?`.
    pub fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        if self.sampling.read().parameters.is_none() {
            self.counts()?;
        }
        let (uv, surface) = {
            let state = self.sampling.read();
            let uv = if let Some(parameters) = &state.parameters {
                explicit_parameters(parameters, state.base_counts[0], index)?
            } else {
                let iv = 1 + index / state.counts[0];
                let iu = 1 + index - (iv - 1) * state.counts[0];
                [
                    state.origin[0] + f64::from(iu) * state.step[0],
                    state.origin[1] + f64::from(iv) * state.step[1],
                ]
            };
            (uv, state.surface.clone())
        };
        Ok((Point2::new(uv[0], uv[1]), surface.value(uv[0], uv[1])?))
    }
    /// Uses the native adaptive B-spline sampler or regular parameter arrays; `tool.sample_pnts(0.01, 10, 10)?`.
    pub fn sample_pnts(&self, deflection: f64, nu: i32, nv: i32) -> Result<(), Failure> {
        let surface = self.sampling.read().surface.clone();
        let (u, v) =
            Adaptor3dTopolTool::sample_pnts_parameters(&*surface, (deflection, nu, nv), || {
                self.compute_sample_points()?;
                let counts = self.counts()?;
                Ok((counts[0], counts[1]))
            })?;
        self.store_parameters(u, v);
        Ok(())
    }
    fn store_parameters(&self, u: Array1<'static, f64>, v: Array1<'static, f64>) {
        let origin = [*u.first().unwrap(), *v.first().unwrap()];
        let counts = [u.length(), v.length()];
        let step = [
            (*u.last().unwrap() - origin[0]) / f64::from(counts[0] - 1),
            (*v.last().unwrap() - origin[1]) / f64::from(counts[1] - 1),
        ];
        let mut state = self.sampling.write();
        state.counts = counts;
        state.base_counts = counts;
        state.origin = origin;
        state.step = step;
        state.parameters = Some([u, v]);
    }
    /// Copies the inherited explicit U parameters; `tool.u_parameters()?`.
    pub fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        self.parameters(0)
    }
    /// Copies the inherited explicit V parameters; `tool.v_parameters()?`.
    pub fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        self.parameters(1)
    }
    fn parameters(&self, axis: usize) -> Result<Array1<'static, f64>, Failure> {
        self.sampling
            .read()
            .parameters
            .as_ref()
            .map(|arrays| arrays[axis].clone())
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NullObject",
                    "IntTools_TopolTool: no parameters; expected SamplePnts()",
                    "",
                )
            })
    }
}
fn explicit_parameters(
    parameters: &[Array1<'static, f64>; 2],
    nu: i32,
    index: i32,
) -> Result<[f64; 2], Failure> {
    let (u, v) = ((index - 1) % nu + 1, (index - 1) / nu + 1);
    let error = |e: rustycad_tkernel::ncollection::ArrayError| {
        e.into_failure("IntTools_TopolTool::SamplePoint")
    };
    Ok([
        *parameters[0].value(u).map_err(error)?,
        *parameters[1].value(v).map_err(error)?,
    ])
}
