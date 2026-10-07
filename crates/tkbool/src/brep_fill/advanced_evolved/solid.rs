// Rust translation of BRepFill_AdvancedEvolved::BuildSolid, ExtractOuterSolid and
// RemoveExcessSolids (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solid of the sweep and its lids: the faces split at their singularities, fused,
//! made volumes, and the solids kept by their lids and the orientation of their faces.
use super::tolerance::{contains_in_list, reduce_vertex_tolerance, remove_internal_wires};
use super::{AdvancedEvolvedStatus, BRepFillAdvancedEvolved};
use rustycad_tkbo::bop_algo::BOPAlgoMakerVolume;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, CompSolid, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The faces of a shape, each once, in explorer order.
fn unique_faces(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut a_map_f = MapOfShape::default();
    let mut out: Vec<Shape> = Vec::new();
    let mut an_exp_f = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while an_exp_f.more() {
        if a_map_f.add(an_exp_f.current().clone()) {
            out.push(an_exp_f.current().clone());
        }
        an_exp_f.next()?;
    }
    Ok(out)
}

/// The list without the first occurrence of each shape (NCollection_List::Remove of a
/// value removes every equal item).
fn remove_value(list: &mut List<Shape>, value: &Shape) {
    let kept: Vec<Shape> = list.iter().filter(|s| *s != value).cloned().collect();
    list.clear();
    for s in kept {
        list.append(s);
    }
}

/// The solids of a shape.
fn solids_of(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)?;
    while ex.more() {
        out.push(ex.current().clone());
        ex.next()?;
    }
    Ok(out)
}

/// The single solid, or a comp-solid of the solids.
fn comp_solid_of(solids: &[Shape]) -> Result<Shape, Failure> {
    let a_bb = BRepBuilder::default();
    let mut a_comp_sol = CompSolid::default();
    a_bb.make_comp_solid(&mut a_comp_sol);
    for s in solids {
        topods::solid(s)?;
        a_bb.add(&mut a_comp_sol, s)?;
    }
    Ok(a_comp_sol.into_shape())
}

impl BRepFillAdvancedEvolved {
    /// `BuildSolid()` (protected): the faces of the sweep (their vertex tolerances
    /// reduced, split at their singularities) fused, made volumes with the lids, the
    /// excess solids removed, unified and without internal wires; status OK;
    /// `ae.build_solid()?`.
    pub fn build_solid(&mut self) -> Result<(), Failure> {
        if self.my_error_status != AdvancedEvolvedStatus::NotSolid {
            return Ok(());
        }
        self.my_error_status = AdvancedEvolvedStatus::NotVolume;
        let (mut a_lf, mut a_l_splits): (List<Shape>, List<Shape>) =
            (List::default(), List::default());
        for f in unique_faces(&self.my_pipe_shell)? {
            reduce_vertex_tolerance(&f)?;
            self.check_singularity_and_add(
                &topods::face(&f)?,
                self.my_fuzzy_value,
                &mut a_lf,
                &mut a_l_splits,
            )?;
        }
        for f in a_l_splits.iter() {
            a_lf.append(f.clone());
        }
        // Split the interfered faces.
        if let Some(shape) = self.perform_boolean(&a_lf)? {
            self.my_pipe_shell = shape;
        }
        let mut a_lf: List<Shape> = List::default();
        for f in unique_faces(&self.my_pipe_shell)? {
            a_lf.append(f);
        }
        if !self.my_top_bottom.is_null() {
            let mut an_it_lids = ShapeIterator::new(self.my_top_bottom.as_ref(), true, true);
            while an_it_lids.more() {
                a_lf.append(an_it_lids.value()?.clone());
                an_it_lids.next();
            }
        }
        self.make_volume(a_lf, &a_l_splits)
    }

    /// The volumes of the faces, then the excess solids removed and the result unified.
    fn make_volume(
        &mut self,
        mut a_lf: List<Shape>,
        a_l_splits: &List<Shape>,
    ) -> Result<(), Failure> {
        let mut a_mv = BOPAlgoMakerVolume::new();
        a_mv.set_arguments(&a_lf);
        a_mv.options_mut().set_fuzzy_value(self.my_fuzzy_value);
        a_mv.set_intersect(true);
        a_mv.options_mut().set_run_parallel(self.my_is_parallel);
        a_mv.set_avoid_internal_shapes(true);
        a_mv.perform();
        if a_mv.options().has_errors() {
            return Ok(());
        }
        self.my_result = a_mv.shape().clone();
        let result = self.my_result.clone();
        self.remove_excess_solids(a_l_splits, &result, &mut a_lf, &a_mv)?;
        self.unify_shape()?;
        remove_internal_wires(&self.my_result)?;
        self.my_error_status = AdvancedEvolvedStatus::Ok;
        Ok(())
    }

    /// `ExtractOuterSolid(theShape, theArgsList)` (protected): the volume of the faces
    /// of the shape that bound one solid only, the other faces removed from the
    /// arguments; `ae.extract_outer_solid(&mut shape, &mut args)?`.
    pub fn extract_outer_solid(
        &self,
        the_shape: &mut Shape,
        the_args_list: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut a_map_s = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            the_shape,
            ShapeEnum::Face,
            ShapeEnum::Solid,
            &mut a_map_s,
        )?;
        let mut a_new_list: List<Shape> = List::default();
        for i in 1..=a_map_s.extent() {
            if a_map_s.find_from_index(i)?.len() == 1 {
                a_new_list.append(a_map_s.find_key(i)?.clone());
            }
        }
        if a_new_list.is_empty() {
            return Ok(());
        }
        while let Some(a_f) = the_args_list
            .iter()
            .find(|f| !contains_in_list(&a_new_list, f))
            .cloned()
        {
            remove_value(the_args_list, &a_f);
        }
        let mut a_mv = BOPAlgoMakerVolume::new();
        a_mv.set_arguments(&a_new_list);
        a_mv.set_intersect(true);
        a_mv.options_mut().set_run_parallel(self.my_is_parallel);
        a_mv.set_avoid_internal_shapes(true);
        a_mv.perform();
        if a_mv.options().has_errors() {
            return Ok(());
        }
        *the_shape = a_mv.shape().clone();
        Ok(())
    }

    /// `RemoveExcessSolids(theLSplits, theShape, theArgsList, theMV)` (protected): the
    /// single solid, else (after extracting the outer solid) the solids containing a
    /// lid as given, else those whose faces kept their argument orientation;
    /// `ae.remove_excess_solids(&splits, &shape, &mut args, &mv)?`.
    pub fn remove_excess_solids(
        &mut self,
        the_l_splits: &List<Shape>,
        the_shape: &Shape,
        the_args_list: &mut List<Shape>,
        the_mv: &BOPAlgoMakerVolume,
    ) -> Result<(), Failure> {
        if self.my_error_status != AdvancedEvolvedStatus::NotVolume {
            return Ok(());
        }
        let mut a_res_shape: Shape = the_shape.clone();
        for i in 0..2 {
            let solids = solids_of(&a_res_shape)?;
            match solids.len() {
                // No solid.
                0 => {
                    self.my_result = a_res_shape;
                    return Ok(());
                }
                // Only one solid has been generated. Natively the explorer is read past
                // its end, which gives the explored shape itself: TopoDS::Solid of it
                // (Standard_TypeMismatch for a compound in the oracle build).
                1 => {
                    self.my_result = topods::solid(&a_res_shape)?.into_shape();
                    return Ok(());
                }
                _ => {}
            }
            if i != 0 {
                break;
            }
            self.extract_outer_solid(&mut a_res_shape, the_args_list)?;
        }
        let lid_solids: Vec<Shape> = self.solids_with_lids(&a_res_shape)?;
        match lid_solids.len() {
            0 => {
                self.my_result = a_res_shape;
                return Ok(());
            }
            1 => {
                self.my_result = lid_solids[0].clone();
                return Ok(());
            }
            _ => a_res_shape = comp_solid_of(&lid_solids)?,
        }
        let valid: Vec<Shape> =
            self.valid_solids(&a_res_shape, the_l_splits, the_args_list, the_mv)?;
        self.my_result = match valid.len() {
            0 => a_res_shape,
            1 => valid[0].clone(),
            _ => comp_solid_of(&valid)?,
        };
        Ok(())
    }

    /// The solids holding their first lid with its orientation (the search stops at
    /// the first solid when there is no lid).
    fn solids_with_lids(&self, a_res_shape: &Shape) -> Result<Vec<Shape>, Failure> {
        let mut a_solid_list: Vec<Shape> = Vec::new();
        for a_sol in solids_of(a_res_shape)? {
            let mut a_map_f = IndexedMapOfShape::default();
            topexp::map_shapes(&a_sol, &mut a_map_f, true, true);
            let mut are_there_lids = false;
            let mut an_exp_lids = Explorer::new(
                self.my_top_bottom.as_ref(),
                ShapeEnum::Face,
                ShapeEnum::Shape,
            )?;
            while an_exp_lids.more() {
                are_there_lids = true;
                let a_f_lid = an_exp_lids.current();
                let a_f_idx = a_map_f.find_index(a_f_lid);
                if a_f_idx < 1 {
                    an_exp_lids.next()?;
                    continue;
                }
                if a_map_f.find_key(a_f_idx)?.is_equal(a_f_lid) {
                    a_solid_list.push(a_sol.clone());
                }
                break;
            }
            if !are_there_lids {
                break;
            }
        }
        Ok(a_solid_list)
    }

    /// The solids without a face whose orientation the volume maker changed (the
    /// splits not checked) and without an argument face of another orientation.
    fn valid_solids(
        &self,
        a_res_shape: &Shape,
        the_l_splits: &List<Shape>,
        the_args_list: &mut List<Shape>,
        the_mv: &BOPAlgoMakerVolume,
    ) -> Result<Vec<Shape>, Failure> {
        // Remove the split faces from the list of arguments.
        for a_f in the_l_splits.iter() {
            remove_value(the_args_list, a_f);
        }
        // The invalid faces: those whose orientation BOPAlgo_MakerVolume changed while
        // creating the solids.
        let mut a_list_inv_faces: Vec<Shape> = Vec::new();
        for a_f in the_args_list.iter() {
            for a_fm in the_mv.modified(a_f)? {
                if a_fm.orientation() != a_f.orientation() {
                    a_list_inv_faces.push(a_fm);
                }
            }
        }
        let mut a_solid_list: Vec<Shape> = Vec::new();
        for a_so in solids_of(a_res_shape)? {
            let mut a_map_f = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(&a_so, ShapeEnum::Face, &mut a_map_f)?;
            if a_list_inv_faces.iter().any(|f| a_map_f.contains(f)) {
                continue;
            }
            let flipped = the_args_list.iter().any(|a_f| {
                let an_idx = a_map_f.find_index(a_f);
                an_idx != 0
                    && a_map_f
                        .find_key(an_idx)
                        .is_ok_and(|a_f1| !a_f.is_equal(a_f1))
            });
            if !flipped {
                a_solid_list.push(a_so);
            }
        }
        Ok(a_solid_list)
    }
}
