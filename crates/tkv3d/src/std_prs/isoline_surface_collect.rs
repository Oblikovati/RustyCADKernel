//! Native surface catch boundary suppresses checked failures while preserving completed intervals and caller lists.
use super::StdPrsIsolineSurfaceSource;
use super::isoline_surface_reference::IsolineSurfaceReference;
use super::{
    StdPrsIsolineDestination, isoline_surface_curves::IsolineSurfaceCurves,
    isoline_surface_hatch::trimmed_isoline_hatcher, isoline_surface_trim::surface_isoline_trimming,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, SharedState, catch_native_failure},
};
use rustycad_tkgeomalgo::hatch::Hatcher;
pub(super) fn collect_surface_isolines<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    surface: &IsolineSurfaceReference<'_, S>,
    drawer: &Prs3dDrawer,
    deflection: f64,
    parameters: [&SharedState<Sequence<f64>>; 2],
    output: &mut StdPrsIsolineDestination<'_>,
) {
    // OCCT3d097a0 catches Standard_Failure here only; unrelated Rust panic payloads must still resume.
    let _ = catch_native_failure(|| {
        sample_surface_isolines(surface, drawer, deflection, parameters, output)
    });
}
fn sample_surface_isolines<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    surface: &IsolineSurfaceReference<'_, S>,
    drawer: &Prs3dDrawer,
    deflection: f64,
    parameters: [&SharedState<Sequence<f64>>; 2],
    output: &mut StdPrsIsolineDestination<'_>,
) -> Result<(), Failure> {
    let hatcher = surface_isoline_hatcher(surface, drawer, parameters)?;
    let curves = IsolineSurfaceCurves::new(surface)?;
    for line in 1..=hatcher.nb_lines() {
        sample_isoline_intervals(&hatcher, &curves, line, drawer, deflection, output)?;
    }
    Ok(())
}
fn sample_isoline_intervals(
    hatcher: &Hatcher,
    curves: &IsolineSurfaceCurves,
    line: i32,
    drawer: &Prs3dDrawer,
    deflection: f64,
    output: &mut StdPrsIsolineDestination<'_>,
) -> Result<(), Failure> {
    let parameter = hatcher.coordinate(line)?;
    let is_u = hatcher.is_x_line(line)?;
    for interval in 1..=hatcher.nb_intervals_on(line)? {
        let range = [hatcher.start(line, interval)?, hatcher.end(line, interval)?];
        let points = curves.sample(is_u, parameter, range, drawer, deflection)?;
        if !points.is_empty() {
            output.append(is_u, points);
        }
    }
    Ok(())
}

fn surface_isoline_hatcher<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    surface: &IsolineSurfaceReference<'_, S>,
    drawer: &Prs3dDrawer,
    parameters: [&SharedState<Sequence<f64>>; 2],
) -> Result<Hatcher, Failure> {
    let region = surface_isoline_trimming(
        surface,
        drawer.maximal_parameter_value(),
        drawer.deviation_coefficient(),
    )?;
    trimmed_isoline_hatcher(&region, parameters)
}
