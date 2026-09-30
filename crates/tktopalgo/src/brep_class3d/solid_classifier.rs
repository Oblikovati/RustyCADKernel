// Rust translation of BRepClass3d_SolidClassifier.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (MARCHEPASSIUNESEULEFACE and LBRCOMPT
// are 0 natively, so their branches are not compiled).

//! `BRepClass3d_SolidClassifier`: the solid classifier on its own explorer.
use super::{BRepClass3dSClassifier, BRepClass3dSolidExplorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point3;
use std::ops::Deref;

/// `BRepClass3d_SolidClassifier` (docs/topalgo-design.md section 5): a
/// `BRepClass3d_SClassifier` with its own `BRepClass3d_SolidExplorer`. The base's
/// `State`, `Rejected`, `IsOnAFace` and `Face` are reached through `Deref`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
/// // A shape without faces is the whole space.
/// let classifier = BRepClass3dSolidClassifier::with_point(&Shape::default(), &Point3::new(0.0, 0.0, 0.0), 1e-7)?;
/// assert_eq!(classifier.state(), State::In);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepClass3dSolidClassifier {
    base: BRepClass3dSClassifier,
    a_solid_loaded: bool,
    explorer: BRepClass3dSolidExplorer,
    // Written by PerformInfinitePoint; only the disabled MARCHEPASSIUNESEULEFACE
    // branches read it natively.
    #[allow(dead_code)]
    isaholeinspace: bool,
}

impl Deref for BRepClass3dSolidClassifier {
    type Target = BRepClass3dSClassifier;
    fn deref(&self) -> &BRepClass3dSClassifier {
        &self.base
    }
}

impl BRepClass3dSolidClassifier {
    /// `BRepClass3d_SolidClassifier()`: no solid loaded (Perform classifies against
    /// the empty explorer, which rejects: IN); `BRepClass3dSolidClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Load(S)`: explores `shape` (the intersectors of an earlier shape deleted);
    /// `classifier.load(&solid)?`.
    pub fn load(&mut self, shape: &Shape) -> Result<(), Failure> {
        if self.a_solid_loaded {
            self.explorer.destroy();
        }
        self.explorer.init_shape(shape)?;
        self.a_solid_loaded = true;
        Ok(())
    }

    /// `BRepClass3d_SolidClassifier(S)`; `BRepClass3dSolidClassifier::with_shape(&solid)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            base: BRepClass3dSClassifier::default(),
            a_solid_loaded: true,
            explorer: BRepClass3dSolidExplorer::with_shape(shape)?,
            isaholeinspace: false,
        })
    }

    /// `BRepClass3d_SolidClassifier(S, P, Tol)`: `Perform(P, Tol)` on the shape;
    /// `BRepClass3dSolidClassifier::with_point(&solid, &p, 1e-7)?`.
    pub fn with_point(shape: &Shape, point: &Point3, tol: f64) -> Result<Self, Failure> {
        let mut classifier = Self::with_shape(shape)?;
        classifier.perform(point, tol)?;
        Ok(classifier)
    }

    /// `Perform(P, Tol)`: `BRepClass3d_SClassifier::Perform` on the explorer;
    /// `classifier.perform(&p, 1e-7)?`.
    pub fn perform(&mut self, point: &Point3, tol: f64) -> Result<(), Failure> {
        self.base.perform(&mut self.explorer, point, tol)
    }

    /// `PerformInfinitePoint(Tol)`: with a solid loaded, the state of the infinite
    /// point (useful for the orientation of a solid), recording whether the solid is a
    /// hole in space (not OUT); `classifier.perform_infinite_point(1e-7)?`.
    pub fn perform_infinite_point(&mut self, tol: f64) -> Result<(), Failure> {
        if self.a_solid_loaded {
            self.base.perform_infinite_point(&mut self.explorer, tol)?;
            self.isaholeinspace = self.base.state() != State::Out;
        }
        Ok(())
    }

    /// `Destroy()`: with a solid loaded, deletes its intersectors and unloads it;
    /// `classifier.destroy()`.
    pub fn destroy(&mut self) {
        if self.a_solid_loaded {
            self.explorer.destroy();
            self.a_solid_loaded = false;
        }
    }
}
