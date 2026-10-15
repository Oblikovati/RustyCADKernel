// Rust translation of ChFi3d_Builder::PerformElement, ChFi3d_Builder::FaceTangency and the
// file statics ReorderFaces, ConcatCurves and MakeOffsetEdge (ChFi3d_Builder_1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contour of an edge: the tangent edges joined before and after it, the faces of
//! each edge in order and the offset edges.
use super::ChFi3dBuilder;
use super::extremity::tangent_on_vertex;
use crate::chfi_ds::{Map, SpineHandle, State, lock};
use crate::chfi3d::{conexfaces, define_connect_type, is_tangent_faces};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBoundedCurve, GeomCurve, GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use std::f64::consts::PI;

/// `ReorderFaces(F1, F2, FirstFace, PrevEdge, CommonVertex, EFmap)`: puts first the face of
/// the next edge reached from `first_face` around the common vertex.
pub(super) fn reorder_faces(
    (f1, f2): (&mut Face, &mut Face),
    first_face: &Face,
    prev_edge: &Edge,
    common_vertex: &Vertex,
    ef_map: &Map,
) -> Result<(), Failure> {
    if f1.is_same(first_face) {
        return Ok(());
    }
    if f2.is_same(first_face) {
        std::mem::swap(f1, f2);
        return Ok(());
    }
    let (mut prev_edge, mut prev_face) = (prev_edge.clone(), first_face.clone());
    let (mut cur_edge, mut cur_face) = (Edge::default(), Face::default());
    loop {
        let mut map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            prev_face.as_ref(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut map,
        )?;
        for an_edge in map.find_from_key(common_vertex.as_ref())?.iter() {
            if an_edge.is_same(prev_edge.as_ref()) {
                continue;
            }
            cur_edge = topods::edge(an_edge)?;
            let flist = ef_map.find_from_key(cur_edge.as_ref())?;
            let first = topods::face(flist.first().ok_or_else(|| Failure::new("ReorderFaces"))?)?;
            cur_face = if prev_face.is_same(&first) {
                topods::face(flist.last().ok_or_else(|| Failure::new("ReorderFaces"))?)?
            } else {
                first
            };
            if cur_face.is_same(f1) {
                return Ok(());
            }
            if cur_face.is_same(f2) {
                std::mem::swap(f1, f2);
                return Ok(());
            }
        }
        prev_edge = cur_edge.clone();
        prev_face = cur_face.clone();
    }
}

/// `ConcatCurves(Curves, NewCurves)`: the curves joined into B-splines, as many as
/// needed.
pub(super) fn concat_curves(
    mut curves: Vec<Handle<dyn GeomCurve>>,
) -> Result<Vec<Handle<dyn GeomCurve>>, Failure> {
    let mut new_curves = Vec::new();
    while !curves.is_empty() {
        let mut concat = CompCurveToBSplineCurve::new(ParameterisationType::TgtThetaOver2);
        let mut i = 0;
        while i < curves.len() {
            let bounded: Handle<dyn GeomBoundedCurve> = curves[i].clone().cast();
            let mut success = concat.add(&bounded, 1.0e-5, true, true, 0)?;
            if !success {
                success = concat.add(&bounded, 1.0e-5, false, true, 0)?;
            }
            if success {
                curves.remove(i);
            } else {
                i += 1;
            }
        }
        new_curves.push(concat.bspline_curve().cast());
    }
    Ok(new_curves)
}

/// The surface of a face trimmed to the bounds of its adaptor.
pub(super) fn trimmed_surface(s: &BRepAdaptorSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    let gs = BRepTool::surface(&s.face())?;
    let u = (s.first_u_parameter()?, s.last_u_parameter()?);
    let v = (s.first_v_parameter()?, s.last_v_parameter()?);
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(&gs, u, v, (true, true))?).cast())
}

/// The intersection of the first face offset by `distance` with the second face, the
/// curve closest to the edge when there are several (None when there is none).
pub(super) fn offset_intersection(
    distance: f64,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
    end0: Point3,
) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    let offset = if s1.face().orientation() == Orientation::Reversed {
        distance
    } else {
        -distance
    };
    let make_offset_surf = GeomOffsetSurface::new(&trimmed_surface(s1)?, offset, false)?;
    let mut offset_tr_gs1 = make_offset_surf.surface()?;
    if offset_tr_gs1.is_null() {
        offset_tr_gs1 = Handle::new(make_offset_surf).cast();
    }
    let inter = GeomIntIntSS::with_surfaces(
        &offset_tr_gs1,
        &trimmed_surface(s2)?,
        precision::confusion(),
        GeomIntIntSSApprox::default(),
    )?;
    if !inter.is_done() || inter.nb_lines() == 0 {
        return Ok(None);
    }
    if inter.nb_lines() == 1 {
        return Ok(Some(inter.line(1)?));
    }
    let curves = (1..=inter.nb_lines())
        .map(|i| inter.line(i))
        .collect::<Result<Vec<_>, _>>()?;
    let new_curves = concat_curves(curves)?;
    let (mut min_dist, mut imin) = (f64::MAX, 0);
    for (i, c) in new_curves.iter().enumerate() {
        let ga = GeomAdaptorCurve::with_curve(c)?;
        let projector = ExtPC::with_curve(end0, &ga as &dyn Adaptor3dCurve, 1.0e-10)?;
        if !projector.is_done() || projector.nb_ext()? == 0 {
            continue;
        }
        for iext in 1..=projector.nb_ext()? {
            let d = projector.square_distance(iext)?;
            if d < min_dist {
                min_dist = d;
                imin = i;
            }
        }
    }
    Ok(Some(new_curves[imin].clone()))
}

/// The parameter on the intersection curve nearest to an end of the edge (the bounds of
/// the curve or its projection).
pub(super) fn nearest_parameter(end: Point3, ga: &GeomAdaptorCurve) -> Result<f64, Failure> {
    let projector = ExtPC::with_curve(end, ga as &dyn Adaptor3dCurve, 1.0e-10)?;
    let mut param = [0.; 4];
    let mut dist = [0.; 4];
    param[1] = ga.first_parameter()?;
    param[2] = ga.last_parameter()?;
    let (d1, d2, _, _) = projector.trimmed_square_distances();
    dist[1] = d1;
    dist[2] = d2;
    dist[3] = f64::MAX;
    if projector.is_done() && projector.nb_ext()? > 0 {
        let mut imin = 1;
        for i in 2..=projector.nb_ext()? {
            if projector.square_distance(i)? < projector.square_distance(imin)? {
                imin = i;
            }
        }
        param[3] = projector.point(imin)?.parameter();
        dist[3] = projector.square_distance(imin)?;
    }
    let mut imin = 1;
    for i in 2..=3 {
        if dist[i] < dist[imin] {
            imin = i;
        }
    }
    Ok(param[imin])
}

/// `MakeOffsetEdge(theEdge, Distance, S1, S2)`: the intersection of the first face offset
/// by `distance` with the second face, as an edge along `the_edge` (null when the faces do
/// not cut).
pub(super) fn make_offset_edge(
    the_edge: &Edge,
    distance: f64,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<Edge, Failure> {
    let ba = BRepAdaptorCurve::with_edge(the_edge)?;
    let ends = [
        ba.value(ba.first_parameter()?)?,
        ba.value(ba.last_parameter()?)?,
    ];
    let Some(int_curve) = offset_intersection(distance, s1, s2, ends[0])? else {
        return Ok(Edge::default());
    };
    let Some(curve) = int_curve.get() else {
        return Ok(Edge::default());
    };
    let ga = GeomAdaptorCurve::with_curve(&int_curve)?;
    let closed = ba.is_closed()?;
    let mut params = [0.; 2];
    for ind_end in 0..2 {
        if ind_end == 1 && closed {
            break;
        }
        params[ind_end] = nearest_parameter(ends[ind_end], &ga)?;
    }
    if closed {
        params[1] = ga.last_parameter()?;
    }
    if params[0] > params[1] {
        let fpnt = curve.value(curve.first_parameter())?;
        let lpnt = curve.value(curve.last_parameter())?;
        if fpnt.square_distance(&lpnt) <= precision::square_confusion() {
            params[1] = curve.last_parameter();
        } else {
            let (nf, nl) = (
                curve.reversed_parameter(params[0])?,
                curve.reversed_parameter(params[1])?,
            );
            curve.reverse()?;
            params = [nf, nl];
        }
    }
    if closed {
        let (_, tangent) = ba.d1(ba.first_parameter()?)?;
        let (_, offset_tangent) = curve.d1(params[0])?;
        if tangent.dot(offset_tangent) < 0. {
            curve.reverse()?;
        }
    }
    BRepLibMakeEdge::from_curve_params(&int_curve, params[0], params[1])?.edge()
}

impl ChFi3dBuilder {
    /// The two faces of an edge from the edge-face map (Standard_ConstructionError beyond
    /// two); false when there are less than two.
    pub(super) fn two_faces(&self, e: &Edge) -> Result<Option<(Face, Face)>, Failure> {
        let mut f = [Face::default(), Face::default()];
        let mut nbf = 0;
        for s in self.my_ef_map.find_from_key(e.as_ref())?.iter() {
            if nbf > 1 {
                return Err(Failure::with_message(
                    "Standard_ConstructionError",
                    "ChFi3d_Builder:only 2 faces",
                ));
            }
            f[nbf] = topods::face(s)?;
            nbf += 1;
        }
        if nbf < 2 {
            return Ok(None);
        }
        let [f0, f1] = f;
        Ok(Some((f0, f1)))
    }

    /// `FaceTangency(E0, E1, V)`: the faces of `e1` are not tangent while those of every
    /// other edge at `v` are.
    pub(crate) fn face_tangency(&self, e0: &Edge, e1: &Edge, v: &Vertex) -> Result<bool, Failure> {
        let Some((f0, f1)) = self.two_faces(e1)? else {
            return Ok(false);
        };
        if is_tangent_faces(e1, &f0, &f1, Continuity::G1)? {
            return Ok(false);
        }
        for s in self.my_ve_map.find_from_key(v.as_ref())?.iter() {
            let ec = topods::edge(s)?;
            if !ec.is_same(e0)
                && !ec.is_same(e1)
                && !matches!(
                    ec.orientation(),
                    Orientation::Internal | Orientation::External
                )
                && !BRepTool::degenerated(&ec)?
            {
                let Some((g0, g1)) = self.two_faces(&ec)? else {
                    return Ok(false);
                };
                if !is_tangent_faces(&ec, &g0, &g1, Continuity::G1)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// The number of faces of an edge.
    pub(super) fn nb_faces(&self, e: &Edge) -> Result<usize, Failure> {
        Ok(self.my_ef_map.find_from_key(e.as_ref())?.iter().count())
    }

    /// `PerformElement(CElement, Offset, theFirstFace)`: the contour of the first edge of
    /// the spine, extended by the tangent edges on both sides, with the faces of each edge
    /// in order and, for a positive offset, the offset edges; false for a degenerated
    /// edge, an edge on less than two faces or between tangent faces;
    /// `b.perform_element(&spine, -1.0, &Face::default())?`.
    pub fn perform_element(
        &mut self,
        spine: &SpineHandle,
        offset: f64,
        the_first_face: &Face,
    ) -> Result<bool, Failure> {
        let ec = lock(spine).edges(1)?.clone();
        if BRepTool::degenerated(&ec)? {
            return Ok(false);
        }
        let (mut ff1, mut ff2) = conexfaces(&ec, &self.my_ef_map)?;
        if ff1.is_null() || ff2.is_null() || is_tangent_faces(&ec, &ff1, &ff2, Continuity::G1)? {
            return Ok(false);
        }
        let mut first_face = ff1.clone();
        if !the_first_face.is_null() && ff2.is_same(the_first_face) {
            first_face = ff2.clone();
            ff2 = ff1;
            ff1 = first_face.clone();
        }
        self.my_edge_first_face
            .bind(ec.clone().into(), first_face.clone().into());
        let type_of_concavity = define_connect_type(&ec, &ff1, &ff2, 1.0e-5, true)?;
        lock(spine).set_type_of_concavity(type_of_concavity);
        if offset > 0. {
            let (sb1, sb2) = (
                BRepAdaptorSurface::with_face(&ff1, true)?,
                BRepAdaptorSurface::with_face(&ff2, true)?,
            );
            let mut offset_edge = make_offset_edge(&ec, offset, &sb1, &sb2)?;
            offset_edge.set_orientation(ec.orientation());
            lock(spine).set_offset_edges(&offset_edge);
        }
        let (vstart, lvec) = topexp::vertices(&ec, false);
        if vstart.is_same(&lvec) {
            self.closed_single_edge(spine, &ec, &vstart, &lvec)?;
            return Ok(true);
        }
        let cur_st = self.extend_downstream(spine, ec, vstart.clone(), lvec, first_face, offset)?;
        lock(spine).set_last_status(cur_st);
        if cur_st == State::Closed {
            lock(spine).set_first_status(cur_st);
        } else {
            let st = self.extend_upstream(spine, vstart, offset)?;
            lock(spine).set_first_status(st);
        }
        Ok(true)
    }

    /// The statuses of a contour of one closed edge.
    pub(super) fn closed_single_edge(
        &self,
        spine: &SpineHandle,
        ec: &Edge,
        vstart: &Vertex,
        lvec: &Vertex,
    ) -> Result<(), Failure> {
        let cec = BRepAdaptorCurve::with_edge(ec)?;
        let (_, v1) = cec.d1(BRepTool::parameter(vstart, ec)?)?;
        let (_, v2) = cec.d1(BRepTool::parameter(lvec, ec)?)?;
        let is_face_tangency = self.face_tangency(ec, ec, vstart)?;
        let cur_st = if (v1.is_parallel(v2, self.angular)? || is_face_tangency) && is_face_tangency
        {
            State::Closed
        } else {
            State::BreakPoint
        };
        let mut sp = lock(spine);
        sp.set_last_status(cur_st);
        sp.set_first_status(cur_st);
        Ok(())
    }

    /// The decision to join `ev` to the contour at a vertex, from the angle of the
    /// tangents and the tangency of the faces.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn joins(
        &self,
        ec: &Edge,
        ev: &Edge,
        vtx: &Vertex,
        v_on: &Vertex,
        av1v2: f64,
        rev: bool,
        degene_on_ec: bool,
    ) -> Result<bool, Failure> {
        let ta = self.angular;
        let mut on_ajoute = false;
        if self.face_tangency(ec, ev, vtx)? {
            on_ajoute = (!rev && av1v2 < PI / 2.) || (rev && av1v2 > PI / 2.);
            if on_ajoute && (degene_on_ec || tangent_on_vertex(v_on, ev, &self.my_ef_map, ta)?) {
                on_ajoute = (!rev && av1v2 < ta) || (rev && (PI - av1v2) < ta);
            }
        }
        Ok(on_ajoute)
    }

    /// The faces of an edge joined to the contour, the face reached from `first_face`
    /// first; it becomes the first face of the edge.
    pub(super) fn joined_faces(
        &mut self,
        ec: &Edge,
        prev_edge: &Edge,
        ev: &Edge,
        first_face: &Face,
    ) -> Result<(Face, Face), Failure> {
        let common = topexp::common_vertex(prev_edge, ev).unwrap_or_default();
        let (mut cf1, mut cf2) = conexfaces(ec, &self.my_ef_map)?;
        reorder_faces(
            (&mut cf1, &mut cf2),
            first_face,
            prev_edge,
            &common,
            &self.my_ef_map,
        )?;
        self.my_edge_first_face
            .bind(ec.clone().into(), cf1.clone().into());
        Ok((cf1, cf2))
    }
}
