//! `Draw_Box` (an oriented bounding box) and `Draw_Grid` (the view grid).
use crate::draw::color::{DrawColor, DrawColorKind};
use crate::draw::display::DrawDisplay;
use crate::draw::drawable::{Drawable3D, Drawable3DBase, drawable_identity};
use rustycad_tkmath::bnd::OrientedBoundingBox;
use rustycad_tkmath::gp::{Point3, Transform3, Xyz};
use std::cell::Cell;

/// `Draw_Box`: the twelve edges of an oriented bounding box.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawBox, DrawColor};
/// use rustycad_tkmath::bnd::OrientedBoundingBox;
/// let drawable = DrawBox::new(OrientedBoundingBox::default(), DrawColor::default());
/// assert!(drawable.obb().is_void());
/// ```
#[derive(Debug)]
pub struct DrawBox {
    base: Drawable3DBase,
    obb: OrientedBoundingBox,
    color: DrawColor,
}

impl DrawBox {
    /// `Draw_Box(theOBB, theColor)`.
    pub fn new(obb: OrientedBoundingBox, color: DrawColor) -> Self {
        Self {
            base: Drawable3DBase::new(),
            obb,
            color,
        }
    }

    /// The box drawn.
    pub fn obb(&self) -> &OrientedBoundingBox {
        &self.obb
    }

    /// `ToWCS`: a point given in the box frame, in world coordinates.
    fn to_wcs(&self, x: f64, y: f64, z: f64) -> Point3 {
        let obb = &self.obb;
        Point3::from(
            obb.center() + x * obb.x_direction() + y * obb.y_direction() + z * obb.z_direction(),
        )
    }

    /// `MoveX`/`MoveY`/`MoveZ`: `point` shifted by `shift` along a box axis.
    fn moved(point: Point3, shift: f64, axis: Xyz) -> Point3 {
        Point3::from(point.xyz() + shift * axis)
    }
}

impl Drawable3D for DrawBox {
    drawable_identity!("Draw_Box");

    /// The bottom and top rectangles, then the four vertical edges; nothing for a void box.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        if self.obb.is_void() {
            return;
        }
        display.set_color(self.color);
        let (hx, hy, hz) = (
            self.obb.x_h_size(),
            self.obb.y_h_size(),
            self.obb.z_h_size(),
        );
        let (ax, ay, az) = (
            self.obb.x_direction(),
            self.obb.y_direction(),
            self.obb.z_direction(),
        );
        let mut point = self.to_wcs(-hx, -hy, -hz);
        display.move_to_3d(point);
        for _ in 0..2 {
            for (shift, axis) in [
                (2.0 * hx, ax),
                (2.0 * hy, ay),
                (-2.0 * hx, ax),
                (-2.0 * hy, ay),
            ] {
                point = Self::moved(point, shift, axis);
                display.draw_to_3d(point);
            }
            point = self.to_wcs(-hx, -hy, hz);
            display.move_to_3d(point);
        }
        for (x, y) in [(-hx, -hy), (hx, -hy), (hx, hy), (-hx, hy)] {
            let bottom = self.to_wcs(x, y, -hz);
            display.move_to_3d(bottom);
            display.draw_to_3d(Self::moved(bottom, 2.0 * hz, az));
        }
    }
}

/// Steps below this are inactive (`MinimumStep`).
const MINIMUM_STEP: f64 = 1.0e-3;
/// The cross size is the frame's smaller side divided by this (`Ratio`).
const CROSS_RATIO: f64 = 200.0;

/// `Draw_Grid`: small blue crosses at the grid nodes of the view plane.
///
/// ```
/// use rustycad_tkdraw::draw::DrawGrid;
/// let grid = DrawGrid::new();
/// grid.set_steps(1.0, -2.0, 0.0);
/// assert_eq!(grid.steps(), (1.0, 2.0, 0.0));
/// assert!(!grid.is_active());
/// ```
#[derive(Debug, Default)]
pub struct DrawGrid {
    base: Drawable3DBase,
    steps: Cell<(f64, f64, f64)>,
    is_active: Cell<bool>,
}

impl DrawGrid {
    /// `Draw_Grid()`: inactive, steps 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Steps`: the absolute steps; active when every step exceeds 1e-3.
    pub fn set_steps(&self, step_x: f64, step_y: f64, step_z: f64) {
        let steps = (step_x.abs(), step_y.abs(), step_z.abs());
        self.steps.set(steps);
        self.is_active
            .set(steps.0 > MINIMUM_STEP && steps.1 > MINIMUM_STEP && steps.2 > MINIMUM_STEP);
    }

    /// `StepX`, `StepY`, `StepZ`.
    pub fn steps(&self) -> (f64, f64, f64) {
        self.steps.get()
    }

    /// `IsActive`.
    pub fn is_active(&self) -> bool {
        self.is_active.get()
    }

    /// The step for the view axis named by `axis` (`X`, `Y`, `Z`; 0 otherwise).
    fn step_for(&self, axis: Option<char>) -> f64 {
        let (x, y, z) = self.steps.get();
        match axis {
            Some('X') => x,
            Some('Y') => y,
            Some('Z') => z,
            _ => 0.0,
        }
    }
}

impl Drawable3D for DrawGrid {
    drawable_identity!("Draw_Grid");

    /// The view type's second and fourth characters name the screen axes (`+X+Y`), which
    /// select the steps; draws a cross at every node of the visible frame.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        if !self.is_active() {
            return;
        }
        let Some(view) = display.view_query() else {
            return;
        };
        let step_x = self.step_for(view.view_type.chars().nth(1));
        let step_y = self.step_for(view.view_type.chars().nth(3));
        if !(step_x > MINIMUM_STEP && step_y > MINIMUM_STEP) {
            return;
        }
        let mut inverse = view.matrix;
        // A view matrix is a rigid motion and always invertible.
        let _ = inverse.invert();
        let (x0, y0, x1, y1) = view.frame;
        let (x_min, x_max) = (x0 as f64 / view.zoom, x1 as f64 / view.zoom);
        let (y_min, y_max) = (y0 as f64 / view.zoom, y1 as f64 / view.zoom);
        // `std::min(a, b)` keeps `a` unless `b < a`.
        let (width, height) = (x_max - x_min, y_max - y_min);
        let offset = if height < width { height } else { width } / CROSS_RATIO;
        let nodes_x = (x_min / step_x) as i32..=(x_max / step_x) as i32;
        for index_x in nodes_x {
            for index_y in (y_min / step_y) as i32..=(y_max / step_y) as i32 {
                let (x, y) = (index_x as f64 * step_x, index_y as f64 * step_y);
                draw_cross(display, inverse, (x, y), offset);
            }
        }
    }
}

/// One blue cross of the grid at `(x, y)` of the view plane.
fn draw_cross(display: &mut DrawDisplay<'_>, inverse: Transform3, (x, y): (f64, f64), offset: f64) {
    let model = |u: f64, v: f64| Point3::new(u, v, 0.0).transformed(inverse);
    display.set_color(DrawColor::new(DrawColorKind::Bleu));
    display.draw_3d(model(x - offset, y), model(x + offset, y));
    display.set_color(DrawColor::new(DrawColorKind::Bleu));
    display.draw_3d(model(x, y - offset), model(x, y + offset));
}
