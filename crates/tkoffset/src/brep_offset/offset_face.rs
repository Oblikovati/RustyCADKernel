//! Regular face-boundary branch of `BRepOffset_Offset::Init`.
mod created_images;

use super::{BRepOffsetStatus, surface};
pub(super) use created_images::FaceCreatedImages;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool, EdgePCurve},
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

struct FaceBoundaryCopy<'a> {
    source: &'a Face,
    target: &'a Face,
    surface: &'a Handle<dyn GeomSurface>,
    created: &'a mut FaceCreatedImages,
    edge_images: EdgeImages,
    vertex_images: VertexImages,
}

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
        Self::from_face_with_created(face, offset, &mut FaceCreatedImages::default())
    }

    pub(super) fn from_face_with_created(
        face: &Face,
        offset: f64,
        created: &mut FaceCreatedImages,
    ) -> Result<Self, Failure> {
        let initial = make_forward(face);
        let (basis, _location) = untrimmed_surface(&initial)?;
        let signed_offset = face_offset(face, offset);
        let (surface, status) = surface(&basis, signed_offset, false)?;
        let mut result = make_offset_face(&initial, &surface)?;
        let (edge_images, vertex_images) =
            copy_face_boundaries(&initial, &mut result, &surface, created)?;
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
    created: &mut FaceCreatedImages,
) -> Result<(EdgeImages, VertexImages), Failure> {
    let target_view = target.clone();
    let mut copy = FaceBoundaryCopy {
        source,
        target: &target_view,
        surface,
        created,
        edge_images: Vec::new(),
        vertex_images: Vec::new(),
    };
    let mut wires = Explorer::new(
        &source.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    while wires.more() {
        let wire = Wire::unchecked(wires.current().clone());
        let wire_image = copy.copy_wire(&wire)?;
        BRepBuilder::default().add(target, &wire_image.oriented(wire.orientation()))?;
        wires.next()?;
    }
    Ok((copy.edge_images, copy.vertex_images))
}

impl FaceBoundaryCopy<'_> {
    fn copy_wire(&mut self, wire: &Wire) -> Result<Wire, Failure> {
        let mut image = Wire::default();
        BRepBuilder::default().make_wire(&mut image);
        self.copy_wire_edges(wire, &mut image)?;
        Ok(image)
    }

    fn copy_wire_edges(&mut self, wire: &Wire, image: &mut Wire) -> Result<(), Failure> {
        let mut edges = Explorer::new(
            &wire.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )?;
        while edges.more() {
            self.append_offset_edge(image, &Edge::unchecked(edges.current().clone()))?;
            edges.next()?;
        }
        Ok(())
    }

    fn append_offset_edge(&mut self, wire: &mut Wire, edge: &Edge) -> Result<(), Failure> {
        let image = self.copy_edge(edge)?;
        BRepBuilder::default().add(wire, &image.oriented(edge.orientation()))
    }

    fn copy_edge(&mut self, edge: &Edge) -> Result<Edge, Failure> {
        let was_created = self.created.has_edge(edge);
        let image = make_edge_image(self, edge)?;
        if !was_created {
            build_image_curve(&image, edge)?;
            self.created.bind_edge(edge, &image);
        }
        self.edge_images.push((edge.clone(), image.clone()));
        Ok(image)
    }
}

fn make_edge_image(copy: &mut FaceBoundaryCopy<'_>, edge: &Edge) -> Result<Edge, Failure> {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    if let Some(image) = copy.created.edge_image(&forward) {
        record_edge_vertices(copy, &forward)?;
        update_existing_edge_pcurves(copy.source, copy.target, &forward, &image)?;
        return Ok(image);
    }
    let (first, last) = edge_vertices(&forward, false);
    let curve = BRepTool::curve_on_face(&forward, copy.source)?;
    if !curve.is_stored {
        return Err(not_implemented("offset face edge has no stored p-curve"));
    }
    let first_image = image_vertex(copy, &forward, &first)?;
    let last_image = image_vertex(copy, &forward, &last)?;
    let second_curve = seam_pcurve(copy.source, &forward)?;
    make_offset_edge(
        copy.target,
        &forward,
        &first_image,
        &last_image,
        &curve,
        second_curve.as_ref(),
    )
}

fn seam_pcurve(source: &Face, edge: &Edge) -> Result<Option<EdgePCurve>, Failure> {
    if !BRepTool::is_closed_on_face(edge, source)? {
        return Ok(None);
    }
    let reversed = Edge::unchecked(edge.clone().reversed());
    let second = BRepTool::curve_on_face(&reversed, source)?;
    if !second.is_stored {
        return Err(not_implemented("seam edge has no second stored p-curve"));
    }
    Ok(Some(second))
}

fn record_edge_vertices(copy: &mut FaceBoundaryCopy<'_>, edge: &Edge) -> Result<(), Failure> {
    let (first, last) = edge_vertices(edge, false);
    let _ = image_vertex(copy, edge, &first)?;
    let _ = image_vertex(copy, edge, &last)?;
    Ok(())
}

fn update_existing_edge_pcurves(
    source: &Face,
    target: &Face,
    edge: &Edge,
    image: &Edge,
) -> Result<(), Failure> {
    let first = BRepTool::curve_on_face(edge, source)?;
    let second = seam_pcurve(source, edge)?;
    update_image_pcurves(
        &BRepBuilder::default(),
        image,
        edge,
        target,
        &first,
        second.as_ref(),
    )?;
    BRepBuilder::default().range_on_face(image, target, first.first, first.last)
}

fn image_vertex(
    copy: &mut FaceBoundaryCopy<'_>,
    edge: &Edge,
    vertex: &Vertex,
) -> Result<Vertex, Failure> {
    if let Some((_, image)) = copy
        .vertex_images
        .iter()
        .find(|(old, _)| old.is_same(vertex))
    {
        return Ok(image.clone());
    }
    if let Some(image) = copy.created.vertex_image(vertex) {
        copy.vertex_images.push((vertex.clone(), image.clone()));
        return Ok(image);
    }
    let image = make_image_vertex(copy.source, edge, vertex, copy.surface)?;
    copy.vertex_images.push((vertex.clone(), image.clone()));
    copy.created.bind_vertex(vertex, &image);
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
    first_curve: &EdgePCurve,
    second_curve: Option<&EdgePCurve>,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let (first, last) = orient_image_vertices(source, first, last);
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.add(&mut edge, &first)?;
    builder.add(&mut edge, &last)?;
    update_image_pcurves(&builder, &edge, source, target, first_curve, second_curve)?;
    builder.range_on_face(&edge, target, first_curve.first, first_curve.last)?;
    builder.degenerated(&edge, BRepTool::degenerated(source)?)?;
    Ok(edge)
}

fn update_image_pcurves(
    builder: &BRepBuilder,
    edge: &Edge,
    source: &Edge,
    target: &Face,
    first: &EdgePCurve,
    second: Option<&EdgePCurve>,
) -> Result<(), Failure> {
    let tolerance = BRepTool::edge_tolerance(source)?;
    if let Some(second) = second {
        return builder.update_edge_pcurves_on_face(
            edge,
            (&first.pcurve, &second.pcurve),
            target,
            tolerance,
        );
    }
    builder.update_edge_pcurve_on_face(edge, &first.pcurve, target, tolerance)
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
