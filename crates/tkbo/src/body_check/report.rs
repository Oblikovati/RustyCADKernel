// SPDX-License-Identifier: GPL-2.0-only
//! The owned validation report: plain data naming topology by kind and index, `Send + Sync`.
use rustycad_tkbrep::topexp::map_shapes_of_type;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::BRepCheckStatus;

/// The four ordered stages; a later stage runs only when every earlier one passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ValidationStage {
    /// Closed shells, manifold edges, consistent orientation, Euler-Poincare, mapped topology.
    Topology,
    /// Curves on surfaces, p-curves, vertices on curves within tolerance (mapped BRepCheck).
    Geometry,
    /// Interferences between sub-shapes (mapped BOPAlgo_CheckerSI).
    SelfIntersection,
    /// Vertex >= edge >= face tolerance, the resolution budget, certified planar residuals.
    Tolerance,
}

/// The outcome of one stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StageVerdict {
    /// The stage ran completely and found no defect.
    Passed,
    /// The stage found at least one defect.
    Failed,
    /// The stage did not complete or a check was inconclusive; never a pass.
    Inconclusive,
    /// Not run because an earlier stage did not pass.
    NotRun,
}

/// How completely a passed stage checked; a sampled or searched pass is not a certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckMode {
    /// Every check of the stage is exact (combinatorial or certified arithmetic).
    Exact,
    /// Some checks sample geometry (BRepCheck samples curve-on-surface deviations).
    Sampled,
    /// A completed native intersection search; it does not certify the absence of contacts.
    NativeSearch,
}

/// Whether a finding is a defect or a gap in coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The body violates the stage's invariant.
    Defect,
    /// The check could not decide; the stage cannot pass.
    Inconclusive,
}

/// A sub-shape of the validated root: its kind and 1-based index in the order of
/// `TopExp::MapShapes(root, kind)` (orientation-independent, deterministic).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TopologyRef {
    /// Shape kind (vertex, edge, face, ...).
    pub kind: ShapeEnum,
    /// 1-based index among the root's sub-shapes of that kind; 0 when not found in the root.
    pub index: usize,
}

impl TopologyRef {
    /// The named sub-shape of `root`, unoriented; `finding.subject.resolve(&solid)`.
    pub fn resolve(&self, root: &Shape) -> Option<Shape> {
        let mut map = IndexedMapOfShape::default();
        map_shapes_of_type(root, self.kind, &mut map).ok()?;
        map.find_key(self.index).ok().cloned()
    }
}

/// Vertex, edge, face, loop and shell counts of one solid and its Euler-Poincare genus:
/// `V - E + F - H = 2 (S - G)` with `H = L - F` inner loops; degenerate edges excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EulerCensus {
    /// The solid.
    pub solid: TopologyRef,
    /// Vertices.
    pub vertices: usize,
    /// Non-degenerate edges.
    pub edges: usize,
    /// Faces.
    pub faces: usize,
    /// Wires (loops) over all faces.
    pub loops: usize,
    /// Shells.
    pub shells: usize,
    /// Genus `S - (V - E + F - H) / 2`; None when the characteristic is odd.
    pub genus: Option<i64>,
}

/// What a finding reports.
#[derive(Clone, Debug, PartialEq)]
pub enum FindingKind {
    /// The root is not a solid, comp-solid, shell or compound of them.
    UnsupportedShape {
        /// The root's kind.
        actual: ShapeEnum,
    },
    /// An edge of a closed solid's shell bounds only one face use (open shell).
    FreeEdge,
    /// An edge bounds more than two face uses in one shell.
    NonManifoldEdge {
        /// Number of face uses.
        uses: usize,
    },
    /// Two face uses traverse the edge in the same direction (a flipped face).
    InconsistentOrientation,
    /// The census violates Euler-Poincare (odd characteristic or negative genus).
    EulerPoincare {
        /// The counts.
        census: EulerCensus,
    },
    /// A mapped BRepCheck status other than `NoError`.
    CheckStatus(BRepCheckStatus),
    /// A checker could not run; its message.
    CheckerFailure {
        /// The native failure text.
        message: String,
    },
    /// A checker result or a required context status is missing.
    CheckerCoverageGap,
    /// The self-intersection search found two interfering sub-shapes (subject, context).
    Interference,
    /// The self-intersection search did not complete.
    SearchIncomplete {
        /// Which part of the search is missing.
        reason: String,
    },
    /// A tolerance is negative or not finite.
    InvalidTolerance {
        /// The stored value.
        tolerance: f64,
    },
    /// An inner entity (subject) has a larger tolerance than its bounding entity (context).
    ToleranceOrder {
        /// Tolerance of the subject (edge or face).
        inner: f64,
        /// Tolerance of the context (vertex or edge).
        outer: f64,
    },
    /// A stored tolerance exceeds the resolution's tolerance cap.
    ToleranceAboveBudget {
        /// The stored tolerance.
        tolerance: f64,
        /// The cap.
        budget: f64,
    },
    /// The certified distance between the edge's curve and its p-curve on the planar
    /// face (context) exceeds the edge tolerance.
    CurveResidualExceedsTolerance {
        /// The edge tolerance.
        tolerance: f64,
    },
}

/// One typed observation of one stage, naming the offending topology.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// Defect or coverage gap.
    pub severity: Severity,
    /// What was found.
    pub kind: FindingKind,
    /// The offending sub-shape.
    pub subject: TopologyRef,
    /// The context it was checked in (face of an edge, second interfering shape, ...).
    pub context: Option<TopologyRef>,
}

/// The result of one stage.
#[derive(Clone, Debug, PartialEq)]
pub struct StageReport {
    /// Which stage.
    pub stage: ValidationStage,
    /// Its outcome.
    pub verdict: StageVerdict,
    /// How completely it checked.
    pub mode: CheckMode,
    /// Every finding, in check order.
    pub findings: Vec<Finding>,
}
