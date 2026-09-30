// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_EdgeIterator.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_EdgeIterator: walks the visible and the hidden parts of an edge status.
use super::edge_status::EdgeStatus;
use rustycad_tkernel::standard::Failure;

/// HLRAlgo_EdgeIterator: iterates the visible parts of an `EdgeStatus` and, between
/// them, its hidden parts (the gaps from the range start to the first visible part, and
/// so on to the range end; gaps whose fuzzy bounds meet are skipped). The native class
/// keeps pointers to the statuses; the port borrows them for its lifetime.
///
/// A hidden walk over a status that is neither all visible nor all hidden and has no
/// visible interval (a default status) reads outside the native sequence; the port
/// returns Standard_OutOfRange there.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::{EdgeIterator, EdgeStatus};
/// let mut status = EdgeStatus::with_bounds(0.0, 1e-6, 1.0, 1e-6);
/// status.hide(0.25, 1e-6, 0.5, 1e-6, false, false);
/// let mut iterator = EdgeIterator::new();
/// iterator.init_hidden(&status).unwrap();
/// assert!(iterator.more_hidden());
/// assert_eq!(iterator.hidden().0, 0.25);
/// ```
#[derive(Clone, Debug, Default)]
pub struct EdgeIterator<'a> {
    nb_vis: i32,
    nb_hid: i32,
    e_vis: Option<&'a EdgeStatus>,
    e_hid: Option<&'a EdgeStatus>,
    i_vis: i32,
    i_hid: i32,
    hid_start: f64,
    hid_end: f64,
    hid_tol_start: f32,
    hid_tol_end: f32,
}

/// The failure of a walk over no status (a null pointer natively).
fn no_status(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("{member}: no edge status, expected an Init call first"),
    )
}

impl<'a> EdgeIterator<'a> {
    /// HLRAlgo_EdgeIterator(): no status, every counter 0; `EdgeIterator::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// InitHidden: starts the hidden walk over `status` at the gap before its first
    /// visible part (or the whole range when all is hidden), skipping it when its
    /// fuzzy bounds meet; `iterator.init_hidden(&status)?`.
    pub fn init_hidden(&mut self, status: &'a EdgeStatus) -> Result<(), Failure> {
        self.e_hid = Some(status);
        self.i_hid = 1;
        if status.all_hidden() {
            (
                self.hid_start,
                self.hid_tol_start,
                self.hid_end,
                self.hid_tol_end,
            ) = status.bounds();
            self.nb_hid = 0;
        } else {
            self.nb_hid = status.nb_visible_part();
            (self.hid_start, self.hid_tol_start, _, _) = status.bounds();
            (self.hid_end, self.hid_tol_end, _, _) = status.visible_part(self.i_hid)?;
        }
        if self.hidden_gap_is_empty() {
            self.next_hidden()?;
        }
        Ok(())
    }

    /// The native test of an empty gap: the fuzzy start reaches the fuzzy end and the
    /// fuzzy end reaches the fuzzy start (double plus float sums).
    fn hidden_gap_is_empty(&self) -> bool {
        let (start_tol, end_tol) = (f64::from(self.hid_tol_start), f64::from(self.hid_tol_end));
        self.hid_start + start_tol >= self.hid_end - end_tol
            && self.hid_end + end_tol >= self.hid_start - start_tol
    }

    /// MoreHidden; `iterator.more_hidden()`.
    pub fn more_hidden(&self) -> bool {
        self.i_hid <= self.nb_hid + 1
    }

    /// NextHidden: moves to the next gap: from the end of the current visible part to
    /// the start of the next one, or to the range end after the last one (skipped when
    /// its fuzzy bounds meet); `iterator.next_hidden()?`.
    pub fn next_hidden(&mut self) -> Result<(), Failure> {
        // `iHid >= myNbHid + 1` natively.
        if self.i_hid > self.nb_hid {
            self.i_hid += 1;
            return Ok(());
        }
        let status = self
            .e_hid
            .ok_or_else(|| no_status("HLRAlgo_EdgeIterator::NextHidden"))?;
        (_, _, self.hid_start, self.hid_tol_start) = status.visible_part(self.i_hid)?;
        self.i_hid += 1;
        if self.i_hid == self.nb_hid + 1 {
            (_, _, self.hid_end, self.hid_tol_end) = status.bounds();
            if self.hidden_gap_is_empty() {
                self.i_hid += 1;
            }
        } else {
            (self.hid_end, self.hid_tol_end, _, _) = status.visible_part(self.i_hid)?;
        }
        Ok(())
    }

    /// Hidden: the current gap, `(start, tol_start, end, tol_end)`; `iterator.hidden()`.
    pub fn hidden(&self) -> (f64, f32, f64, f32) {
        (
            self.hid_start,
            self.hid_tol_start,
            self.hid_end,
            self.hid_tol_end,
        )
    }

    /// InitVisible: starts the visible walk over `status`; `iterator.init_visible(&status)`.
    pub fn init_visible(&mut self, status: &'a EdgeStatus) {
        self.e_vis = Some(status);
        self.i_vis = 1;
        self.nb_vis = status.nb_visible_part();
    }

    /// MoreVisible; `iterator.more_visible()`.
    pub fn more_visible(&self) -> bool {
        self.i_vis <= self.nb_vis
    }

    /// NextVisible; `iterator.next_visible()`.
    pub fn next_visible(&mut self) {
        self.i_vis += 1;
    }

    /// Visible: the current visible part, `(start, tol_start, end, tol_end)`;
    /// `iterator.visible()?`.
    pub fn visible(&self) -> Result<(f64, f32, f64, f32), Failure> {
        let status = self
            .e_vis
            .ok_or_else(|| no_status("HLRAlgo_EdgeIterator::Visible"))?;
        status.visible_part(self.i_vis)
    }
}
