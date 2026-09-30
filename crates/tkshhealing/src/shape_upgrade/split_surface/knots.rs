//! Native advancing knot scan snaps split values without re-sorting them.
use super::*;
impl SplitSurfaceData {
    pub(super) fn snap_knots(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        let spline = surface.cast::<GeomBSplineSurface>();
        if spline.is_null() {
            return Ok(());
        }
        let copy: Handle<GeomBSplineSurface> = spline.copy()?.cast();
        snap_direction(&copy, &self.values(true), true)?;
        snap_direction(&copy, &self.values(false), false)
    }
}
fn snap_direction(
    surface: &GeomBSplineSurface,
    values: &SurfaceSplitValues,
    u: bool,
) -> Result<(), Failure> {
    let mut knot = if u {
        surface.first_u_knot_index()?
    } else {
        surface.first_v_knot_index()?
    };
    let last = if u {
        surface.last_u_knot_index()?
    } else {
        surface.last_v_knot_index()?
    };
    for index in 1..=values.sequence(|s| s.len()) {
        let value = values.sequence(|s| s.value(index).copied())?;
        snap_value(surface, values, (u, index), value, &mut knot, last)?;
        if knot == last {
            break;
        }
    }
    Ok(())
}
fn snap_value(
    surface: &GeomBSplineSurface,
    values: &SurfaceSplitValues,
    index: (bool, usize),
    value: f64,
    knot: &mut i32,
    last: i32,
) -> Result<(), Failure> {
    while *knot <= last {
        let candidate = if index.0 {
            surface.u_knot(*knot)?
        } else {
            surface.v_knot(*knot)?
        };
        if value > candidate + precision::p_confusion() {
            *knot += 1;
            continue;
        }
        if value < candidate - precision::p_confusion() {
            break;
        }
        values.change_sequence(|s| s.set_value(index.1, candidate))?;
        *knot += 1;
    }
    Ok(())
}
