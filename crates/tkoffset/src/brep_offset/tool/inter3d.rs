// Rust translation of BRepOffset_Tool::Inter3D with the static PerformPlanes,
// UpdateVertexTolerances, IsInf, IsAutonomVertex, AreConnex, AreClosed and CheckIntersFF of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The section edges of two offset faces: planes directly by `IntTools_FaceFace`, any
//! other pair by the pave filler, then glued into longer edges where the sections of
//! non-elementary faces were split.
use super::glue::{angle_wire_edge, assemble_edge, reconstruct_pcurves};
use super::pipe_inter::append_section;
use super::{BRepOffsetTool, THE_INFINI, live};
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbo::bop_tools::{algo_tools2d, make_connexity_blocks};
use rustycad_tkbo::int_tools::{Context, FaceFace};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;
use rustycad_tktopalgo::brep_lib::{
    BRepLibMakeVertex, build_curves3d_default, same_parameter_shape,
};

/// `IsInf(theVal)`: whether the value is close to the enlargement bound `TheInfini`.
fn is_inf(the_val: f64) -> bool {
    the_val > THE_INFINI * 0.9
}

/// Whether both faces are planes of infinite (enlarged) bounds.
fn are_infinite_planes(f1: &Face, f2: &Face) -> Result<bool, Failure> {
    let bas1 = BRepAdaptorSurface::with_face(f1, false)?;
    let bas2 = BRepAdaptorSurface::with_face(f2, false)?;
    if bas1.get_type()? != SurfaceType::Plane || bas2.get_type()? != SurfaceType::Plane {
        return Ok(false);
    }
    let bas1 = BRepAdaptorSurface::with_face(f1, true)?;
    if !(is_inf(bas1.last_u_parameter()?) && is_inf(bas1.last_v_parameter()?)) {
        return Ok(false);
    }
    let bas2 = BRepAdaptorSurface::with_face(f2, true)?;
    Ok(is_inf(bas2.last_u_parameter()?) && is_inf(bas2.last_v_parameter()?))
}

/// `PerformPlanes(theFace1, theFace2, theSide, theL1, theL2)`: the single section edge
/// of two planes by `IntTools_FaceFace`, oriented and made same parameter.
fn perform_planes(
    face1: &Face,
    face2: &Face,
    side: State,
    l1: &mut List<Shape>,
    l2: &mut List<Shape>,
) -> Result<(), Failure> {
    l1.clear();
    l2.clear();
    let mut ff = FaceFace::default();
    ff.set_parameters(true, true, true, precision::confusion());
    ff.perform(face1, face2, false)?;
    if !ff.is_done() {
        return Ok(());
    }
    let lines = ff.lines()?;
    let Some(ic) = lines.first() else {
        return Ok(());
    };
    let builder = BRepBuilder::default();
    let mut e = Edge::default();
    builder.make_edge_with_curve(&mut e, ic.curve(), ic.tolerance())?;
    let (mut tf, mut tl, mut pf, mut pl) = (0., 0., Point3::default(), Point3::default());
    ic.bounds(&mut tf, &mut tl, &mut pf, &mut pl)?;
    let mut vf = Vertex::default();
    let mut vl = Vertex::default();
    builder.make_vertex_at(&mut vf, pf, ic.tolerance())?;
    builder.make_vertex_at(&mut vl, pl, ic.tolerance())?;
    vl.set_orientation(Orientation::Reversed);
    builder.add(&mut e, vf.as_ref())?;
    builder.add(&mut e, vl.as_ref())?;
    builder.update_edge_pcurve_on_face(&e, ic.first_curve2d(), face1, ic.tolerance())?;
    builder.update_edge_pcurve_on_face(&e, ic.second_curve2d(), face2, ic.tolerance())?;
    builder.range(&e, tf, tl, false)?;
    let (mut o1, mut o2) = BRepOffsetTool::orient_section(&e, face1, face2)?;
    if side == State::Out {
        o1 = rustycad_tkg3d::top_abs::reverse(o1);
        o2 = rustycad_tkg3d::top_abs::reverse(o2);
    }
    same_parameter_shape(e.as_ref(), precision::confusion(), true)?;
    l1.append(e.oriented(o1));
    l2.append(e.oriented(o2));
    Ok(())
}

/// `UpdateVertexTolerances(theFace)`: each vertex of the face covers the 3d curve and
/// the p-curve ends of its edges.
fn update_vertex_tolerances(the_face: &Face) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        the_face.as_ref(),
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut ve_map,
    )?;
    for i in 1..=ve_map.extent() {
        let a_vertex = topods::vertex(ve_map.find_key(i)?)?;
        let pnt_vtx = BRepTool::pnt(&a_vertex)?;
        for e in ve_map.find_from_index(i)?.iter() {
            let an_edge = topods::edge(e)?;
            let (v1, v2) = topexp::vertices(&an_edge, false);
            let (fpar, lpar) = BRepTool::range(&an_edge)?;
            let a_param = if v1.is_same(&a_vertex) { fpar } else { lpar };
            let closed = v1.is_same(&v2);
            if !BRepTool::degenerated(&an_edge)? {
                let curve = BRepAdaptorCurve::with_edge(&an_edge)?;
                cover_point(
                    &builder,
                    &a_vertex,
                    pnt_vtx,
                    &curve,
                    (a_param, lpar),
                    closed,
                )?;
            }
            let on_surf = BRepAdaptorCurve::with_edge_on_face(&an_edge, the_face)?;
            cover_point(
                &builder,
                &a_vertex,
                pnt_vtx,
                &on_surf,
                (a_param, lpar),
                closed,
            )?;
        }
    }
    Ok(())
}

/// Raises the vertex tolerance to its distance to the curve point (and to the last
/// point of a closed edge).
fn cover_point(
    builder: &BRepBuilder,
    v: &Vertex,
    p: Point3,
    c: &BRepAdaptorCurve,
    (param, lpar): (f64, f64),
    closed: bool,
) -> Result<(), Failure> {
    builder.update_vertex_tolerance(v, p.distance(&c.value(param)?))?;
    if closed {
        builder.update_vertex_tolerance(v, p.distance(&c.value(lpar)?))?;
    }
    Ok(())
}

/// The data structure of a performed pave filler.
fn pave_ds(pf: &BOPAlgoPaveFiller) -> Result<&DataStructure, Failure> {
    pf.data_structure().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BOPAlgo_PaveFiller: no data structure",
        )
    })
}

/// `IsAutonomVertex(theVertex, thePDS, theFace1, theFace2)`: a vertex not on either face.
pub(super) fn is_autonom_vertex_on_faces(
    v: &Shape,
    ds: &DataStructure,
    f1: &Face,
    f2: &Face,
) -> Result<bool, Failure> {
    let n_v = ds.index(v);
    for f in [f1, f2] {
        let n_f = ds.index(f.as_ref());
        if ds.face_info(n_f)?.vertices_on().contains(n_v) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `IsAutonomVertex(aVertex, pDS)`: a new vertex not created by a vertex/vertex,
/// edge/edge or edge/face interference.
fn is_autonom_vertex(v: &Shape, ds: &DataStructure) -> Result<bool, Failure> {
    let mut index = ds.index(v);
    if index == -1 {
        for i in ds.nb_source_shapes()..ds.nb_shapes() {
            if ds.shape(i)?.is_same(v) {
                index = i;
                break;
            }
        }
    }
    if !ds.is_new_shape(index) {
        return Ok(false);
    }
    for vv in ds.interf_vv_records().iter() {
        if vv.has_index_new() && vv.index_new() == index {
            return Ok(false);
        }
    }
    for ee in ds.interf_ee_records().iter() {
        if ee.common_part().shape_type() == ShapeEnum::Vertex && ee.index_new() == index {
            return Ok(false);
        }
    }
    for ef in ds.interf_ef_records().iter() {
        if ef.common_part().shape_type() == ShapeEnum::Vertex && ef.index_new() == index {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `AreConnex(W1, W2)`: two wires sharing an end vertex.
fn are_connex(w1: &Wire, w2: &Wire) -> bool {
    let (v11, v12) = topexp::wire_vertices(w1);
    let (v21, v22) = topexp::wire_vertices(w2);
    v11.is_same(&v21) || v11.is_same(&v22) || v12.is_same(&v21) || v12.is_same(&v22)
}

/// The section edges of the face/face interferences of the data structure, in their
/// native order.
fn section_edges(ds: &DataStructure) -> Result<Vec<(Edge, usize, usize)>, Failure> {
    let mut edges = Vec::new();
    for (i, ff) in ds.interf_ff_records().iter().enumerate() {
        for (j, bc) in ff.curves().iter().enumerate() {
            for pb in bc.pave_blocks() {
                let n_sect = pb.lock().map_err(|_| poisoned())?.edge();
                edges.push((topods::edge(ds.shape(n_sect)?)?, i, j));
            }
        }
    }
    Ok(edges)
}

/// `CheckIntersFF(pDS, RefEdge, TrueEdges)`: the section edges of the connexity block
/// nearest to the middle of the reference edge (the only block when there is one).
fn check_inters_ff(
    ds: &DataStructure,
    ref_edge: &Edge,
    true_edges: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut edges = Compound::default();
    builder.make_compound(&mut edges);
    let sections = section_edges(ds)?;
    if sections.is_empty() {
        return Ok(());
    }
    for (e, _, _) in &sections {
        builder.add(&mut edges, e.as_ref())?;
    }
    let comp_list = connexity_compounds(edges.as_ref())?;
    let nearest = if comp_list.len() == 1 {
        comp_list[0].clone()
    } else {
        nearest_compound(&comp_list, ref_edge)?
    };
    topexp::map_shapes_of_type(&nearest, ShapeEnum::Edge, true_edges)
}

/// `BOPTools_AlgoTools::MakeConnexityBlocks(theS, VERTEX, EDGE, theLCB)`: a compound of
/// each connexity block.
fn connexity_compounds(s: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut blocks = Vec::new();
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(s, ShapeEnum::Vertex, ShapeEnum::Edge, &mut blocks, &mut map)?;
    let builder = BRepBuilder::default();
    let mut compounds = Vec::with_capacity(blocks.len());
    for block in blocks {
        let mut c = Compound::default();
        builder.make_compound(&mut c);
        for shape in &block {
            builder.add(&mut c, shape)?;
        }
        compounds.push(c.into_shape());
    }
    Ok(compounds)
}

/// The compound at the least distance from the middle point of the reference edge
/// (a null shape when no distance is computed).
fn nearest_compound(comp_list: &[Shape], ref_edge: &Edge) -> Result<Shape, Failure> {
    let ba_curve = BRepAdaptorCurve::with_edge(ref_edge)?;
    let p_ref = ba_curve.value((ba_curve.first_parameter()? + ba_curve.last_parameter()?) / 2.)?;
    let v_ref = BRepLibMakeVertex::new(p_ref)?.vertex()?;
    let mut min_dist = f64::MAX;
    let mut nearest = Shape::default();
    for a_compound in comp_list {
        let projector =
            BRepExtremaDistShapeShape::with_shapes(v_ref.as_ref(), a_compound, Default::default())?;
        if !projector.is_done() || projector.nb_solution() == 0 {
            continue;
        }
        let a_dist = projector.value()?;
        if a_dist < min_dist {
            min_dist = a_dist;
            nearest = a_compound.clone();
        }
    }
    Ok(nearest)
}

/// Whether the basis surface of the face is a plane (`Some(true)`), another elementary
/// surface (`Some(false)`), or neither (`None`).
fn plane_or_elementary(f: &Face) -> Result<Option<bool>, Failure> {
    let mut a_surf = BRepTool::surface(f)?;
    if a_surf.is_instance_name("Geom_RectangularTrimmedSurface") {
        a_surf = live(
            &a_surf.clone().cast::<GeomRectangularTrimmedSurface>(),
            "surface",
        )?
        .basis_surface();
    }
    if a_surf.is_instance_name("Geom_Plane") {
        return Ok(Some(true));
    }
    if a_surf.is_kind_name("Geom_ElementarySurface") {
        return Ok(Some(false));
    }
    Ok(None)
}

impl BRepOffsetTool {
    /// `Inter3D(F1, F2, L1, L2, Side, RefEdge, theRefFace1, theRefFace2)`: the section
    /// edges of two faces oriented in each, with missing p-curves, keeping only the
    /// block nearest the reference edge and glueing the splits of non-elementary faces;
    /// `BRepOffsetTool::inter3d(&f1, &f2, &mut l1, &mut l2, State::Out, &edge, &rf1, &rf2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn inter3d(
        f1: &Face,
        f2: &Face,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
        side: State,
        ref_edge: &Edge,
        the_ref_face1: &Face,
        the_ref_face2: &Face,
    ) -> Result<(), Failure> {
        if are_infinite_planes(f1, f2)? {
            return perform_planes(f1, f2, side, l1, l2);
        }
        build_curves3d_default(f1.as_ref())?;
        build_curves3d_default(f2.as_ref())?;
        update_vertex_tolerances(f1)?;
        update_vertex_tolerances(f2)?;
        let mut a_pf = BOPAlgoPaveFiller::new();
        let mut a_ls = List::default();
        a_ls.append(f1.as_ref().clone());
        a_ls.append(f2.as_ref().clone());
        a_pf.set_arguments(&a_ls);
        a_pf.perform();
        let ds = pave_ds(&a_pf)?;
        let mut true_edges = IndexedMapOfShape::default();
        if !ref_edge.is_null() {
            check_inters_ff(ds, ref_edge, &mut true_edges)?;
        }
        l1.clear();
        l2.clear();
        store_sections(ds, a_pf.context(), (f1, f2), side, &true_edges, (l1, l2))?;
        let (add_pcurve1, is_el1) = pcurve_flags(f1)?;
        let (add_pcurve2, is_el2) = pcurve_flags(f2)?;
        let elementary = is_el1 && is_el2;
        if l1.len() > 1 && !elementary && !the_ref_face1.is_null() {
            keep_min_angle_edge(ref_edge, (the_ref_face1, the_ref_face2), l1, l2)?;
        }
        if l1.len() > 1 && !elementary {
            let context = GlueContext {
                ds,
                f1,
                f2,
                add_pcurve1,
                add_pcurve2,
            };
            return glue_sections(&context, &true_edges, side, l1, l2);
        }
        for e in l1.iter() {
            same_parameter_shape(e, precision::confusion(), true)?;
        }
        Ok(())
    }
}

/// `(addPCurve, isEl)` of a face: no p-curves are added on planes; elementary surfaces
/// other than planes are flagged.
fn pcurve_flags(f: &Face) -> Result<(bool, bool), Failure> {
    Ok(match plane_or_elementary(f)? {
        Some(true) => (false, false),
        Some(false) => (true, true),
        None => (true, false),
    })
}

/// The face/face section edges of the pave filler (only the true edges when known),
/// given the p-curves the interference curves carry, oriented in each face.
fn store_sections(
    ds: &DataStructure,
    context: Option<&Context>,
    (f1, f2): (&Face, &Face),
    side: State,
    true_edges: &IndexedMapOfShape,
    (l1, l2): (&mut List<Shape>, &mut List<Shape>),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for (an_edge, i, j) in section_edges(ds)? {
        if !true_edges.is_empty() && !true_edges.contains(an_edge.as_ref()) {
            continue;
        }
        let bc_curve = ds
            .interf_ff_records()
            .value(i)
            .map_err(array_error)?
            .curves()
            .value(j)
            .map_err(array_error)?
            .curve()
            .clone();
        let (c3de, f, l) = BRepTool::curve(&an_edge)?;
        let trim: Handle<dyn GeomCurve> = if c3de.is_null() {
            Handle::default()
        } else {
            Handle::new(GeomTrimmedCurve::new(&c3de, f, l, true, true)?).cast()
        };
        let tol_edge = BRepTool::edge_tolerance(&an_edge)?;
        for (face, c2d) in [
            (f1, bc_curve.first_curve2d()),
            (f2, bc_curve.second_curve2d()),
        ] {
            if algo_tools2d::has_curve_on_surface_simple(&an_edge, face)? {
                continue;
            }
            let c2d = adjusted_pcurve(face, c2d, (&c3de, &trim, f, l), context)?;
            builder.update_edge_pcurve_on_face(&an_edge, &c2d, face, tol_edge)?;
        }
        append_section(&an_edge, f1, f2, side, (l1, l2))?;
    }
    Ok(())
}

/// The interference p-curve adjusted on the face to the periodic domain (by the edge
/// range for a periodic curve, by the trimmed curve range otherwise) when the edge has
/// a 3d curve.
fn adjusted_pcurve(
    face: &Face,
    c2d: &Handle<dyn Geom2dCurve>,
    (c3de, trim, f, l): (&Handle<dyn GeomCurve>, &Handle<dyn GeomCurve>, f64, f64),
    context: Option<&Context>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if trim.is_null() {
        return Ok(c2d.clone());
    }
    if live(c3de, "curve")?.is_periodic() {
        return algo_tools2d::adjust_pcurve_on_face(face, f, l, c2d, context);
    }
    let t = live(trim, "curve")?;
    algo_tools2d::adjust_pcurve_on_face(face, t.first_parameter(), t.last_parameter(), c2d, context)
}

/// For an open reference edge on a closed reference surface, keeps only the section
/// edge whose nearest point to the middle of the reference edge is in the direction
/// of its own middle (the least angle).
fn keep_min_angle_edge(
    ref_edge: &Edge,
    (rf1, rf2): (&Face, &Face),
    l1: &mut List<Shape>,
    l2: &mut List<Shape>,
) -> Result<(), Failure> {
    let (v1, v2) = topexp::vertices(ref_edge, false);
    if v1.is_same(&v2) {
        return Ok(());
    }
    let s1 = BRepTool::surface(rf1)?;
    let s2 = BRepTool::surface(rf2)?;
    let (s1, s2) = (live(&s1, "surface")?, live(&s2, "surface")?);
    if !(s1.is_u_closed()? || s1.is_v_closed()? || s2.is_u_closed()? || s2.is_v_closed()?) {
        return Ok(());
    }
    let ref_curve = BRepAdaptorCurve::with_edge(ref_edge)?;
    let ref_pnt =
        ref_curve.value((ref_curve.first_parameter()? + ref_curve.last_parameter()?) / 2.)?;
    let mut min_angle = precision::infinite();
    let mut min_angle_edge = Shape::default();
    for an_edge in l1.iter() {
        let Some(angle) = projection_angle(ref_pnt, &topods::edge(an_edge)?)? else {
            continue;
        };
        if angle < min_angle {
            min_angle = angle;
            min_angle_edge = an_edge.clone();
        }
    }
    if min_angle_edge.is_null() {
        return Ok(());
    }
    let keep: Vec<bool> = l1.iter().map(|e| e.is_same(&min_angle_edge)).collect();
    retain_flags(l1, &keep);
    retain_flags(l2, &keep);
    Ok(())
}

/// Keeps the list items whose flag is set.
fn retain_flags(list: &mut List<Shape>, keep: &[bool]) {
    let mut index = 0;
    list.retain(|_| {
        let kept = keep.get(index).copied().unwrap_or(false);
        index += 1;
        kept
    });
}

/// The angle between the vectors from the point to the middle of the edge and to its
/// nearest projection on the edge, when the extrema find one.
fn projection_angle(ref_pnt: Point3, an_edge: &Edge) -> Result<Option<f64>, Failure> {
    let ba_curve = BRepAdaptorCurve::with_edge(an_edge)?;
    let mid = ba_curve.value((ba_curve.first_parameter()? + ba_curve.last_parameter()?) / 2.)?;
    let ref_to_mid = Vector3::between(ref_pnt, mid);
    let projector = ExtPC::with_curve(ref_pnt, &ba_curve, 1.0e-10)?;
    if !projector.is_done() {
        return Ok(None);
    }
    let mut imin = 0;
    let mut min_sq_dist = precision::infinite();
    for ind in 1..=projector.nb_ext()? {
        let sq = projector.square_distance(ind)?;
        if sq < min_sq_dist {
            min_sq_dist = sq;
            imin = ind;
        }
    }
    if imin == 0 {
        return Ok(None);
    }
    let projection = *projector.point(imin)?.value();
    Ok(Some(
        Vector3::between(ref_pnt, projection).angle(ref_to_mid)?,
    ))
}

/// The faces, flags and data structure of the glueing of the section edges.
pub(super) struct GlueContext<'a> {
    pub(super) ds: &'a DataStructure,
    pub(super) f1: &'a Face,
    pub(super) f2: &'a Face,
    pub(super) add_pcurve1: bool,
    pub(super) add_pcurve2: bool,
}

/// The glueing branch of `Inter3D`: the true edges assembled (in reverse map order), or
/// each wire of connected section edges assembled; when fewer edges result, the
/// assembled edges (same parameter, the p-curves reprojected above a 1e-4 tolerance,
/// dropped above 1e-2) replace the sections.
fn glue_sections(
    context: &GlueContext,
    true_edges: &IndexedMapOfShape,
    side: State,
    l1: &mut List<Shape>,
    l2: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut eseq: Vec<Shape> = Vec::new();
    if !true_edges.is_empty() {
        let edges_for_concat: Vec<Shape> = (1..=true_edges.extent())
            .rev()
            .map(|i| true_edges.find_key(i).cloned())
            .collect::<Result<_, _>>()?;
        let assembled = assemble_edge(context, &edges_for_concat)?;
        if assembled.is_null() {
            eseq.extend(edges_for_concat);
        } else {
            eseq.push(assembled.into_shape());
        }
    } else {
        let wires = connected_wires(l1)?;
        for a_wire in &wires {
            let local = wire_edges_for_concat(a_wire, context.ds)?;
            let assembled = assemble_edge(context, &local)?;
            if assembled.is_null() {
                eseq.extend(local.into_iter().rev());
            } else {
                eseq.push(assembled.into_shape());
            }
        }
    }
    if eseq.len() >= l1.len() {
        return Ok(());
    }
    l1.clear();
    l2.clear();
    for shape in &eseq {
        let an_edge = topods::edge(shape)?;
        same_parameter_shape(an_edge.as_ref(), precision::confusion(), true)?;
        let edge_tol = BRepTool::edge_tolerance(&an_edge)?;
        if edge_tol > 1.0e-2 {
            continue;
        }
        if edge_tol >= 1.0e-4 {
            reconstruct_pcurves(&an_edge)?;
            same_parameter_shape(an_edge.as_ref(), precision::confusion(), true)?;
        }
        append_section(&an_edge, context.f1, context.f2, side, (l1, l2))?;
    }
    Ok(())
}

/// The wires of connected section edges: each edge starts a wire unless it connects
/// to an existing wire, which then grows by the connecting edge of least angle.
fn connected_wires(l1: &List<Shape>) -> Result<Vec<Wire>, Failure> {
    let builder = BRepBuilder::default();
    let mut wseq: Vec<Wire> = Vec::new();
    let mut edges: Vec<Shape> = l1.iter().cloned().collect();
    while let Some(first) = edges.first().cloned() {
        let a_wire = single_edge_wire(&builder, &first)?;
        let Some(k) = wseq.iter().position(|w| are_connex(w, &a_wire)) else {
            wseq.push(a_wire);
            edges.remove(0);
            continue;
        };
        let res_wire = wseq[k].clone();
        let mut candidates = vec![0usize];
        for (j, e) in edges.iter().enumerate().skip(1) {
            if are_connex(&res_wire, &single_edge_wire(&builder, e)?) {
                candidates.push(j);
            }
        }
        let minind = least_angle_candidate(&res_wire, &edges, &candidates)?;
        let mut grown = res_wire;
        builder.add(&mut grown, &edges[candidates[minind]])?;
        wseq[k] = grown;
        edges.remove(candidates[minind]);
    }
    Ok(wseq)
}

/// A wire of one edge.
fn single_edge_wire(builder: &BRepBuilder, e: &Shape) -> Result<Wire, Failure> {
    let mut w = Wire::default();
    builder.make_wire(&mut w);
    builder.add(&mut w, e)?;
    Ok(w)
}

/// The index of the candidate of least `AngleWireEdge` (the first when only one).
fn least_angle_candidate(
    res_wire: &Wire,
    edges: &[Shape],
    candidates: &[usize],
) -> Result<usize, Failure> {
    if candidates.len() <= 1 {
        return Ok(0);
    }
    let mut min_angle = f64::MAX;
    let mut minind = 0;
    for (j, &c) in candidates.iter().enumerate() {
        let angle = angle_wire_edge(res_wire, &topods::edge(&edges[c])?)?;
        if angle < min_angle {
            min_angle = angle;
            minind = j;
        }
    }
    Ok(minind)
}

/// The edges of a wire in concatenation order: from a non-autonomous vertex along a
/// closed wire, in wire explorer order otherwise.
fn wire_edges_for_concat(a_wire: &Wire, ds: &DataStructure) -> Result<Vec<Shape>, Failure> {
    if !a_wire.closed()? {
        let mut local = Vec::new();
        let mut wexp = WireExplorer::with_wire(a_wire)?;
        while wexp.more() {
            local.push(wexp.current().as_ref().clone());
            wexp.next()?;
        }
        return Ok(local);
    }
    let (mut start_vertex, start_edge, mut elist) = closed_wire_start(a_wire, ds)?;
    let mut local = vec![start_edge.into_shape()];
    while !elist.is_empty() {
        let Some(pos) = next_edge_from(&elist, &mut start_vertex)? else {
            break;
        };
        local.push(elist.remove(pos));
    }
    Ok(local)
}

/// The start of a closed wire: the first edge with a non-autonomous vertex (starting
/// at its other vertex), else the first edge; the remaining edges.
fn closed_wire_start(
    a_wire: &Wire,
    ds: &DataStructure,
) -> Result<(Vertex, Edge, Vec<Shape>), Failure> {
    let mut start: Option<(Vertex, Edge)> = None;
    let mut elist = Vec::new();
    let mut itw = ShapeIterator::new(a_wire.as_ref(), true, true);
    while itw.more() {
        let an_edge = topods::edge(itw.value()?)?;
        itw.next();
        if start.is_some() {
            elist.push(an_edge.into_shape());
            continue;
        }
        let (v1, v2) = topexp::vertices(&an_edge, false);
        if !is_autonom_vertex(v1.as_ref(), ds)? {
            start = Some((v2, an_edge));
        } else if !is_autonom_vertex(v2.as_ref(), ds)? {
            start = Some((v1, an_edge));
        } else {
            elist.push(an_edge.into_shape());
        }
    }
    if let Some((v, e)) = start {
        return Ok((v, e, elist));
    }
    let first = topods::edge(&elist.remove(0))?;
    let (v1, _) = topexp::vertices(&first, false);
    Ok((v1, first, elist))
}

/// The position of the first edge of the list touching the vertex, which moves to its
/// other vertex.
fn next_edge_from(elist: &[Shape], start_vertex: &mut Vertex) -> Result<Option<usize>, Failure> {
    for (pos, e) in elist.iter().enumerate() {
        let (v1, v2) = topexp::vertices(&topods::edge(e)?, false);
        if v1.is_same(start_vertex) {
            *start_vertex = v2;
            return Ok(Some(pos));
        }
        if v2.is_same(start_vertex) {
            *start_vertex = v1;
            return Ok(Some(pos));
        }
    }
    Ok(None)
}

/// Standard_Failure of a poisoned pave block lock.
fn poisoned() -> Failure {
    Failure::with_message("Standard_ProgramError", "BOPDS_PaveBlock lock poisoned")
}

/// Standard_OutOfRange of a data structure array index.
fn array_error(e: impl std::fmt::Debug) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("BOPDS interference index: {e:?}"),
    )
}
