// Rust translation of BRepClass3d_SClassifier.hxx and the state and infinite point
// members of BRepClass3d_SClassifier.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass3d_SClassifier`: the state of a point (or of the infinite point) with
//! respect to the solid a `BRepClass3d_SolidExplorer` explores.
use super::BRepClass3dSolidExplorer;
use super::FacePointSample;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkmath::gp::{self, Direction3, Line3, Point3};
use rustycad_tkmath::math::BullardGenerator;

/// `BRepClass3d_SClassifier`: the classification state (1 error, 2 ON, 3 IN, 4
/// OUT; 0 before a classification) and the face that decided it.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_class3d::{BRepClass3dSClassifier, BRepClass3dSolidExplorer};
/// // A solid without faces is the whole space.
/// let mut explorer = BRepClass3dSolidExplorer::with_shape(&Shape::default())?;
/// let classifier = BRepClass3dSClassifier::with_point(&mut explorer, &Point3::new(1.0, 0.0, 0.0), 1e-7)?;
/// assert_eq!(classifier.state(), State::In);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepClass3dSClassifier {
    pub(super) my_face: Face,
    /// 1 - Error inside of the algorithm, 2 - ON, 3 - IN, 4 - OUT.
    pub(super) my_state: i32,
}

/// The number of probing points tried from each face by `PerformInfinitePoint`.
const NB_MAX_POINTS_PER_FACE: i32 = 10;

impl BRepClass3dSClassifier {
    /// `BRepClass3d_SClassifier()`: state 0 (OUT); `BRepClass3dSClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepClass3d_SClassifier(S, P, Tol)`: IN for a solid without faces, else
    /// `Perform(S, P, Tol)`; `BRepClass3dSClassifier::with_point(&mut explorer, &p, 1e-7)?`.
    pub fn with_point(
        explorer: &mut BRepClass3dSolidExplorer,
        point: &Point3,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut classifier = Self::default();
        if explorer.reject(point) {
            classifier.my_state = 3; //-- in ds solid case without face
        } else {
            classifier.perform(explorer, point, tol)?;
        }
        Ok(classifier)
    }

    /// `PerformInfinitePoint(S, Tol)`: IN for a solid without faces; else, up to 10
    /// times over the faces, the line from a random point of the face (its parameter
    /// in [0.1, 0.9] from a `math_BullardGenerator`) against the face normal: when the
    /// nearest intersection is IN a face, OUT for an entering and IN for a leaving
    /// transition. Without a decisive probe the state stays ON; `tol` is unused;
    /// `classifier.perform_infinite_point(&mut explorer, 1e-7)?`.
    pub fn perform_infinite_point(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        _tol: f64,
    ) -> Result<(), Failure> {
        if explorer.reject(&Point3::new(0.0, 0.0, 0.0)) {
            self.my_state = 3; //-- in ds solid case without face
            return Ok(());
        }
        let mut random = BullardGenerator::default();
        self.my_face = Face::default();
        self.my_state = 2;
        let faces = collect_faces(explorer)?;
        for _ in 0..NB_MAX_POINTS_PER_FACE {
            for face in &faces {
                let param = 0.1 + 0.8 * random.next_real(); // random number in range [0.1, 0.9]
                if let Some(state) = self.probe_face(explorer, face, param)? {
                    self.my_state = state;
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// One probe of `PerformInfinitePoint`: `Some(3)` or `Some(4)` when the nearest
    /// intersection of the reversed normal line is IN a face and not tangent.
    fn probe_face(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        face: &Face,
        param: f64,
    ) -> Result<Option<i32>, Failure> {
        let mut sample = FacePointSample::default();
        let mut param = param;
        if !BRepClass3dSolidExplorer::find_a_point_in_the_face_d1(face, &mut sample, &mut param)? {
            return Ok(None);
        }
        let Some(normal) = face_normal(face, sample.u, sample.v)? else {
            return Ok(None);
        };
        let line = Line3::new(sample.point, normal.reversed());
        let (state, transition) = self.nearest_on_solid(explorer, &line)?;
        Ok(match (state, transition) {
            (State::In, TransitionOnCurve::Out) => Some(3), //-- from inside the solid to outside
            (State::In, TransitionOnCurve::In) => Some(4),
            _ => None,
        })
    }

    /// The state and transition of the intersection of smallest parameter of the
    /// whole line with the faces (OUT and tangent without one); a rejected shell
    /// would set the error state.
    fn nearest_on_solid(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        line: &Line3,
    ) -> Result<(State, TransitionOnCurve), Failure> {
        let (mut state, mut transition) = (State::Out, TransitionOnCurve::Tangent);
        let mut parmin = f64::MAX;
        explorer.init_shell()?;
        while explorer.more_shell() {
            if explorer.reject_shell(line) {
                self.my_state = 1;
            } else {
                explorer.init_face()?;
                while explorer.more_face() {
                    if !explorer.reject_face(line) {
                        let face = explorer.current_face()?;
                        let intersector = explorer.change_intersector(&face)?;
                        intersector.perform_line(line, -f64::MAX, parmin)?;
                        if intersector.is_done() && intersector.nb_pnt() > 0 {
                            let imin = nearest_point(
                                |i| intersector.w_parameter(i),
                                intersector.nb_pnt(),
                            )?;
                            parmin = intersector.w_parameter(imin)?;
                            state = intersector.state(imin)?;
                            transition = intersector.transition(imin)?;
                        }
                    }
                    explorer.next_face()?;
                }
            }
            explorer.next_shell()?;
        }
        Ok((state, transition))
    }

    /// `Rejected()`: the error state; `classifier.rejected()`.
    pub fn rejected(&self) -> bool {
        self.my_state == 1
    }

    /// `State()`: ON, IN or OUT (OUT for an error or before a classification);
    /// `classifier.state()`.
    pub fn state(&self) -> State {
        match self.my_state {
            2 => State::On,
            3 => State::In,
            // return OUT state when there is an error during execution.
            _ => State::Out,
        }
    }

    /// `IsOnAFace()`: the ON state; `classifier.is_on_a_face()`.
    pub fn is_on_a_face(&self) -> bool {
        self.my_state == 2
    }

    /// `Face()`: the face that decided the state (the face holding the point when
    /// ON); meaningless when rejected; `classifier.face()`.
    pub fn face(&self) -> Face {
        self.my_face.clone()
    }

    /// Protected `ForceIn()`: the IN state; `classifier.force_in()`.
    pub fn force_in(&mut self) {
        self.my_state = 3;
    }

    /// Protected `ForceOut()`: the OUT state; `classifier.force_out()`.
    pub fn force_out(&mut self) {
        self.my_state = 4;
    }
}

/// The faces of every shell of the explorer, in exploration order.
fn collect_faces(explorer: &mut BRepClass3dSolidExplorer) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    explorer.init_shell()?;
    while explorer.more_shell() {
        explorer.init_face()?;
        while explorer.more_face() {
            faces.push(explorer.current_face()?);
            explorer.next_face()?;
        }
        explorer.next_shell()?;
    }
    Ok(faces)
}

/// The index of the smallest parameter among `1..=count` (the first on ties).
pub(super) fn nearest_point(
    parameter: impl Fn(i32) -> Result<f64, Failure>,
    count: i32,
) -> Result<i32, Failure> {
    let mut imin = 1;
    for i in 2..=count {
        if parameter(i)? < parameter(imin)? {
            imin = i;
        }
    }
    Ok(imin)
}

/// `FaceNormal(aF, U, V, aDN)`: the unit normal of the face surface (reversed for a
/// REVERSED face); `None` where the derivatives are parallel.
pub(super) fn face_normal(face: &Face, u: f64, v: f64) -> Result<Option<Direction3>, Failure> {
    let surface = BRepTool::surface(face)?;
    let surface = surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepClass3d_SClassifier: FaceNormal of a face without surface",
        )
    })?;
    let d1 = surface.d1(u, v)?;
    let mut normal = d1.d1u.crossed(d1.d1v);
    if normal.magnitude() <= gp::resolution() {
        return Ok(None);
    }
    normal.normalize()?;
    let mut direction = Direction3::from_vector(normal)?;
    if face.orientation() == Orientation::Reversed {
        direction.reverse();
    }
    Ok(Some(direction))
}
