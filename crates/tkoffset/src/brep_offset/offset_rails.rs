//! Edge-rail branch of `BRepOffset_Offset::Init`.
use super::offset_rails_geometry::{
    assemble_pipe_face, construction_error, install_rail_pcurves, make_connectors, unsupported_edge,
};
use super::sphere_join::build_vertex_arc_join;
use super::{BRepOffsetOffsetFace, BRepOffsetStatus};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::update_face,
    topexp::{Explorer, vertices},
    topods::{Edge, Face, Shape, Vertex, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomCurve, GeomSurface},
    geom_adaptor::GeomAdaptorCurve,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkgeomalgo::geom_fill::GeomFillPipe;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

/// The edge-rail branch of `BRepOffset_Offset`, retaining the source path edge.
/// This port currently accepts identity-located edges with 3D curves.
pub struct BRepOffsetOffset {
    initial: Shape,
    face: Face,
    status: BRepOffsetStatus,
    connectors: Vec<Edge>,
    wire_reversed: bool,
    face_image: Option<BRepOffsetOffsetFace>,
    vertex_mode: bool,
    tube_mode: bool,
}

pub(super) struct PipeFaceOptions<'a> {
    pub(super) surface: &'a Handle<dyn GeomSurface>,
    pub(super) exchange_uv: bool,
    pub(super) error: f64,
    pub(super) tolerance: f64,
}

impl Default for BRepOffsetOffset {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepOffsetOffset {
    /// Creates an uninitialized offset object; `BRepOffsetOffset::new()`.
    pub fn new() -> Self {
        Self {
            initial: Shape::default(),
            face: Face::default(),
            status: BRepOffsetStatus::Good,
            connectors: Vec::new(),
            wire_reversed: false,
            face_image: None,
            vertex_mode: false,
            tube_mode: false,
        }
    }

    /// Builds the face between two 3D edge rails with the native pipe construction.
    /// `BRepOffsetOffset::from_edges(&path, &edge1, &edge2, 0.5, false, 1e-4, Continuity::C1)?`.
    pub fn from_edges(
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        polynomial: bool,
        tolerance: f64,
        continuity: Continuity,
    ) -> Result<Self, Failure> {
        let mut result = Self::new();
        result.init_edges(
            path, edge1, edge2, offset, polynomial, tolerance, continuity,
        )?;
        Ok(result)
    }

    /// Builds the default outside/arc image for a source face.
    /// `BRepOffsetOffset::from_face(&face, 0.5)?`.
    pub fn from_face(face: &Face, offset: f64) -> Result<Self, Failure> {
        let image = BRepOffsetOffsetFace::from_face(face, offset)?;
        Ok(Self {
            initial: face.clone().into_shape(),
            face: image.face().clone(),
            status: image.status(),
            connectors: Vec::new(),
            wire_reversed: false,
            face_image: Some(image),
            vertex_mode: false,
            tube_mode: false,
        })
    }

    /// Builds the default spherical join for a vertex and its offset edges.
    /// `BRepOffsetOffset::from_vertex(&vertex, &edges, 2.0)?`.
    pub fn from_vertex(vertex: &Vertex, edges: &[Edge], offset: f64) -> Result<Self, Failure> {
        let face = build_vertex_arc_join(vertex, edges, offset)?;
        Ok(Self {
            initial: vertex.clone().into_shape(),
            face,
            status: BRepOffsetStatus::Good,
            connectors: Vec::new(),
            wire_reversed: false,
            face_image: None,
            vertex_mode: true,
            tube_mode: false,
        })
    }

    /// Builds the free-boundary tube around one identity-located 3D edge.
    /// `let mut offset = BRepOffsetOffset::new(); offset.init_edge_tube(&edge, 1.0)?`.
    pub fn init_edge_tube(&mut self, edge: &Edge, offset: f64) -> Result<(), Failure> {
        self.initial = edge.clone().into_shape();
        self.face_image = None;
        self.vertex_mode = false;
        if !edge.location().is_identity() {
            return Err(unsupported_edge("Edge"));
        }
        let curve = trimmed_curve(edge)?;
        let surface = edge_tube_surface(&curve, offset.abs())?;
        let face = edge_tube_face(&surface, offset)?;
        self.face = face;
        self.connectors.clear();
        self.wire_reversed = false;
        self.tube_mode = true;
        Ok(())
    }

    /// Reinitializes the pipe face, mutating its rail edges' face p-curves like OCCT.
    /// `offset.init_edges(&path, &edge1, &edge2, 0.5, false, 1e-4, Continuity::C1)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn init_edges(
        &mut self,
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        polynomial: bool,
        tolerance: f64,
        continuity: Continuity,
    ) -> Result<(), Failure> {
        self.initial = path.clone().into_shape();
        self.status = BRepOffsetStatus::Good;
        self.face_image = None;
        self.vertex_mode = false;
        self.tube_mode = false;
        validate_edges(path, edge1, edge2)?;
        let pipe = build_ready_rail_pipe(
            path, edge1, edge2, offset, tolerance, polynomial, continuity,
        )?;
        let surface = pipe.surface();
        self.install_result(
            path,
            [edge1, edge2],
            pipe_face_options(&pipe, &surface, tolerance),
        )
    }

    /// Returns the initialized face; `offset.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }

    /// Returns the edge used as the pipe path; `offset.initial_shape()`.
    pub fn initial_shape(&self) -> &Shape {
        &self.initial
    }

    /// Returns the native offset status; `offset.status()`.
    pub const fn status(&self) -> BRepOffsetStatus {
        self.status
    }

    /// Returns a generated connector edge for a path endpoint, or a null shape.
    /// `offset.generated(&vertex.clone().into_shape())`.
    pub fn generated(&self, source: &Shape) -> Shape {
        if self.vertex_mode {
            return Shape::default();
        }
        if let Some(image) = &self.face_image {
            return image.generated(source);
        }
        if self.tube_mode {
            return generated_tube_edge(&self.face, &self.initial, source);
        }
        let (_, last) = vertices(&Edge::unchecked(self.initial.clone()), false);
        let index = usize::from(last.is_same(source));
        let Some(edge) = self.connectors.get(index) else {
            return Shape::default();
        };
        let mut image = edge.clone().into_shape();
        if self.wire_reversed {
            image = image.reversed();
        }
        if self.face.orientation() == Orientation::Reversed {
            image = image.reversed();
        }
        image
    }

    fn install_result(
        &mut self,
        path: &Edge,
        rails: [&Edge; 2],
        options: PipeFaceOptions<'_>,
    ) -> Result<(), Failure> {
        let (face, connectors) = build_pipe_face(path, rails, &options)?;
        self.save_result(face, connectors, options.exchange_uv);
        Ok(())
    }

    fn save_result(&mut self, face: Face, connectors: Vec<Edge>, wire_reversed: bool) {
        self.face = face;
        self.connectors = connectors;
        self.wire_reversed = wire_reversed;
    }
}

fn pipe_face_options<'a>(
    pipe: &GeomFillPipe,
    surface: &'a Handle<dyn GeomSurface>,
    tolerance: f64,
) -> PipeFaceOptions<'a> {
    PipeFaceOptions {
        surface,
        exchange_uv: pipe.exchange_uv(),
        error: pipe.error_on_surf(),
        tolerance,
    }
}

fn build_pipe_face(
    path: &Edge,
    rails: [&Edge; 2],
    options: &PipeFaceOptions<'_>,
) -> Result<(Face, Vec<Edge>), Failure> {
    let mut face = new_pipe_face(path, options.surface)?;
    let builder = BRepBuilder::default();
    let connectors = build_pipe_boundaries(&builder, &face, path, rails, options)?;
    assemble_pipe_face(&builder, &mut face, rails, &connectors, options.exchange_uv)?;
    update_face(&face)?;
    apply_source_orientation(&mut face, rails[0]);
    Ok((face, connectors))
}

fn build_pipe_boundaries(
    builder: &BRepBuilder,
    face: &Face,
    path: &Edge,
    rails: [&Edge; 2],
    options: &PipeFaceOptions<'_>,
) -> Result<Vec<Edge>, Failure> {
    let bounds = options.surface.bounds();
    let path_tolerance = BRepTool::edge_tolerance(path)?;
    install_rail_pcurves(builder, face, rails, options, path_tolerance)?;
    make_connectors(
        options.surface,
        rails,
        options.exchange_uv,
        bounds,
        path_tolerance,
    )
}

fn new_pipe_face(path: &Edge, surface: &Handle<dyn GeomSurface>) -> Result<Face, Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(
        &mut face,
        surface,
        BRepTool::edge_tolerance(path)?,
    )?;
    Ok(face)
}

fn apply_source_orientation(face: &mut Face, edge1: &Edge) {
    if edge1.orientation() == Orientation::Reversed {
        face.reverse();
    }
}

fn validate_edges(path: &Edge, edge1: &Edge, edge2: &Edge) -> Result<(), Failure> {
    for (name, edge) in [("Path", path), ("Edge1", edge1), ("Edge2", edge2)] {
        if !edge.location().is_identity() || BRepTool::curve_with_location(edge)?.curve.is_null() {
            return Err(unsupported_edge(name));
        }
    }
    Ok(())
}

fn build_rail_pipe(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
    offset: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let curves = rail_curves(path, edge1, edge2)?;
    let adaptors = rail_adaptors(&curves)?;
    perform_rail_pipe(&adaptors, offset.abs(), tolerance, polynomial, continuity)
}

fn build_ready_rail_pipe(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
    offset: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let pipe = build_rail_pipe(
        path, edge1, edge2, offset, tolerance, polynomial, continuity,
    )?;
    if pipe.is_done() {
        return Ok(pipe);
    }
    Err(construction_error("GeomFill_Pipe: Cannot make a surface"))
}

fn rail_curves(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
) -> Result<[Handle<dyn GeomCurve>; 3], Failure> {
    Ok([
        trimmed_curve(path)?,
        trimmed_curve(edge1)?,
        trimmed_curve(edge2)?,
    ])
}

fn rail_adaptors(
    curves: &[Handle<dyn GeomCurve>; 3],
) -> Result<[Handle<dyn Adaptor3dCurve>; 3], Failure> {
    Ok([
        adaptor(&curves[0])?,
        adaptor(&curves[1])?,
        adaptor(&curves[2])?,
    ])
}

fn perform_rail_pipe(
    adaptors: &[Handle<dyn Adaptor3dCurve>; 3],
    radius: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let mut pipe =
        GeomFillPipe::with_adaptor_rails(&adaptors[0], &adaptors[1], &adaptors[2], radius);
    pipe.perform(tolerance, polynomial, continuity, 11, 30)?;
    Ok(pipe)
}

fn trimmed_curve(edge: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let trimmed = rustycad_tkg3d::geom::GeomTrimmedCurve::new(&curve, first, last, true, true)?;
    Ok(Handle::new(trimmed).cast())
}

fn adaptor(curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    Ok(Handle::new(GeomAdaptorCurve::with_curve(curve)?).cast())
}

fn generated_tube_edge(face: &Face, initial: &Shape, source: &Shape) -> Shape {
    let (_, last) = vertices(&Edge::unchecked(initial.clone()), false);
    let index = if last.is_same(source) { 2 } else { 3 };
    let Ok(wires) = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    ) else {
        return Shape::default();
    };
    if !wires.more() {
        return Shape::default();
    }
    let wire_shape = wires.current().clone();
    tube_edge_at(&Wire::unchecked(wire_shape), face, index)
}

fn tube_edge_at(wire: &Wire, face: &Face, index: usize) -> Shape {
    let Ok(mut edges) = Explorer::new(
        &wire.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    ) else {
        return Shape::default();
    };
    let Some(edge) = explorer_edge(&mut edges, index) else {
        return Shape::default();
    };
    orient_generated_tube_edge(Edge::unchecked(edge), wire, face).into_shape()
}

fn explorer_edge(explorer: &mut Explorer, index: usize) -> Option<Shape> {
    for current in 0..=index {
        if !explorer.more() {
            return None;
        }
        let edge = explorer.current().clone();
        if current == index {
            return Some(edge);
        }
        explorer.next().ok()?;
    }
    None
}

fn orient_generated_tube_edge(mut edge: Edge, wire: &Wire, face: &Face) -> Edge {
    if wire.orientation() == Orientation::Reversed {
        edge.reverse();
    }
    if face.orientation() == Orientation::Reversed {
        edge.reverse();
    }
    edge
}

fn edge_tube_surface(
    curve: &Handle<dyn GeomCurve>,
    radius: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut pipe = GeomFillPipe::with_radius(curve, radius)?;
    pipe.perform(1.0e-4, false, Continuity::C1, 11, 30)?;
    if !pipe.is_done() {
        return Err(construction_error("GeomFill_Pipe : Cannot make a surface"));
    }
    Ok(pipe.surface())
}

fn edge_tube_face(surface: &Handle<dyn GeomSurface>, offset: f64) -> Result<Face, Failure> {
    let maker =
        BRepBuilderAPIMakeFace::from_surface(surface, rustycad_tkernel::precision::confusion())?;
    let mut face = maker.face()?;
    if offset < 0.0 {
        face.reverse();
    }
    Ok(face)
}
