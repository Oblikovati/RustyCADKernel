// Rust translation of BRepBuilderAPI_Sewing::Perform (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sewing steps in order: analysis, free boundaries, vertices assembling, cutting,
//! merging, edge processing, sewn shape, regularity, SameParameter and the output.
use super::BRepBuilderAPISewing;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;

impl BRepBuilderAPISewing {
    /// `Perform(theProgress)`: sews the added shapes (or the context shape). An
    /// exception of a step is returned with the state reached, as it leaves the native
    /// member; a user break returns with the sewn shape nullified after it was built;
    /// `sewing.perform(&ProgressRange::default())?`.
    pub fn perform(&mut self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let stages = i32::from(self.my_analysis)
            + i32::from(self.my_cutting)
            + i32::from(self.my_sewing)
            + 2;
        let scope = ProgressScope::with_literal(progress, "Sewing", f64::from(stages), false);
        if self.my_analysis {
            self.face_analysis(&scope.next(1.0))?;
            if !scope.more() {
                return Ok(());
            }
        }
        if self.my_nb_shapes == 0 && self.my_shape.is_null() {
            return Ok(());
        }
        self.find_free_boundaries()?;
        if !self.assemble_and_merge(&scope)? {
            return Ok(());
        }
        if self.my_sewing && !self.sew(&scope)? {
            return Ok(());
        }
        self.create_output_informations()?;
        if !scope.more() {
            self.my_sewed_shape.nullify();
        }
        Ok(())
    }

    /// The vertices assembling, cutting and merging of the boundaries (their progress
    /// steps skipped when there is no boundary); false on a user break.
    fn assemble_and_merge(&mut self, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        if self.my_bound_faces.is_empty() {
            scope.next(1.0);
            if self.my_cutting {
                scope.next(1.0);
            }
            scope.next(1.0);
            return Ok(scope.more());
        }
        self.vertices_assembling(&scope.next(1.0))?;
        if !scope.more() {
            return Ok(false);
        }
        if self.my_cutting {
            self.cutting(&scope.next(1.0))?;
            if !scope.more() {
                return Ok(false);
            }
        }
        self.merging(true, &scope.next(1.0))?;
        Ok(scope.more())
    }

    /// The edge processing, the sewn shape, its regularity and SameParameter; false on
    /// a user break (the sewn shape nullified once built).
    fn sew(&mut self, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        // Examine the multiple edges if any and process sameparameter for edges if
        // necessary.
        self.edge_processing(&scope.next(1.0))?;
        if !scope.more() {
            return Ok(false);
        }
        self.create_sewed_shape()?;
        if !scope.more() {
            self.my_sewed_shape.nullify();
            return Ok(false);
        }
        self.edge_regularity(&scope.next(1.0))?;
        if self.same_parameter_mode() && self.face_mode() {
            self.same_parameter_shape()?;
        }
        if !scope.more() {
            self.my_sewed_shape.nullify();
            return Ok(false);
        }
        Ok(true)
    }
}
