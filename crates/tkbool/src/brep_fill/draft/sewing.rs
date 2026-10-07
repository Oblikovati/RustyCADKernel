// Rust translation of BRepFill_Draft::Sewing
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Draft.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The assembly of the draft surface with the input face or shell.
use super::{BRepFillDraft, null};
use crate::brep_fill::location_law::BRepFillLocationLaw as _;
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Solid};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// The shape `(1, ii)` of the table replaced by its sewn shape when it is modified.
fn replace_by_sewn(
    ass: &BRepBuilderAPISewing,
    table: &mut ShapeTable,
    ii: i32,
) -> Result<(), Failure> {
    let s: Shape = cell(table, 1, ii)?.clone();
    if ass.is_modified(&s) {
        *cell_mut(table, 1, ii)? = ass.modified(&s);
    }
    Ok(())
}

impl BRepFillDraft {
    /// `Sewing()` (private): the draft sewn to the input face or shell (not for a wire
    /// input or a failed draft); the sewn shell or solid (the only part of a compound)
    /// becomes the result, a solid when closed (reversed when its infinite point is
    /// inside); false when there is no real assembly.
    pub(super) fn sewing(&mut self) -> Result<bool, Failure> {
        if self.my_top.shape_type()? == ShapeEnum::Wire || !self.my_done {
            return Ok(false);
        }
        let mut ass = BRepBuilderAPISewing::new(5. * self.my_tol, true, true, false, false);
        ass.add(&self.my_shape)?;
        ass.add(&self.my_top)?;
        ass.perform(&ProgressRange::default())?;
        let mut ok: bool = false;
        if ass.nb_contigous_edges() > 0 {
            let res: Shape = ass.sewed_shape().clone();
            match res.shape_type()? {
                ShapeEnum::Shell | ShapeEnum::Solid => {
                    self.my_shape = res;
                    ok = true;
                }
                ShapeEnum::Compound => {
                    let mut it = ShapeIterator::new(&res, true, true);
                    let first: Shape = it.value()?.clone();
                    it.next();
                    if !it.more() {
                        self.my_shape = first;
                        ok = true;
                    }
                }
                _ => {}
            }
        }
        if ok {
            self.sewn_history(&ass)?;
            if self.my_shape.closed()? {
                self.close_solid()?;
            }
        }
        Ok(ok)
    }

    /// The faces and sections of the draft replaced by their sewn shapes.
    fn sewn_history(&mut self, ass: &BRepBuilderAPISewing) -> Result<(), Failure> {
        let nb_law: i32 = self.loc()?.nb_law()?;
        let faces: &mut ShapeTable = self.my_faces.as_mut().ok_or_else(|| null("face table"))?;
        for ii in 1..=nb_law {
            replace_by_sewn(ass, faces, ii)?;
        }
        let sections: &mut ShapeTable = self
            .my_sections
            .as_mut()
            .ok_or_else(|| null("section table"))?;
        for ii in 1..=nb_law + 1 {
            replace_by_sewn(ass, sections, ii)?;
        }
        Ok(())
    }

    /// The solid of the closed shell, reversed when its infinite point is inside.
    fn close_solid(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        let shell = topods::shell(&self.my_shape)?;
        builder.add(&mut solid, &shell)?;
        let mut sc = BRepClass3dSolidClassifier::with_shape(&solid)?;
        sc.perform_infinite_point(precision::confusion())?;
        if sc.state() == State::In {
            builder.make_solid(&mut solid);
            self.my_shape.reverse();
            let shell = topods::shell(&self.my_shape)?;
            builder.add(&mut solid, &shell)?;
        }
        self.my_shape = solid.into_shape();
        Ok(())
    }
}
