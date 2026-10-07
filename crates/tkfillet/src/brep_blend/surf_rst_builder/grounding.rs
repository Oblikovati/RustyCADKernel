// Rust translation of the choice of the section on the restrictions in
// BRepBlend_SurfRstLineBuilder::PerformFirstSection and InternalPerform
// (BRepBlend_SurfRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframings of a section that left a domain, and the section they give: on the
//! surface domain ("ground on surf"), on the restriction ("ground on rst"), or both.
use super::{Arc, BRepBlendSurfRstLineBuilder, VertexFlag};
use crate::blend::{
    BlendFuncInv, BlendSurfCurvFuncInv, BlendSurfPointFuncInv, BlendSurfRstFunction, Status,
};
use crate::brep_blend::BRepBlendBlendTool;
use rustycad_tkernel::standard::Failure;

/// The reframings of a step and what the native code keeps of them between the steps:
/// the guide parameters (`ws`, `wp`, `wrst`), the inverse solutions, the vertices, the
/// arc of the surface, the section on the restriction (`U`, `V`, `trst`) and `decroch`.
#[derive(Default)]
pub(super) struct Grounding {
    pub(super) recads: bool,
    pub(super) recadp: bool,
    pub(super) recadrst: bool,
    pub(super) ws: f64,
    pub(super) wp: f64,
    pub(super) wrst: f64,
    pub(super) solinvs: [f64; 3],
    pub(super) solinvp: [f64; 3],
    pub(super) solinvrst: [f64; 4],
    pub(super) vtxs: VertexFlag,
    pub(super) vtxp: VertexFlag,
    pub(super) vtxrst: VertexFlag,
    pub(super) arc: Arc,
    pub(super) u: f64,
    pub(super) v: f64,
    pub(super) trst: f64,
    pub(super) decroch: bool,
}

impl Grounding {
    /// No reframing yet, every guide parameter at `bound`.
    pub(super) fn new(bound: f64) -> Self {
        Self {
            ws: bound,
            wp: bound,
            wrst: bound,
            ..Self::default()
        }
    }

    /// A new step: no reframing, every guide parameter at `bound`.
    pub(super) fn reset(&mut self, bound: f64) {
        (self.ws, self.wp, self.wrst) = (bound, bound, bound);
        (self.recads, self.recadp, self.recadrst) = (false, false, false);
    }

    /// Whether a reframing succeeded.
    pub(super) fn any(&self) -> bool {
        self.recads || self.recadp || self.recadrst
    }

    /// The reframing on an arc of the surface domain (`FinvC`).
    pub(super) fn reframe_on_surface(
        &mut self,
        b: &BRepBlendSurfRstLineBuilder,
        finv_c: &mut dyn BlendSurfCurvFuncInv,
    ) -> Result<bool, Failure> {
        self.recads = b.recadre_on_arc(finv_c, &mut self.solinvs, &mut self.arc, &mut self.vtxs)?;
        if self.recads {
            self.ws = self.solinvs[0];
        }
        Ok(self.recads)
    }

    /// The reframing on the end point of the restriction (`FinvP`).
    pub(super) fn reframe_on_end_point(
        &mut self,
        b: &BRepBlendSurfRstLineBuilder,
        finv_p: &mut dyn BlendSurfPointFuncInv,
    ) -> Result<bool, Failure> {
        self.recadp = b.recadre_on_point(finv_p, &mut self.solinvp, &mut self.vtxp)?;
        if self.recadp {
            self.wp = self.solinvp[0];
        }
        Ok(self.recadp)
    }

    /// The reframing on the restriction, where the section leaves it (`Finv`).
    pub(super) fn reframe_on_restriction(
        &mut self,
        b: &BRepBlendSurfRstLineBuilder,
        func: &mut dyn BlendSurfRstFunction,
        finv: &mut dyn BlendFuncInv,
    ) -> Result<bool, Failure> {
        self.recadrst = b.recadre_on_rst(func, finv, &mut self.solinvrst, &mut self.vtxrst)?;
        if self.recadrst {
            self.wrst = self.solinvrst[1];
        }
        Ok(self.recadrst)
    }

    /// The section on the restriction from the reframings on its end point and where it
    /// is left: the first one met leaves the domain (`U`, `V`, `trst`, `wrst`).
    fn on_restriction(
        &mut self,
        rst: &Arc,
        sens: f64,
        tolgui: f64,
        internal: bool,
    ) -> Result<(), Failure> {
        let point_first = self.recadp && (!self.recadrst || sens * (self.wrst - self.wp) > tolgui);
        if point_first {
            // first one leaves the domain
            self.wrst = self.wp;
            (self.u, self.v) = (self.solinvp[1], self.solinvp[2]);
            self.trst = BRepBlendBlendTool::parameter(&self.vtxp.vtx, rst)?;
            self.vtxrst = self.vtxp.clone();
            self.recadrst = true;
            return Ok(());
        }
        if self.recadrst {
            self.decroch |= internal;
            (self.u, self.v, self.trst) = (self.solinvrst[2], self.solinvrst[3], self.solinvrst[0]);
        }
        Ok(())
    }
}

impl BRepBlendSurfRstLineBuilder {
    /// The section given by the reframings, `param` and `sol` set on it, and its status
    /// (`state` when there is no reframing). `internal` selects the InternalPerform
    /// text: `decroch` follows the restriction, and the section on both takes the
    /// third value of the surface reframing where the first section takes the second.
    pub(super) fn ground(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        g: &mut Grounding,
        internal: bool,
        state: Status,
    ) -> Result<Status, Failure> {
        g.on_restriction(&self.rst, self.sens, self.tolgui, internal)?;
        let state = match (g.recads, g.recadrst) {
            (true, true) if (g.ws - g.wrst).abs() < self.tolgui => {
                self.param = 0.5 * (g.ws + g.wrst);
                let w = if internal { g.solinvs[2] } else { g.solinvs[1] };
                self.sol = [g.u, g.v, w];
                Status::OnRst12
            }
            (true, true) if self.sens * (g.ws - g.wrst) < 0.0 => {
                self.ground_on_surf(g, internal)?
            }
            (true, true) | (false, true) => {
                // ground on rst
                self.param = g.wrst;
                self.sol = [g.u, g.v, g.trst];
                Status::OnRst2
            }
            (true, false) => self.ground_on_surf(g, false)?,
            (false, false) => return Ok(if internal { Status::Ok } else { state }),
        };
        func.set_param(self.param)?;
        Ok(state)
    }

    /// Ground on surf: the section on the arc of the surface reframing (`decroch` reset
    /// when it wins over the restriction in InternalPerform).
    fn ground_on_surf(
        &mut self,
        g: &mut Grounding,
        reset_decroch: bool,
    ) -> Result<Status, Failure> {
        if reset_decroch {
            g.decroch = false;
        }
        self.param = g.ws;
        let p2d = crate::brep_blend::BRepBlendHCurve2dTool::value(&g.arc, g.solinvs[2])?;
        (g.u, g.v) = (p2d.x(), p2d.y());
        self.sol = [g.u, g.v, g.solinvs[1]];
        Ok(Status::OnRst1)
    }
}
