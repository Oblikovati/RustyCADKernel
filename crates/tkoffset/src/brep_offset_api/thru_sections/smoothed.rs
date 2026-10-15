// Rust translation of BRepOffsetAPI_ThruSections::CreateSmoothed
// (ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The smoothed loft: one face per section edge, cut from the total surface of the
//! sections between the knots of its edge, bounded by iso edges.
use super::BRepOffsetAPIThruSections;
use super::solid::{make_solid, oriented_solid};
use super::total_surf::SectionEdges;
use rustycad_tkbool::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape, WireExplorer};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomCurve, GeomGeometry, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tktopalgo::brep_lib::{same_parameter_edge, same_parameter_edge_with_tolerance};

/// The section edges of the smoothed loft and their shape flags.
struct SmoothedSections {
    shapes: Vec<Shape>,
    nb_sects: usize,
    nb_edges: usize,
    w1_point: bool,
    w2_point: bool,
    u_closed: bool,
    v_closed: bool,
}

impl SmoothedSections {
    /// The view of the sections that `TotalSurf` reads.
    fn edges(&self) -> SectionEdges<'_> {
        SectionEdges {
            shapes: &self.shapes,
            nb_sects: self.nb_sects,
            nb_edges: self.nb_edges,
            w1_point: self.w1_point,
            w2_point: self.w2_point,
            v_closed: self.v_closed,
        }
    }
}

/// The edges carried from one face of the loop to the next (`edge4`, `couture`,
/// `vcouture`) and the wires of the section ends.
struct FaceLoop {
    shell: Shell,
    edge4: Edge,
    couture: Edge,
    vcouture: Vec<Edge>,
    new_w1: Wire,
    new_w2: Wire,
}

/// The vertices of a face: the first-section ends `v1f`, `v1l` and the last-section
/// ends `v2f`, `v2l`, each in the sense of its edge.
struct FaceVertices {
    v1f: Vertex,
    v1l: Vertex,
    v2f: Vertex,
    v2l: Vertex,
}

/// The parameter bounds of a face surface `(f1, l1, f2, l2)`.
type Bounds = (f64, f64, f64, f64);

impl BRepOffsetAPIThruSections {
    /// `CreateSmoothed()`: the faces of the total surface (Failed without it), the
    /// shell or solid, done, then the edges of the result made same parameter.
    pub(super) fn create_smoothed(&mut self) -> Result<(), Failure> {
        let sections = self.smoothed_sections()?;
        let surface = self.total_surf(&sections.edges())?;
        if surface.is_null() {
            self.my_status = ThruSectionErrorStatus::Failed;
            return Ok(());
        }
        let face_loop = self.smoothed_faces(&sections, &surface)?;
        if sections.u_closed && sections.w1_point && sections.w2_point {
            face_loop.shell.set_closed(true)?;
        }
        self.base.my_shape = self.smoothed_result(&sections, face_loop)?;
        self.base.set_done();
        self.same_parameter_result()
    }

    /// The punctual ends, the looping flag, the edge count and the edges of each
    /// section (the degenerated edge repeated for a punctual one).
    fn smoothed_sections(&self) -> Result<SmoothedSections, Failure> {
        let nb_sects = self.my_wires.len();
        let w1_point = is_punctual_wire(&self.my_wires[0])?;
        let w2_point = is_punctual_wire(&self.my_wires[nb_sects - 1])?;
        let v_closed = self.my_wires[0].is_same(&self.my_wires[nb_sects - 1]);
        let counted = if w1_point { 1 } else { 0 };
        let nb_edges = wire_edges(&self.my_wires[counted])?.len();
        let mut sections = SmoothedSections {
            shapes: vec![Shape::default(); nb_sects * nb_edges],
            nb_sects,
            nb_edges,
            w1_point,
            w2_point,
            u_closed: true,
            v_closed,
        };
        let mut nb = 0;
        for (i, wire) in self.my_wires.iter().enumerate() {
            sections.u_closed &= is_closed_section(wire)?;
            let punctual = (i == 0 && w1_point) || (i + 1 == nb_sects && w2_point);
            let edges = wire_edges(wire)?;
            let section: Vec<Shape> = if punctual {
                vec![edges.first().cloned().unwrap_or_default(); nb_edges]
            } else {
                edges
            };
            for edge in section {
                // The native NCollection_Array1 raises past its nbSects * nbEdges cells.
                let cell = sections.shapes.get_mut(nb).ok_or_else(|| {
                    Failure::with_message("Standard_OutOfRange", "NCollection_Array1::ChangeValue")
                })?;
                *cell = edge;
                nb += 1;
            }
        }
        Ok(sections)
    }

    /// The faces of each section edge added to a new shell.
    fn smoothed_faces(
        &mut self,
        sections: &SmoothedSections,
        total: &Handle<GeomBSplineSurface>,
    ) -> Result<FaceLoop, Failure> {
        let builder = BRepBuilder::default();
        let mut face_loop = FaceLoop {
            shell: Shell::default(),
            edge4: Edge::default(),
            couture: Edge::default(),
            vcouture: vec![Edge::default(); sections.nb_edges],
            new_w1: Wire::default(),
            new_w2: Wire::default(),
        };
        builder.make_shell(&mut face_loop.shell);
        builder.make_wire(&mut face_loop.new_w1);
        builder.make_wire(&mut face_loop.new_w2);
        for i in 1..=sections.nb_edges {
            self.smoothed_face(sections, total, i, &mut face_loop)?;
        }
        Ok(face_loop)
    }

    /// The face of section edge `i` (1-based) and its history.
    fn smoothed_face(
        &mut self,
        sections: &SmoothedSections,
        total: &Handle<GeomBSplineSurface>,
        i: usize,
        face_loop: &mut FaceLoop,
    ) -> Result<(), Failure> {
        let surface = segmented_surface(total, i)?;
        let first_edge = sections.edges().edge(i)?;
        let last_edge = sections
            .edges()
            .edge((sections.nb_sects - 1) * sections.nb_edges + i)?;
        let mut vertices = face_vertices(&first_edge, &last_edge);
        let builder = BRepBuilder::default();
        let mut face = Face::default();
        let surface_handle: Handle<dyn GeomSurface> = surface.clone().cast();
        builder.make_face_on_surface(&mut face, &surface_handle, precision::confusion())?;
        let bounds: Bounds = live_surface(&surface)?.bounds();
        let edges = face_edges(sections, &surface, i, (&mut vertices, bounds), face_loop)?;
        let wire = boundary_wire(&edges)?;
        set_pcurves(&edges, &face, sections, bounds)?;
        builder.add(&mut face, wire.as_ref())?;
        builder.add(&mut face_loop.shell, face.as_ref())?;
        builder.add(&mut face_loop.new_w1, &edges[0].reversed())?;
        builder.add(&mut face_loop.new_w2, &edges[2].reversed())?;
        self.my_edge_face
            .bind(first_edge.into_shape(), face.into_shape());
        Ok(())
    }

    /// The shell, or its solid (closed by itself for looping sections, by caps on the
    /// end wires otherwise).
    fn smoothed_result(
        &mut self,
        sections: &SmoothedSections,
        face_loop: FaceLoop,
    ) -> Result<Shape, Failure> {
        let FaceLoop {
            mut shell,
            new_w1,
            new_w2,
            ..
        } = face_loop;
        if !self.my_is_solid {
            return Ok(shell.into_shape());
        }
        if sections.v_closed {
            return Ok(oriented_solid(&shell)?.into_shape());
        }
        let caps = (&mut self.my_first, &mut self.my_last);
        Ok(make_solid(&mut shell, (&new_w1, &new_w2), self.my_pres3d, caps)?.into_shape())
    }

    /// Each edge of the result loses its SameRange and SameParameter flags and is made
    /// same parameter within its tolerance; for an immutable input the vertices whose
    /// tolerance must grow are replaced by copies.
    fn same_parameter_result(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut vertex_tolerances: DataMap<Shape, f64, ShapeMapHasher> = DataMap::default();
        let mut explorer = Explorer::new(&self.base.my_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while explorer.more() {
            let edge = topods::edge(explorer.current())?;
            builder.same_range(&edge, false)?;
            builder.same_parameter(&edge, false)?;
            let tolerance = BRepTool::edge_tolerance(&edge)?;
            if self.my_mutable_input {
                same_parameter_edge(&edge, tolerance)?;
            } else {
                record_vertex_tolerances(&edge, tolerance, &mut vertex_tolerances)?;
            }
            explorer.next()?;
        }
        if !self.my_mutable_input {
            self.base.my_shape = replaced_vertices(&self.base.my_shape, &vertex_tolerances)?;
        }
        Ok(())
    }
}

/// The edges of a wire in wire explorer order.
fn wire_edges(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut edges = Vec::new();
    let mut explorer = WireExplorer::with_wire(&topods::wire(shape)?)?;
    while explorer.more() {
        edges.push(explorer.current().as_ref().clone());
        explorer.next()?;
    }
    Ok(edges)
}

/// Whether every edge of the wire (in wire explorer order) is degenerated.
fn is_punctual_wire(shape: &Shape) -> Result<bool, Failure> {
    for edge in wire_edges(shape)? {
        if !BRepTool::degenerated(&topods::edge(&edge)?)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether a section is closed: flagged so, or with the same end vertices.
fn is_closed_section(shape: &Shape) -> Result<bool, Failure> {
    let wire = topods::wire(shape)?;
    if wire.closed()? {
        return Ok(true);
    }
    let (v1, v2) = topexp::wire_vertices(&wire);
    Ok(v1.is_same(&v2))
}

/// `PreciseUpar(anUpar, aSurface)`: the U knot of the knot interval of `u` nearest to
/// it.
fn precise_upar(u: f64, surface: &GeomBSplineSurface) -> Result<f64, Failure> {
    let (i1, i2) = surface.locate_u(u, precision::p_confusion(), false)?;
    let u1 = surface.u_knot(i1)?;
    let u2 = surface.u_knot(i2)?;
    Ok(if u - u1 < u2 - u { u1 } else { u2 })
}

/// A copy of the total surface segmented to the U span `[i - 1, i]` (pinned to its
/// knots) and its whole V span.
fn segmented_surface(
    total: &Handle<GeomBSplineSurface>,
    i: usize,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let copy: Handle<GeomBSplineSurface> = live_surface(total)?.copy()?.cast();
    let surface = live_surface(&copy)?;
    let ui1 = precise_upar((i - 1) as f64, surface)?;
    let ui2 = precise_upar(i as f64, surface)?;
    let v0 = surface.v_knot(surface.first_v_knot_index()?)?;
    let v1 = surface.v_knot(surface.last_v_knot_index()?)?;
    let tolerance = precision::p_confusion();
    surface.segment((ui1, ui2), (v0, v1), (tolerance, tolerance))?;
    Ok(copy)
}

/// The surface of a live handle.
fn live_surface(surface: &Handle<GeomBSplineSurface>) -> Result<&GeomBSplineSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepOffsetAPI_ThruSections: null surface; expected a B-spline surface",
        )
    })
}

/// The ends of an edge in its sense.
fn oriented_ends(edge: &Edge) -> (Vertex, Vertex) {
    let (first, last) = topexp::vertices(edge, false);
    if edge.orientation() == Orientation::Reversed {
        return (last, first);
    }
    (first, last)
}

/// The vertices of the face of the first and last section edges.
fn face_vertices(first_edge: &Edge, last_edge: &Edge) -> FaceVertices {
    let (v1f, v1l) = oriented_ends(first_edge);
    let (v2f, v2l) = oriented_ends(last_edge);
    FaceVertices { v1f, v1l, v2f, v2l }
}

/// Builds an edge on the curve with the two vertices and the range.
fn make_iso_edge(
    curve: &Handle<dyn GeomCurve>,
    ends: (&mut Vertex, &mut Vertex),
    range: (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, curve, precision::confusion())?;
    add_ends(&mut edge, ends, range)?;
    Ok(edge)
}

/// Adds the first vertex FORWARD and the second REVERSED (orienting both variables, as
/// natively) and sets the range of the edge.
fn add_ends(
    edge: &mut Edge,
    (first, last): (&mut Vertex, &mut Vertex),
    range: (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    first.set_orientation(Orientation::Forward);
    builder.add(edge, first.as_ref())?;
    last.set_orientation(Orientation::Reversed);
    builder.add(edge, last.as_ref())?;
    builder.range(edge, range.0, range.1, false)
}

/// An empty copy of a punctual section edge, FORWARD.
fn degenerated_copy(edge: &Edge) -> Result<Edge, Failure> {
    let mut copy = topods::edge(&edge.empty_copied()?)?;
    copy.set_orientation(Orientation::Forward);
    Ok(copy)
}

/// The four edges of the face of section edge `i`: the first-section edge, the right
/// iso, the reversed last-section edge and the reversed left iso.
fn face_edges(
    sections: &SmoothedSections,
    surface: &Handle<GeomBSplineSurface>,
    i: usize,
    (vertices, bounds): (&mut FaceVertices, Bounds),
    face_loop: &mut FaceLoop,
) -> Result<[Edge; 4], Failure> {
    let (f1, l1, f2, l2) = bounds;
    let s = live_surface(surface)?;
    let edge1 = if sections.w1_point {
        let mut edge = degenerated_copy(&sections.edges().edge(1)?)?;
        add_ends(&mut edge, (&mut vertices.v1f, &mut vertices.v1l), (f1, l1))?;
        edge
    } else {
        make_iso_edge(
            &s.v_iso(f2)?,
            (&mut vertices.v1f, &mut vertices.v1l),
            (f1, l1),
        )?
    };
    if sections.v_closed {
        face_loop.vcouture[i - 1] = edge1.clone();
    }
    let mut edge2 = last_section_edge(sections, s, i, (vertices, bounds), face_loop)?;
    edge2.reverse();
    let mut edge3 = if i == 1 {
        let edge = make_iso_edge(
            &s.u_iso(f1)?,
            (&mut vertices.v1f, &mut vertices.v2f),
            (f2, l2),
        )?;
        if sections.u_closed {
            face_loop.couture = edge.clone();
        }
        edge
    } else {
        face_loop.edge4.clone()
    };
    edge3.reverse();
    let edge4 = if sections.u_closed && i == sections.nb_edges {
        face_loop.couture.clone()
    } else {
        make_iso_edge(
            &s.u_iso(l1)?,
            (&mut vertices.v1l, &mut vertices.v2l),
            (f2, l2),
        )?
    };
    face_loop.edge4 = edge4.clone();
    Ok([edge1, edge4, edge2, edge3])
}

/// The last-section edge of the face (unreversed): the stored first-section edge of
/// looping sections, a copy of a punctual one, or the V iso at the end.
fn last_section_edge(
    sections: &SmoothedSections,
    s: &GeomBSplineSurface,
    i: usize,
    (vertices, bounds): (&mut FaceVertices, Bounds),
    face_loop: &FaceLoop,
) -> Result<Edge, Failure> {
    let (f1, l1, _, l2) = bounds;
    if sections.v_closed {
        return Ok(face_loop.vcouture[i - 1].clone());
    }
    if sections.w2_point {
        let mut edge = degenerated_copy(
            &sections
                .edges()
                .edge(sections.nb_sects * sections.nb_edges)?,
        )?;
        add_ends(&mut edge, (&mut vertices.v2f, &mut vertices.v2l), (f1, l1))?;
        return Ok(edge);
    }
    make_iso_edge(
        &s.v_iso(l2)?,
        (&mut vertices.v2f, &mut vertices.v2l),
        (f1, l1),
    )
}

/// The wire of the four edges in the order edge1, edge4, edge2, edge3.
fn boundary_wire(edges: &[Edge; 4]) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge.as_ref())?;
    }
    Ok(wire)
}

/// A 2D line through the point along X (`along_u`) or Y.
fn line2d(x: f64, y: f64, along_u: bool) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let direction = if along_u {
        Direction2::new(1., 0.)?
    } else {
        Direction2::new(0., 1.)?
    };
    Ok(Handle::new(Geom2dLine::from_point(Point2::new(x, y), direction)).cast())
}

/// The p-curves of the four edges on the face: V isos for the section edges (a seam
/// pair for looping sections) and U isos for the side edges (a seam pair for one
/// closed edge per section).
fn set_pcurves(
    edges: &[Edge; 4],
    face: &Face,
    sections: &SmoothedSections,
    bounds: Bounds,
) -> Result<(), Failure> {
    let (f1, l1, f2, l2) = bounds;
    let [edge1, edge4, edge2, edge3] = edges;
    let builder = BRepBuilder::default();
    let tol = precision::confusion();
    if sections.v_closed {
        let pair = (&line2d(0., f2, true)?, &line2d(0., l2, true)?);
        builder.update_edge_pcurves_on_face(edge1, pair, face, tol)?;
        builder.range_on_face(edge1, face, f1, l1)?;
    } else {
        builder.update_edge_pcurve_on_face(edge1, &line2d(0., f2, true)?, face, tol)?;
        builder.range_on_face(edge1, face, f1, l1)?;
        builder.update_edge_pcurve_on_face(edge2, &line2d(0., l2, true)?, face, tol)?;
        builder.range_on_face(edge2, face, f1, l1)?;
    }
    if sections.u_closed && sections.nb_edges == 1 {
        let pair = (&line2d(l1, 0., false)?, &line2d(f1, 0., false)?);
        builder.update_edge_pcurves_on_face(edge3, pair, face, tol)?;
        return builder.range_on_face(edge3, face, f2, l2);
    }
    builder.update_edge_pcurve_on_face(edge3, &line2d(f1, 0., false)?, face, tol)?;
    builder.range_on_face(edge3, face, f2, l2)?;
    builder.update_edge_pcurve_on_face(edge4, &line2d(l1, 0., false)?, face, tol)?;
    builder.range_on_face(edge4, face, f2, l2)
}

/// The immutable-input branch of the same-parameter loop: the edge is made same
/// parameter in place and its vertices record the greatest new tolerance.
fn record_vertex_tolerances(
    edge: &Edge,
    tolerance: f64,
    vertex_tolerances: &mut DataMap<Shape, f64, ShapeMapHasher>,
) -> Result<(), Failure> {
    let mut new_tolerance = -1.;
    same_parameter_edge_with_tolerance(edge, tolerance, &mut new_tolerance, true)?;
    if new_tolerance <= 0. {
        return Ok(());
    }
    let (v1, v2) = topexp::vertices(edge, false);
    for vertex in [v1, v2] {
        if vertex.is_null() {
            continue;
        }
        let grows = vertex_tolerances
            .find(vertex.as_ref())
            .map_or(true, |old| *old < new_tolerance);
        if grows {
            vertex_tolerances.bind(vertex.into_shape(), new_tolerance);
        }
    }
    Ok(())
}

/// The shape with each vertex whose tolerance is below its recorded one replaced by an
/// empty copy of that tolerance.
fn replaced_vertices(
    shape: &Shape,
    vertex_tolerances: &DataMap<Shape, f64, ShapeMapHasher>,
) -> Result<Shape, Failure> {
    let reshaper = BRepToolsReShape::new();
    let builder = BRepBuilder::default();
    for (key, new_tolerance) in vertex_tolerances.items() {
        let vertex = topods::vertex(key)?;
        if BRepTool::vertex_tolerance(&vertex)? < *new_tolerance {
            let new_vertex = topods::vertex(&vertex.empty_copied()?)?;
            builder.update_vertex_tolerance(&new_vertex, *new_tolerance)?;
            reshaper.replace(vertex.as_ref(), new_vertex.as_ref())?;
        }
    }
    reshaper.apply(shape, ShapeEnum::Shape)
}
