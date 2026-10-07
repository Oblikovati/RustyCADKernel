// Rust translation of the body of the edge loop of BRepFill_Generator::Perform
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One ruled face between the current edges of two consecutive sections: its two
//! section edges and their vertices, and its surface.
use super::BRepFillGenerator;
use super::kpart_create::create_kpart;
use super::kpart_detect::{KPART_NONE, KPART_NULL_CURVE, detect_kpart};
use super::lateral::segment;
use super::section_pair::SectionPair;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::GeomFillGenerator;

/// The section edges of one face, their vertices in the orientation of the edges and
/// the vertices that key the lateral edges.
pub(super) struct FaceSections {
    pub(super) edge1: Edge,
    pub(super) edge2: Edge,
    pub(super) degen1: bool,
    pub(super) degen2: bool,
    pub(super) v1f: Vertex,
    pub(super) v1l: Vertex,
    pub(super) v2f: Vertex,
    pub(super) v2l: Vertex,
    pub(super) vf_to_map: Vertex,
    pub(super) vl_to_map: Vertex,
    /// Both section edges are closed (or degenerated): the face closes on a seam.
    pub(super) periodic: bool,
}

impl BRepFillGenerator {
    /// The section edges of the next face: a degenerated edge is replaced by an
    /// empty copy with its two vertices on `[0, 1]` (recorded as modified).
    pub(super) fn face_sections(&mut self, pair: &SectionPair) -> Result<FaceSections, Failure> {
        let or_edge1: Edge = pair.ex1.current().clone();
        let or_edge2: Edge = pair.ex2.current().clone();
        let degen1: bool = BRepTool::degenerated(&or_edge1)?;
        let degen2: bool = BRepTool::degenerated(&or_edge2)?;
        let edge1: Edge = self.section_edge(&or_edge1, degen1)?;
        let edge2: Edge = self.section_edge(&or_edge2, degen2)?;
        let periodic: bool =
            (BRepTool::is_closed(&edge1)? || degen1) && (BRepTool::is_closed(&edge2)? || degen2);
        let (v1f, v1l) = oriented_vertices(&edge1);
        let (v2f, v2l) = oriented_vertices(&edge2);
        let (vf_to_map, vl_to_map) = if degen1 {
            (v2f.clone(), v2l.clone())
        } else {
            (v1f.clone(), v1l.clone())
        };
        Ok(FaceSections {
            edge1,
            edge2,
            degen1,
            degen2,
            v1f,
            v1l,
            v2f,
            v2l,
            vf_to_map,
            vl_to_map,
            periodic,
        })
    }

    /// The edge itself, or the empty copy of a degenerated edge bounded by its own
    /// vertices on `[0, 1]`, bound in the old-new map.
    fn section_edge(&mut self, or_edge: &Edge, degenerated: bool) -> Result<Edge, Failure> {
        if !degenerated {
            return Ok(or_edge.clone());
        }
        let builder = BRepBuilder::default();
        let mut edge: Edge = topods::edge(&or_edge.empty_copied()?)?;
        let (mut vf, mut vl) = topexp::vertices(or_edge, false);
        vf.set_orientation(Orientation::Forward);
        builder.add(&mut edge, &vf)?;
        vl.set_orientation(Orientation::Reversed);
        builder.add(&mut edge, &vl)?;
        builder.range(&edge, 0., 1., false)?;
        self.my_old_new_shapes
            .bind(or_edge.as_ref().clone(), edge.as_ref().clone());
        Ok(edge)
    }
}

/// The vertices of an edge in its own orientation (swapped for a REVERSED edge).
fn oriented_vertices(edge: &Edge) -> (Vertex, Vertex) {
    let (first, last) = topexp::vertices(edge, false);
    if edge.orientation() == Orientation::Reversed {
        return (last, first);
    }
    (first, last)
}

/// The outcome of the surface of a face: the surface and the particular case code,
/// or a section edge without a 3d curve.
pub(super) enum RuledSurface {
    Built(Handle<dyn GeomSurface>, i32),
    Null3DCurve,
}

/// The surface ruled between the two section edges: an analytic surface for a
/// particular case, the GeomFill_Generator surface between the trimmed, located and
/// oriented curves otherwise.
pub(super) fn ruled_surface(sections: &FaceSections) -> Result<RuledSurface, Failure> {
    let itype: i32 = detect_kpart(&sections.edge1, &sections.edge2)?;
    if itype == KPART_NULL_CURVE {
        return Ok(RuledSurface::Null3DCurve);
    }
    if itype != KPART_NONE {
        return Ok(
            match create_kpart(&sections.edge1, &sections.edge2, itype)? {
                Some(surface) => RuledSurface::Built(surface, itype),
                None => RuledSurface::Null3DCurve,
            },
        );
    }
    let Some(c1) = general_curve(
        &sections.edge1,
        sections.degen1,
        (&sections.v1f, &sections.v1l),
    )?
    else {
        return Ok(RuledSurface::Null3DCurve);
    };
    // The punctual second side runs from its last vertex to its first, as natively.
    let Some(c2) = general_curve(
        &sections.edge2,
        sections.degen2,
        (&sections.v2l, &sections.v2f),
    )?
    else {
        return Ok(RuledSurface::Null3DCurve);
    };
    let mut generator = GeomFillGenerator::new();
    generator.add_curve(&c1)?;
    generator.add_curve(&c2)?;
    generator.perform(precision::p_confusion())?;
    Ok(RuledSurface::Built(generator.surface(), KPART_NONE))
}

/// The curve of one side of the general ruled surface: a Bezier segment between the
/// given vertices for a degenerated edge; else the 3d curve, trimmed to the edge range
/// when it differs from the curve bounds (a copy otherwise), moved by its location and
/// reversed for a REVERSED edge. `None` when the edge has no 3d curve.
fn general_curve(
    edge: &Edge,
    degenerated: bool,
    ends: (&Vertex, &Vertex),
) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    if degenerated {
        return Ok(Some(segment(
            BRepTool::pnt(ends.0)?,
            BRepTool::pnt(ends.1)?,
        )?));
    }
    let found = BRepTool::curve_with_location(edge)?;
    let Some(stored) = found.curve.get() else {
        return Ok(None);
    };
    let (f, l) = (found.first, found.last);
    let trimmed: bool = (f - stored.first_parameter()).abs() > precision::p_confusion()
        || (l - stored.last_parameter()).abs() > precision::p_confusion();
    let curve: Handle<dyn GeomCurve> = if trimmed {
        Handle::new(GeomTrimmedCurve::new(&found.curve, f, l, true, true)?).cast::<dyn GeomCurve>()
    } else {
        stored.copy()?.cast::<dyn GeomCurve>()
    };
    let Some(owned) = curve.get() else {
        return Ok(None);
    };
    owned.transform(found.location.transformation())?;
    if edge.orientation() == Orientation::Reversed {
        owned.reverse()?;
    }
    Ok(Some(curve))
}
