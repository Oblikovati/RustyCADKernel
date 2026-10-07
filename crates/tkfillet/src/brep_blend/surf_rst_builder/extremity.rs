// Rust translation of BRepBlend_SurfRstLineBuilder::MakeExtremity (and Transition through
// brep_blend::arc_ends) (BRepBlend_SurfRstLineBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of the surface-restriction marching on the arcs.
use super::{Arc, BRepBlendSurfRstLineBuilder, CLASS, VertexFlag};
use crate::brep_blend::BRepBlendExtremity;
use crate::brep_blend::arc_ends::ArcEnds;
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;

impl BRepBlendSurfRstLineBuilder {
    /// `MakeExtremity(Extrem, OnFirst, Arc, Param, IsVtx, Vtx)`: the end at the previous
    /// point on the arc `arc` of the surface (`on_first`) or on the restriction, and on
    /// every arc through its vertex.
    pub(super) fn make_extremity(
        &self,
        extrem: &mut BRepBlendExtremity,
        on_first: bool,
        (arc, param): (&Arc, f64),
        vtx: &VertexFlag,
    ) -> Result<(), Failure> {
        let previous = &self.previous_p;
        let (w, tol) = (previous.parameter(), self.tolpoint3d);
        if on_first {
            extrem.set_value(previous.point_on_s(), self.sol[0], self.sol[1], w, tol);
        } else {
            extrem.set_value_on_curve(previous.point_on_c(), self.sol[2], w, tol);
        }
        if !previous.is_tangency_point() {
            extrem.set_tangent(if on_first {
                previous.tangent_on_s()?
            } else {
                previous.tangent_on_c()?
            });
        }
        let line = lock(self.line_handle()?);
        let ends = ArcEnds {
            class: CLASS,
            line: &line,
            previous,
            sens: self.sens,
            on_first,
            surface: self.surface(on_first)?,
            domain: if on_first {
                &self.domain1
            } else {
                &self.domain2
            },
        };
        ends.add_arcs(extrem, (arc, param), vtx.vertex())
    }
}
