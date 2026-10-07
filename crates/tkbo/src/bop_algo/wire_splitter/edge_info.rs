//! `BOPAlgo_EdgeInfo` (BOPAlgo_WireSplitter.lxx): one edge seen from one of its vertices.
use rustycad_tkbrep::topods::Edge;

/// An edge at a vertex: entering or leaving it, already passed, inside the face, and its
/// 2D angle at the vertex.
///
/// ```
/// use rustycad_tkbo::bop_algo::EdgeInfo;
/// let info = EdgeInfo::new();
/// assert_eq!(info.angle(), -1.);
/// assert!(!info.passed() && !info.is_in() && !info.is_inside());
/// ```
#[derive(Clone)]
pub struct EdgeInfo {
    edge: Edge,
    passed: bool,
    in_flag: bool,
    is_inside: bool,
    angle: f64,
}

impl Default for EdgeInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeInfo {
    /// An info with no edge and the angle -1 (`BOPAlgo_EdgeInfo()`); `EdgeInfo::new()`.
    pub fn new() -> Self {
        Self {
            edge: Edge::default(),
            passed: false,
            in_flag: false,
            is_inside: false,
            angle: -1.,
        }
    }

    /// Sets the edge; `info.set_edge(&edge)`.
    pub fn set_edge(&mut self, edge: &Edge) {
        self.edge = edge.clone();
    }

    /// The edge; `info.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.edge
    }

    /// Marks the edge as used by a built wire; `info.set_passed(true)`.
    pub fn set_passed(&mut self, flag: bool) {
        self.passed = flag;
    }

    /// Whether a built wire used the edge; `info.passed()`.
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// Marks the edge as entering the vertex; `info.set_in_flag(true)`.
    pub fn set_in_flag(&mut self, flag: bool) {
        self.in_flag = flag;
    }

    /// Whether the edge enters the vertex; `info.is_in()`.
    pub fn is_in(&self) -> bool {
        self.in_flag
    }

    /// Sets the 2D angle of the edge at the vertex; `info.set_angle(0.5)`.
    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }

    /// The 2D angle of the edge at the vertex; `info.angle()`.
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// Whether the edge lies inside the face rather than on its boundary; `info.is_inside()`.
    pub fn is_inside(&self) -> bool {
        self.is_inside
    }

    /// Marks the edge as inside the face; `info.set_is_inside(true)`.
    pub fn set_is_inside(&mut self, is_inside: bool) {
        self.is_inside = is_inside;
    }
}
