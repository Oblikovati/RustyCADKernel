// Rust translation of Transition and the arc part of MakeExtremity, whose text
// BRepBlend_SurfRstLineBuilder.cxx and BRepBlend_RstRstLineBuilder.cxx share, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The arcs of the end of a line builder: the transitions of the line and of an arc,
//! and the arcs through the vertex the end may be.
use super::marching::{WalkingDomain, line_tangent, live_rst};
use super::{BRepBlendBlendTool, BRepBlendExtremity, BRepBlendLine};
use crate::blend::BlendPoint;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkgeomalgo::int_surf::{IntSurfTransition, make_transition};
use rustycad_tkmath::gp::{Direction3, Vector3};

/// A restriction arc.
type Arc = Handle<dyn Adaptor2dCurve2d>;

/// The end of a line on one side: the line and its previous point, the direction, and
/// the surface and domain of that side.
pub(crate) struct ArcEnds<'a> {
    pub(crate) class: &'a str,
    pub(crate) line: &'a BRepBlendLine,
    pub(crate) previous: &'a BlendPoint,
    pub(crate) sens: f64,
    pub(crate) on_first: bool,
    pub(crate) surface: &'a dyn Adaptor3dSurface,
    pub(crate) domain: &'a WalkingDomain,
}

impl ArcEnds<'_> {
    /// `Transition(OnFirst, Arc, Param, TLine, TArc)`: the transitions of the line and of
    /// the arc at `param`; the default ones on a line too short for a chord.
    pub(crate) fn transition(
        &self,
        arc: &Arc,
        param: f64,
    ) -> Result<(IntSurfTransition, IntSurfTransition), Failure> {
        let (mut tline, mut tarc) = (IntSurfTransition::new(), IntSurfTransition::new());
        let Some(tgline) = line_tangent(self.line, self.previous, self.sens, self.on_first)? else {
            return Ok((tline, tarc));
        };
        let (p2d, dp2d) = live_rst(arc, self.class)?.d1(param)?;
        let d1 = self.surface.d1(p2d.x(), p2d.y())?;
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

    /// The arcs of `MakeExtremity`: the arc `arc` at `param`, or, at the vertex `vtx`,
    /// every arc of the domain through it (the arc itself at `param`).
    pub(crate) fn add_arcs(
        &self,
        extrem: &mut BRepBlendExtremity,
        (arc, param): (&Arc, f64),
        vtx: Option<&Handle<dyn Adaptor3dHVertex>>,
    ) -> Result<(), Failure> {
        self.domain.init();
        let Some(vertex) = vtx else {
            let (tline, tarc) = self.transition(arc, param)?;
            extrem.add_arc(arc, param, tline, tarc);
            return Ok(());
        };
        extrem.set_vertex(vertex);
        while self.domain.more() {
            let other = self.domain.value()?;
            if &other == arc {
                let (tline, tarc) = self.transition(&other, param)?;
                extrem.add_arc(&other, param, tline, tarc);
            } else {
                self.add_vertex_arc(extrem, &other, vertex)?;
            }
            self.domain.next();
        }
        Ok(())
    }

    /// The arc `other` at each of its vertices identical to `vertex`.
    fn add_vertex_arc(
        &self,
        extrem: &mut BRepBlendExtremity,
        other: &Arc,
        vertex: &Handle<dyn Adaptor3dHVertex>,
    ) -> Result<(), Failure> {
        self.domain.initialize_arc(other)?;
        self.domain.init_vertex_iterator()?;
        while self.domain.more_vertex() {
            if self.domain.identical(vertex, &self.domain.vertex()?)? {
                let prm = BRepBlendBlendTool::parameter(vertex, other)?;
                let (tline, tarc) = self.transition(other, prm)?;
                extrem.add_arc(other, prm, tline, tarc);
            }
            self.domain.next_vertex()?;
        }
        Ok(())
    }
}
