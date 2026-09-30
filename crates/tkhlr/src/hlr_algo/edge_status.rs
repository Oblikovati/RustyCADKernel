// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_EdgeStatus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_EdgeStatus: the visible parts of an edge.
use crate::intrv::{Interval, Intervals};
use rustycad_tkernel::standard::Failure;

/// HLRAlgo_EdgeStatus: the fuzzy parameter range of an edge and its visible parts, an
/// `Intervals` from which every hiding range is subtracted. Two flags short-cut the
/// states where the whole edge is visible (the intervals are then not built) or hidden.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::EdgeStatus;
/// let mut status = EdgeStatus::with_bounds(0.0, 1e-6, 1.0, 1e-6);
/// status.hide(0.25, 1e-6, 0.5, 1e-6, false, false);
/// assert_eq!(status.nb_visible_part(), 2);
/// assert_eq!(status.visible_part(2).unwrap().0, 0.5);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EdgeStatus {
    start: f64,
    end: f64,
    tol_start: f32,
    tol_end: f32,
    all_hidden: bool,
    all_visible: bool,
    visibles: Intervals,
}

impl EdgeStatus {
    /// HLRAlgo_EdgeStatus(): the range `[0, 0]`, neither all visible nor all hidden,
    /// with no visible part; `EdgeStatus::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// HLRAlgo_EdgeStatus(Start, TolStart, End, TolEnd): the range, all visible;
    /// `EdgeStatus::with_bounds(0.0, 1e-6, 1.0, 1e-6)`.
    pub fn with_bounds(start: f64, tol_start: f32, end: f64, tol_end: f32) -> Self {
        let mut status = Self::new();
        status.initialize(start, tol_start, end, tol_end);
        status
    }

    /// Initialize: replaces the range and shows the whole edge; the visible intervals
    /// of an earlier state are kept but unused while all is visible, as natively;
    /// `status.initialize(0.0, 1e-6, 1.0, 1e-6)`.
    pub fn initialize(&mut self, start: f64, tol_start: f32, end: f64, tol_end: f32) {
        self.start = start;
        self.tol_start = tol_start;
        self.end = end;
        self.tol_end = tol_end;
        self.show_all();
    }

    /// Bounds: `(start, tol_start, end, tol_end)`; `status.bounds()`.
    pub fn bounds(&self) -> (f64, f32, f64, f32) {
        (self.start, self.tol_start, self.end, self.tol_end)
    }

    /// NbVisiblePart: 0 when all hidden, 1 when all visible, else the number of
    /// visible intervals; `status.nb_visible_part()`.
    pub fn nb_visible_part(&self) -> i32 {
        if self.all_hidden {
            0
        } else if self.all_visible {
            1
        } else {
            self.visibles.nb_intervals()
        }
    }

    /// VisiblePart: the visible part of rank `index`, `(start, tol_start, end,
    /// tol_end)`: the whole range when all is visible (whatever the index), else the
    /// visible interval. A rank outside the intervals reads outside the native sequence
    /// (undefined behaviour); the port returns Standard_OutOfRange;
    /// `status.visible_part(1)?`.
    pub fn visible_part(&self, index: i32) -> Result<(f64, f32, f64, f32), Failure> {
        if self.all_visible {
            return Ok(self.bounds());
        }
        Ok(self.visibles.value(index)?.bounds())
    }

    /// Hide: hides the fuzzy range `[start, end]` of the edge, unless `on_face` (the
    /// edge lies on the hiding face). The first hiding turns the whole range into one
    /// visible interval; the edge becomes all hidden when nothing is left.
    /// `on_boundary` is unused, as natively; `status.hide(0.2, 1e-6, 0.4, 1e-6, false, false)`.
    pub fn hide(
        &mut self,
        start: f64,
        tol_start: f32,
        end: f64,
        tol_end: f32,
        on_face: bool,
        _on_boundary: bool,
    ) {
        if on_face {
            return;
        }
        if self.all_visible {
            let whole =
                Interval::with_tolerances(self.start, self.tol_start, self.end, self.tol_end);
            self.visibles = Intervals::from_interval(whole);
            self.all_visible = false;
        }
        self.visibles
            .subtract_interval(&Interval::with_tolerances(start, tol_start, end, tol_end));
        if !self.all_hidden {
            self.all_hidden = self.visibles.nb_intervals() == 0;
        }
    }

    /// HideAll: all hidden, not all visible; `status.hide_all()`.
    pub fn hide_all(&mut self) {
        self.all_visible = false;
        self.all_hidden = true;
    }

    /// ShowAll: all visible, not all hidden; `status.show_all()`.
    pub fn show_all(&mut self) {
        self.all_visible = true;
        self.all_hidden = false;
    }

    /// AllHidden(); `status.all_hidden()`.
    pub fn all_hidden(&self) -> bool {
        self.all_hidden
    }

    /// AllHidden(B); `status.set_all_hidden(true)`.
    pub fn set_all_hidden(&mut self, on: bool) {
        self.all_hidden = on;
    }

    /// AllVisible(); `status.all_visible()`.
    pub fn all_visible(&self) -> bool {
        self.all_visible
    }

    /// AllVisible(B); `status.set_all_visible(true)`.
    pub fn set_all_visible(&mut self, on: bool) {
        self.all_visible = on;
    }
}
