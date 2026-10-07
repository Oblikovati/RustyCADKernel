// Rust translation of the choice of the section on the restrictions in
// BRepBlend_RstRstLineBuilder::PerformFirstSection and InternalPerform
// (BRepBlend_RstRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframings of a section of the restriction-restriction marching and the section
//! they give: where the contact is lost on a restriction, or where a restriction ends.
use super::{BRepBlendRstRstLineBuilder, VertexFlag};
use crate::blend::{
    BlendCurvPointFuncInv, BlendRstRstFunction, BlendSurfCurvFuncInv, DecrochStatus, Status,
};
use crate::brep_blend::BRepBlendBlendTool;
use rustycad_tkernel::standard::Failure;

/// The reframings of one restriction (a side): where the contact is lost (`recadrst`,
/// `wrst`, `solinvrst`, `vtxrst`), at its end point (`recadp`, `wp`, `solinvp`, `vtxp`),
/// and the section on it (`trst`: its own parameter, then the other one).
#[derive(Default)]
pub(super) struct SideGrounding {
    pub(super) recadp: bool,
    pub(super) recadrst: bool,
    pub(super) wp: f64,
    pub(super) wrst: f64,
    pub(super) solinvp: [f64; 2],
    pub(super) solinvrst: [f64; 3],
    pub(super) vtxp: VertexFlag,
    pub(super) vtxrst: VertexFlag,
    pub(super) trst: (f64, f64),
}

/// The reframings of both sides and what the native code keeps of them between the
/// steps, with the unhooking status.
pub(super) struct Grounding {
    pub(super) side: [SideGrounding; 2],
    pub(super) decroch: DecrochStatus,
}

impl Grounding {
    /// No reframing yet, every guide parameter at `bound`.
    pub(super) fn new(bound: f64) -> Self {
        let mut g = Self {
            side: Default::default(),
            decroch: DecrochStatus::NoDecroch,
        };
        g.reset(bound);
        g
    }

    /// A new step: no reframing, every guide parameter at `bound`.
    pub(super) fn reset(&mut self, bound: f64) {
        for s in &mut self.side {
            (s.wp, s.wrst, s.recadp, s.recadrst) = (bound, bound, false, false);
        }
    }

    /// Whether a reframing succeeded.
    pub(super) fn any(&self) -> bool {
        self.side.iter().any(|s| s.recadp || s.recadrst)
    }

    /// The reframing where the contact with the restriction of `side` is lost.
    pub(super) fn reframe_on_rst(
        &mut self,
        b: &BRepBlendRstRstLineBuilder,
        side: usize,
        func: &mut dyn BlendRstRstFunction,
        finv: &mut dyn BlendSurfCurvFuncInv,
    ) -> Result<bool, Failure> {
        let s = &mut self.side[side];
        s.recadrst = b.recadre_on_rst(side, func, finv, &mut s.solinvrst, &mut s.vtxrst)?;
        if s.recadrst {
            s.wrst = s.solinvrst[0];
        }
        Ok(s.recadrst)
    }

    /// The reframing on the end point of the restriction of `side`.
    pub(super) fn reframe_on_end_point(
        &mut self,
        b: &BRepBlendRstRstLineBuilder,
        side: usize,
        finv_p: &mut dyn BlendCurvPointFuncInv,
    ) -> Result<bool, Failure> {
        let s = &mut self.side[side];
        s.recadp = b.recadre_on_point(side, finv_p, &mut s.solinvp, &mut s.vtxp)?;
        if s.recadp {
            s.wp = s.solinvp[0];
        }
        Ok(s.recadp)
    }

    /// It is checked if the contact was lost or the domain of `side` was left: at
    /// first one leaves the domain (the end point wins), or the contact is lost. The
    /// vertex of the end point goes to the second restriction when both reframings
    /// succeeded on the first one, as natively.
    fn settle_side(&mut self, side: usize, b: &BRepBlendRstRstLineBuilder) -> Result<(), Failure> {
        let (sens, tolgui) = (b.sens, b.tolgui);
        let s = &self.side[side];
        let both = s.recadp && s.recadrst;
        if s.recadp && (!s.recadrst || sens * (s.wrst - s.wp) > tolgui) {
            let own = BRepBlendBlendTool::parameter(&s.vtxp.vtx, &b.rst[side])?;
            let (other, vtxp, wp) = (s.solinvp[1], s.vtxp.clone(), s.wp);
            let target = if both { 1 } else { side };
            self.side[target].vtxrst = vtxp;
            let s = &mut self.side[side];
            (s.wrst, s.trst) = (wp, (own, other));
            s.recadrst &= !both;
            return Ok(());
        }
        let s = &mut self.side[side];
        if s.recadrst {
            s.trst = (s.solinvrst[2], s.solinvrst[1]);
            s.recadp &= !both;
        }
        Ok(())
    }
}

impl BRepBlendRstRstLineBuilder {
    /// The section given by the reframings, `param` and `sol` set on it, and its status
    /// (`state` when there is no reframing). `internal` selects the InternalPerform text,
    /// where the unhooking status follows the restriction the contact is lost on.
    pub(super) fn ground(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        g: &mut Grounding,
        internal: bool,
        state: Status,
    ) -> Result<Status, Failure> {
        g.settle_side(0, self)?;
        g.settle_side(1, self)?;
        let (rst, p) = (
            [g.side[0].recadrst, g.side[1].recadrst],
            [g.side[0].recadp, g.side[1].recadp],
        );
        let lost = rst[0] || rst[1];
        let by = if lost { rst } else { p };
        if !by[0] && !by[1] {
            return Ok(if internal { Status::Ok } else { state });
        }
        let state = self.choose_section(g, by);
        if internal && lost {
            g.decroch = match state {
                Status::OnRst12 => DecrochStatus::DecrochBoth,
                Status::OnRst1 => DecrochStatus::DecrochRst1,
                _ => DecrochStatus::DecrochRst2,
            };
        }
        func.set_param(self.param)?;
        Ok(state)
    }

    /// The section on the restriction where the contact is lost (or the domain left)
    /// earlier, or on both when they meet the guide at the same parameter.
    fn choose_section(&mut self, g: &Grounding, by: [bool; 2]) -> Status {
        let (s1, s2) = (&g.side[0], &g.side[1]);
        let on_both = by[0] && by[1];
        if on_both && (s1.wrst - s2.wrst).abs() < self.tolgui {
            self.param = 0.5 * (s1.wrst + s2.wrst);
            self.sol = [s1.trst.0, s2.trst.0];
            return Status::OnRst12;
        }
        if by[0] && (!on_both || self.sens * (s1.wrst - s2.wrst) < 0.0) {
            self.param = s1.wrst;
            self.sol = [s1.trst.0, s1.trst.1];
            return Status::OnRst1;
        }
        self.param = s2.wrst;
        self.sol = [s2.trst.1, s2.trst.0];
        Status::OnRst2
    }
}
