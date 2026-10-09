// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_C2DF.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_C2DF`: a p-curve with its range and tolerance on a face.
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;

/// A p-curve on a face (`TopOpeBRepTool_C2DF`); natively the range and tolerance of the
/// default constructor are uninitialized, they are 0 here;
/// `let c = TopOpeBRepToolC2DF::new(&pc, 0., 1., 1e-7, &face);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepToolC2DF {
    my_pc: Handle<dyn Geom2dCurve>,
    myf2d: f64,
    myl2d: f64,
    mytol: f64,
    my_face: Face,
}

impl TopOpeBRepToolC2DF {
    /// `TopOpeBRepTool_C2DF(PC, f2d, l2d, tol, F)`.
    pub fn new(pc: &Handle<dyn Geom2dCurve>, f2d: f64, l2d: f64, tol: f64, f: &Face) -> Self {
        Self {
            my_pc: pc.clone(),
            myf2d: f2d,
            myl2d: l2d,
            mytol: tol,
            my_face: f.clone(),
        }
    }

    /// `SetPC(PC, f2d, l2d, tol)`; `c.set_pc(&pc, 0., 1., 1e-7)`.
    pub fn set_pc(&mut self, pc: &Handle<dyn Geom2dCurve>, f2d: f64, l2d: f64, tol: f64) {
        (self.my_pc, self.myf2d, self.myl2d, self.mytol) = (pc.clone(), f2d, l2d, tol);
    }

    /// `SetFace(F)`; `c.set_face(&face)`.
    pub fn set_face(&mut self, f: &Face) {
        self.my_face = f.clone();
    }

    /// `PC(f2d, l2d, tol)`: the p-curve, its range and tolerance;
    /// `let (pc, f, l, tol) = c.pc();`.
    pub fn pc(&self) -> (Handle<dyn Geom2dCurve>, f64, f64, f64) {
        (self.my_pc.clone(), self.myf2d, self.myl2d, self.mytol)
    }

    /// `Face()`; `c.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `IsPC(PC)`: the same curve object; `c.is_pc(&pc)`.
    pub fn is_pc(&self, pc: &Handle<dyn Geom2dCurve>) -> bool {
        pc.object_address() == self.my_pc.object_address()
    }

    /// `IsFace(F)`: the equal face (same orientation); `c.is_face(&face)`.
    pub fn is_face(&self, f: &Face) -> bool {
        f.as_ref().is_equal(self.my_face.as_ref())
    }
}
