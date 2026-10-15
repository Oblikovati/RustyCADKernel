// Rust translation of ChFi3d_Builder::FindFace (both overloads), ChFi3d_Builder::MoreSurfdata
// and the file statics cherche_face, cherche_edge1, containV, containE and IsShrink
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face closing a corner and the faces and edges around a vertex.
use super::ChFi3dBuilder;
use crate::chfi_ds::{CommonPoint, lock};
use crate::chfi3d::{index_of_surf_data, surf_data_at};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::math::FunctionSample;

/// The sub-shapes of a type of a shape.
fn sub_shapes(s: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, kind, &mut m)?;
    Ok(m)
}

/// `cherche_face(map, E, F1, F2, F3, F)`: the face of the list other than the three
/// containing the edge (Standard_ConstructionError when none).
pub(super) fn cherche_face(
    map: &List<Shape>,
    e: &Edge,
    (f1, f2, f3): (&Face, &Face, &Face),
) -> Result<Face, Failure> {
    for s in map.iter() {
        let fcur = topods::face(s)?;
        if fcur.is_same(f1) || fcur.is_same(f2) || fcur.is_same(f3) {
            continue;
        }
        if contain_e(&fcur, e)? {
            return Ok(fcur);
        }
    }
    Err(Failure::with_message(
        "Standard_ConstructionError",
        "Failed to find face.",
    ))
}

/// `cherche_edge1(F1, F2, Edge)`: the first edge common to the two faces
/// (Standard_ConstructionError when none).
pub(super) fn cherche_edge1(f1: &Face, f2: &Face) -> Result<Edge, Failure> {
    let (m1, m2) = (
        sub_shapes(f1.as_ref(), ShapeEnum::Edge)?,
        sub_shapes(f2.as_ref(), ShapeEnum::Edge)?,
    );
    for i in 1..=m1.extent() {
        let e1 = m1.find_key(i)?;
        for j in 1..=m2.extent() {
            if m2.find_key(j)?.is_same(e1) {
                return topods::edge(e1);
            }
        }
    }
    Err(Failure::with_message(
        "Standard_ConstructionError",
        "Failed to find edge.",
    ))
}

/// `containV(F1, V)`: the face has the vertex.
pub(super) fn contain_v(f1: &Face, v: &Vertex) -> Result<bool, Failure> {
    let m = sub_shapes(f1.as_ref(), ShapeEnum::Vertex)?;
    Ok((1..=m.extent()).any(|i| m.find_key(i).is_ok_and(|s| s.is_same(v.as_ref()))))
}

/// `containE(F1, E)`: the face has the edge.
pub(super) fn contain_e(f1: &Face, e: &Edge) -> Result<bool, Failure> {
    let m = sub_shapes(f1.as_ref(), ShapeEnum::Edge)?;
    Ok((1..=m.extent()).any(|i| m.find_key(i).is_ok_and(|s| s.is_same(e.as_ref()))))
}

/// `IsShrink(PC, Pf, Pl, Param, isU, tol)`: the pcurve stays on the iso `param` (of U or
/// V) between `pf` and `pl` (a line, or a spline sampled at 10 points).
pub(super) fn is_shrink(
    pc: &Geom2dAdaptorCurve,
    (pf, pl): (f64, f64),
    param: f64,
    is_u: bool,
    tol: f64,
) -> Result<bool, Failure> {
    let coord = |p: rustycad_tkmath::gp::Point2| if is_u { p.x() } else { p.y() };
    match pc.get_type()? {
        CurveType::Line => Ok((coord(pc.value(pf)?) - param).abs() <= tol
            && (coord(pc.value(pl)?) - param).abs() <= tol),
        CurveType::BezierCurve | CurveType::BSplineCurve => {
            let sample = FunctionSample::new(pf, pl, 10);
            for i in 1..=sample.number_of_points() {
                if (coord(pc.value(sample.parameter(i))?) - param).abs() > tol {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

impl ChFi3dBuilder {
    /// `FindFace(V, P1, P2, Fv, Favoid)`: a face other than `favoid` containing the arcs of
    /// the two common points (`fv` left at the last face examined).
    pub(crate) fn find_face_avoid(
        &self,
        _v: &Vertex,
        (p1, p2): (&CommonPoint, &CommonPoint),
        fv: &mut Face,
        favoid: &Face,
    ) -> Result<bool, Failure> {
        if !(p1.is_on_arc() && p2.is_on_arc()) {
            return Ok(false);
        }
        let l2 = self.my_ef_map.find_from_key(p2.arc()?.as_ref())?;
        for s in self.my_ef_map.find_from_key(p1.arc()?.as_ref())?.iter() {
            *fv = topods::face(s)?;
            if !fv.is_same(favoid) && l2.iter().any(|f| f.is_same(fv.as_ref())) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `FindFace(V, P1, P2, Fv)`: a face containing the arcs of the two common points.
    pub(crate) fn find_face(
        &self,
        v: &Vertex,
        p: (&CommonPoint, &CommonPoint),
        fv: &mut Face,
    ) -> Result<bool, Failure> {
        self.find_face_avoid(v, p, fv, &Face::default())
    }

    /// The first and second edges at the vertex lying on the face.
    fn edges_on_face(&self, vtx: &Vertex, fv: &Face) -> Result<(Edge, Edge), Failure> {
        let list = self.my_ve_map.find_from_key(vtx.as_ref())?;
        let mut arc1 = Edge::default();
        for s in list.iter() {
            arc1 = topods::edge(s)?;
            if contain_e(fv, &arc1)? {
                break;
            }
        }
        let mut arc2 = Edge::default();
        for s in list.iter() {
            arc2 = topods::edge(s)?;
            if contain_e(fv, &arc2)? && !arc2.is_same(&arc1) {
                break;
            }
        }
        Ok((arc1, arc2))
    }

    /// `MoreSurfdata(Index)`: the corner of one contour at the vertex has a second surface
    /// data meeting the first one on the edges of the face closing it.
    pub(crate) fn more_surfdata(&self, index: i32) -> Result<bool, Failure> {
        let stripe = self
            .my_vdata_map
            .find_from_index(index)?
            .first()
            .cloned()
            .ok_or_else(|| Failure::new("MoreSurfdata"))?;
        let vtx = self.my_vdata_map.find_key(index)?;
        let (num, sens) = index_of_surf_data(&vtx, &stripe)?;
        let isfirst = sens == 1;
        let fd = surf_data_at(&stripe, num)?;
        let (cv1, cv2) = {
            let g = lock(&fd);
            (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
        };
        let nbsurf = lock(&stripe)
            .set_of_surf_data()
            .map_or(0, |s| lock(s).len() as i32);
        let mut fv = Face::default();
        let inters = self.find_face(&vtx, (&cv1, &cv2), &mut fv)?;
        let num2 = if sens == 1 { 2 } else { nbsurf - 1 };
        if nbsurf == 1 || !inters {
            return Ok(false);
        }
        let (arc1, arc2) = self.edges_on_face(&vtx, &fv)?;
        let fd1 = surf_data_at(&stripe, num2)?;
        let (cv3, cv4) = {
            let g = lock(&fd1);
            (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
        };
        let mut oksurf = false;
        if cv3.is_on_arc() {
            if cv3.arc()?.is_same(&arc1) {
                oksurf |= cv1.point().distance(&cv3.point()) < 1.0e-4;
            } else if cv3.arc()?.is_same(&arc2) {
                oksurf |= cv2.point().distance(&cv3.point()) < 1.0e-4;
            }
        }
        if cv4.is_on_arc()
            && (cv1.point().distance(&cv4.point()) < 1.0e-4
                || (cv4.arc()?.is_same(&arc2) && cv2.point().distance(&cv4.point()) < 1.0e-4))
        {
            oksurf = true;
        }
        Ok(oksurf)
    }
}
