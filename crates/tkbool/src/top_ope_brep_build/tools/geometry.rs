//! The geometric tools of `TopOpeBRepBuild_Tools`: normals and tangents at the sample
//! point of an edge, the adjacent face, the p-curves moved from a face to another and
//! the face normalizations.
use super::{PAR_T, TopOpeBRepBuildTools, sub_shapes};
use crate::top_ope_brep_build::{MapOfTrans2dInfo, TopOpeBRepBuildCorrectFace2d};
use crate::top_ope_brep_tool::{fc2d_curve_on_surface, min_duv};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, Vector3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The sample parameter `f * PAR_T + (1 - PAR_T) * l`.
fn sample(f: f64, l: f64) -> f64 {
    f * PAR_T + (1. - PAR_T) * l
}

/// The p-curve of the edge on the face (Standard_NullObject when there is none).
fn pcurve(e: &Edge, f: &Face) -> Result<(Handle<dyn Geom2dCurve>, f64, f64, f64), Failure> {
    let (c2d, f2, l2, tolpc) = fc2d_curve_on_surface(e, f, true)?;
    if c2d.is_null() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "TopOpeBRepBuild_Tools: no p-curve of the edge on the face",
        ));
    }
    Ok((c2d, f2, l2, tolpc))
}

/// The cross product of the first derivatives of the face surface at `uv`.
fn surface_normal(face: &Face, uv: Point2) -> Result<Vector3, Failure> {
    let d1 = BRepAdaptorSurface::with_face(face, true)?.d1(uv.x(), uv.y())?;
    Ok(d1.d1u.crossed(d1.d1v))
}

/// The 2d curve of a geometry copy.
fn copy2d(c: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let curve = c.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "TopOpeBRepBuild_Tools: null 2d curve",
        )
    })?;
    Ok(curve.copy()?.cast::<dyn Geom2dCurve>())
}

impl TopOpeBRepBuildTools {
    /// `GetNormalToFaceOnEdge(aFObj, anEdgeObj, aNormal)`: the (not normalized) normal of
    /// the face at the sample point of the edge p-curve;
    /// `TopOpeBRepBuildTools::get_normal_to_face_on_edge(&f, &e)?`.
    pub fn get_normal_to_face_on_edge(f_obj: &Face, edge_obj: &Edge) -> Result<Vector3, Failure> {
        let (c2d, _, _, _) = pcurve(edge_obj, f_obj)?;
        let ca = BRepAdaptorCurve::with_edge(edge_obj)?;
        let par = sample(ca.first_parameter()?, ca.last_parameter()?);
        let uv = c2d.get().expect("checked p-curve").value(par)?;
        surface_normal(f_obj, uv)
    }

    /// `GetNormalInNearestPoint(F, E, aNormal)`: the face normal at a point moved from the
    /// sample point of the p-curve by a hundredth of the smallest face span, to the
    /// left of the p-curve (to the right when the left point is out of the face);
    /// `TopOpeBRepBuildTools::get_normal_in_nearest_point(&f, &e)?`.
    pub fn get_normal_in_nearest_point(f: &Face, e: &Edge) -> Result<Vector3, Failure> {
        let (c2d, f2, l2, _) = pcurve(e, f)?;
        let (p, tangent) = c2d.get().expect("checked p-curve").d1(sample(f2, l2))?;
        let step = min_duv(f)? * 1e-2;
        let mut step_v = Direction2::new(-tangent.y(), tangent.x())?;
        let moved = |d: Direction2| Point2::new(p.x() + step * d.x(), p.y() + step * d.y());
        let mut uv = moved(step_v);
        let fc = BRepTopAdaptorFClass2d::new(f, precision::p_confusion())?;
        if fc.perform(uv, true)? == State::Out {
            step_v.reverse();
            uv = moved(step_v);
        }
        surface_normal(f, uv)
    }

    /// `GetTangentToEdgeEdge(aFObj, anEdgeObj, aOriEObj, aTangent)`: the tangent of the
    /// piece at its sample point (reversed for a reversed piece) and true when it is
    /// opposite to the original edge's (then also reversed); the plain tangent for a
    /// degenerated edge; `TopOpeBRepBuildTools::get_tangent_to_edge_edge(&f, &piece, &e)?`.
    pub fn get_tangent_to_edge_edge(
        _f_obj: &Face,
        edge_obj: &Edge,
        ori_e_obj: &Edge,
    ) -> Result<(bool, Vector3), Failure> {
        if BRepTool::degenerated(ori_e_obj)? || BRepTool::degenerated(edge_obj)? {
            let (ok, t) = Self::get_tangent_to_edge(edge_obj)?;
            return Ok((ok, t));
        }
        let ca = BRepAdaptorCurve::with_edge(edge_obj)?;
        let par = sample(ca.first_parameter()?, ca.last_parameter()?);
        let (p, tg_piece) = ca.d1(par)?;
        let mut tangent = tg_piece;
        let ori_curve = BRepAdaptorCurve::with_edge(ori_e_obj)?
            .transformed_curve()
            .curve()
            .get()
            .map(|c| c.curve())
            .unwrap_or_default();
        let copy = ori_curve
            .get()
            .ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "TopOpeBRepBuild_Tools::GetTangentToEdgeEdge: no 3d curve",
                )
            })?
            .transformed(ori_e_obj.location().transformation())?
            .cast::<dyn GeomCurve>();
        let c = copy.get().expect("transformed copy");
        let pp =
            ProjectPointOnCurve::with_range(p, &copy, c.first_parameter(), c.last_parameter())?;
        let (_, tg_ori) = c.d1(pp.lower_distance_parameter()?)?;
        if edge_obj.orientation() == Orientation::Reversed {
            tangent.reverse();
        }
        if tg_ori.dot(tg_piece) < 0. {
            tangent.reverse();
            return Ok((true, tangent));
        }
        Ok((false, tangent))
    }

    /// `GetTangentToEdge(anEdgeObj, aTangent)`: the tangent of the edge curve at its
    /// sample point (always true); `TopOpeBRepBuildTools::get_tangent_to_edge(&e)?`.
    pub fn get_tangent_to_edge(edge_obj: &Edge) -> Result<(bool, Vector3), Failure> {
        let ca = BRepAdaptorCurve::with_edge(edge_obj)?;
        let par = sample(ca.first_parameter()?, ca.last_parameter()?);
        Ok((true, ca.d1(par)?.1))
    }

    /// `GetAdjacentFace(aFaceObj, anEObj, anEdgeFaceMap, anAdjFaceObj)`: the first face of
    /// the edge not equal to the face (None when none);
    /// `TopOpeBRepBuildTools::get_adjacent_face(&f, &e, &map)?`.
    pub fn get_adjacent_face(
        face_obj: &Shape,
        e_obj: &Shape,
        edge_face_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<Face>, Failure> {
        let adj = edge_face_map
            .find_from_key(e_obj)?
            .iter()
            .find(|f| *f != face_obj)
            .cloned();
        adj.map(|s| topods::face(&s)).transpose()
    }

    /// `UpdatePCurves(aWire, fromFace, toFace)`: each edge of the wire updated on
    /// `to_face`; `TopOpeBRepBuildTools::update_pcurves(&w, &f1, &f2)?`.
    pub fn update_pcurves(wire: &Wire, from_face: &Face, to_face: &Face) -> Result<(), Failure> {
        for e in Explorer::new(wire.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            Self::update_edge_on_face(&topods::edge(&e)?, from_face, to_face)?;
        }
        Ok(())
    }

    /// `UpdateEdgeOnFace(aEdgeToUpdate, fromFace, toFace)`: the p-curve of the edge on
    /// `to_face` stored: a trimmed copy of the one on `from_face` for a degenerated edge,
    /// the seam pair for an edge closed on `from_face`, else its p-curve on `to_face`;
    /// `TopOpeBRepBuildTools::update_edge_on_face(&e, &f1, &f2)?`.
    pub fn update_edge_on_face(e: &Edge, from_face: &Face, to_face: &Face) -> Result<(), Failure> {
        let tol_e = BRepTool::edge_tolerance(e)?;
        let bb = BRepBuilder::default();
        if BRepTool::degenerated(e)? {
            let (c2d, f2, l2, tolpc) = fc2d_curve_on_surface(e, from_face, true)?;
            let trimmed: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dTrimmedCurve::new(&copy2d(&c2d)?, f2, l2, true, true)?).cast();
            return bb.update_edge_pcurve_on_face(e, &trimmed, to_face, tol_e.max(tolpc));
        }
        if BRepTool::is_closed_on_face(e, from_face)? {
            return Self::update_edge_on_periodical_face(e, from_face, to_face);
        }
        let (c2d, _, _, tolpc) = fc2d_curve_on_surface(e, to_face, true)?;
        bb.update_edge_pcurve_on_face(e, &c2d, to_face, tol_e.max(tolpc))
    }

    /// `IsDegEdgesTheSame(anE1, anE2)`: whether the first vertices of the two edges are
    /// the same (false when one has none);
    /// `TopOpeBRepBuildTools::is_deg_edges_the_same(&e1, &e2)?`.
    pub fn is_deg_edges_the_same(e1: &Shape, e2: &Shape) -> Result<bool, Failure> {
        let (m1, m2) = (
            sub_shapes(e1, ShapeEnum::Vertex)?,
            sub_shapes(e2, ShapeEnum::Vertex)?,
        );
        if m1.extent() == 0 || m2.extent() == 0 {
            return Ok(false);
        }
        Ok(m1.find_key(1)?.is_same(m2.find_key(1)?))
    }

    /// The wire without its EXTERNAL and INTERNAL edges (None when no edge is left),
    /// oriented as `w`.
    fn normalized_wire(bb: &BRepBuilder, w: &Shape) -> Result<Option<Wire>, Failure> {
        let mut new_wire = Wire::default();
        bb.make_wire(&mut new_wire);
        let mut nb_good = 0;
        for e in Explorer::new(
            &w.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            if matches!(
                e.orientation(),
                Orientation::External | Orientation::Internal
            ) {
                continue;
            }
            bb.add(&mut new_wire, &e)?;
            nb_good += 1;
        }
        new_wire.set_orientation(w.orientation());
        Ok((nb_good > 0).then_some(new_wire))
    }

    /// `NormalizeFace(oldFace, corrFace)`: a copy of the face on its surface with the
    /// EXTERNAL and INTERNAL edges removed from its wires (the empty wires dropped);
    /// `TopOpeBRepBuildTools::normalize_face(&f)?`.
    pub fn normalize_face(old_face: &Shape) -> Result<Shape, Failure> {
        let f1 = topods::face(&old_face.oriented(Orientation::Forward))?;
        let (surf, loc) = BRepTool::surface_with_location(&f1)?;
        let tol_f1 = BRepTool::face_tolerance(&f1)?;
        let bb = BRepBuilder::default();
        let mut new_face = Face::default();
        bb.make_face_on_located_surface(&mut new_face, &surf, &loc, tol_f1)?;
        for w in Explorer::new(f1.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)? {
            if let Some(nw) = Self::normalized_wire(&bb, &w)? {
                bb.add(&mut new_face, &nw)?;
            }
        }
        new_face.set_orientation(old_face.orientation());
        Ok(new_face.into())
    }

    /// `CorrectFace2d(oldFace, corrFace, aSourceShapes, aMapOfCorrect2dEdges)`: the 2d
    /// correction of the face performed (its edges moved in place), the face returned
    /// as is; `TopOpeBRepBuildTools::correct_face2d(&f, &src, &mut map)?`.
    pub fn correct_face2d(
        old_face: &Shape,
        source_shapes: &IndexedMap<Shape>,
        map_of_correct2d_edges: &mut MapOfTrans2dInfo,
    ) -> Result<Shape, Failure> {
        let mut cf = TopOpeBRepBuildCorrectFace2d::with_face(
            &topods::face(old_face)?,
            source_shapes,
            map_of_correct2d_edges,
        );
        cf.perform()?;
        Ok(old_face.clone())
    }

    /// The translation of the seam p-curve: from the first point of the forward p-curve
    /// on `from_face` to the reversed one's (the inverse for different orientations).
    fn seam_translation(
        e: &Edge,
        from_face: &Face,
        diff_oriented: bool,
    ) -> Result<Vector2, Failure> {
        let pc_for = BRepTool::curve_on_face(e, from_face)?;
        let pc_rev = BRepTool::curve_on_face(&topods::edge(&e.reversed())?, from_face)?;
        let null = || {
            Failure::with_message(
                "Standard_NullObject",
                "TopOpeBRepBuild_Tools::UpdateEdgeOnPeriodicalFace: no seam p-curve",
            )
        };
        let uvf = pc_for.pcurve.get().ok_or_else(null)?.value(pc_for.first)?;
        let uvr = pc_rev.pcurve.get().ok_or_else(null)?.value(pc_rev.first)?;
        Ok(if diff_oriented {
            Vector2::between(uvr, uvf)
        } else {
            Vector2::between(uvf, uvr)
        })
    }

    /// Whether the p-curve goes first in the seam pair: by its V direction (downwards),
    /// along U (forwards) when it is horizontal.
    fn first_order(c2d: &Handle<dyn Geom2dCurve>, f2: f64, l2: f64) -> Result<bool, Failure> {
        let c = c2d.get().expect("checked p-curve");
        let c2d_vec = Vector2::between(c.value(f2)?, c.value(l2)?);
        let scalar = Vector2::new(0., 1.).dot(c2d_vec);
        if scalar.abs() <= 1e-10 {
            return Ok(Vector2::new(1., 0.).dot(c2d_vec) >= 0.);
        }
        Ok(scalar <= 0.)
    }

    /// `UpdateEdgeOnPeriodicalFace(aEdgeToUpdate, fromFace, toFace)`: the seam p-curves on
    /// `to_face` of an edge closed on `from_face` (nothing when it has a p-curve on
    /// `to_face`): its p-curve on `to_face` and a copy translated by the seam offset of
    /// `from_face`, ordered by the offset and the p-curve directions;
    /// `TopOpeBRepBuildTools::update_edge_on_periodical_face(&e, &f1, &f2)?`.
    pub fn update_edge_on_periodical_face(
        e: &Edge,
        from_face: &Face,
        to_face: &Face,
    ) -> Result<(), Failure> {
        if !BRepTool::curve_on_face(e, to_face)?.pcurve.is_null() {
            return Ok(());
        }
        let n1 = Self::get_normal_to_face_on_edge(from_face, e)?;
        let n2 = Self::get_normal_to_face_on_edge(to_face, e)?;
        let diff_oriented = n1.dot(n2) < 0.;
        let tol_e = BRepTool::edge_tolerance(e)?;
        let (c2d, f2, l2, tolpc) = pcurve(e, to_face)?;
        let tol = tolpc.max(tol_e);
        let tr_v = Self::seam_translation(e, from_face, diff_oriented)?;
        let dir = Vector2::new(1., 1.).dot(tr_v) >= 0.;
        let first_order = Self::first_order(&c2d, f2, l2)?;
        let tr_c = copy2d(&c2d)?;
        tr_c.get().expect("copied p-curve").translate(tr_v)?;
        let bb = BRepBuilder::default();
        if dir == first_order {
            bb.update_edge_pcurves_on_face(e, (&c2d, &tr_c), to_face, tol)
        } else {
            bb.update_edge_pcurves_on_face(e, (&tr_c, &c2d), to_face, tol)
        }
    }
}
