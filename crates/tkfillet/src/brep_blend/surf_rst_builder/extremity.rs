// Rust translation of BRepBlend_SurfRstLineBuilder::Transition and MakeExtremity
// (BRepBlend_SurfRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of the surface-restriction marching on the arcs: their transitions, and the
//! arcs through the vertex they may be.
use super::{Arc, BRepBlendSurfRstLineBuilder, VertexFlag};
use crate::brep_blend::marching::{WalkingDomain, line_tangent};
use crate::brep_blend::{BRepBlendBlendTool, BRepBlendExtremity};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_surf::{IntSurfTransition, make_transition};
use rustycad_tkmath::gp::{Direction3, Vector3};

impl BRepBlendSurfRstLineBuilder {
    /// `Transition(OnFirst, Arc, Param, TLine, TArc)`: the transitions of the line and of
    /// the arc at `param`, on the surface (`on_first`) or on the surface of the
    /// restriction; the default ones on a line too short for a chord.
    fn transition(
        &self,
        on_first: bool,
        arc: &Arc,
        param: f64,
    ) -> Result<(IntSurfTransition, IntSurfTransition), Failure> {
        let (mut tline, mut tarc) = (IntSurfTransition::new(), IntSurfTransition::new());
        let tgline = {
            let line = lock(self.line_handle()?);
            line_tangent(&line, &self.previous_p, self.sens, on_first)?
        };
        let Some(tgline) = tgline else {
            return Ok((tline, tarc));
        };
        let live_arc = crate::brep_blend::marching::live_rst(arc, super::CLASS)?;
        let (p2d, dp2d) = live_arc.d1(param)?;
        let d1 = self.surface(on_first)?.d1(p2d.x(), p2d.y())?;
        let mut tgrst = Vector3::default();
        tgrst.set_linear_form(dp2d.x(), d1.d1u, dp2d.y(), d1.d1v);
        let normale = d1.d1u.crossed(d1.d1v);
        make_transition(
            tgline,
            tgrst,
            Direction3::from_vector(normale)?,
            &mut tline,
            &mut tarc,
        );
        Ok((tline, tarc))
    }

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
        let iter: &WalkingDomain = if on_first {
            &self.domain1
        } else {
            &self.domain2
        };
        iter.init();
        let Some(vertex) = vtx.vertex() else {
            let (tline, tarc) = self.transition(on_first, arc, param)?;
            extrem.add_arc(arc, param, tline, tarc);
            return Ok(());
        };
        extrem.set_vertex(vertex);
        while iter.more() {
            let other = iter.value()?;
            self.add_vertex_arcs(extrem, on_first, (&other, arc, param), vertex)?;
            iter.next();
        }
        Ok(())
    }

    /// The arc `other` of the domain at the end: the arc of the end itself at `param`,
    /// another arc at each of its vertices identical to `vertex`.
    fn add_vertex_arcs(
        &self,
        extrem: &mut BRepBlendExtremity,
        on_first: bool,
        (other, arc, param): (&Arc, &Arc, f64),
        vertex: &super::Vertex,
    ) -> Result<(), Failure> {
        if other == arc {
            let (tline, tarc) = self.transition(on_first, other, param)?;
            extrem.add_arc(other, param, tline, tarc);
            return Ok(());
        }
        let iter: &WalkingDomain = if on_first {
            &self.domain1
        } else {
            &self.domain2
        };
        iter.initialize_arc(other)?;
        iter.init_vertex_iterator()?;
        while iter.more_vertex() {
            if iter.identical(vertex, &iter.vertex()?)? {
                let prm = BRepBlendBlendTool::parameter(vertex, other)?;
                let (tline, tarc) = self.transition(on_first, other, prm)?;
                extrem.add_arc(other, prm, tline, tarc);
            }
            iter.next_vertex()?;
        }
        Ok(())
    }
}
