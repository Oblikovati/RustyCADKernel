//! `Aspect_RectangularGrid`: a grid of two line networks with their own steps and angles.
//!
//! The class is still abstract natively (the drawing members come from TKV3d), so it is the
//! trait [`AspectRectangularGrid`] over the embedded [`AspectRectangularGridBase`], which
//! holds the overrides of `Compute`, `Init` and `DumpJson` that an implementer forwards to.
use super::grid::{AspectGrid, AspectGridBase, init_and_update};
use crate::cast::int_from_f64;
use crate::json_dump::{base_class, class_begin, field_real};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::{negative_value, null_value, numeric_error};
use rustycad_tkernel::standard::{Failure, SharedState};
use std::f64::consts::PI;
use std::io;

/// The private fields of `Aspect_RectangularGrid`, including the cached line equations
/// `a1 x + b1 y = c1` (first network) and `a2 x + b2 y = c2` (second network).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RectangularFields {
    x_step: f64,
    y_step: f64,
    first_angle: f64,
    second_angle: f64,
    size_x: f64,
    size_y: f64,
    z_offset: f64,
    a1: f64,
    b1: f64,
    c1: f64,
    a2: f64,
    b2: f64,
    c2: f64,
}

/// The rectangular grid state an implementer embeds.
#[derive(Debug)]
pub struct AspectRectangularGridBase {
    grid: AspectGridBase,
    fields: SharedState<RectangularFields>,
}

/// Rejects a negative or null step with the native exception and the offending value.
fn check_step(step: f64, what: &str) -> Result<(), Failure> {
    if step < 0.0 {
        return Err(negative_value(&format!(
            "invalid {what} step {step}; expected > 0"
        )));
    }
    if step == 0.0 {
        return Err(null_value(&format!(
            "invalid {what} step {step}; expected > 0"
        )));
    }
    Ok(())
}

/// `CheckAngle`: true when the two networks are not parallel.
fn check_angle(alpha: f64, beta: f64) -> bool {
    (alpha.sin() * (beta + PI / 2.0).cos() - alpha.cos() * (beta + PI / 2.0).sin()).abs() != 0.0
}

impl AspectRectangularGridBase {
    /// `Aspect_RectangularGrid(XStep, YStep, XOrigin, YOrigin, FirstAngle, SecondAngle,
    /// RotationAngle)`: `Standard_NumericError` for parallel networks, `Standard_NegativeValue`
    /// or `Standard_NullValue` for a bad step.
    ///
    /// Native quirk: the constructor does not call `Init`, so the line equations stay
    /// indeterminate natively until the first setter; here they start at zero.
    ///
    /// Example: `AspectRectangularGridBase::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0)?`.
    pub fn new(
        x_step: f64,
        y_step: f64,
        x_origin: f64,
        y_origin: f64,
        first_angle: f64,
        second_angle: f64,
        rotation_angle: f64,
    ) -> Result<Self, Failure> {
        if !check_angle(first_angle, second_angle) {
            return Err(numeric_error(&format!(
                "networks are parallel: angles {first_angle} and {second_angle}"
            )));
        }
        check_step(x_step, "x")?;
        check_step(y_step, "y")?;
        let fields = RectangularFields {
            x_step,
            y_step,
            first_angle,
            second_angle,
            ..Default::default()
        };
        Ok(Self {
            grid: AspectGridBase::with_default_colors(x_origin, y_origin, rotation_angle),
            fields: SharedState::new(fields),
        })
    }
    /// The `Aspect_Grid` base; implementers return it from `grid_base`.
    pub fn grid(&self) -> &AspectGridBase {
        &self.grid
    }
    /// `Compute`: the intersection of the closest line of each network.
    ///
    /// Example: `base.compute(0.4, 1.6)` is `(0.0, 2.0)` for unit steps after `init`.
    pub fn compute(&self, x: f64, y: f64) -> (f64, f64) {
        let f = *self.fields.read();
        let d1 = f.b1 * x - f.a1 * y - f.c1;
        let d2 = f.b2 * x - f.a2 * y - f.c2;
        let n1 = int_from_f64(d1.abs() / f.x_step + 0.5);
        let n2 = int_from_f64(d2.abs() / f.y_step + 0.5);
        let offset1 = f.c1 + f64::from(n1) * f.x_step.copysign(d1);
        let offset2 = f.c2 + f64::from(n2) * f.y_step.copysign(d2);
        let delta = f.a1 * f.b2 - f.b1 * f.a2;
        (
            (offset2 * f.a1 - offset1 * f.a2) / delta,
            (offset2 * f.b1 - offset1 * f.b2) / delta,
        )
    }
    /// `Init`: recomputes the line equations from the angles and the origin.
    ///
    /// Example: `base.init()`.
    pub fn init(&self) {
        let grid = *self.grid.fields();
        let mut f = self.fields.write();
        let angle1 = f.first_angle + grid.rotation_angle;
        let mut angle2 = f.second_angle + grid.rotation_angle;
        (f.a1, f.b1, f.c1) = if angle1 != 0.0 {
            let (a1, b1) = (-angle1.sin(), angle1.cos());
            (a1, b1, grid.x_origin * b1 - grid.y_origin * a1)
        } else {
            (0.0, 1.0, grid.x_origin)
        };
        (f.a2, f.b2, f.c2) = if angle2 != 0.0 {
            angle2 += PI / 2.0;
            let (a2, b2) = (-angle2.sin(), angle2.cos());
            (a2, b2, grid.x_origin * b2 - grid.y_origin * a2)
        } else {
            (-1.0, 0.0, grid.y_origin)
        };
    }
    /// `Aspect_RectangularGrid::DumpJson`: the base grid, then steps, sizes and equations.
    ///
    /// Example: `base.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let f = *self.fields.read();
        class_begin(stream, "Aspect_RectangularGrid")?;
        base_class(stream, depth, &|s, d| self.grid.dump_json(s, d))?;
        let values = [
            ("myXStep", f.x_step),
            ("myYStep", f.y_step),
            ("myFirstAngle", f.first_angle),
            ("mySecondAngle", f.second_angle),
            ("mySizeX", f.size_x),
            ("mySizeY", f.size_y),
            ("myZOffset", f.z_offset),
            ("a1", f.a1),
            ("b1", f.b1),
            ("c1", f.c1),
            ("a2", f.a2),
            ("b2", f.b2),
            ("c2", f.c2),
        ];
        values
            .iter()
            .try_for_each(|(name, value)| field_real(stream, name, *value))
    }
}

/// A rectangular grid; implementers forward `compute`, `init` and `dump_json` to the base.
pub trait AspectRectangularGrid: AspectGrid {
    /// The embedded base; `grid.rectangular_base().compute(0.4, 1.6)`.
    fn rectangular_base(&self) -> &AspectRectangularGridBase;
}

/// The non-virtual members of `Aspect_RectangularGrid`.
pub trait AspectRectangularGridMembers: AspectRectangularGrid {
    /// Sets the X step; `grid.set_x_step(2.0)?`.
    fn set_x_step(&self, step: f64) -> Result<(), Failure> {
        // Native quirk kept: both checks name the x step for a negative and the y step for a
        // null value.
        check_step_messages(step, "x", "y")?;
        self.rectangular_base().fields.write().x_step = step;
        init_and_update(self);
        Ok(())
    }
    /// Sets the Y step; the native messages name the x step when negative.
    ///
    /// Example: `grid.set_y_step(2.0)?`.
    fn set_y_step(&self, step: f64) -> Result<(), Failure> {
        check_step_messages(step, "x", "y")?;
        self.rectangular_base().fields.write().y_step = step;
        init_and_update(self);
        Ok(())
    }
    /// Sets both network angles; `Standard_NumericError` when they are parallel.
    ///
    /// Example: `grid.set_angle(0.0, 0.3)?`.
    fn set_angle(&self, angle1: f64, angle2: f64) -> Result<(), Failure> {
        if !check_angle(angle1, angle2) {
            return Err(numeric_error(&format!(
                "axis are parallel: angles {angle1} and {angle2}"
            )));
        }
        {
            let mut f = self.rectangular_base().fields.write();
            (f.first_angle, f.second_angle) = (angle1, angle2);
        }
        init_and_update(self);
        Ok(())
    }
    /// Sets origin, steps and angle; native quirk kept: the origin (and the X step) are
    /// already stored when a later step check fails.
    ///
    /// Example: `grid.set_grid_values(0.0, 0.0, 1.0, 1.0, 0.0)?`.
    fn set_grid_values(
        &self,
        x_origin: f64,
        y_origin: f64,
        x_step: f64,
        y_step: f64,
        angle: f64,
    ) -> Result<(), Failure> {
        {
            let mut grid = self.grid_base().fields_mut();
            (grid.x_origin, grid.y_origin) = (x_origin, y_origin);
        }
        check_step(x_step, "x")?;
        self.rectangular_base().fields.write().x_step = x_step;
        check_step(y_step, "y")?;
        self.rectangular_base().fields.write().y_step = y_step;
        self.grid_base().fields_mut().rotation_angle = angle;
        init_and_update(self);
        Ok(())
    }
    /// The X step; `grid.x_step()`.
    fn x_step(&self) -> f64 {
        self.rectangular_base().fields.read().x_step
    }
    /// The Y step; `grid.y_step()`.
    fn y_step(&self) -> f64 {
        self.rectangular_base().fields.read().y_step
    }
    /// The angle of the first network; `grid.first_angle()`.
    fn first_angle(&self) -> f64 {
        self.rectangular_base().fields.read().first_angle
    }
    /// The angle of the second network; `grid.second_angle()`.
    fn second_angle(&self) -> f64 {
        self.rectangular_base().fields.read().second_angle
    }
    /// Sets the drawn size along X; `Standard_NegativeValue` when negative; redisplays only
    /// when the size changes. Example: `grid.set_size_x(8.0)?`.
    fn set_size_x(&self, size: f64) -> Result<(), Failure> {
        set_size(self, size, "X", |f| &mut f.size_x)
    }
    /// The drawn size along X (0 means unbounded); `grid.size_x()`.
    fn size_x(&self) -> f64 {
        self.rectangular_base().fields.read().size_x
    }
    /// Sets the drawn size along Y; `grid.set_size_y(4.0)?`.
    fn set_size_y(&self, size: f64) -> Result<(), Failure> {
        set_size(self, size, "Y", |f| &mut f.size_y)
    }
    /// The drawn size along Y; `grid.size_y()`.
    fn size_y(&self) -> f64 {
        self.rectangular_base().fields.read().size_y
    }
    /// Sets the Z offset; redisplays only when it changes; `grid.set_z_offset(-0.005)`.
    fn set_z_offset(&self, offset: f64) {
        let changed =
            replace_if_different(&mut self.rectangular_base().fields.write().z_offset, offset);
        if changed {
            self.update_display();
        }
    }
    /// The Z offset; `grid.z_offset()`.
    fn z_offset(&self) -> f64 {
        self.rectangular_base().fields.read().z_offset
    }
}
impl<T: AspectRectangularGrid + ?Sized> AspectRectangularGridMembers for T {}

/// The step checks of `SetXStep`/`SetYStep`, whose messages name the x step when negative
/// and the y step when null, whatever the axis.
fn check_step_messages(step: f64, negative: &str, null: &str) -> Result<(), Failure> {
    if step < 0.0 {
        return Err(negative_value(&format!(
            "invalid {negative} step {step}; expected > 0"
        )));
    }
    if step == 0.0 {
        return Err(null_value(&format!(
            "invalid {null} step {step}; expected > 0"
        )));
    }
    Ok(())
}

/// Stores `value` in `slot` and reports whether it changed.
pub(crate) fn replace_if_different(slot: &mut f64, value: f64) -> bool {
    if *slot == value {
        return false;
    }
    *slot = value;
    true
}

fn set_size<G: AspectRectangularGrid + ?Sized>(
    grid: &G,
    size: f64,
    axis: &str,
    slot: fn(&mut RectangularFields) -> &mut f64,
) -> Result<(), Failure> {
    if size < 0.0 {
        return Err(negative_value(&format!(
            "invalid grid size {axis} {size}; expected >= 0"
        )));
    }
    let changed = replace_if_different(slot(&mut grid.rectangular_base().fields.write()), size);
    if changed {
        grid.update_display();
    }
    Ok(())
}

rustycad_tkernel::impl_transient_interface!(
    dyn AspectRectangularGrid,
    "Aspect_RectangularGrid",
    dyn AspectGrid
);
