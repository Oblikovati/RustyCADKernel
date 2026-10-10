// Rust translation of BRepFill_AdvancedEvolved::GetLids
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The top and bottom lids of the sweep: its free edges parallel to the plane of the
//! spine, split, chained into wires and made planar faces oriented outward.
use super::{AdvancedEvolvedStatus, BRepFillAdvancedEvolved};
use rustycad_tkbo::bop_algo::tools::{edges_to_wires, wires_to_faces};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Xyz};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::BRepLibFindSurface;

/// The free edges of the sweep (with one face) whose tangent at the middle is parallel
/// to the plane of the spine within the angular tolerance (squared).
fn planar_free_edges(
    pipe_shell: &Shape,
    a_normal: Direction3,
    a_sq_angular_tol: f64,
) -> Result<List<Shape>, Failure> {
    let mut a_map_ef = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(pipe_shell, ShapeEnum::Edge, ShapeEnum::Face, &mut a_map_ef)?;
    let mut a_le: List<Shape> = List::default();
    for i in 1..=a_map_ef.extent() {
        if a_map_ef.find_from_index(i)?.len() != 1 {
            continue;
        }
        let an_e = topods::edge(a_map_ef.find_key(i)?)?;
        let an_ac = BRepAdaptorCurve::with_edge(&an_e)?;
        if !an_ac.transformed_curve().is_3d_curve() {
            // Degenerated edges are not of interest.
            continue;
        }
        let (_, a_tan) = an_ac.d1(0.5 * (an_ac.first_parameter()? + an_ac.last_parameter()?))?;
        let a_sq_modulus: f64 = a_tan.square_magnitude();
        if a_sq_modulus < precision::confusion() {
            continue;
        }
        let a_dp: f64 = a_tan.xyz().dot(Xyz::from(a_normal));
        if a_dp * a_dp > a_sq_modulus * a_sq_angular_tol {
            // Only planar edges are considered.
            continue;
        }
        a_le.append(an_e.into_shape());
    }
    Ok(a_le)
}

/// Whether a face of a lid has a vertex of the sweep above (true) or below (false)
/// its plane; `None` when all of them are in the plane.
fn side_of_vertices(a_f: &Face, a_map_v: &IndexedMapOfShape) -> Result<Option<bool>, Failure> {
    let a_pln: Handle<GeomPlane> = BRepTool::surface(a_f)?.cast();
    let position = a_pln.get().ok_or_else(no_plane)?.pln().position();
    let (a_norm, a_loc_p): (Xyz, Xyz) =
        (Xyz::from(position.direction()), position.location().xyz());
    for i in 1..=a_map_v.extent() {
        let a_p: Xyz = BRepTool::pnt(&topods::vertex(a_map_v.find_key(i)?)?)?.xyz();
        let a_delta: Xyz = a_p - a_loc_p;
        let a_sq_d: f64 = a_delta.square_modulus();
        if a_sq_d < precision::square_confusion() {
            continue;
        }
        let a_dp: f64 = a_delta.dot(a_norm);
        if a_dp * a_dp < a_sq_d * precision::square_confusion() {
            // a_p is in the plane.
            continue;
        }
        return Ok(Some(a_dp > 0.0));
    }
    Ok(None)
}

impl BRepFillAdvancedEvolved {
    /// `GetLids()` (protected): the lids of an open profile (status NotSolid; none for
    /// a closed one), NotPlanarSpine when the spine has no plane or no free edge is
    /// planar, NoLids otherwise; `ae.get_lids()?`.
    pub fn get_lids(&mut self) -> Result<(), Failure> {
        if self.my_pipe_shell.is_null() {
            return Ok(());
        }
        if BRepTool::is_closed(self.my_profile.as_ref())? {
            // No need in lids creation.
            self.my_error_status = AdvancedEvolvedStatus::NotSolid;
            return Ok(());
        }
        self.my_error_status = AdvancedEvolvedStatus::NoLids;
        let a_fs = BRepLibFindSurface::new(self.my_spine.as_ref(), -1.0, true, false)?;
        let a_surf: Handle<GeomPlane> = a_fs.surface().cast();
        let Some(plane) = a_surf.get() else {
            self.my_error_status = AdvancedEvolvedStatus::NotPlanarSpine;
            return Ok(());
        };
        let a_tol: f64 = a_fs.tolerance().max(a_fs.tolerance_reached()) + self.my_fuzzy_value;
        let mut a_prof_box = BoundingBox3::default();
        BRepBndLib::add(self.my_profile.as_ref(), &mut a_prof_box, true)?;
        // The square of the default angular tolerance of BOPAlgo_Tools::EdgesToWires
        // and BOPAlgo_Tools::WiresToFaces.
        let a_sq_anguar_tol: f64 = a_tol * a_tol / a_prof_box.square_extent();
        let a_le = planar_free_edges(
            &self.my_pipe_shell,
            plane.pln().position().direction(),
            a_sq_anguar_tol,
        )?;
        if a_le.is_empty() {
            self.my_error_status = AdvancedEvolvedStatus::NotPlanarSpine;
            return Ok(());
        }
        // Split the interfered edges.
        let Some(a_free_edges) = self.perform_boolean(&a_le)? else {
            self.my_error_status = AdvancedEvolvedStatus::NotPlanarSpine;
            return Ok(());
        };
        self.make_lids(&a_free_edges, a_sq_anguar_tol.sqrt())?;
        self.my_error_status = AdvancedEvolvedStatus::NotSolid;
        Ok(())
    }

    /// The free edges chained into wires and made planar faces; each face reversed
    /// when the first vertex of the sweep off its plane is above it.
    fn make_lids(&mut self, a_free_edges: &Shape, an_ang_tol: f64) -> Result<(), Failure> {
        let a_bb = BRepBuilder::default();
        let (mut a_comp_w, mut a_comp_f) = (Compound::default(), Compound::default());
        a_bb.make_compound(&mut a_comp_w);
        a_bb.make_compound(&mut a_comp_f);
        a_bb.make_compound(&mut self.my_top_bottom);
        let mut comp_w: Shape = a_comp_w.into_shape();
        let mut comp_f: Shape = a_comp_f.into_shape();
        edges_to_wires(a_free_edges, &mut comp_w, true, an_ang_tol)?;
        wires_to_faces(&comp_w, &mut comp_f, an_ang_tol)?;
        // Check the orientation.
        let mut a_map_v = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.my_pipe_shell, ShapeEnum::Vertex, &mut a_map_v)?;
        let mut an_exp = Explorer::new(&comp_f, ShapeEnum::Face, ShapeEnum::Shape)?;
        while an_exp.more() {
            let a_f: Face = topods::face(an_exp.current())?;
            let lid: Shape = match side_of_vertices(&a_f, &a_map_v)? {
                Some(true) => a_f.reversed(),
                _ => a_f.into_shape(),
            };
            a_bb.add(&mut self.my_top_bottom, &lid)?;
            an_exp.next()?;
        }
        Ok(())
    }
}

/// Standard_NullObject of a lid without plane (natively a crash).
fn no_plane() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_AdvancedEvolved: lid without plane; expected a planar face",
        "BRepFill_AdvancedEvolved",
    )
}
