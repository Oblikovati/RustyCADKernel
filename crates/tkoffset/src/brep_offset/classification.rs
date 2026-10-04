//! Native offset construction modes and failure classifications.
// BRepOffset_Mode.hxx/BRepOffset_Error.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

/// Selects the native offset construction; `BRepOffsetMode::Skin as i32 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BRepOffsetMode {
    /// Offset the surface of a solid to obtain manifold topology.
    Skin = 0,
    /// Offset a curve to construct a surface.
    Pipe = 1,
    /// Offset a surface shell on both sides.
    RectoVerso = 2,
}

/// Native offset failure codes; `BRepOffsetError::NoError as i32 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BRepOffsetError {
    /// No error was recorded.
    NoError = 0,
    /// A failure without a more specific classification.
    UnknownError = 1,
    /// Geometry does not provide usable normals.
    BadNormalsOnGeometry = 2,
    /// Geometry is only position-continuous.
    C0Geometry = 3,
    /// A null offset was requested.
    NullOffset = 4,
    /// The input shell is disconnected.
    NotConnectedShell = 5,
    /// An exception occurred while trimming edges.
    CannotTrimEdges = 6,
    /// An exception occurred while fusing vertices.
    CannotFuseVertices = 7,
    /// An exception occurred while extending edges; native spelling is `Extent`.
    CannotExtentEdge = 8,
    /// The user interrupted construction.
    UserBreak = 9,
    /// Face connectivity changes between C0 and tangential along an edge.
    MixedConnectivity = 10,
}
