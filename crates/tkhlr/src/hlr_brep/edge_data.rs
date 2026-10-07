//! HLRBRep_EdgeData.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Curve;
use crate::hlr_algo::{EdgeStatus, MinMaxIndices};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;

// The bits of `myFlags` (`EMaskFlags`).
/// HLRBRep_EdgeData::EMaskFlags: the bits of `myFlags`; protected natively, public here so that the packed
/// flags can be read; `EdgeDataMask::EMaskSelected as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum EdgeDataMask {
    /// `EMaskSelected`.
    EMaskSelected = 1,
    /// `EMaskUsed`.
    EMaskUsed = 2,
    /// `EMaskRg1Line`.
    EMaskRg1Line = 4,
    /// `EMaskVertical`.
    EMaskVertical = 8,
    /// `EMaskSimple`.
    EMaskSimple = 16,
    /// `EMaskOutLVSta`.
    EMaskOutLVSta = 32,
    /// `EMaskOutLVEnd`.
    EMaskOutLVEnd = 64,
    /// `EMaskIntDone`.
    EMaskIntDone = 128,
    /// `EMaskCutAtSta`.
    EMaskCutAtSta = 256,
    /// `EMaskCutAtEnd`.
    EMaskCutAtEnd = 512,
    /// `EMaskVerAtSta`.
    EMaskVerAtSta = 1024,
    /// `EMaskVerAtEnd`.
    EMaskVerAtEnd = 2048,
    /// `EMaskRgNLine`.
    EMaskRgNLine = 4096,
}

const EMASK_SELECTED: i32 = EdgeDataMask::EMaskSelected as i32;
const EMASK_USED: i32 = EdgeDataMask::EMaskUsed as i32;
const EMASK_RG1_LINE: i32 = EdgeDataMask::EMaskRg1Line as i32;
const EMASK_VERTICAL: i32 = EdgeDataMask::EMaskVertical as i32;
const EMASK_SIMPLE: i32 = EdgeDataMask::EMaskSimple as i32;
const EMASK_OUT_LV_STA: i32 = EdgeDataMask::EMaskOutLVSta as i32;
const EMASK_OUT_LV_END: i32 = EdgeDataMask::EMaskOutLVEnd as i32;
const EMASK_INT_DONE: i32 = EdgeDataMask::EMaskIntDone as i32;
const EMASK_CUT_AT_STA: i32 = EdgeDataMask::EMaskCutAtSta as i32;
const EMASK_CUT_AT_END: i32 = EdgeDataMask::EMaskCutAtEnd as i32;
const EMASK_VER_AT_STA: i32 = EdgeDataMask::EMaskVerAtSta as i32;
const EMASK_VER_AT_END: i32 = EdgeDataMask::EMaskVerAtEnd as i32;
const EMASK_RGN_LINE: i32 = EdgeDataMask::EMaskRgNLine as i32;

/// One flag of `myFlags`: the getter and the setter of the native overload pair.
macro_rules! edge_flag {
    ($get:ident, $set:ident, $mask:expr, $name:literal) => {
        #[doc = concat!("`", $name, "()`; `edge.", stringify!($get), "()`.")]
        pub fn $get(&self) -> bool {
            (self.flags & $mask) != 0
        }
        #[doc = concat!("`", $name, "(B)`; `edge.", stringify!($set), "(true)`.")]
        pub fn $set(&mut self, b: bool) {
            if b {
                self.flags |= $mask;
            } else {
                self.flags &= !$mask;
            }
        }
    };
}

/// HLRBRep_EdgeData: the hidden line data of one edge: its flags (selected,
/// regularity, outline ends, cuts, vertical ends...), hide count, end vertices, box,
/// visible parts ([`EdgeStatus`]) and projected curve ([`Curve`]).
///
/// ```
/// use rustycad_tkhlr::hlr_brep::EdgeData;
/// let mut edge = EdgeData::new();
/// assert!(edge.selected());
/// edge.set_used(true);
/// assert!(edge.used());
/// ```
pub struct EdgeData {
    flags: i32,
    hide_count: i32,
    // The native vertex indices and tolerance are uninitialized until Set; 0 here.
    v_sta: i32,
    v_end: i32,
    min_max: MinMaxIndices,
    status: EdgeStatus,
    geometry: Curve,
    tolerance: f32,
}

impl Default for EdgeData {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeData {
    /// HLRBRep_EdgeData(): selected, no other flag, hide count 0; `EdgeData::new()`.
    pub fn new() -> Self {
        let mut data = Self {
            flags: 0,
            hide_count: 0,
            v_sta: 0,
            v_end: 0,
            min_max: MinMaxIndices::default(),
            status: EdgeStatus::new(),
            geometry: Curve::new(),
            tolerance: 0.,
        };
        data.set_selected(true);
        data
    }

    /// The native copy assignment (`*ed = *e1` in HLRBRep_Data::Write): every field,
    /// the curve on a new adaptor of the same edge; `let copy = edge.try_clone()?;`.
    pub fn try_clone(&self) -> Result<Self, Failure> {
        Ok(Self {
            status: self.status.clone(),
            geometry: self.geometry.try_clone()?,
            ..*self
        })
    }

    /// Set(Reg1, RegN, EG, V1, V2, Out1, Out2, Cut1, Cut2, Start, TolStart, End,
    /// TolEnd): loads the edge and its flags; the status tolerances are the 3D curve
    /// resolutions of the given tolerances, rounded to `float`;
    /// `edge.set(false, false, &e, 1, 2, false, false, false, false, 0.0, 1e-7, 1.0, 1e-7)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn set(
        &mut self,
        rg1l: bool,
        rgnl: bool,
        eg: &Edge,
        v1: i32,
        v2: i32,
        out1: bool,
        out2: bool,
        cut1: bool,
        cut2: bool,
        start: f64,
        tol_start: f32,
        end: f64,
        tol_end: f32,
    ) -> Result<(), Failure> {
        self.set_rg1_line(rg1l);
        self.set_rgn_line(rgnl);
        self.set_used(false);
        self.geometry.set_curve(eg)?;
        self.tolerance = BRepTool::edge_tolerance(eg)? as f32;
        self.set_v_sta(v1);
        self.set_v_end(v2);
        self.set_out_lv_sta(out1);
        self.set_out_lv_end(out2);
        self.set_cut_at_sta(cut1);
        self.set_cut_at_end(cut2);
        let curve = self.geometry.curve();
        let res_start = curve.resolution(f64::from(tol_start))? as f32;
        let res_end = curve.resolution(f64::from(tol_end))? as f32;
        self.status.initialize(start, res_start, end, res_end);
        Ok(())
    }

    edge_flag!(selected, set_selected, EMASK_SELECTED, "Selected");
    edge_flag!(rg1_line, set_rg1_line, EMASK_RG1_LINE, "Rg1Line");
    edge_flag!(rgn_line, set_rgn_line, EMASK_RGN_LINE, "RgNLine");
    edge_flag!(vertical, set_vertical, EMASK_VERTICAL, "Vertical");
    edge_flag!(simple, set_simple, EMASK_SIMPLE, "Simple");
    edge_flag!(out_lv_sta, set_out_lv_sta, EMASK_OUT_LV_STA, "OutLVSta");
    edge_flag!(out_lv_end, set_out_lv_end, EMASK_OUT_LV_END, "OutLVEnd");
    edge_flag!(cut_at_sta, set_cut_at_sta, EMASK_CUT_AT_STA, "CutAtSta");
    edge_flag!(cut_at_end, set_cut_at_end, EMASK_CUT_AT_END, "CutAtEnd");
    edge_flag!(ver_at_sta, set_ver_at_sta, EMASK_VER_AT_STA, "VerAtSta");
    edge_flag!(ver_at_end, set_ver_at_end, EMASK_VER_AT_END, "VerAtEnd");
    edge_flag!(
        auto_intersection_done,
        set_auto_intersection_done,
        EMASK_INT_DONE,
        "AutoIntersectionDone"
    );
    edge_flag!(used, set_used, EMASK_USED, "Used");

    /// HideCount(); `edge.hide_count()`.
    pub fn hide_count(&self) -> i32 {
        self.hide_count
    }
    /// HideCount(I); `edge.set_hide_count(2)`.
    pub fn set_hide_count(&mut self, i: i32) {
        self.hide_count = i;
    }
    /// VSta(); `edge.v_sta()`.
    pub fn v_sta(&self) -> i32 {
        self.v_sta
    }
    /// VSta(I); `edge.set_v_sta(1)`.
    pub fn set_v_sta(&mut self, i: i32) {
        self.v_sta = i;
    }
    /// VEnd(); `edge.v_end()`.
    pub fn v_end(&self) -> i32 {
        self.v_end
    }
    /// VEnd(I); `edge.set_v_end(2)`.
    pub fn set_v_end(&mut self, i: i32) {
        self.v_end = i;
    }
    /// UpdateMinMax(theTotMinMax): stores the box; `edge.update_min_max(&box_indices)`.
    pub fn update_min_max(&mut self, tot_min_max: &MinMaxIndices) {
        self.min_max = *tot_min_max;
    }
    /// MinMax(): the box, for edition; `edge.min_max_mut()`.
    pub fn min_max_mut(&mut self) -> &mut MinMaxIndices {
        &mut self.min_max
    }
    /// The box, read only; `edge.min_max()`.
    pub fn min_max(&self) -> &MinMaxIndices {
        &self.min_max
    }
    /// Status(): the visible parts, for edition; `edge.status_mut()`.
    pub fn status_mut(&mut self) -> &mut EdgeStatus {
        &mut self.status
    }
    /// The visible parts, read only; `edge.status()`.
    pub fn status(&self) -> &EdgeStatus {
        &self.status
    }
    /// ChangeGeometry(): the projected curve, for edition; `edge.change_geometry()`.
    pub fn change_geometry(&mut self) -> &mut Curve {
        &mut self.geometry
    }
    /// Geometry(): the projected curve; `edge.geometry()`.
    pub fn geometry(&self) -> &Curve {
        &self.geometry
    }
    /// Curve(): the projected curve as the templates take it (the native
    /// `HLRBRep_Curve*`); `edge.curve()`.
    pub fn curve(&self) -> &Curve {
        &self.geometry
    }
    /// Tolerance(): the edge tolerance as a `float`; `edge.tolerance()`.
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }
}
