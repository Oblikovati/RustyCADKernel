//! Synthetic surface polylines expose root search, insertion and index mutation.
use super::super::{
    common::{Reader, attempt, end, int, show, text},
    contap_surface, walking_internal,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkhlr::contap::{ContapLine, ContapPoint};
fn fixture(
    surface: &Adaptor3dSurfaceHandle,
    count: i32,
    path: i32,
    seed: bool,
) -> Result<ContapLine, Failure> {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for index in 0..count {
        let t = if count == 1 {
            0.
        } else {
            -1.5 + 3. * f64::from(index) / f64::from(count - 1)
        };
        let (u, v) = match path {
            0 => (t, 0.),
            1 => (std::f64::consts::FRAC_PI_2, t),
            _ => (t, t * 0.25),
        };
        let mut point = IntSurfPntOn2S::new();
        point.set_value_all(h_surface_tool::value(surface, u, v)?, 17., 19., u, v);
        points.add(&point);
    }
    let mut line = ContapLine::new();
    line.set_line_on_2s(points);
    if seed && count > 0 {
        seed_vertices(&line, count)?;
    }
    Ok(line)
}
fn seed_vertices(line: &ContapLine, count: i32) -> Result<(), Failure> {
    for index in [1, (count + 1) / 2, count] {
        let source = line.point(index)?;
        let (u, v) = source.parameters_on_s2();
        let mut point = ContapPoint::with_value(source.value(), u, v);
        point.set_parameter(f64::from(index));
        line.add_vertex(&point);
    }
    Ok(())
}
pub fn walking_internal_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let (_, mut function) = super::attachment::functions(&surface, &mut reader)?;
        let (count, path, seed) = (reader.integer(), reader.integer(), reader.flag());
        let line = fixture(&surface, count, path, seed)?;
        let mut console = Vec::new();
        let result = walking_internal::compute_walking_internal(
            &line,
            &mut function,
            1e-7,
            1e-7,
            &mut console,
        );
        text(&console);
        show(result, |()| {});
        super::sections::report(&line)?;
        int(-1);
        Ok(())
    });
    end();
}
