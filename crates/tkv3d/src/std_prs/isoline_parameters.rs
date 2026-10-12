//! Native UV output ordering, aliased scalar references and asymmetric open-boundary inset.
use super::StdPrsIsolines;
use rustycad_tkbrep::{brep::BRepTool, brep_tools, topods::Face};
use rustycad_tkernel::{
    ncollection::Sequence,
    precision,
    standard::{Failure, SharedState},
};
/// Native output references permit aliasing without copies; `StdPrsIsolineUvOutputs { u, v, bounds }`.
pub struct StdPrsIsolineUvOutputs<'a> {
    /// Appended U parameters; may refer to the same sequence as V.
    pub u: &'a SharedState<Sequence<f64>>,
    /// Appended V parameters.
    pub v: &'a SharedState<Sequence<f64>>,
    /// Umin/Umax/Vmin/Vmax references, including valid aliases among them.
    pub bounds: [&'a SharedState<f64>; 4],
}
impl StdPrsIsolineUvOutputs<'_> {
    fn write_bounds(&self, limits: [f64; 4]) {
        for (destination, value) in self.bounds.iter().zip(limits) {
            *destination.write() = value;
        }
    }
    fn read_bounds(&self) -> [f64; 4] {
        // BRepTools writes precede local copies, so scalar-reference aliases affect all subsequent limits.
        self.bounds.map(|cell| *cell.read())
    }
}
impl StdPrsIsolines {
    /// Append UV isoline parameters, preserving caller references; `StdPrsIsolines::uv_iso_parameters(&face,[2,3],100.,outputs)?`.
    pub fn uv_iso_parameters(
        face: &Face,
        counts: [i32; 2],
        limit: f64,
        outputs: StdPrsIsolineUvOutputs<'_>,
    ) -> Result<(), Failure> {
        let (surface, _) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(());
        }
        let (u0, u1, v0, v1) = brep_tools::uv_bounds(face)?;
        outputs.write_bounds([u0, u1, v0, v1]);
        let limits = outputs.read_bounds();
        let closed = [surface.is_u_closed()?, surface.is_v_closed()?];
        append_isoline_parameter_axes(&outputs, limits, counts, limit, closed);
        Ok(())
    }
}
fn append_isoline_parameters(
    output: &SharedState<Sequence<f64>>,
    endpoints: [f64; 2],
    count: i32,
    limit: f64,
    closed: bool,
) {
    let [first, last] = isoline_parameter_range(endpoints, limit, closed);
    let step = (last - first) / f64::from(count + 1);
    for index in 1..=count {
        output.write().append(first + step * f64::from(index));
    }
}
fn isoline_parameter_range([mut first, mut last]: [f64; 2], limit: f64, closed: bool) -> [f64; 2] {
    if precision::is_infinite(first) {
        first = -limit;
    }
    if precision::is_infinite(last) {
        last = limit;
    }
    if !closed {
        // OCCT3d097a0 computes the upper inset from the already adjusted lower bound.
        first += (last - first) / 1000.;
        last -= (last - first) / 1000.;
    }
    [first, last]
}
pub(super) fn face_isoline_parameters(
    face: &Face,
    counts: [i32; 2],
    limit: f64,
) -> Result<([Sequence<f64>; 2], [f64; 4]), Failure> {
    let u = SharedState::new(Sequence::default());
    let v = SharedState::new(Sequence::default());
    let bounds: [SharedState<f64>; 4] = std::array::from_fn(|_| SharedState::new(0.));
    let outputs = StdPrsIsolineUvOutputs {
        u: &u,
        v: &v,
        bounds: bounds.each_ref(),
    };
    StdPrsIsolines::uv_iso_parameters(face, counts, limit, outputs)?;
    Ok((
        [u.snapshot(), v.snapshot()],
        bounds.each_ref().map(|value| *value.read()),
    ))
}

fn append_isoline_parameter_axes(
    outputs: &StdPrsIsolineUvOutputs<'_>,
    limits: [f64; 4],
    counts: [i32; 2],
    limit: f64,
    closed: [bool; 2],
) {
    for (axis, output) in [outputs.u, outputs.v].into_iter().enumerate() {
        append_isoline_parameters(
            output,
            [limits[axis * 2], limits[axis * 2 + 1]],
            counts[axis],
            limit,
            closed[axis],
        );
    }
}
