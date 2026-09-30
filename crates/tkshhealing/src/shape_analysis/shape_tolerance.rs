// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_ShapeTolerance.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_ShapeTolerance`: minimal, maximal and average tolerances of the
//! sub-shapes of a shape, and the sub-shapes whose tolerance lies in an interval.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ShapeAnalysis_ShapeTolerance`: tolerances of faces, edges and vertices.
///
/// `type` selects the sub-shapes as natively: `Shape` all faces, edges and vertices,
/// `Vertex`, `Edge` or `Face` only those, `Shell` (for `InTolerance`) the shells and faces
/// that hold a face, edge or vertex in the interval.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisShapeTolerance;
/// let mut tool = ShapeAnalysisShapeTolerance::new();
/// assert_eq!(tool.tolerance(&Shape::default(), 1, ShapeEnum::Shape).unwrap(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeAnalysisShapeTolerance {
    /// `myTols`: minimum, sum and maximum. Natively uninitialized until
    /// `InitTolerance` or the first `AddTolerance` (see the progress doc); zeros here.
    tols: [f64; 3],
    /// `myNbTol`: the count of tolerances added.
    nb_tol: i32,
}

/// The running minimum, sum and maximum of one `AddTolerance` call.
#[derive(Default)]
struct Cumul {
    nbt: i32,
    cmin: f64,
    cmoy: f64,
    cmax: f64,
}

impl Cumul {
    /// The static `AddTol` (#76 rln 11.03.99 S4135: the average has no weights, so
    /// `mult` stays 1).
    fn add(&mut self, tol: f64) {
        self.nbt += 1;
        if self.nbt == 1 {
            (self.cmin, self.cmoy, self.cmax) = (tol, tol, tol);
            return;
        }
        if self.cmin > tol {
            self.cmin = tol;
        }
        if self.cmax < tol {
            self.cmax = tol;
        }
        let mult = 1;
        self.nbt += mult - 1;
        self.cmoy += tol * f64::from(mult);
    }
}

/// The tolerance of a face, edge or vertex.
fn sub_shape_tolerance(shape: &Shape, kind: ShapeEnum) -> Result<f64, Failure> {
    match kind {
        ShapeEnum::Face => BRepTool::face_tolerance(&topods::face(shape)?),
        ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(shape)?),
        _ => BRepTool::vertex_tolerance(&topods::vertex(shape)?),
    }
}

/// Calls `visit` on the tolerance of each sub-shape of `kind`, in explorer order.
fn each_tolerance(
    shape: &Shape,
    kind: ShapeEnum,
    mut visit: impl FnMut(&Shape, f64),
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        let current = explorer.current().clone();
        visit(&current, sub_shape_tolerance(&current, kind)?);
        explorer.next()?;
    }
    Ok(())
}

/// The kinds of sub-shapes one `type` selects, in the native order (faces, edges,
/// vertices).
fn selected_kinds(kind: ShapeEnum) -> impl Iterator<Item = ShapeEnum> {
    [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex]
        .into_iter()
        .filter(move |k| kind == *k || kind == ShapeEnum::Shape)
}

impl ShapeAnalysisShapeTolerance {
    /// `ShapeAnalysis_ShapeTolerance()`; `ShapeAnalysisShapeTolerance::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Tolerance(shape, mode, type)`: `InitTolerance`, `AddTolerance` and
    /// `GlobalTolerance(mode)` (the average for 0, the maximum above, the minimum
    /// below); `tool.tolerance(&shape, 1, ShapeEnum::Shape)?`.
    pub fn tolerance(&mut self, shape: &Shape, mode: i32, kind: ShapeEnum) -> Result<f64, Failure> {
        self.init_tolerance();
        self.add_tolerance(shape, kind)?;
        Ok(self.global_tolerance(mode))
    }

    /// `OverTolerance(shape, value, type)`: `InTolerance(value, 0)` (no upper bound) for a
    /// non-negative `value`, else `InTolerance(0, value)`;
    /// `tool.over_tolerance(&shape, 1e-3, ShapeEnum::Shape)?`.
    pub fn over_tolerance(
        &self,
        shape: &Shape,
        value: f64,
        kind: ShapeEnum,
    ) -> Result<Handle<HSequence<Shape>>, Failure> {
        if value >= 0.0 {
            return self.in_tolerance(shape, value, 0.0, kind);
        }
        self.in_tolerance(shape, 0.0, value, kind)
    }

    /// `InTolerance(shape, valmin, valmax, type)`: the faces, then edges, then vertices
    /// with a tolerance at least `valmin` and at most `valmax` (no upper bound when
    /// `valmax < valmin`). Natively a vertex passes with a tolerance at least `valmax`,
    /// not at most. With `Shell`, each shell holding a face in the interval (recursively
    /// with `Shell`) follows its faces, then the free faces that are in the interval or
    /// hold an edge or vertex in it; `tool.in_tolerance(&shape, 0.0, 1.0, ShapeEnum::Edge)?`.
    pub fn in_tolerance(
        &self,
        shape: &Shape,
        valmin: f64,
        valmax: f64,
        kind: ShapeEnum,
    ) -> Result<Handle<HSequence<Shape>>, Failure> {
        let over = valmax < valmin;
        let found = Handle::new(HSequence::<Shape>::default());
        let list = found.get().expect("new sequence");
        for sub in selected_kinds(kind) {
            each_tolerance(shape, sub, |current, tol| {
                let upper = if sub == ShapeEnum::Vertex {
                    tol >= valmax
                } else {
                    tol <= valmax
                };
                if tol >= valmin && (over || upper) {
                    list.append(current.clone());
                }
            })?;
        }
        if kind == ShapeEnum::Shell {
            self.shells_in_tolerance(shape, valmin, valmax, list)?;
        }
        Ok(found)
    }

    /// The `SHELL` branch of `InTolerance`.
    fn shells_in_tolerance(
        &self,
        shape: &Shape,
        valmin: f64,
        valmax: f64,
        list: &HSequence<Shape>,
    ) -> Result<(), Failure> {
        let mut faces = MapOfShape::default();
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            let mut has_face = false;
            for face in Explorer::new(&shell, ShapeEnum::Face, ShapeEnum::Shape)? {
                faces.add(face.clone());
                let found = self.in_tolerance(&face, valmin, valmax, ShapeEnum::Shell)?;
                if found.get().is_some_and(|f| f.sequence(|s| s.length()) > 0) {
                    list.append_handle(&found)?;
                    has_face = true;
                }
            }
            if has_face {
                list.append(shell);
            }
        }
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if faces.contains(&face) {
                continue;
            }
            if self.face_in_tolerance(&face, valmin, valmax)? {
                list.append(face);
            }
        }
        Ok(())
    }

    /// A free face of the `SHELL` branch: in the interval itself, or with an edge or a
    /// vertex in it.
    fn face_in_tolerance(&self, face: &Shape, valmin: f64, valmax: f64) -> Result<bool, Failure> {
        let tol = sub_shape_tolerance(face, ShapeEnum::Face)?;
        if tol >= valmin && (valmax < valmin || tol <= valmax) {
            return Ok(true);
        }
        for sub in [ShapeEnum::Edge, ShapeEnum::Vertex] {
            let found = self.in_tolerance(face, valmin, valmax, sub)?;
            if found.get().is_some_and(|f| f.sequence(|s| s.length()) > 0) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `InitTolerance()`: starts a new cumulation (the sum and the count are reset, the
    /// minimum and maximum are replaced by the next `AddTolerance`);
    /// `tool.init_tolerance()`.
    pub fn init_tolerance(&mut self) {
        self.nb_tol = 0;
        self.tols[1] = 0.0;
    }

    /// `AddTolerance(shape, type)`: adds the tolerances of the selected sub-shapes to the
    /// cumulation (nothing when there is none); `tool.add_tolerance(&shape, ShapeEnum::Shape)?`.
    pub fn add_tolerance(&mut self, shape: &Shape, kind: ShapeEnum) -> Result<(), Failure> {
        let mut cumul = Cumul::default();
        for sub in selected_kinds(kind) {
            each_tolerance(shape, sub, |_, tol| cumul.add(tol))?;
        }
        if cumul.nbt == 0 {
            return Ok(());
        }
        if self.nb_tol == 0 || self.tols[0] > cumul.cmin {
            self.tols[0] = cumul.cmin;
        }
        if self.nb_tol == 0 || self.tols[2] < cumul.cmax {
            self.tols[2] = cumul.cmax;
        }
        self.tols[1] += cumul.cmoy;
        self.nb_tol += cumul.nbt;
        Ok(())
    }

    /// `GlobalTolerance(mode)`: 0 before any tolerance, else the minimum (`mode < 0`), the
    /// maximum (`mode > 0`) or the average (`mode == 0`, the minimum when it equals the
    /// maximum); `tool.global_tolerance(0)`.
    pub fn global_tolerance(&self, mode: i32) -> f64 {
        if self.nb_tol == 0 {
            return 0.0;
        }
        if mode < 0 {
            return self.tols[0];
        }
        if mode > 0 {
            return self.tols[2];
        }
        if self.tols[0] == self.tols[2] {
            return self.tols[0];
        }
        self.tols[1] / f64::from(self.nb_tol)
    }
}
