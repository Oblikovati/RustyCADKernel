// ShapeUpgrade_ShellSewing.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::shape_analysis::ShapeAnalysisShapeTolerance;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::{
    brep_tools::ReShape, topexp::Explorer, topods::Shape, toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPISewing, brep_class3d::BRepClass3dSolidClassifier,
};

/// Sews shells independently and applies their replacements to the original shape.
/// ```
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeShellSewing;
/// use rustycad_tkbrep::topods::Shape;
/// let mut sewing=ShapeUpgradeShellSewing::new();
/// assert!(sewing.apply_sewing(&Shape::default(),0.).unwrap().is_null());
/// ```
#[derive(Default)]
pub struct ShapeUpgradeShellSewing {
    shells: IndexedMapOfShape,
    reshape: ShapeBuildReShape,
}
impl ShapeUpgradeShellSewing {
    /// Creates the persistent native shell set; `ShapeUpgradeShellSewing::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sews the input's shells, using mean shape tolerance for non-positive values.
    /// `let result=sewing.apply_sewing(&shape,1e-6)?;`
    pub fn apply_sewing(&mut self, shape: &Shape, tolerance: f64) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(shape.clone());
        }
        let tolerance = if tolerance <= 0. {
            ShapeAnalysisShapeTolerance::new().tolerance(shape, 0, ShapeEnum::Shape)?
        } else {
            tolerance
        };
        self.init(shape)?;
        if self.prepare(tolerance)? == 0 {
            return Ok(Shape::default());
        }
        self.apply(shape, tolerance)
    }

    fn init(&mut self, shape: &Shape) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        if shape.shape_type()? == ShapeEnum::Shell {
            self.shells.add(shape.clone());
            return Ok(());
        }
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            self.shells.add(shell);
        }
        Ok(())
    }

    fn prepare(&self, tolerance: f64) -> Result<usize, Failure> {
        let mut count = 0;
        for shell in &self.shells {
            let result = sew_shell(shell, tolerance)?;
            if result.is_null() {
                continue;
            }
            self.reshape.replace(shell, &result)?;
            count += 1;
        }
        Ok(count)
    }

    fn apply(&self, shape: &Shape, tolerance: f64) -> Result<Shape, Failure> {
        if shape.is_null() || self.shells.is_empty() {
            return Ok(shape.clone());
        }
        let result = self
            .reshape
            .apply_with_build_mode(shape, ShapeEnum::Face, 2)?;
        self.reshape.clear();
        if self.reverse_inside_solids(shape, tolerance)? {
            return self
                .reshape
                .apply_with_build_mode(&result, ShapeEnum::Shell, 2);
        }
        Ok(result)
    }

    fn reverse_inside_solids(&self, shape: &Shape, tolerance: f64) -> Result<bool, Failure> {
        let mut reversed = false;
        // Native classifies the original solids, then applies those identities to the rebuilt result.
        for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
            let mut classifier = BRepClass3dSolidClassifier::with_shape(&solid)?;
            classifier.perform_infinite_point(tolerance)?;
            if classifier.state() != State::In {
                continue;
            }
            self.reshape.replace(&solid, &solid.reversed())?;
            reversed = true;
        }
        Ok(reversed)
    }
}
fn sew_shell(shell: &Shape, tolerance: f64) -> Result<Shape, Failure> {
    let mut sewing = BRepBuilderAPISewing::new(tolerance, true, true, true, false);
    for face in Explorer::new(shell, ShapeEnum::Face, ShapeEnum::Shape)? {
        sewing.add(&face)?;
    }
    sewing.perform(&ProgressRange::default())?;
    Ok(sewing.sewed_shape().clone())
}
