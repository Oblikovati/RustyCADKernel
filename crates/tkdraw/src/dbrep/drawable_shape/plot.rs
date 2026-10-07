// Rust translation of PlotIso, PlotEdge and the pick statics of
// Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The adaptive plotting of isos and edges, and the last pick of a drawable shape.
use crate::draw::DrawDisplay;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::IsoType;
use std::cell::RefCell;

/// `IsoRatio`: a chord is kept when the midpoint is at most 0.1% off.
const ISO_RATIO: f64 = 1.001;

/// `MaxPlotCount`: the bisection stops after this many calls of one plot (bad
/// curves and surfaces would otherwise recurse very deep).
const MAX_PLOT_COUNT: i32 = 5;

/// `pickshape`, `upick`, `vpick`: the sub-shape the last pick hit and the parameters
/// of the closest point.
#[derive(Debug, Clone, Default)]
pub struct LastShapePick {
    /// The face, edge or vertex.
    pub shape: Shape,
    /// The first parameter (on the face or the edge).
    pub u: f64,
    /// The second parameter (on a face).
    pub v: f64,
}

thread_local! {
    // The C++ file-scope statics: `DrawOn` sets them while it draws, and only the
    // display reaches `DrawOn`, so they stay global (per thread here).
    static LAST_PICK: RefCell<LastShapePick> = RefCell::new(LastShapePick::default());
}

/// `DBRep_DrawableShape::LastPick`: the sub-shape and parameters of the last pick.
///
/// ```
/// let pick = rustycad_tkdraw::dbrep::last_pick();
/// assert!(pick.shape.is_null());
/// ```
pub fn last_pick() -> LastShapePick {
    LAST_PICK.with(|pick| pick.borrow().clone())
}

/// Records a pick of `shape` at `(u, v)`.
pub(crate) fn record_pick(shape: &Shape, u: f64, v: f64) {
    LAST_PICK.with(|pick| {
        *pick.borrow_mut() = LastShapePick {
            shape: shape.clone(),
            u,
            v,
        }
    });
}

/// Draws to `point` and records `shape` at `(u, v)` when the pick hits; `halt` is set.
pub(crate) fn draw_and_check(
    display: &mut DrawDisplay<'_>,
    point: rustycad_tkmath::gp::Point3,
    (shape, u, v): (&Shape, f64, f64),
    halt: &mut bool,
) {
    display.draw_to_3d(point);
    if display.has_picked() {
        record_pick(shape, u, v);
        *halt = true;
    }
}

/// One adaptive plot (`PlotIso` or `PlotEdge` with its `PlotCount`).
pub(crate) struct Plot<'a, 'd> {
    pub(crate) display: &'a mut DrawDisplay<'d>,
    pub(crate) shape: &'a Shape,
    pub(crate) halt: &'a mut bool,
    count: i32,
}

impl<'a, 'd> Plot<'a, 'd> {
    /// A plot with `PlotCount = 0`.
    pub(crate) fn new(
        display: &'a mut DrawDisplay<'d>,
        shape: &'a Shape,
        halt: &'a mut bool,
    ) -> Self {
        Self {
            display,
            shape,
            halt,
            count: 0,
        }
    }

    /// `PlotIso`: from `(u, v)` one `step` along the iso, bisected while the midpoint is
    /// off the chord (at most [`MAX_PLOT_COUNT`] calls).
    pub(crate) fn iso(
        &mut self,
        surface: &BRepAdaptorSurface,
        kind: IsoType,
        (u, v): (f64, f64),
        step: f64,
    ) -> Result<(), Failure> {
        self.count += 1;
        let along_v = kind == IsoType::IsoU;
        let at = |t: f64| {
            if along_v {
                surface.d0(u, v + t)
            } else {
                surface.d0(u + t, v)
            }
        };
        let (left, middle, right) = (surface.d0(u, v)?, at(step / 2.)?, at(step)?);
        let end = if along_v {
            (u, v + step)
        } else {
            (u + step, v)
        };
        if self.count > MAX_PLOT_COUNT || is_straight(left, middle, right) {
            draw_and_check(self.display, right, (self.shape, end.0, end.1), self.halt);
            return Ok(());
        }
        self.iso(surface, kind, (u, v), step / 2.)?;
        let middle_start = if along_v {
            (u, v + step / 2.)
        } else {
            (u + step / 2., v)
        };
        self.iso(surface, kind, middle_start, step / 2.)
    }

    /// `PlotEdge`: from `f` one `step` along the curve, bisected while the midpoint is
    /// off the chord (at most [`MAX_PLOT_COUNT`] calls).
    pub(crate) fn edge(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        f: f64,
        step: f64,
    ) -> Result<(), Failure> {
        self.count += 1;
        let (left, middle, right) = (curve.d0(f)?, curve.d0(f + step / 2.)?, curve.d0(f + step)?);
        if self.count > MAX_PLOT_COUNT || is_straight(left, middle, right) {
            draw_and_check(self.display, right, (self.shape, f + step, 0.0), self.halt);
            return Ok(());
        }
        self.edge(curve, f, step / 2.)?;
        self.edge(curve, f + step / 2., step / 2.)
    }
}

/// `Pm.Distance(Pl) + Pm.Distance(Pr) <= IsoRatio * Pl.Distance(Pr)`.
fn is_straight(
    left: rustycad_tkmath::gp::Point3,
    middle: rustycad_tkmath::gp::Point3,
    right: rustycad_tkmath::gp::Point3,
) -> bool {
    middle.distance(&left) + middle.distance(&right) <= ISO_RATIO * left.distance(&right)
}
