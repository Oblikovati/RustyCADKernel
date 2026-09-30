// Rust translation of ProcessSegments of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The restriction arcs that lie on the other quadric become restriction lines, with
//! the multiple points of the other lines at their ends.
use super::super::line::IntPatchLine;
use super::super::line::IntPatchLineTransitions;
use super::super::path_point::{
    IntPatchThePathPointOfTheSOnBounds, IntPatchTheSegmentOfTheSOnBounds,
};
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::lines::VertexLine;
use super::transitions::by_sign_tol;
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfQuadric, IntSurfTransition, IntSurfTypeTrans, make_transition};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2, Vector3};

/// The locals of `ProcessSegments` that keep their values from one segment (or vertex)
/// to the next: the vertex `ptvtx`, the derivatives, the tangents and the transitions.
#[derive(Default)]
struct SegmentState {
    ptvtx: IntPatchPoint,
    d1u: Vector3,
    d1v: Vector3,
    tgline: Vector3,
    tgarc: Vector3,
    norm1: Vector3,
    norm2: Vector3,
    trest: IntSurfTransition,
    tarc: IntSurfTransition,
}

/// The quadrics and the domain side of the segments.
pub(super) struct SegmentInput<'a> {
    pub(super) quad1: &'a IntSurfQuadric,
    pub(super) quad2: &'a IntSurfQuadric,
    pub(super) on_first: bool,
    pub(super) tol_arc: f64,
}

impl SegmentInput<'_> {
    /// The quadric of the restrictions.
    fn own_quad(&self) -> &IntSurfQuadric {
        if self.on_first {
            self.quad1
        } else {
            self.quad2
        }
    }
}

impl SegmentState {
    /// The quadric derivatives at the arc point of parameter `prm` (kept when the
    /// quadric is undefined, as natively); returns the arc derivative and the point.
    fn derivatives_at(
        &mut self,
        quad: &IntSurfQuadric,
        arc: &TheArc,
        prm: f64,
    ) -> Result<(Vector2, Option<Point3>), Failure> {
        let (p2d, d2d): (Point2, Vector2) = arc.d1(prm)?;
        let valpt = quad.d1(p2d.x(), p2d.y()).map(|(p, d1u, d1v)| {
            (self.d1u, self.d1v) = (d1u, d1v);
            p
        });
        Ok((d2d, valpt))
    }
}

/// The restriction line of an arc, with its arc on the side of the segments.
fn new_rline(
    arc: &TheArc,
    on_first: bool,
    transitions: Option<(IntSurfTypeTrans, IntSurfTypeTrans)>,
) -> Handle<IntPatchRLine> {
    let rline = match transitions {
        Some((t1, t2)) => IntPatchRLine::new(false, t1, t2),
        None => IntPatchRLine::new_undecided(false),
    };
    if on_first {
        rline.set_arc_on_s1(arc);
    } else {
        rline.set_arc_on_s2(arc);
    }
    Handle::new(rline)
}

/// `ProcessSegments(listedg, slin, Quad1, Quad2, OnFirst, TolArc)`: each segment gives a
/// restriction line (with the transitions of its middle, undecided on a degenerated
/// edge), whose ends take the vertices of the lines within `tol_arc` (made multiple),
/// or the end points themselves.
pub(super) fn process_segments(
    listedg: &[IntPatchTheSegmentOfTheSOnBounds],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    input: &SegmentInput<'_>,
) -> Result<(), Failure> {
    let mut st = SegmentState::default();
    for thesegsol in listedg {
        let arc_ref: TheArc = thesegsol.curve();
        let (pstartf, pstartl) = (end_point(thesegsol, true)?, end_point(thesegsol, false)?);
        let (rline, edge_degenere) = segment_line(input, &mut st, &arc_ref, (&pstartf, &pstartl))?;
        let mut ends = Ends {
            dofirst: pstartf.is_some(),
            dolast: pstartl.is_some(),
            procf: false,
            procl: false,
        };
        if ends.dofirst || ends.dolast {
            let segment = Segment {
                arc_ref: &arc_ref,
                rline: &rline,
                pstartf: &pstartf,
                pstartl: &pstartl,
                edge_degenere,
            };
            for slinj in slin.iter() {
                ends_on_line(input, &mut st, &segment, slinj, &mut ends)?;
            }
        }
        own_ends(input, &mut st, &rline, (&pstartf, &pstartl), &ends)?;
        slin.push(rline.cast());
    }
    Ok(())
}

/// The first (or last) point of the segment and its parameter, when it has one.
fn end_point(
    seg: &IntPatchTheSegmentOfTheSOnBounds,
    first: bool,
) -> Result<Option<(IntPatchThePathPointOfTheSOnBounds, f64)>, Failure> {
    let has = if first {
        seg.has_first_point()
    } else {
        seg.has_last_point()
    };
    if !has {
        return Ok(None);
    }
    let p = if first {
        seg.first_point()?
    } else {
        seg.last_point()?
    }
    .clone();
    let prm: f64 = p.parameter();
    Ok(Some((p, prm)))
}

type EndPoint = Option<(IntPatchThePathPointOfTheSOnBounds, f64)>;

/// The restriction line of the segment: with both ends, its transitions come from the
/// middle of the arc, unless the edge is degenerated (a null U derivative there and at
/// the 11 samples).
fn segment_line(
    input: &SegmentInput<'_>,
    st: &mut SegmentState,
    arc_ref: &TheArc,
    (pstartf, pstartl): (&EndPoint, &EndPoint),
) -> Result<(Handle<IntPatchRLine>, bool), Failure> {
    let (Some((_, paramf)), Some((_, paraml))) = (pstartf, pstartl) else {
        return Ok((new_rline(arc_ref, input.on_first, None), false));
    };
    let quad = input.own_quad();
    let (d2d, valpt) = st.derivatives_at(quad, arc_ref, 0.5 * (paramf + paraml))?;
    st.tgline.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
    if st.d1u.magnitude() < 1e-7 {
        // A degenerated edge?
        let mut edge_degenere: bool = true;
        for edg in 0..=10 {
            let prm: f64 = paramf + (paraml - paramf) * f64::from(edg) * 0.1;
            st.derivatives_at(quad, arc_ref, prm)?;
            if st.d1u.magnitude() > 1e-7 {
                edge_degenere = false;
            }
        }
        return Ok((new_rline(arc_ref, input.on_first, None), edge_degenere));
    }
    let valpt: Point3 = valpt.unwrap_or_default();
    st.norm2 = input.quad2.normale_at(valpt)?;
    st.norm1 = input.quad1.normale_at(valpt)?;
    let transitions = match by_sign_tol(st.tgline.dot_cross(st.norm2, st.norm1), 0.000000001) {
        IntPatchLineTransitions::Types(t1, t2) => (t1, t2),
        _ => (IntSurfTypeTrans::Undecided, IntSurfTypeTrans::Undecided),
    };
    Ok((new_rline(arc_ref, input.on_first, Some(transitions)), false))
}

/// Whether each end is still to process, and whether it was processed.
struct Ends {
    dofirst: bool,
    dolast: bool,
    procf: bool,
    procl: bool,
}

/// The segment being processed.
struct Segment<'a> {
    arc_ref: &'a TheArc,
    rline: &'a Handle<IntPatchRLine>,
    pstartf: &'a EndPoint,
    pstartl: &'a EndPoint,
    edge_degenere: bool,
}

/// The vertices of the line `slinj` at the ends of the segment; after the line, the
/// processed ends are no longer searched (they may be multiple points).
fn ends_on_line(
    input: &SegmentInput<'_>,
    st: &mut SegmentState,
    seg: &Segment<'_>,
    slinj: &Handle<dyn IntPatchLine>,
    ends: &mut Ends,
) -> Result<(), Failure> {
    let vline = VertexLine::of(slinj)?;
    let nbpts: i32 = vline.nb_vertex();
    for k in 1..=nbpts {
        st.ptvtx = vline.vertex(k)?;
        if !seg.edge_degenere && ends.dofirst {
            let (p, prm) = seg
                .pstartf
                .as_ref()
                .map(|(p, prm)| (p.value(), *prm))
                .unwrap_or_default();
            if st.ptvtx.value().distance(&p) <= input.tol_arc {
                end_vertex(input, st, seg, (&vline, k), prm)?;
                if !ends.procf {
                    ends.procf = true;
                    seg.rline.set_first_point(seg.rline.nb_vertex());
                }
            }
        }
        if !seg.edge_degenere && ends.dolast {
            let (p, prm) = seg
                .pstartl
                .as_ref()
                .map(|(p, prm)| (p.value(), *prm))
                .unwrap_or_default();
            if st.ptvtx.value().distance(&p) <= input.tol_arc {
                end_vertex(input, st, seg, (&vline, k), prm)?;
                if !ends.procl {
                    ends.procl = true;
                    seg.rline.set_last_point(seg.rline.nb_vertex());
                }
            }
        }
    }
    if ends.procf {
        ends.dofirst = false;
    }
    if ends.procl {
        ends.dolast = false;
    }
    Ok(())
}

/// The vertex `k` of the line is made multiple (replaced), and a copy at the end
/// parameter `prm` of the arc, with its transitions on the arcs it lies on, is added to
/// the restriction line.
fn end_vertex(
    input: &SegmentInput<'_>,
    st: &mut SegmentState,
    seg: &Segment<'_>,
    (vline, k): (&VertexLine, i32),
    prm: f64,
) -> Result<(), Failure> {
    st.ptvtx.set_multiple(true);
    st.ptvtx.set_tolerance(input.tol_arc);
    vline.replace(k, &st.ptvtx)?;
    let mut newptvtx: IntPatchPoint = st.ptvtx.clone();
    newptvtx.set_parameter(prm);
    // The transitions again, when the point is on a restriction.
    let (d2d, _) = st.derivatives_at(input.own_quad(), seg.arc_ref, prm)?;
    st.tgline.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
    if st.ptvtx.is_on_dom_s1() {
        let (arc, prm1) = (st.ptvtx.arc_on_s1()?, st.ptvtx.parameter_on_arc1()?);
        arc_transitions(st, input.quad1, &arc, prm1, true)?;
        newptvtx.set_arc(true, &arc, prm1, &st.trest, &st.tarc);
    }
    if st.ptvtx.is_on_dom_s2() {
        let (arc, prm2) = (st.ptvtx.arc_on_s2()?, st.ptvtx.parameter_on_arc2()?);
        arc_transitions(st, input.quad2, &arc, prm2, false)?;
        newptvtx.set_arc(false, &arc, prm2, &st.trest, &st.tarc);
    }
    seg.rline.add_vertex(&newptvtx, false);
    Ok(())
}

/// The transitions of the restriction line (`tgline`) and of the arc of the vertex.
fn arc_transitions(
    st: &mut SegmentState,
    quad: &IntSurfQuadric,
    arc: &TheArc,
    prm: f64,
    first: bool,
) -> Result<(), Failure> {
    let (d2d, _) = st.derivatives_at(quad, arc, prm)?;
    st.tgarc.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
    let norm: Vector3 = st.d1u.crossed(st.d1v);
    if first {
        st.norm1 = norm;
    } else {
        st.norm2 = norm;
    }
    if norm.square_magnitude() < 1e-16 {
        st.trest.set_type(true, IntSurfTypeTrans::Undecided);
        st.tarc.set_type(true, IntSurfTypeTrans::Undecided);
        return Ok(());
    }
    make_transition(
        st.tgline,
        st.tgarc,
        Direction3::from_vector(norm)?,
        &mut st.trest,
        &mut st.tarc,
    );
    Ok(())
}

/// The ends not found on the lines are placed on the restriction line anyway, with the
/// vertex and arc of the end point (default transitions) when it is on a vertex.
fn own_ends(
    input: &SegmentInput<'_>,
    st: &mut SegmentState,
    rline: &Handle<IntPatchRLine>,
    (pstartf, pstartl): (&EndPoint, &EndPoint),
    ends: &Ends,
) -> Result<(), Failure> {
    if ends.dofirst
        && let Some((p, prm)) = pstartf
    {
        own_end(input, st, rline, (p, *prm))?;
        rline.set_first_point(rline.nb_vertex());
    }
    if ends.dolast
        && let Some((p, prm)) = pstartl
    {
        own_end(input, st, rline, (p, *prm))?;
        rline.set_last_point(rline.nb_vertex());
    }
    Ok(())
}

fn own_end(
    input: &SegmentInput<'_>,
    st: &mut SegmentState,
    rline: &Handle<IntPatchRLine>,
    (p, prm): (&IntPatchThePathPointOfTheSOnBounds, f64),
) -> Result<(), Failure> {
    st.ptvtx.set_value_tangent(p.value(), p.tolerance(), false);
    let (u1, v1) = input.quad1.parameters(p.value()).unwrap_or((0.0, 0.0));
    let (u2, v2) = input.quad2.parameters(p.value()).unwrap_or((0.0, 0.0));
    st.ptvtx.set_parameters(u1, v1, u2, v2);
    st.ptvtx.set_parameter(prm);
    if !p.is_new() {
        let (transline, transarc) = (IntSurfTransition::default(), IntSurfTransition::default());
        st.ptvtx.set_vertex(input.on_first, &p.vertex()?);
        st.ptvtx.set_arc(
            input.on_first,
            &p.arc(),
            p.parameter(),
            &transline,
            &transarc,
        );
    }
    rline.add_vertex(&st.ptvtx, false);
    Ok(())
}
