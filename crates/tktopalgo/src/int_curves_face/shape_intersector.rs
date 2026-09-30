// Rust translation of IntCurvesFace_ShapeIntersector.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurvesFace_ShapeIntersector`: the intersection points of a line or a curve
//! with the faces of a shape, sorted by their parameter on the curve.
use super::intersector::{IntCurvesFaceIntersector, sequence_slot};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TransitionOnCurve};
use rustycad_tkmath::gp::{Line3, Point3};

/// `IntCurvesFace_ShapeIntersector` (docs/topalgo-design.md section 4): one
/// `IntCurvesFace_Intersector` per face of the loaded shape, and the points of all
/// faces sorted by their curve parameter (`myIndexPt` holds, in sorted order, the
/// index of each point in face order).
///
/// ```
/// use rustycad_tktopalgo::int_curves_face::IntCurvesFaceShapeIntersector;
/// let intersector = IntCurvesFaceShapeIntersector::new();
/// assert!(!intersector.is_done());
/// assert_eq!(intersector.nb_pnt(), 0);
/// ```
#[derive(Default)]
pub struct IntCurvesFaceShapeIntersector {
    my_is_done: bool,
    my_nb_faces: i32,
    my_ptr_nums: Vec<i32>,
    my_ptr_index_nums: Vec<i32>,
    my_intersector: Vec<IntCurvesFaceIntersector>,
    my_index_pt: Vec<i32>,
    my_index_face: Vec<i32>,
    my_index_int_pnt: Vec<i32>,
    my_index_par: Vec<f64>,
}

impl IntCurvesFaceShapeIntersector {
    /// `IntCurvesFace_ShapeIntersector()`: no face, not done;
    /// `IntCurvesFaceShapeIntersector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Load(Sh, Tol)`: an intersector (tolerance `tol`, restricted surface,
    /// boundary tolerances) for each face of `shape`. The intersectors are appended
    /// to those of an earlier Load while the face count restarts, so a second Load
    /// performs on the first faces loaded (native behaviour);
    /// `intersector.load(&shape, 1e-7)?`.
    pub fn load(&mut self, shape: &Shape, tol: f64) -> Result<(), Failure> {
        self.my_nb_faces = 0;
        let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        while faces.more() {
            self.my_nb_faces += 1;
            let face: Face = topods::face(faces.current())?;
            self.my_intersector
                .push(IntCurvesFaceIntersector::new(&face, tol, true, true)?);
            faces.next()?;
        }
        Ok(())
    }

    /// `Perform(L, PInf, PSup)`: every face intersector on the line, then the sorted
    /// points; `intersector.perform_line(&line, -1e100, 1e100)?`.
    pub fn perform_line(&mut self, line: &Line3, pinf: f64, psup: f64) -> Result<(), Failure> {
        self.my_is_done = false;
        for intersector in self.loaded_faces() {
            intersector.perform_line(line, pinf, psup)?;
        }
        self.sort_result()
    }

    /// `Perform(HCu, PInf, PSup)`: the same for an Adaptor3d curve;
    /// `intersector.perform_curve(&curve, 0.0, 1.0)?`.
    pub fn perform_curve(&mut self, curve: &HCurve, pinf: f64, psup: f64) -> Result<(), Failure> {
        self.my_is_done = false;
        for intersector in self.loaded_faces() {
            intersector.perform_curve(curve, pinf, psup)?;
        }
        self.sort_result()
    }

    // The intersectors of the last Load's face count, from the first one appended.
    fn loaded_faces(&mut self) -> std::slice::IterMut<'_, IntCurvesFaceIntersector> {
        let count: usize = (self.my_nb_faces.max(0) as usize).min(self.my_intersector.len());
        self.my_intersector[..count].iter_mut()
    }

    /// `PerformNearest(L, PInf, PSup)`: the faces in the order of their past success
    /// (above two faces), each on `[pinf, w]` with `w` the smallest parameter found so
    /// far; a face whose range became empty keeps its earlier results. Not done as soon
    /// as a face intersector is not done; `intersector.perform_nearest(&line, 0.0, 1e100)?`.
    pub fn perform_nearest(&mut self, line: &Line3, pinf: f64, psup: f64) -> Result<(), Failure> {
        self.init_ptr_nums();
        let mut index_face: i32 = -1;
        let mut par_max: f64 = psup;
        self.my_is_done = false;
        for ii in 1..=self.my_nb_faces {
            let face_index: i32 = if self.my_ptr_nums.is_empty() {
                ii
            } else {
                let slot: usize = (ii - 1) as usize;
                *self
                    .my_ptr_index_nums
                    .get(slot)
                    .ok_or_else(|| face_out_of_range(ii - 1, self.my_ptr_index_nums.len()))?
            };
            let Some(intersector) = self.my_intersector.get_mut((face_index - 1) as usize) else {
                return Err(face_out_of_range(face_index, self.my_intersector.len()));
            };
            let performs: bool = pinf < par_max;
            if !performs {
                continue;
            }
            intersector.perform_line(line, pinf, par_max)?;
            if !intersector.is_done() {
                self.my_is_done = false;
                return Ok(());
            }
            for j in 1..=intersector.nb_pnt() {
                let w: f64 = intersector.w_parameter(j)?;
                if w < par_max {
                    par_max = w;
                    index_face = ii - 1;
                }
            }
        }
        self.promote(index_face)?;
        self.sort_result()
    }

    // The success counters and face order of PerformNearest, created once above two
    // faces (and never resized by a later Load).
    fn init_ptr_nums(&mut self) {
        if self.my_nb_faces > 2 && self.my_ptr_nums.is_empty() {
            let count: usize = self.my_nb_faces as usize;
            self.my_ptr_nums = vec![0; count];
            self.my_ptr_index_nums = (1..=self.my_nb_faces).collect();
        }
    }

    // Counts one more success at the position `index_face` of the order and bubbles
    // it up past the positions with fewer successes.
    // A position past the counters of an earlier, smaller Load raises
    // Standard_OutOfRange, as the native array access does.
    fn promote(&mut self, index_face: i32) -> Result<(), Failure> {
        if self.my_ptr_nums.is_empty() || index_face < 0 {
            return Ok(());
        }
        let mut i: usize = index_face as usize;
        let len: usize = self.my_ptr_nums.len();
        *self
            .my_ptr_nums
            .get_mut(i)
            .ok_or_else(|| face_out_of_range(index_face, len))? += 1;
        while i >= 1 && self.my_ptr_nums[i] > self.my_ptr_nums[i - 1] {
            self.my_ptr_index_nums.swap(i, i - 1);
            self.my_ptr_nums.swap(i, i - 1);
            i -= 1;
        }
        Ok(())
    }

    /// `SortResult()`: collects the points of every face intersector (not done, and
    /// stopped, at the first one that is not done), then orders them by their curve
    /// parameter with a bubble sort (equal parameters keep face order);
    /// `intersector.sort_result()?`.
    pub fn sort_result(&mut self) -> Result<(), Failure> {
        self.my_is_done = true;
        self.my_index_pt.clear();
        self.my_index_face.clear();
        self.my_index_int_pnt.clear();
        self.my_index_par.clear();
        // Retrieval of the results
        let count: usize = (self.my_nb_faces.max(0) as usize).min(self.my_intersector.len());
        for (f, intersector) in self.my_intersector[..count].iter().enumerate() {
            if !intersector.is_done() {
                self.my_is_done = false;
                return Ok(());
            }
            for j in 1..=intersector.nb_pnt() {
                self.my_index_pt.push(self.my_index_pt.len() as i32 + 1);
                self.my_index_face.push(f as i32 + 1);
                self.my_index_int_pnt.push(j);
                self.my_index_par.push(intersector.w_parameter(j)?);
            }
        }
        self.bubble_sort();
        Ok(())
    }

    // Sort according to parameter w
    fn bubble_sort(&mut self) {
        let nb_pnt: usize = self.my_index_pt.len();
        let mut is_ok: bool = false;
        while !is_ok {
            is_ok = true;
            for ind0 in 0..nb_pnt.saturating_sub(1) {
                let (ind, indp1) = (self.my_index_pt[ind0], self.my_index_pt[ind0 + 1]);
                if self.my_index_par[(ind - 1) as usize] > self.my_index_par[(indp1 - 1) as usize] {
                    self.my_index_pt[ind0] = indp1;
                    self.my_index_pt[ind0 + 1] = ind;
                    is_ok = false;
                }
            }
        }
    }

    /// `IsDone()`; `intersector.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `NbPnt()`: the number of sorted points; `intersector.nb_pnt()`.
    pub fn nb_pnt(&self) -> i32 {
        self.my_index_pt.len() as i32
    }

    // The face intersector and the point index of the i-th sorted point;
    // Standard_OutOfRange outside [1, NbPnt].
    fn sorted(&self, index: i32) -> Result<(&IntCurvesFaceIntersector, i32), Failure> {
        let k: i32 = self.my_index_pt[sequence_slot(index, self.my_index_pt.len())?];
        let slot: usize = (k - 1) as usize;
        let face: i32 = self.my_index_face[slot];
        let intersector = self
            .my_intersector
            .get((face - 1) as usize)
            .ok_or_else(|| face_out_of_range(face, self.my_intersector.len()))?;
        Ok((intersector, self.my_index_int_pnt[slot]))
    }

    /// `UParameter(I)`: the U parameter of the i-th sorted point on its face surface;
    /// Standard_OutOfRange outside `[1, NbPnt]`; `intersector.u_parameter(1)?`.
    pub fn u_parameter(&self, index: i32) -> Result<f64, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.u_parameter(j)
    }

    /// `VParameter(I)`; `intersector.v_parameter(1)?`.
    pub fn v_parameter(&self, index: i32) -> Result<f64, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.v_parameter(j)
    }

    /// `WParameter(I)`: the curve parameter of the i-th sorted point;
    /// `intersector.w_parameter(1)?`.
    pub fn w_parameter(&self, index: i32) -> Result<f64, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.w_parameter(j)
    }

    /// `Pnt(I)`; `intersector.pnt(1)?`.
    pub fn pnt(&self, index: i32) -> Result<Point3, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.pnt(j)
    }

    /// `Transition(I)`; `intersector.transition(1)?`.
    pub fn transition(&self, index: i32) -> Result<TransitionOnCurve, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.transition(j)
    }

    /// `State(I)`: IN or ON; `intersector.state(1)?`.
    pub fn state(&self, index: i32) -> Result<State, Failure> {
        let (intersector, j) = self.sorted(index)?;
        intersector.state(j)
    }

    /// `Face(I)`: the face of the i-th sorted point; `intersector.face(1)?`.
    pub fn face(&self, index: i32) -> Result<&Face, Failure> {
        let (intersector, _) = self.sorted(index)?;
        Ok(intersector.face())
    }
}

fn face_out_of_range(index: i32, len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "IntCurvesFace_ShapeIntersector: face intersector {index} outside [1, {len}]"
    ))
}
