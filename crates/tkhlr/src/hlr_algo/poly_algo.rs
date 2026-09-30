// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyAlgo.hxx/.cxx (the class and the hide and show walks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyAlgo: the polyhedral hidden line algorithm over shells.
use super::bi_point::{BiPoint, BiPointPoints};
use super::edge_status::EdgeStatus;
use super::failures::{array_failure, null_handle};
use super::poly_data::{PolyData, PolyTriangle, boxes_meet};
use super::poly_shell_data::PolyShellData;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The segment answered by `hide` and `show`: its projected points (the native
/// returned reference, copied), its shape index and its four regularity flags.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // The native output names, documented on the type.
pub struct PolySegment {
    pub points: BiPointPoints,
    pub index: i32,
    pub reg1: bool,
    pub regn: bool,
    pub outl: bool,
    pub intl: bool,
}

pub(super) struct PolyAlgoState {
    pub(super) triangle: PolyTriangle,
    /// The native `mySegListIt`: the position in the edges of the current shell.
    seg_pos: usize,
    pub(super) nbr_shell: i32,
    cur_shell: i32,
    found: bool,
}

/// HLRAlgo_PolyAlgo: the shells of the polyhedral algorithm, `Update` that packs the
/// boxes of their segments, triangles, faces and shells and computes the hiding
/// planes, and the walks over every segment of every shell: `init_hide`/`next_hide`
/// with `hide` (the visible parts of the segment behind every hiding face), and
/// `init_show`/`next_show` with `show`. A transient; the shell array and the walk
/// state are behind their own locks (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyAlgo;
/// let algo = PolyAlgo::new();
/// algo.init(2).unwrap();
/// assert_eq!(algo.poly_shell().upper(), 2);
/// algo.clear();
/// algo.init_show().unwrap();
/// assert!(!algo.more_show());
/// ```
pub struct PolyAlgo {
    base: StandardTransient,
    pub(super) h_shell: SharedState<Array1<'static, Handle<PolyShellData>>>,
    pub(super) state: SharedState<PolyAlgoState>,
}
impl_transient!(PolyAlgo, "HLRAlgo_PolyAlgo", StandardTransient, base);

impl Default for PolyAlgo {
    fn default() -> Self {
        Self::new()
    }
}

impl PolyAlgo {
    /// HLRAlgo_PolyAlgo(): no shell, the parameter tolerance 1e-8 and the angular
    /// tolerance 1e-4 (the distance tolerance is set by `update`; natively it is
    /// uninitialized before, zero here); `PolyAlgo::new()`.
    pub fn new() -> Self {
        let triangle = PolyTriangle {
            tol_param: 0.00000001,
            tol_ang: 0.0001,
            ..PolyTriangle::default()
        };
        let state = PolyAlgoState {
            triangle,
            seg_pos: 0,
            nbr_shell: 0,
            cur_shell: 0,
            found: false,
        };
        Self {
            base: StandardTransient::default(),
            h_shell: SharedState::new(Array1::default()),
            state: SharedState::new(state),
        }
    }

    /// Init: `1..nb_shells` null shells (the old ones dropped unless the count is
    /// unchanged, as the native Resize does); `algo.init(2)?`.
    pub fn init(&self, nb_shells: i32) -> Result<(), Failure> {
        let mut shells = self.h_shell.write();
        let failure = array_failure("HLRAlgo_PolyAlgo::Init");
        shells.resize_bounds(1, nb_shells, false).map_err(failure)
    }

    /// PolyShell(): the shells, read through the guard; `algo.poly_shell().upper()`.
    pub fn poly_shell(&self) -> RwLockReadGuard<'_, Array1<'static, Handle<PolyShellData>>> {
        self.h_shell.read()
    }

    /// ChangePolyShell(): the shells, writable through the guard;
    /// `algo.change_poly_shell().set_value(1, shell)?`.
    pub fn change_poly_shell(
        &self,
    ) -> RwLockWriteGuard<'_, Array1<'static, Handle<PolyShellData>>> {
        self.h_shell.write()
    }

    /// Clear: no shell; `algo.clear()`.
    pub fn clear(&self) {
        *self.h_shell.write() = Array1::default();
        self.state.write().nbr_shell = 0;
    }

    /// InitHide: starts the walk at the first segment; `algo.init_hide()?`.
    pub fn init_hide(&self) -> Result<(), Failure> {
        self.state.write().cur_shell = 0;
        self.next_hide()
    }

    /// MoreHide; `algo.more_hide()`.
    pub fn more_hide(&self) -> bool {
        self.state.read().found
    }

    /// NextHide: the next segment of the current shell, else the first segment of the
    /// next shell with segments; a null shell crashes natively and gives
    /// Standard_NullObject; `algo.next_hide()?`.
    pub fn next_hide(&self) -> Result<(), Failure> {
        self.next_segment("HLRAlgo_PolyAlgo::NextHide")
    }

    /// InitShow: as `init_hide`; `algo.init_show()?`.
    pub fn init_show(&self) -> Result<(), Failure> {
        self.state.write().cur_shell = 0;
        self.next_show()
    }

    /// MoreShow; `algo.more_show()`.
    pub fn more_show(&self) -> bool {
        self.state.read().found
    }

    /// NextShow: as `next_hide`; `algo.next_show()?`.
    pub fn next_show(&self) -> Result<(), Failure> {
        self.next_segment("HLRAlgo_PolyAlgo::NextShow")
    }

    /// The shell at `index` of the shell array.
    fn shell(&self, index: i32, member: &'static str) -> Result<Handle<PolyShellData>, Failure> {
        let shells = self.h_shell.read();
        let shell = shells.value(index).map_err(array_failure(member))?;
        if shell.is_null() {
            return Err(null_handle(member, "myHShell(i)"));
        }
        Ok(shell.clone())
    }

    fn shell_edges_len(&self, index: i32, member: &'static str) -> Result<usize, Failure> {
        let shell = self.shell(index, member)?;
        let shell = shell
            .get()
            .ok_or_else(|| null_handle(member, "myHShell(i)"))?;
        let length = shell.edges().len();
        Ok(length)
    }

    /// The common body of NextHide and NextShow.
    fn next_segment(&self, member: &'static str) -> Result<(), Failure> {
        let (cur_shell, seg_pos, nbr_shell) = {
            let state = self.state.read();
            (state.cur_shell, state.seg_pos, state.nbr_shell)
        };
        if cur_shell != 0 && seg_pos + 1 < self.shell_edges_len(cur_shell, member)? {
            let mut state = self.state.write();
            state.seg_pos += 1;
            state.found = true;
            return Ok(());
        }
        let mut shell = cur_shell + 1;
        while shell <= nbr_shell {
            if self.shell_edges_len(shell, member)? > 0 {
                break;
            }
            shell += 1;
        }
        let mut state = self.state.write();
        state.found = shell <= nbr_shell;
        state.cur_shell = shell;
        state.seg_pos = 0;
        Ok(())
    }

    /// The current segment of the walk (a copy).
    fn current_segment(&self, member: &'static str) -> Result<BiPoint, Failure> {
        let (cur_shell, seg_pos) = {
            let state = self.state.read();
            (state.cur_shell, state.seg_pos)
        };
        let shell = self.shell(cur_shell, member)?;
        let shell = shell
            .get()
            .ok_or_else(|| null_handle(member, "myHShell(i)"))?;
        let edges = shell.edges();
        let segment = edges.iter().nth(seg_pos).copied();
        segment.ok_or_else(|| {
            Failure::with_message(
                "Standard_NoSuchObject",
                &format!("{member}: no current segment at position {seg_pos}, expected a walk with More true"),
            )
        })
    }

    /// Hide: the visible parts of the current segment into `status` (reset to the
    /// range `[0, 1]` at the parameter tolerance): all hidden when the segment is
    /// flagged hidden, else hidden by every hiding face of every hiding shell whose
    /// box meets the segment box; `algo.hide(&mut status)?`.
    pub fn hide(&self, status: &mut EdgeStatus) -> Result<PolySegment, Failure> {
        const MEMBER: &str = "HLRAlgo_PolyAlgo::Hide";
        let segment = self.current_segment(MEMBER)?;
        let tol = self.state.read().triangle.tol_param as f32;
        *status = EdgeStatus::with_bounds(0.0, tol, 1.0, tol);
        if segment.hidden() {
            status.hide_all();
            return Ok(report(&segment));
        }
        let (nbr_shell, cur_shell) = {
            let state = self.state.read();
            (state.nbr_shell, state.cur_shell)
        };
        let mut triangle = self.state.read().triangle;
        let hidden = (1..=nbr_shell).try_for_each(|s| {
            self.hide_by_shell(s, s == cur_shell, &segment, &mut triangle, status)
        });
        // The working triangle keeps what the faces wrote, also when one raised.
        self.state.write().triangle = triangle;
        hidden?;
        Ok(report(&segment))
    }

    /// One shell of Hide: skipped unless it hides and its box meets the segment box.
    fn hide_by_shell(
        &self,
        s: i32,
        is_hiding_shell: bool,
        segment: &BiPoint,
        triangle: &mut PolyTriangle,
        status: &mut EdgeStatus,
    ) -> Result<(), Failure> {
        const MEMBER: &str = "HLRAlgo_PolyAlgo::Hide";
        let shell = self.shell(s, MEMBER)?;
        let shell = shell
            .get()
            .ok_or_else(|| null_handle(MEMBER, "myHShell(i)"))?;
        if !shell.hiding() {
            return Ok(());
        }
        let box_indices = *shell.indices();
        if !boxes_meet(box_indices.max, box_indices.min, segment.indices()) {
            return Ok(());
        }
        let faces: Vec<Handle<PolyData>> = shell.hiding_poly_data().iter().cloned().collect();
        for face in faces {
            let face = face
                .get()
                .ok_or_else(|| null_handle(MEMBER, "a hiding face"))?;
            face.hide_by_poly_data(
                segment.points(),
                triangle,
                segment.indices(),
                is_hiding_shell,
                status,
            )?;
        }
        Ok(())
    }

    /// Show: the current segment; `algo.show()?`.
    pub fn show(&self) -> Result<PolySegment, Failure> {
        Ok(report(&self.current_segment("HLRAlgo_PolyAlgo::Show")?))
    }
}

/// The outputs of Hide and Show for `segment`.
fn report(segment: &BiPoint) -> PolySegment {
    PolySegment {
        points: *segment.points(),
        index: segment.indices().shape_index,
        reg1: segment.rg1_line(),
        regn: segment.rgn_line(),
        outl: segment.out_line(),
        intl: segment.int_line(),
    }
}
