// Rust translation of Draw/TKDraw/DBRep/DBRep_HideData.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep_HideData`: the visible and hidden segments of a shape in one view, from the
//! polyhedral hidden-line removal.
use crate::draw::{DrawColor, DrawDisplay};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::hlr_algo::{EdgeIterator, EdgeStatus, Projector};
use rustycad_tkhlr::hlr_brep::{BiPntFlags, BiPoint, PolyAlgo, PolyAlgoSegment};
use rustycad_tkmath::gp::Transform3;

/// `DBRep_HideData`: the hidden-line result of one view (its projection, focal
/// distance and discretisation angle) and the shape the last pick touched.
#[derive(Debug, Clone)]
pub struct DBRepHideData {
    view: i32,
    trsf: Transform3,
    focal: f64,
    visible: Vec<BiPoint>,
    hidden: Vec<BiPoint>,
    pick_shape: Shape,
    angle: f64,
}

impl Default for DBRepHideData {
    /// `DBRep_HideData()`: no view (-1), focal and angle 0.
    fn default() -> Self {
        Self {
            view: -1,
            trsf: Transform3::default(),
            focal: 0.0,
            visible: Vec::new(),
            hidden: Vec::new(),
            pick_shape: Shape::default(),
            angle: 0.0,
        }
    }
}

impl DBRepHideData {
    /// `Set(ViewId, TProj, Focal, S, ang)`: runs `HLRBRep_PolyAlgo` on `shape` with the
    /// projector of the view (perspective when `focal > 0`) and keeps each visible
    /// and hidden part of each segment in model coordinates.
    pub fn set(
        &mut self,
        view: i32,
        (trsf, focal): (Transform3, f64),
        shape: &Shape,
        angle: f64,
    ) -> Result<(), Failure> {
        (self.view, self.trsf, self.focal, self.angle) = (view, trsf, focal, angle);
        let hider = PolyAlgo::with_shape(shape)?;
        hider.set_projector(&Projector::with_transformation(trsf, focal > 0., focal)?);
        hider.update()?;
        self.visible.clear();
        self.hidden.clear();
        hider.init_hide()?;
        while hider.more_hide() {
            let mut status = EdgeStatus::default();
            let segment = hider.hide(&mut status)?;
            self.add_parts(&segment, &status)?;
            hider.next_hide()?;
        }
        Ok(())
    }

    /// The visible then hidden parts of one segment, as `BiPoint`s between the
    /// parameters `sta` and `end` of the segment.
    fn add_parts(&mut self, segment: &PolyAlgoSegment, status: &EdgeStatus) -> Result<(), Failure> {
        let (p1, p2) = (segment.points.pnt1, segment.points.pnt2);
        let (dx, dy, dz) = (p2.x() - p1.x(), p2.y() - p1.y(), p2.z() - p1.z());
        let flags = BiPntFlags {
            reg1: segment.reg1,
            regn: segment.regn,
            outl: segment.outl,
            intl: segment.intl,
        };
        let part = |start: f64, end: f64| {
            BiPoint::new(
                [
                    p1.x() + start * dx,
                    p1.y() + start * dy,
                    p1.z() + start * dz,
                ],
                [p1.x() + end * dx, p1.y() + end * dy, p1.z() + end * dz],
                &segment.shape,
                flags,
            )
        };
        let mut parts = EdgeIterator::new();
        parts.init_visible(status);
        while parts.more_visible() {
            let (start, _, end, _) = parts.visible()?;
            self.visible.push(part(start, end));
            parts.next_visible();
        }
        parts.init_hidden(status)?;
        while parts.more_hidden() {
            let (start, _, end, _) = parts.hidden();
            self.hidden.push(part(start, end));
            parts.next_hidden()?;
        }
        Ok(())
    }

    /// `ViewId`.
    pub fn view_id(&self) -> i32 {
        self.view
    }

    /// `Angle`.
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// `IsSame(TProj, Focal)`: the same rotation (and, in perspective, the same focal
    /// distance and translation), compared exactly.
    pub fn is_same(&self, trsf: &Transform3, focal: f64) -> bool {
        if focal > 0. {
            if self.focal <= 0. || self.focal != focal {
                return false;
            }
            if trsf.translation_part() != self.trsf.translation_part() {
                return false;
            }
        }
        // `HVectorialPart` is the unscaled matrix.
        trsf.matrix() == self.trsf.matrix()
    }

    /// `DrawOn(D, withRg1, withRgN, withHid, VisCol, HidCol)`: the hidden parts (when
    /// asked) in `HidCol`, then the visible ones in `VisCol`; smooth (Rg1) and
    /// seam-like (RgN) lines that are not outlines are left out unless asked. The
    /// first segment the pick touches gives `LastPick`.
    pub fn draw_on(
        &mut self,
        display: &mut DrawDisplay<'_>,
        (with_rg1, with_rgn, with_hid): (bool, bool, bool),
        (visible_color, hidden_color): (DrawColor, DrawColor),
    ) {
        let mut first_pick = true;
        if with_hid {
            display.set_color(hidden_color);
            let picked = draw_parts(display, &self.hidden, (with_rg1, with_rgn), &mut first_pick);
            if let Some(shape) = picked {
                self.pick_shape = shape;
            }
        }
        display.set_color(visible_color);
        if let Some(shape) = draw_parts(
            display,
            &self.visible,
            (with_rg1, with_rgn),
            &mut first_pick,
        ) {
            self.pick_shape = shape;
        }
    }

    /// `LastPick`.
    pub fn last_pick(&self) -> &Shape {
        &self.pick_shape
    }
}

/// Draws the kept parts; the shape of the first one picked while `first_pick` holds.
fn draw_parts(
    display: &mut DrawDisplay<'_>,
    parts: &[BiPoint],
    (with_rg1, with_rgn): (bool, bool),
    first_pick: &mut bool,
) -> Option<Shape> {
    let mut picked = None;
    for part in parts {
        let skipped = (!with_rg1 && part.rg1_line() && !part.out_line())
            || (!with_rgn && part.rgn_line() && !part.out_line());
        if skipped {
            continue;
        }
        display.move_to_3d(part.p1());
        display.draw_to_3d(part.p2());
        if *first_pick && display.has_picked() {
            *first_pick = false;
            picked = Some(part.shape().clone());
        }
    }
    picked
}
