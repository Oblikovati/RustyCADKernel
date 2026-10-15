// Rust translation of BRepFeat_RibSlot::LFPerform of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LFPerform`: the form glued on the basis when its glued faces lie inside the basis
//! faces and the gluer makes the expected operation, else the Boolean operation of the
//! feature builder keeping the parts of the form holding both ends of the wire.
use super::RibSlot;
use crate::brep_feat::{BRepFeatBuilder, PerfSelection, StatusError, is_inside};
use crate::loc_ope::{FindEdges, Gluer, Operation};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
use rustycad_tktopalgo::brep_lib;

impl RibSlot {
    /// The gluer of the form on the basis, or None (operation 2) when a glued face is
    /// not inside its basis face or the gluing is not the expected operation.
    fn prepare_glue(&self) -> Result<Option<Gluer>, Failure> {
        let mut the_glue = Gluer::new();
        let mut collage = true;
        the_glue.init(&self.sbase, &self.g_shape);
        for (glface, fac) in self.glued_f.items() {
            if !Explorer::new(&self.g_shape, ShapeEnum::Face, ShapeEnum::Shape)?
                .any(|x| x.is_same(glface))
            {
                continue;
            }
            let (glface, fac) = (topods::face(glface)?, topods::face(fac)?);
            collage = is_inside(&glface, &fac)?;
            if !collage {
                return Ok(None);
            }
            the_glue.bind_faces(&glface, &fac)?;
            let mut the_fe = FindEdges::new();
            the_fe.set(&glface, &fac)?;
            the_fe.init_iterator();
            while the_fe.more() {
                the_glue.bind_edges(&the_fe.edge_from()?, &the_fe.edge_to()?)?;
                the_fe.next();
            }
        }
        let ope = the_glue.ope_type();
        if ope == Operation::Invalid
            || (self.fuse && ope != Operation::Fuse)
            || (!self.fuse && ope != Operation::Cut)
            || !collage
        {
            return Ok(None);
        }
        Ok(Some(the_glue))
    }

    /// The gluing performed: the result and the history of the gluer (false when not
    /// done).
    fn glue(&mut self, mut the_glue: Gluer) -> Result<bool, Failure> {
        the_glue.perform()?;
        if !the_glue.is_done() {
            return Ok(false);
        }
        self.update_descendants_glue(&the_glue)?;
        self.new_edges = the_glue.edges().clone();
        self.tgt_edges = the_glue.tgt_edges().clone();
        self.base.set_done();
        self.base.my_shape = the_glue.resulting_shape()?.clone();
        brep_lib::same_parameter_shape(&self.base.my_shape, 1.0e-7, true)?;
        Ok(true)
    }

    /// The parts of the tool whose classification keeps both end points of the wire
    /// (not OUT).
    fn keep_end_parts(&self, the_builder: &mut BRepFeatBuilder) -> Result<(), Failure> {
        let mut partsoftool = List::default();
        the_builder.parts_of_tool(&mut partsoftool)?;
        if partsoftool.is_empty() || self.perf_selection == PerfSelection::NoSelection {
            return Ok(());
        }
        let toler = BRepTool::face_tolerance(&self.pbase)? * 2.;
        let mut oussa = BRepClass3dSolidClassifier::new();
        for s in partsoftool.iter() {
            oussa.load(s)?;
            oussa.perform(&self.first_pnt, toler)?;
            let sp1 = oussa.state();
            oussa.perform(&self.last_pnt, toler)?;
            let sp2 = oussa.state();
            if sp1 != State::Out && sp2 != State::Out {
                the_builder.keep_part(s)?;
            }
        }
        Ok(())
    }

    /// The Boolean operation of the form with the basis (operation 2).
    fn boolean(&mut self) -> Result<(), Failure> {
        let mut the_builder = BRepFeatBuilder::new();
        let b_flag = self.perf_selection != PerfSelection::NoSelection;
        the_builder.init_with_tool(&self.sbase, &self.g_shape);
        the_builder.set_operation_with_flag(self.fuse as i32, b_flag);
        the_builder.perform();
        if b_flag {
            self.keep_end_parts(&mut the_builder)?;
            the_builder.perform_result()?;
        }
        self.base.my_shape = the_builder.shape().clone();
        self.base.set_done();
        Ok(())
    }

    /// `LFPerform()`: the form glued on or combined with the basis (NotInitialized
    /// without basis, profile, sketch face, form or profile map); `rib.lf_perform()?`.
    pub(crate) fn lf_perform(&mut self) -> Result<(), Failure> {
        if self.sbase.is_null()
            || self.pbase.is_null()
            || self.skface.is_null()
            || self.g_shape.is_null()
            || self.lf_map.is_empty()
        {
            self.status_error = StatusError::NotInitialized;
            self.base.set_not_done();
            return Ok(());
        }
        // Hope that there is just a solid in the result (natively a search of the faces
        // of the shape Until in the basis, whose outcome is unused)
        let mut the_ope = if self.glued_f.is_empty() { 2 } else { 1 };
        // case of gluing
        let mut the_glue = None;
        if the_ope == 1 {
            the_glue = self.prepare_glue()?;
            if the_glue.is_none() {
                the_ope = 2;
            }
        }
        // gluing is always applicable
        if let Some(g) = the_glue
            && !self.glue(g)?
        {
            the_ope = 2;
        }
        // case without gluing
        if the_ope == 2 {
            self.boolean()?;
        }
        Ok(())
    }

    /// The shape of the result (`myShape`); `rib.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.base.my_shape
    }
}
