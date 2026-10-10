// Rust translation of BRepFill_OffsetWire::PrepareSpine and of the static EdgeVertices,
// CutEdge and CutCurve (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The work spine: the edges of the spine cut at the extrema of curvature and the
//! inflexions (a lone closed edge in three, two edges in two), each part mapped to its
//! edge.
use super::BRepFillOffsetWire;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
use rustycad_tktopalgo::mat2d::Mat2dCutCurve;
use std::f64::consts::PI;

/// `EdgeVertices(E, V1, V2)`: the vertices of the edge in its orientation.
pub(super) fn edge_vertices(e: &Edge) -> (Vertex, Vertex) {
    let (v1, v2) = topexp::vertices(e, false);
    if e.orientation() == Orientation::Reversed {
        (v2, v1)
    } else {
        (v1, v2)
    }
}

/// `CutCurve(C, nbParts, theCurves)`: the curve cut in `nb_parts` equal parameter
/// steps, a step too close to the start or the end skipped; the whole curve when no
/// cut is made.
fn cut_curve(
    c: &Handle<Geom2dTrimmedCurve>,
    nb_parts: i32,
    the_curves: &mut Vec<Handle<dyn Geom2dCurve>>,
) -> Result<(), Failure> {
    let (p_tol, tol): (f64, f64) = (precision::p_confusion() * 10., precision::confusion() * 10.);
    let curve = c.get().ok_or_else(null_curve)?;
    let basis: Handle<dyn Geom2dCurve> = c.clone().cast();
    let (mut uf, ul) = (curve.first_parameter(), curve.last_parameter());
    let (mut pf, pl) = (curve.value(uf)?, curve.value(ul)?);
    let step: f64 = (ul - uf) / f64::from(nb_parts);
    let mut ya_cut = false;
    for i in 1..nb_parts {
        let uc: f64 = uf + f64::from(i) * step;
        let pc = curve.value(uc)?;
        if uc - uf > p_tol && pc.distance(pf) > tol {
            if ul - uc < p_tol || pl.distance(pc) < tol {
                continue;
            }
            the_curves
                .push(Handle::new(Geom2dTrimmedCurve::new(&basis, uf, uc, true, true)?).cast());
            (uf, pf) = (uc, pc);
            ya_cut = true;
        }
    }
    if ya_cut {
        the_curves.push(Handle::new(Geom2dTrimmedCurve::new(&basis, uf, ul, true, true)?).cast());
    } else {
        the_curves.push(basis);
    }
    Ok(())
}

/// The pieces of MAT2d_CutCurve as curves.
fn cutter_pieces(
    cuter: &Mat2dCutCurve,
    the_curves: &mut Vec<Handle<dyn Geom2dCurve>>,
) -> Result<(), Failure> {
    for k in 1..=cuter.nb_curves()? {
        the_curves.push(cuter.value(k)?.cast());
    }
    Ok(())
}

/// The pieces of the trimmed p-curve by the forced cut: none (only the cutter's), two
/// when unmodified (1), three when unmodified or a halved larger piece of two (2).
fn forced_pieces(
    cuter: &Mat2dCutCurve,
    ct2d: &Handle<Geom2dTrimmedCurve>,
    force_cut: i32,
    (f, l): (f64, f64),
) -> Result<Vec<Handle<dyn Geom2dCurve>>, Failure> {
    let mut the_curves: Vec<Handle<dyn Geom2dCurve>> = Vec::new();
    if cuter.un_modified() {
        match force_cut {
            1 => cut_curve(ct2d, 2, &mut the_curves)?,
            2 => cut_curve(ct2d, 3, &mut the_curves)?,
            _ => {}
        }
        return Ok(the_curves);
    }
    if force_cut != 2 || cuter.nb_curves()? != 2 {
        cutter_pieces(cuter, &mut the_curves)?;
        return Ok(the_curves);
    }
    let cc = cuter.value(1)?;
    if cc.get().ok_or_else(null_curve)?.last_parameter() > (l + f) / 2. {
        cut_curve(&cc, 2, &mut the_curves)?;
        the_curves.push(cuter.value(2)?.cast());
    } else {
        the_curves.push(cc.cast());
        cut_curve(&cuter.value(2)?, 2, &mut the_curves)?;
    }
    Ok(the_curves)
}

/// `CutEdge(E, F, ForceCut, Cuts)`: the parts of the edge cut at the extrema of
/// curvature and the inflexions of its p-curve (forced in two or three parts), with
/// new vertices at the cuts (of the edge tolerance); no part for an arc of half a
/// circle or more, or an unmodified curve without forced cut. Returns the part count.
pub(super) fn cut_edge(
    e: &Edge,
    f_face: &Face,
    force_cut: i32,
    cuts: &mut Vec<Shape>,
) -> Result<usize, Failure> {
    cuts.clear();
    let a_tol: f64 = BRepTool::edge_tolerance(e)?;
    let (v1, v2) = topexp::vertices(e, false);
    let pc = BRepTool::curve_on_face(e, f_face)?;
    let (a_c, f, l) = BRepTool::curve(e)?;
    let ct2d: Handle<Geom2dTrimmedCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&pc.pcurve, f, l, true, true)?);
    let basis = ct2d.get().ok_or_else(null_curve)?.basis_curve();
    if !basis.cast::<Geom2dCircle>().is_null() && (f - l).abs() >= PI {
        return Ok(0);
    }
    let mut cuter = Mat2dCutCurve::new();
    cuter.perform(&ct2d.clone().cast())?;
    if force_cut == 0 && cuter.un_modified() {
        return Ok(0);
    }
    let the_curves = forced_pieces(&cuter, &ct2d, force_cut, (f, l))?;
    let curve3d = a_c.get().ok_or_else(null_curve)?;
    let mut vf: Vertex = v1;
    for (k, piece) in the_curves.iter().enumerate() {
        let cc: Handle<Geom2dTrimmedCurve> = piece.cast();
        let cc = cc.get().ok_or_else(null_curve)?;
        let vl: Vertex = if k + 1 == the_curves.len() {
            v2.clone()
        } else {
            cut_vertex(curve3d.value(cc.last_parameter())?, a_tol)?
        };
        cuts.push(cut_part(
            e,
            &vf,
            &vl,
            (cc.first_parameter(), cc.last_parameter()),
        )?);
        vf = vl;
    }
    Ok(the_curves.len())
}

/// A vertex at a cut point with the edge tolerance.
fn cut_vertex(p: rustycad_tkmath::gp::Point3, a_tol: f64) -> Result<Vertex, Failure> {
    let vl: Vertex = BRepLibMakeVertex::new(p)?.vertex()?;
    BRepBuilder::default().update_vertex_tolerance(&vl, a_tol)?;
    Ok(vl)
}

/// A part of the edge: an empty copy between the two vertices on the range, in the
/// orientation of the edge.
fn cut_part(
    e: &Edge,
    vf: &Vertex,
    vl: &Vertex,
    (first, last): (f64, f64),
) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut ne: Edge = topods::edge(&e.empty_copied()?)?;
    ne.set_orientation(Orientation::Forward);
    b.add(&mut ne, &vf.oriented(Orientation::Forward))?;
    b.add(&mut ne, &vl.oriented(Orientation::Reversed))?;
    b.range(&ne, first, last, false)?;
    Ok(ne.oriented(e.orientation()))
}

impl BRepFillOffsetWire {
    /// `PrepareSpine()` (private): the work spine of the cut wires of the spine, each
    /// part (and its new vertices) mapped to its edge in myMapSpine.
    pub(super) fn prepare_spine(&mut self) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        self.my_work_spine = Face::default();
        self.my_map_spine.clear(true);
        let (s, l) = BRepTool::surface_with_location(&self.my_spine)?;
        let tol_f: f64 = BRepTool::face_tolerance(&self.my_spine)?;
        b.make_face_on_located_surface(&mut self.my_work_spine, &s, &l, tol_f)?;
        let mut ite_f = ShapeIterator::new(self.my_spine.as_ref(), true, true);
        while ite_f.more() {
            let nw: Wire = self.cut_wire(ite_f.value()?)?;
            let mut work = self.my_work_spine.clone();
            b.add(&mut work, &nw)?;
            ite_f.next();
        }
        Ok(())
    }

    /// The new wire of the cut edges of a wire of the spine, flagged closed when its
    /// ends are the same vertex.
    fn cut_wire(&mut self, wire: &Shape) -> Result<Wire, Failure> {
        let b = BRepBuilder::default();
        let mut nw = Wire::default();
        b.make_wire(&mut nw);
        let mut edge_map = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(wire, ShapeEnum::Edge, &mut edge_map)?;
        let nb_edges: usize = edge_map.extent();
        // In case of an open wire there is no need to force the cut of a lone edge.
        let mut forced_cut: i32 = if nb_edges == 1 && !self.my_is_open_result {
            2
        } else {
            0
        };
        let mut nb_res_edges: i64 = -1;
        let mut ite_w = ShapeIterator::new(wire, true, true);
        while ite_w.more() {
            let e: Edge = topods::edge(ite_w.value()?)?;
            let a_num_curves_in_edge: usize = edge_tshape(&e, "BRepFill_OffsetWire::PrepareSpine")?
                .curves()
                .len();
            if nb_edges == 2 && nb_res_edges == 0 && a_num_curves_in_edge > 1 {
                forced_cut = 1;
            }
            nb_res_edges = self.add_cut_edge(&mut nw, &e, forced_cut)? as i64;
            ite_w.next();
        }
        let (a_v1, a_v2) = topexp::wire_vertices(&nw);
        nw.set_closed(a_v1.is_same(&a_v2))?;
        Ok(nw)
    }

    /// The parts of an edge added to the new wire and mapped to the edge; the count of
    /// parts.
    fn add_cut_edge(&mut self, nw: &mut Wire, e: &Edge, forced_cut: i32) -> Result<usize, Failure> {
        let b = BRepBuilder::default();
        let (v1, v2) = edge_vertices(e);
        self.my_map_spine
            .bind(v1.as_ref().clone(), v1.as_ref().clone());
        self.my_map_spine
            .bind(v2.as_ref().clone(), v2.as_ref().clone());
        let mut cuts: Vec<Shape> = Vec::new();
        let nb_res: usize = cut_edge(
            &topods::edge(&e.oriented(Orientation::Forward))?,
            &self.my_spine,
            forced_cut,
            &mut cuts,
        )?;
        if cuts.is_empty() {
            b.add(nw, e)?;
            self.my_map_spine
                .bind(e.as_ref().clone(), e.as_ref().clone());
            return Ok(nb_res);
        }
        for cut in cuts {
            let mut ne: Edge = topods::edge(&cut)?;
            ne.set_orientation(e.orientation());
            b.add(nw, &ne)?;
            self.my_map_spine
                .bind(ne.as_ref().clone(), e.as_ref().clone());
            let (n1, n2) = edge_vertices(&ne);
            for v in [n1, n2] {
                if !self.my_map_spine.is_bound(v.as_ref()) {
                    self.my_map_spine.bind(v.into_shape(), e.as_ref().clone());
                }
            }
        }
        Ok(nb_res)
    }
}

/// Standard_NullObject of a null curve.
fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null curve of a spine edge; expected a curve",
        "BRepFill_OffsetWire",
    )
}
