// Rust translation of BRepFill_AdvancedEvolved::CheckSingularityAndAdd and of the static
// RebuildFaces (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces of the sweep split where their edges interfere: a planar face rebuilt
//! from its split edges, a cone, sphere, revolution or spline face from its split
//! wires with degenerated edges inserted at the singular vertices.
use super::BRepFillAdvancedEvolved;
use super::degenerated::insert_e_degenerated;
use rustycad_tkbo::bop_algo::tools::{edges_to_wires, wires_to_faces};
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderFace};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

/// `RebuildFaces(theLE, theSourceFace, theList)`: the faces of the edges on the
/// forward source face (the source face when none is built).
fn rebuild_faces(
    the_le: &List<Shape>,
    the_source_face: &Face,
    the_list: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut a_bf = BuilderFace::new();
    a_bf.set_face(&topods::face(
        &the_source_face.oriented(Orientation::Forward),
    )?);
    a_bf.set_shapes(the_le);
    a_bf.perform()?;
    let a_lfr = a_bf.areas();
    if a_lfr.is_empty() {
        the_list.append(the_source_face.as_ref().clone());
        return Ok(());
    }
    for a_fr in a_lfr.iter() {
        the_list.append(topods::face(a_fr)?.into_shape());
    }
    Ok(())
}

/// Whether a pave filler created new shapes (interfered edges were found).
fn has_new_shapes(a_pf: &BOPAlgoPaveFiller) -> bool {
    a_pf.data_structure()
        .is_some_and(|ds| ds.nb_shapes() != ds.nb_source_shapes())
}

/// The surface type of the face, that of the basis of an offset surface.
fn basis_type(the_f: &Face) -> Result<SurfaceType, Failure> {
    let an_as = BRepAdaptorSurface::with_face(the_f, false)?;
    let a_s_type = an_as.get_type()?;
    if a_s_type != SurfaceType::OffsetSurface {
        return Ok(a_s_type);
    }
    let basis = an_as.basis_surface()?;
    basis.get().ok_or_else(null_basis)?.get_type()
}

/// The edges of the face, each once.
fn unique_edges(the_f: &Face) -> Result<List<Shape>, Failure> {
    let mut a_me = MapOfShape::default();
    let mut a_le: List<Shape> = List::default();
    let mut an_exp = Explorer::new(the_f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while an_exp.more() {
        if a_me.add(an_exp.current().clone()) {
            a_le.append(an_exp.current().clone());
        }
        an_exp.next()?;
    }
    Ok(a_le)
}

impl BRepFillAdvancedEvolved {
    /// `CheckSingularityAndAdd(theF, theFuzzyToler, theListOfFaces, theListOfSplits)`
    /// (protected): the face split where its edges interfere (its splits added to the
    /// splits, the cone, sphere, spline and revolution splits also to the faces in the
    /// orientation of the face), else the face added to the faces; true when split;
    /// `ae.check_singularity_and_add(&face, 1e-7, &mut faces, &mut splits)?`.
    pub fn check_singularity_and_add(
        &self,
        the_f: &Face,
        the_fuzzy_toler: f64,
        the_list_of_faces: &mut List<Shape>,
        the_list_of_splits: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let a_s_type = basis_type(the_f)?;
        if a_s_type == SurfaceType::Plane {
            return self.split_planar_face(the_f, the_list_of_faces, the_list_of_splits);
        }
        let singular = matches!(
            a_s_type,
            SurfaceType::Cone
                | SurfaceType::Sphere
                | SurfaceType::BezierSurface
                | SurfaceType::BSplineSurface
                | SurfaceType::SurfaceOfRevolution
        );
        if !singular {
            the_list_of_faces.append(the_f.as_ref().clone());
            return Ok(false);
        }
        let Some(a_list_edges) = self.split_wires(the_f, the_fuzzy_toler)? else {
            the_list_of_faces.append(the_f.as_ref().clone());
            return Ok(false);
        };
        rebuild_faces(&a_list_edges, the_f, the_list_of_splits)?;
        for a_f in the_list_of_splits.iter() {
            the_list_of_faces.append(topods::face(a_f)?.oriented(the_f.orientation()));
        }
        Ok(true)
    }

    /// A planar face: its edges fused; the faces of the wires of the split edges added
    /// to the splits when they interfere, else the face added to the faces.
    fn split_planar_face(
        &self,
        the_f: &Face,
        the_list_of_faces: &mut List<Shape>,
        the_list_of_splits: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let a_le = unique_edges(the_f)?;
        let mut a_pf = BOPAlgoPaveFiller::new();
        a_pf.set_arguments(&a_le);
        a_pf.options_mut().set_run_parallel(self.my_is_parallel);
        a_pf.perform();
        if a_pf.options().has_errors() || !has_new_shapes(&a_pf) {
            // Interfered edges have not been detected.
            the_list_of_faces.append(the_f.as_ref().clone());
            return Ok(false);
        }
        let mut a_builder = BOPAlgoBuilder::new();
        a_builder.set_arguments(&a_le);
        a_builder
            .options_mut()
            .set_run_parallel(self.my_is_parallel);
        a_builder.perform_with_filler(&mut a_pf);
        if a_builder.options().has_errors() {
            the_list_of_faces.append(the_f.as_ref().clone());
            return Ok(false);
        }
        let a_bb = BRepBuilder::default();
        let (mut a_comp_w, mut a_comp_f) = (Compound::default(), Compound::default());
        a_bb.make_compound(&mut a_comp_w);
        a_bb.make_compound(&mut a_comp_f);
        let (mut comp_w, mut comp_f): (Shape, Shape) =
            (a_comp_w.into_shape(), a_comp_f.into_shape());
        edges_to_wires(a_builder.shape(), &mut comp_w, true, 1.0e-8)?;
        wires_to_faces(&comp_w, &mut comp_f, 1.0e-8)?;
        let mut an_exp = Explorer::new(&comp_f, ShapeEnum::Face, ShapeEnum::Shape)?;
        while an_exp.more() {
            the_list_of_splits.append(an_exp.current().clone());
            an_exp.next()?;
        }
        Ok(true)
    }

    /// The edges of the wires of the forward face that interfere, split (in a
    /// non-destructive mode) with degenerated edges inserted; `None` when no wire is
    /// split.
    fn split_wires(
        &self,
        the_f: &Face,
        the_fuzzy_toler: f64,
    ) -> Result<Option<List<Shape>>, Failure> {
        let a_face: Face = topods::face(&the_f.oriented(Orientation::Forward))?;
        let mut is_split = false;
        let mut a_list_edges: List<Shape> = List::default();
        let mut an_exp_w = ShapeIterator::new(a_face.as_ref(), true, true);
        while an_exp_w.more() {
            let a_wir = an_exp_w.value()?.clone();
            an_exp_w.next();
            let Some(mut a_le) = self.split_wire(&a_wir, the_fuzzy_toler)? else {
                continue;
            };
            is_split = true;
            insert_e_degenerated(&a_face, &mut a_le)?;
            a_list_edges.append_list(&mut a_le);
        }
        Ok(is_split.then_some(a_list_edges))
    }

    /// The edges of one wire with the interfering ones replaced by their splits
    /// (degenerated edges kept); `None` when nothing is split.
    fn split_wire(
        &self,
        a_wir: &Shape,
        the_fuzzy_toler: f64,
    ) -> Result<Option<List<Shape>>, Failure> {
        let mut a_lgf: List<Shape> = List::default();
        let mut an_e_exp = Explorer::new(a_wir, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while an_e_exp.more() {
            a_lgf.append(an_e_exp.current().clone());
            an_e_exp.next()?;
        }
        let mut a_pf = BOPAlgoPaveFiller::new();
        a_pf.set_arguments(&a_lgf);
        a_pf.options_mut().set_fuzzy_value(the_fuzzy_toler);
        a_pf.perform();
        if a_pf.options().has_errors() || !has_new_shapes(&a_pf) {
            // No new shapes have been created.
            return Ok(None);
        }
        let mut a_builder = BOPAlgoBuilder::new();
        a_builder.set_arguments(&a_lgf);
        a_builder
            .options_mut()
            .set_run_parallel(self.my_is_parallel);
        a_builder.set_non_destructive(true);
        a_builder.perform_with_filler(&mut a_pf);
        if a_builder.options().has_errors() {
            return Ok(None);
        }
        let mut a_le: List<Shape> = List::default();
        for a_sh in a_lgf.iter() {
            let a_lm = a_builder.modified(a_sh)?;
            if a_lm.is_empty() || BRepTool::degenerated(&topods::edge(a_sh)?)? {
                a_le.append(a_sh.clone());
                continue;
            }
            for an_em in a_lm {
                a_le.append(topods::edge(&an_em)?.into_shape());
            }
        }
        Ok(Some(a_le))
    }
}

/// Standard_NullObject of an offset surface without basis.
fn null_basis() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_AdvancedEvolved: offset surface without basis; expected a surface",
        "BRepFill_AdvancedEvolved",
    )
}
