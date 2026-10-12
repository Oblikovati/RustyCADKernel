//! Real Hatch_Hatcher insertion/trim ordering; no rectangular-face shortcut replaces p-curve boundaries.
use super::isoline_surface_trim::IsolineTrimRegion;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, SharedState},
};
use rustycad_tkgeomalgo::hatch::Hatcher;
pub(super) fn trimmed_isoline_hatcher(
    region: &IsolineTrimRegion,
    parameters: [&SharedState<Sequence<f64>>; 2],
) -> Result<Hatcher, Failure> {
    let mut hatcher = Hatcher::new(region.tolerance, true);
    load_isoline_hatch_axis(&mut hatcher, region, parameters[0], true)?;
    load_isoline_hatch_axis(&mut hatcher, region, parameters[1], false)?;
    for pair in region.points.as_chunks::<2>().0 {
        hatcher.trim_segment(pair[0], pair[1], 0)?;
    }
    Ok(hatcher)
}
fn load_isoline_hatch_axis(
    hatcher: &mut Hatcher,
    region: &IsolineTrimRegion,
    parameters: &SharedState<Sequence<f64>>,
    is_u: bool,
) -> Result<(), Failure> {
    let range = &region.ranges[usize::from(!is_u)];
    let mut rank = 1;
    while rank <= parameters.read().length() {
        let coordinate = *parameters.read().value(rank as usize)?;
        if range.is_void() || !range.is_out(coordinate) {
            insert_isoline_hatch(hatcher, is_u, coordinate);
        }
        rank += 1;
    }
    Ok(())
}
fn insert_isoline_hatch(hatcher: &mut Hatcher, is_u: bool, coordinate: f64) {
    if is_u {
        hatcher.add_x_line(coordinate);
    } else {
        hatcher.add_y_line(coordinate);
    }
}
