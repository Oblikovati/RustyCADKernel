//! `Aspect_CircularGrid`: concentric circles crossed by radial divisions.
//!
//! Abstract natively like the rectangular grid: the trait [`AspectCircularGrid`] over the
//! embedded [`AspectCircularGridBase`], which holds the `Compute`, `Init` and `DumpJson`
//! overrides.
use super::grid::{AspectGrid, AspectGridBase, init_and_update};
use super::rectangular_grid::replace_if_different;
use crate::cast::int_from_f64;
use crate::json_dump::{base_class, class_begin, field_integer, field_real};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::{negative_value, null_value};
use rustycad_tkernel::standard::{Failure, SharedState};
use std::f64::consts::PI;
use std::io;

/// The private fields of `Aspect_CircularGrid`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct CircularFields {
    radius_step: f64,
    division_number: i32,
    alpha: f64,
    a1: f64,
    b1: f64,
    radius: f64,
    z_offset: f64,
    angle_start: f64,
    angle_end: f64,
}

/// The circular grid state an implementer embeds.
#[derive(Debug)]
pub struct AspectCircularGridBase {
    grid: AspectGridBase,
    fields: SharedState<CircularFields>,
}

/// The native radius-step checks (negative, then null), with the offending value.
fn check_radius_step(step: f64) -> Result<(), Failure> {
    if step < 0.0 {
        return Err(negative_value(&format!(
            "invalid radius step {step}; expected > 0"
        )));
    }
    if step == 0.0 {
        return Err(null_value(&format!(
            "invalid radius step {step}; expected > 0"
        )));
    }
    Ok(())
}

/// The native division-number checks (negative, then null), with the offending value.
fn check_division_number(number: i32) -> Result<(), Failure> {
    if number < 0 {
        return Err(negative_value(&format!(
            "invalid division number {number}; expected > 0"
        )));
    }
    if number == 0 {
        return Err(null_value(&format!(
            "invalid division number {number}; expected > 0"
        )));
    }
    Ok(())
}

/// The exact cosine and sine of the quadrant `q` of `nquad` (4 or 2) quadrants, as the
/// `switch` statements of `Compute` select them.
fn quadrant_direction(q: i32, nquad: i32) -> (f64, f64) {
    match (nquad, q) {
        (4, 1) => (0.0, 1.0),
        (4, 2) | (2, 1) => (-1.0, 0.0),
        (4, 3) => (0.0, -1.0),
        _ => (1.0, 0.0),
    }
}

/// `n / qmax`; natively an integer division by zero (SIGFPE) only when the division number
/// is 0, which the constructor accepts. The port then takes the first quadrant.
fn quadrant_index(n: i32, qmax: i32) -> i32 {
    n.checked_div(qmax).unwrap_or(0)
}

/// The exact direction when the snapped division lies on an axis and the grid is not
/// rotated (the two `nquad` cases of `Compute`), else `None`.
fn axis_direction(n: i32, division_number: i32) -> Option<(f64, f64)> {
    let nmax = 2 * division_number;
    let qmax = nmax / 4;
    if n == 0 || (nmax % 4 == 0 && n.checked_rem(qmax) == Some(0)) {
        return Some(quadrant_direction(quadrant_index(n, qmax), 4));
    }
    let qmax = nmax / 2;
    if nmax % 2 == 0 && n.checked_rem(qmax) == Some(0) {
        return Some(quadrant_direction(quadrant_index(n, qmax), 2));
    }
    None
}

impl AspectCircularGridBase {
    /// `Aspect_CircularGrid(RadiusStep, DivisionNumber, XOrigin, YOrigin, RotationAngle)`.
    ///
    /// Native quirk: the constructor validates nothing and does not call `Init`; the
    /// cached angle stays 0 until the first setter.
    ///
    /// Example: `AspectCircularGridBase::new(1.0, 8, 0.0, 0.0, 0.0)`.
    pub fn new(
        radius_step: f64,
        division_number: i32,
        x_origin: f64,
        y_origin: f64,
        rotation_angle: f64,
    ) -> Self {
        let fields = CircularFields {
            radius_step,
            division_number,
            ..Default::default()
        };
        Self {
            grid: AspectGridBase::with_default_colors(x_origin, y_origin, rotation_angle),
            fields: SharedState::new(fields),
        }
    }
    /// The `Aspect_Grid` base; implementers return it from `grid_base`.
    pub fn grid(&self) -> &AspectGridBase {
        &self.grid
    }
    /// `Compute`: the closest circle and division; on an unrotated grid, axis directions
    /// are exact. Example: `base.compute(1.2, 0.1)` is `(1.0, 0.0)` for 8 divisions.
    pub fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        let f = *self.fields.read();
        let grid = *self.grid.fields();
        let (xo, yo, ra) = (grid.x_origin, grid.y_origin, grid.rotation_angle);
        let d = ((xo - x) * (xo - x) + (yo - y) * (yo - y)).sqrt();
        let radius = f64::from(int_from_f64(d / f.radius_step + 0.5)) * f.radius_step;
        let mut a = ((x - xo) / d).acos();
        if y < yo {
            a = 2.0 * PI - a;
        }
        let n = int_from_f64((a - ra) / f.alpha + 0.5_f64.copysign(a - ra));
        let exact = if ra == 0.0 {
            axis_direction(n, f.division_number)
        } else {
            None
        };
        let (cs, sn) = exact.unwrap_or_else(|| {
            let angle = ra + f64::from(n) * f.alpha;
            (angle.cos(), angle.sin())
        });
        (xo + cs * radius, yo + sn * radius)
    }
    /// `Init`: the angle between two divisions and its cosine and sine; `base.init()`.
    pub fn init(&self) {
        let mut f = self.fields.write();
        f.alpha = PI / f64::from(f.division_number);
        f.a1 = f.alpha.cos();
        f.b1 = f.alpha.sin();
    }
    /// `Aspect_CircularGrid::DumpJson`: the base grid, then the circular fields.
    ///
    /// Example: `base.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let f = *self.fields.read();
        class_begin(stream, "Aspect_CircularGrid")?;
        base_class(stream, depth, &|s, d| self.grid.dump_json(s, d))?;
        field_real(stream, "myRadiusStep", f.radius_step)?;
        field_integer(stream, "myDivisionNumber", f.division_number)?;
        let values = [
            ("myAlpha", f.alpha),
            ("myA1", f.a1),
            ("myB1", f.b1),
            ("myRadius", f.radius),
            ("myZOffset", f.z_offset),
            ("myAngleStart", f.angle_start),
            ("myAngleEnd", f.angle_end),
        ];
        values
            .iter()
            .try_for_each(|(name, value)| field_real(stream, name, *value))
    }
}

/// A circular grid; implementers forward `compute`, `init` and `dump_json` to the base.
pub trait AspectCircularGrid: AspectGrid {
    /// The embedded base; `grid.circular_base().compute(1.2, 0.1)`.
    fn circular_base(&self) -> &AspectCircularGridBase;
}

/// The non-virtual members of `Aspect_CircularGrid`.
pub trait AspectCircularGridMembers: AspectCircularGrid {
    /// Sets the radius step; `grid.set_radius_step(2.0)?`.
    fn set_radius_step(&self, step: f64) -> Result<(), Failure> {
        check_radius_step(step)?;
        self.circular_base().fields.write().radius_step = step;
        init_and_update(self);
        Ok(())
    }
    /// Sets the number of divisions; `grid.set_division_number(12)?`.
    fn set_division_number(&self, number: i32) -> Result<(), Failure> {
        check_division_number(number)?;
        self.circular_base().fields.write().division_number = number;
        init_and_update(self);
        Ok(())
    }
    /// Sets origin, radius step, divisions and angle; native quirk kept: the origin (and
    /// the radius step) are already stored when a later check fails.
    ///
    /// Example: `grid.set_grid_values(0.0, 0.0, 1.0, 8, 0.0)?`.
    fn set_grid_values(
        &self,
        x_origin: f64,
        y_origin: f64,
        radius_step: f64,
        division_number: i32,
        angle: f64,
    ) -> Result<(), Failure> {
        {
            let mut grid = self.grid_base().fields_mut();
            (grid.x_origin, grid.y_origin) = (x_origin, y_origin);
        }
        check_radius_step(radius_step)?;
        self.circular_base().fields.write().radius_step = radius_step;
        check_division_number(division_number)?;
        self.circular_base().fields.write().division_number = division_number;
        self.grid_base().fields_mut().rotation_angle = angle;
        init_and_update(self);
        Ok(())
    }
    /// The radius step; `grid.radius_step()`.
    fn radius_step(&self) -> f64 {
        self.circular_base().fields.read().radius_step
    }
    /// The number of divisions; `grid.division_number()`.
    fn division_number(&self) -> i32 {
        self.circular_base().fields.read().division_number
    }
    /// Sets the drawn radius (0 means unbounded); `Standard_NegativeValue` when negative.
    ///
    /// Example: `grid.set_radius(2.5)?`.
    fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(negative_value(&format!(
                "invalid grid radius {radius}; expected >= 0"
            )));
        }
        if replace_if_different(&mut self.circular_base().fields.write().radius, radius) {
            self.update_display();
        }
        Ok(())
    }
    /// The drawn radius; `grid.radius()`.
    fn radius(&self) -> f64 {
        self.circular_base().fields.read().radius
    }
    /// Sets the Z offset; redisplays only when it changes; `grid.set_z_offset(-0.005)`.
    fn set_z_offset(&self, offset: f64) {
        if replace_if_different(&mut self.circular_base().fields.write().z_offset, offset) {
            self.update_display();
        }
    }
    /// The Z offset; `grid.z_offset()`.
    fn z_offset(&self) -> f64 {
        self.circular_base().fields.read().z_offset
    }
    /// Sets the arc range; equal angles mean the full circle; `grid.set_arc_range(0.0, PI)`.
    fn set_arc_range(&self, start: f64, end: f64) {
        {
            let mut f = self.circular_base().fields.write();
            if f.angle_start == start && f.angle_end == end {
                return;
            }
            (f.angle_start, f.angle_end) = (start, end);
        }
        self.update_display();
    }
    /// The start angle of the arc; `grid.angle_start()`.
    fn angle_start(&self) -> f64 {
        self.circular_base().fields.read().angle_start
    }
    /// The end angle of the arc; `grid.angle_end()`.
    fn angle_end(&self) -> f64 {
        self.circular_base().fields.read().angle_end
    }
    /// True when only an arc is drawn; `grid.is_arc()`.
    fn is_arc(&self) -> bool {
        let f = self.circular_base().fields.read();
        f.angle_start != f.angle_end
    }
}
impl<T: AspectCircularGrid + ?Sized> AspectCircularGridMembers for T {}

rustycad_tkernel::impl_transient_interface!(
    dyn AspectCircularGrid,
    "Aspect_CircularGrid",
    dyn AspectGrid
);
