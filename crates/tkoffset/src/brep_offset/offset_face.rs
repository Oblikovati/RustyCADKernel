//! Regular face-boundary branch of `BRepOffset_Offset::Init`.
use super::{BRepOffsetStatus, surface};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::update_face,
    topexp::{Explorer, vertices as edge_vertices},
    topods::{Edge, Face, Shape, Vertex, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::{GeomRectangularTrimmedSurface, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::toploc::Location;

type EdgeImages = Vec<(Edge, Edge)>;
type VertexImages = Vec<(Vertex, Vertex)>;

/// Result of the regular face branch of `BRepOffset_Offset`; `BRepOffsetOffsetFace::from_face(&face, 2.0)?`.
pub struct BRepOffsetOffsetFace {
    initial: Face,
    offset: Face,
    status: BRepOffsetStatus,
    edge_images: EdgeImages,
    vertex_images: VertexImages,
}

impl BRepOffsetOffsetFace {
    /// Builds the regular face image using OCCT's default outside/Arc options.
    /// Seam edges and non-identity face locations are explicit unsupported cases;
    /// `BRepOffsetOffsetFace::from_face(&face, 2.0)?`.
    pub fn from_face(face: &Face, offset: f64) -> Result<Self, Failure> {
        let initial = make_forward(face);
        reject_unsupported_boundary(&initial)?;
        let (basis, _location) = untrimmed_surface(&initial)?;
        let signed_offset = face_offset(face, offset);
        let (surface, status) = surface(&basis, signed_offset, false)?;
        let mut result = make_offset_face(&initial, &surface)?;
        let (edge_images, vertex_images) = copy_face_boundaries(&initial, &mut result, &surface)?;
        result.set_orientation(face.orientation());
        update_face(&result)?;
        Ok(Self {
            initial: face.clone(),
            offset: result,
            status,
            edge_images,
            vertex_images,
        })
    }

    /// Returns the constructed offset face (`Face`); `offset.face()`.
    pub fn face(&self) -> &Face {
        &self.offset
    }

    pub(super) fn edge_images(&self) -> &EdgeImages {
        &self.edge_images
    }

    /// Returns the status from `BRepOffset::Surface`; `offset.status()`.
    pub const fn status(&self) -> BRepOffsetStatus {
        self.status
    }

    /// Returns the image of a source edge, or a null shape when no image exists.
    /// `offset.generated(&edge.clone().into_shape())`.
    pub fn generated(&self, source: &Shape) -> Shape {
        let Some(image) = self.image_for(source) else {
            return Shape::default();
        };
        if self.initial.orientation() == Orientation::Reversed {
            return image.reversed();
        }
        image
    }

    fn image_for(&self, source: &Shape) -> Option<Shape> {
        self.edge_images
            .iter()
            .find(|(edge, _)| edge.is_same(source))
            .map(|(_, image)| image.clone().into_shape())
            .or_else(|| {
                self.vertex_images
                    .iter()
                    .find(|(vertex, _)| vertex.is_same(source))
                    .map(|(_, image)| image.clone().into_shape())
            })
    }
}

fn make_forward(face: &Face) -> Face {
    Face::unchecked(face.oriented(Orientation::Forward))
}

fn reject_unsupported_boundary(face: &Face) -> Result<(), Failure> {
    let mut wires = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    while wires.more() {
        let wire = Wire::unchecked(wires.current().clone());
        reject_wire_edges(face, &wire)?;
        wires.next()?;
    }
    Ok(())
}

fn reject_wire_edges(face: &Face, wire: &Wire) -> Result<(), Failure> {
    let mut edges = Explorer::new(
        &wire.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let edge = Edge::unchecked(edges.current().clone());
        if BRepTool::is_closed_on_face(&edge, face)? {
            return Err(not_implemented("offset face has a seam edge"));
        }
        edges.next()?;
    }
    Ok(())
}

fn untrimmed_surface(face: &Face) -> Result<(Handle<dyn GeomSurface>, Location), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if !location.is_identity() {
        return Err(not_implemented("offset face has a non-identity location"));
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    let Some(trimmed) = trimmed.get() else {
        return Ok((surface, location));
    };
    Ok((trimmed.basis_surface(), location))
}

fn face_offset(face: &Face, offset: f64) -> f64 {
    if face.orientation() == Orientation::Reversed {
        -offset
    } else {
        offset
    }
}

fn make_offset_face(source: &Face, surface: &Handle<dyn GeomSurface>) -> Result<Face, Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(
        &mut face,
        surface,
        BRepTool::face_tolerance(source)?,
    )?;
    Ok(face)
}

fn copy_face_boundaries(
    source: &Face,
    target: &mut Face,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(EdgeImages, VertexImages), Failure> {
    let mut edge_images = Vec::new();
    let mut vertex_images = Vec::new();
    let mut wires = Explorer::new(
        &source.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    while wires.more() {
        let wire = Wire::unchecked(wires.current().clone());
        let wire_image = copy_wire(
            source,
            target,
            &wire,
            surface,
            &mut edge_images,
            &mut vertex_images,
        )?;
        BRepBuilder::default().add(target, &wire_image.oriented(wire.orientation()))?;
        wires.next()?;
    }
    Ok((edge_images, vertex_images))
}

fn copy_wire(
    source: &Face,
    target: &Face,
    wire: &Wire,
    surface: &Handle<dyn GeomSurface>,
    edge_images: &mut EdgeImages,
    vertex_images: &mut VertexImages,
) -> Result<Wire, Failure> {
    let mut image = Wire::default();
    BRepBuilder::default().make_wire(&mut image);
    copy_wire_edges(
        source,
        target,
        wire,
        surface,
        edge_images,
        vertex_images,
        &mut image,
    )?;
    Ok(image)
}

fn copy_wire_edges(
    source: &Face,
    target: &Face,
    wire: &Wire,
    surface: &Handle<dyn GeomSurface>,
    edge_images: &mut EdgeImages,
    vertex_images: &mut VertexImages,
    image: &mut Wire,
) -> Result<(), Failure> {
    let mut edges = Explorer::new(
        &wire.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let source_edge = Edge::unchecked(edges.current().clone());
        append_offset_edge(
            source,
            target,
            surface,
            edge_images,
            vertex_images,
            image,
            &source_edge,
        )?;
        edges.next()?;
    }
    Ok(())
}

fn append_offset_edge(
    source: &Face,
    target: &Face,
    surface: &Handle<dyn GeomSurface>,
    edge_images: &mut EdgeImages,
    vertex_images: &mut VertexImages,
    wire: &mut Wire,
    edge: &Edge,
) -> Result<(), Failure> {
    let image = copy_edge(source, target, edge, surface, edge_images, vertex_images)?;
    BRepBuilder::default().add(wire, &image.oriented(edge.orientation()))
}

fn copy_edge(
    source: &Face,
    target: &Face,
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    edge_images: &mut Vec<(Edge, Edge)>,
    vertex_images: &mut Vec<(Vertex, Vertex)>,
) -> Result<Edge, Failure> {
    let image = make_edge_image(source, target, edge, surface, vertex_images)?;
    build_image_curve(&image, edge)?;
    edge_images.push((edge.clone(), image.clone()));
    Ok(image)
}

fn make_edge_image(
    source: &Face,
    target: &Face,
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    vertices: &mut Vec<(Vertex, Vertex)>,
) -> Result<Edge, Failure> {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let (first, last) = edge_vertices(&forward, false);
    let curve = BRepTool::curve_on_face(&forward, source)?;
    if !curve.is_stored {
        return Err(not_implemented("offset face edge has no stored p-curve"));
    }
    let first_image = image_vertex(source, &forward, &first, surface, vertices)?;
    let last_image = image_vertex(source, &forward, &last, surface, vertices)?;
    make_offset_edge(
        target,
        &forward,
        &first_image,
        &last_image,
        &curve.pcurve,
        curve.first,
        curve.last,
    )
}

fn image_vertex(
    source: &Face,
    edge: &Edge,
    vertex: &Vertex,
    surface: &Handle<dyn GeomSurface>,
    images: &mut Vec<(Vertex, Vertex)>,
) -> Result<Vertex, Failure> {
    if let Some((_, image)) = images.iter().find(|(old, _)| old.is_same(vertex)) {
        return Ok(image.clone());
    }
    let image = make_image_vertex(source, edge, vertex, surface)?;
    images.push((vertex.clone(), image.clone()));
    Ok(image)
}

fn make_image_vertex(
    source: &Face,
    edge: &Edge,
    vertex: &Vertex,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Vertex, Failure> {
    let parameter = BRepTool::parameter_on_face(vertex, edge, source)?;
    let curve = BRepTool::curve_on_face(edge, source)?;
    let uv = curve.pcurve.value(parameter)?;
    let point = surface.value(uv.x(), uv.y())?;
    let mut image = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut image,
        point,
        BRepTool::vertex_tolerance(vertex)?,
    )?;
    Ok(image)
}

fn make_offset_edge(
    target: &Face,
    source: &Edge,
    first: &Vertex,
    last: &Vertex,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    start: f64,
    end: f64,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let (first, last) = orient_image_vertices(source, first, last);
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.add(&mut edge, &first)?;
    builder.add(&mut edge, &last)?;
    builder.update_edge_pcurve_on_face(&edge, pcurve, target, BRepTool::edge_tolerance(source)?)?;
    builder.range_on_face(&edge, target, start, end)?;
    builder.degenerated(&edge, BRepTool::degenerated(source)?)?;
    Ok(edge)
}

fn orient_image_vertices(source: &Edge, first: &Vertex, last: &Vertex) -> (Shape, Shape) {
    let (source_first, source_last) = edge_vertices(source, false);
    // `BRepOffset_Offset::Init(Face)` attaches generated vertices with source orientations.
    (
        first.oriented(source_first.orientation()),
        last.oriented(source_last.orientation()),
    )
}

fn build_image_curve(image: &Edge, source: &Edge) -> Result<(), Failure> {
    if !BRepTool::degenerated(source)? {
        let _ = rustycad_tktopalgo::brep_lib::build_curves3d_default(&image.clone().into_shape())?;
    }
    Ok(())
}

fn not_implemented(detail: &str) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        detail,
        "BRepOffset_Offset::Init(Face)",
    )
}
