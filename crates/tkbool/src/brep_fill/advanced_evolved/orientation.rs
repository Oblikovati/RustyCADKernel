// Rust translation of BRepFill_AdvancedEvolved::GetSpineAndProfile
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The spine and the profile, the profile reversed so that it runs from the spine to
//! the side of its normal at the point nearest to the spine.
use super::BRepFillAdvancedEvolved;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Vector3};
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The tangents at a vertex of its first and last edges, both evaluated on the first
/// edge (`anAC1.D1(aPar2, ...)` natively: the second parameter on the first curve).
fn vertex_tangents(
    v: &Vertex,
    map: &IndexedDataMapOfShapeListOfShape,
) -> Result<(Vector3, Vector3), Failure> {
    let a_le = map.find_from_key(v.as_ref())?;
    let an_e1: Edge = topods::edge(a_le.first().ok_or_else(no_edge)?)?;
    let an_e2: Edge = topods::edge(a_le.last().ok_or_else(no_edge)?)?;
    let an_ac1 = BRepAdaptorCurve::with_edge(&an_e1)?;
    let _an_ac2 = BRepAdaptorCurve::with_edge(&an_e2)?;
    let a_par1: f64 = BRepTool::parameter(v, &an_e1)?;
    let a_par2: f64 = BRepTool::parameter(v, &an_e2)?;
    let (_, a_t1) = an_ac1.d1(a_par1)?;
    let (_, a_t2) = an_ac1.d1(a_par2)?;
    Ok((a_t1, a_t2))
}

/// The normal of the profile at its first vertex of two edges whose tangents cross,
/// with that vertex; a null normal when there is none.
fn profile_normal(the_profile: &Wire) -> Result<(Vector3, Point3), Failure> {
    let mut a_mvep = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        the_profile.as_ref(),
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut a_mvep,
    )?;
    for i in 1..=a_mvep.extent() {
        if a_mvep.find_from_index(i)?.len() < 2 {
            continue;
        }
        let a_vc: Vertex = topods::vertex(a_mvep.find_key(i)?)?;
        let (a_t1, a_t2) = vertex_tangents(&a_vc, &a_mvep)?;
        let a_n2: Vector3 = a_t1.crossed(a_t2);
        if a_n2.square_magnitude() > precision::square_confusion() {
            return Ok((a_n2, BRepTool::pnt(&a_vc)?));
        }
    }
    Ok((Vector3::default(), Point3::default()))
}

/// The second shape of the distance: the face of the profile in its plane, else the
/// profile; `None` when the face cannot be made.
fn profile_shape(
    the_profile: &Wire,
    a_n2: Vector3,
    a_loc: Point3,
) -> Result<Option<Shape>, Failure> {
    if a_n2.square_magnitude() <= precision::square_confusion() {
        return Ok(Some(the_profile.as_ref().clone()));
    }
    let a_pln = Plane3::new(a_loc, Direction3::from_vector(a_n2)?)?;
    let a_mf = BRepLibMakeFace::from_plane_wire(a_pln, the_profile, true)?;
    if !a_mf.is_done() {
        return Ok(None);
    }
    Ok(Some(a_mf.face()?.into_shape()))
}

/// The index of the solution of minimal distance: the last one, every `Value()` being
/// the same minimal distance (a native quirk).
fn last_minimal(an_extr: &BRepExtremaDistShapeShape) -> Result<(i32, f64), Failure> {
    let (mut a_dist_min, mut an_idx_min) = (f64::MAX, 0);
    for a_sol_id in 1..=an_extr.nb_solution() {
        let a_d: f64 = an_extr.value()?;
        if a_d > a_dist_min {
            continue;
        }
        (a_dist_min, an_idx_min) = (a_d, a_sol_id);
    }
    Ok((an_idx_min, a_dist_min))
}

impl BRepFillAdvancedEvolved {
    /// `GetSpineAndProfile(theSpine, theProfile)` (protected): the spine and the
    /// profile, reversed by the side of the profile at the solution of the distance
    /// between them; `ae.get_spine_and_profile(&spine, &profile)?`.
    pub fn get_spine_and_profile(
        &mut self,
        the_spine: &Wire,
        the_profile: &Wire,
    ) -> Result<(), Failure> {
        self.my_spine = the_spine.clone();
        self.my_profile = the_profile.clone();
        let (a_n2, a_loc) = profile_normal(the_profile)?;
        let mut an_extr = BRepExtremaDistShapeShape::new();
        an_extr.load_s1(the_spine.as_ref())?;
        let Some(s2) = profile_shape(the_profile, a_n2, a_loc)? else {
            return Ok(());
        };
        an_extr.load_s2(&s2)?;
        if !an_extr.perform(&ProgressRange::default())? || an_extr.nb_solution() < 1 {
            return Ok(());
        }
        let (an_idx_min, a_dist_min) = last_minimal(&an_extr)?;
        let mut an_extr_type2 = an_extr.support_type_shape2(an_idx_min)?;
        if a_dist_min < precision::confusion() {
            an_extr_type2 = BRepExtremaSupportType::IsInFace;
        }
        match an_extr_type2 {
            BRepExtremaSupportType::IsInFace => self.orient_in_face(&an_extr, an_idx_min, a_n2),
            _ => self.orient_on_spine_face(&an_extr, an_idx_min),
        }
    }

    /// The solution in the face of the profile: reversed when the spine tangent (of
    /// minimal sine with the normal at a vertex) goes along the normal.
    fn orient_in_face(
        &mut self,
        an_extr: &BRepExtremaDistShapeShape,
        an_idx_min: i32,
        a_n2: Vector3,
    ) -> Result<(), Failure> {
        let a_t: Vector3 =
            if an_extr.support_type_shape1(an_idx_min)? == BRepExtremaSupportType::IsVertex {
                let a_v: Vertex = topods::vertex(&an_extr.support_on_shape1(an_idx_min)?)?;
                let mut a_mves = IndexedDataMapOfShapeListOfShape::default();
                topexp::map_shapes_and_ancestors(
                    self.my_spine.as_ref(),
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut a_mves,
                )?;
                let (a_t1, a_t2) = vertex_tangents(&a_v, &a_mves)?;
                // Find the minimal sine.
                let a_sq_t1: f64 = a_t1.square_magnitude().max(1.0 / precision::infinite());
                let a_sq_t2: f64 = a_t2.square_magnitude().max(1.0 / precision::infinite());
                let a_sq_sin1: f64 = a_t1.cross_square_magnitude(a_n2) / a_sq_t1;
                let a_sq_sin2: f64 = a_t2.cross_square_magnitude(a_n2) / a_sq_t2;
                if a_sq_sin1 < a_sq_sin2 { a_t1 } else { a_t2 }
            } else {
                let an_e: Edge = topods::edge(&an_extr.support_on_shape1(an_idx_min)?)?;
                let an_ac = BRepAdaptorCurve::with_edge(&an_e)?;
                an_ac.d1(an_extr.par_on_edge_s1(an_idx_min)?)?.1
            };
        if a_t.dot(a_n2) > 0.0 {
            self.my_profile.reverse();
        }
        Ok(())
    }

    /// The solution on an edge or a vertex of the profile: reversed by the profile
    /// tangent there (of maximal cosine with the spine normal at a vertex) and the side
    /// of the nearest point to the face of the spine.
    fn orient_on_spine_face(
        &mut self,
        an_extr: &BRepExtremaDistShapeShape,
        an_idx_min: i32,
    ) -> Result<(), Failure> {
        let a_mk_f_spine = BRepLibMakeFace::from_wire(&self.my_spine, true)?;
        if !a_mk_f_spine.is_done() {
            return Ok(());
        }
        let a_f_spine = a_mk_f_spine.face()?;
        let a_pln_spine: Handle<GeomPlane> = BRepTool::surface(&a_f_spine)?.cast();
        let a_n1 = Vector3::from(
            a_pln_spine
                .get()
                .ok_or_else(no_edge)?
                .pln()
                .axis()
                .direction(),
        );
        let a_tan_v: Vector3 = self.profile_tangent(an_extr, an_idx_min, a_n1)?;
        // The point in the profile which is the nearest to the spine.
        let a_pnear = an_extr.point_on_shape2(an_idx_min)?;
        let a_f_class = BRepClassFaceClassifier::with_face_point3d(
            &a_f_spine,
            &a_pnear,
            precision::confusion(),
            false,
            0.1,
        )?;
        let reverse: bool = if a_f_class.state() != State::Out {
            a_n1.dot(a_tan_v) < 0.0
        } else {
            a_n1.dot(a_tan_v) > 0.0
        };
        if reverse {
            self.my_profile.reverse();
        }
        Ok(())
    }

    /// The tangent of the profile at the solution: at a vertex, the edge tangent of
    /// maximal cosine with the spine normal (a null tangent counting as RealLast).
    fn profile_tangent(
        &self,
        an_extr: &BRepExtremaDistShapeShape,
        an_idx_min: i32,
        a_n1: Vector3,
    ) -> Result<Vector3, Failure> {
        if an_extr.support_type_shape2(an_idx_min)? != BRepExtremaSupportType::IsVertex {
            let an_e: Edge = topods::edge(&an_extr.support_on_shape2(an_idx_min)?)?;
            let an_ac = BRepAdaptorCurve::with_edge(&an_e)?;
            return Ok(an_ac.d1(an_extr.par_on_edge_s2(an_idx_min)?)?.1);
        }
        let a_v: Vertex = topods::vertex(&an_extr.support_on_shape2(an_idx_min)?)?;
        let mut a_mves = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            self.my_profile.as_ref(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut a_mves,
        )?;
        let (a_t1, a_t2) = vertex_tangents(&a_v, &a_mves)?;
        // Find the maximal cosine.
        let null = |sq: f64| {
            if sq < precision::square_confusion() {
                f64::MAX
            } else {
                sq
            }
        };
        let (a_sq_t1, a_sq_t2) = (null(a_t1.square_magnitude()), null(a_t2.square_magnitude()));
        let (a_dp1, a_dp2) = (a_t1.dot(a_n1), a_t2.dot(a_n1));
        Ok(if a_dp1 * a_dp1 * a_sq_t2 > a_dp2 * a_dp2 * a_sq_t1 {
            a_t1
        } else {
            a_t2
        })
    }
}

/// Standard_NoSuchObject of a vertex without edge or a spine without plane.
fn no_edge() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "BRepFill_AdvancedEvolved: vertex without edge or spine without plane; expected one",
        "BRepFill_AdvancedEvolved",
    )
}
