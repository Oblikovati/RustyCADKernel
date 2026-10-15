//! The ON patches of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_GridSS.cxx): the faces
//! of the ON parts of the faces coinciding with faces of the other shape, built and
//! paired by their normals.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::{fdscnx_edge_connexity_shape_index, fdscnx_face_edge_connex_faces};
use crate::top_ope_brep_tool::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Direction2, Line2, Vector3};
use rustycad_tktopalgo::brep_class::{BRepClassEdge, BRepClassIntersector};

/// A p-curve with its range.
type PCurveRange = (Handle<dyn Geom2dCurve>, f64, f64);
/// The unit normal of the face at a UV point (oriented by the face), None when null.
fn face_normal(f: &Face, u: f64, v: f64) -> Result<Option<Vector3>, Failure> {
    let bas = BRepAdaptorSurface::with_face(f, true)?;
    let d = bas.d1(u, v)?;
    let n = d.d1u.crossed(d.d1v);
    let norm = n.magnitude();
    if norm < precision::confusion() {
        return Ok(None);
    }
    let mut n = n / norm;
    if f.orientation() == Orientation::Reversed {
        n.reverse();
    }
    Ok(Some(n))
}

/// The p-curve of the edge on the face, made (and stored with the tolerance) when
/// missing; None when it cannot be made.
fn pcurve_or_make(
    e: &Edge,
    f: &Face,
    tol_store: Option<f64>,
) -> Result<Option<PCurveRange>, Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    if !pc.pcurve.is_null() {
        return Ok(Some((pc.pcurve, pc.first, pc.last)));
    }
    let (c, pf, pl, _) = fc2d_curve_on_surface(e, f, true)?;
    if c.is_null() {
        return Ok(None);
    }
    let tol = match tol_store {
        Some(t) => t,
        None => BRepTool::edge_tolerance(e)?,
    };
    BRepBuilder::default().update_edge_pcurve_on_face(e, &c, f, tol)?;
    Ok(Some((c, pf, pl)))
}

/// The nearest intersection (beyond the origin) of the line with the other edges;
/// None on an error (an edge without p-curve, a segment over the origin).
fn nearest_inside(
    a_lin: &Line2,
    an_edge: &Edge,
    base: &Face,
    all_edges: &List<Shape>,
    max_dist: &mut f64,
) -> Result<Option<Option<f64>>, Failure> {
    let tol2d = precision::p_confusion();
    let (mut p_lin_min, mut has_int) = (f64::MAX, false);
    let mut an_inter = BRepClassIntersector::new();
    let mut a_bce = BRepClassEdge::new();
    a_bce.set_face(base.clone());
    for e in all_edges.iter() {
        let ae = topods::edge(e)?;
        let tol_e = BRepTool::edge_tolerance(&ae)?;
        *max_dist = max_dist.max(tol_e);
        let not_oriented = !matches!(
            ae.orientation(),
            Orientation::Forward | Orientation::Reversed
        );
        if ae.as_ref().is_equal(an_edge.as_ref())
            || (not_oriented && ae.as_ref().is_same(an_edge.as_ref()))
        {
            continue;
        }
        if pcurve_or_make(&ae, base, Some(tol_e))?.is_none() {
            return Ok(None);
        }
        a_bce.set_edge(ae);
        an_inter.perform(a_lin, p_lin_min, tol2d, &a_bce)?;
        if !an_inter.is_done() {
            continue;
        }
        for i in 1..=an_inter.nb_points()? {
            let p_lin = an_inter.point(i)?.param_on_first();
            if p_lin > tol2d && p_lin < p_lin_min {
                (p_lin_min, has_int) = (p_lin, true);
            }
        }
        for i in 1..=an_inter.nb_segments()? {
            let s = an_inter.segment(i)?;
            let p_lin_f = if s.has_first_point() {
                s.first_point()?.param_on_first()
            } else {
                -precision::infinite()
            };
            let p_lin_l = if s.has_last_point() {
                s.last_point()?.param_on_first()
            } else {
                precision::infinite()
            };
            if p_lin_f < tol2d && p_lin_l > -tol2d {
                return Ok(None);
            } else if p_lin_f > tol2d && p_lin_f < p_lin_min {
                (p_lin_min, has_int) = (p_lin_f, true);
            }
        }
    }
    Ok(Some(has_int.then_some(p_lin_min)))
}

/// `AreFacesCoincideInArea(theBaseFace, theFace, theEdge, allEdges, isSameOri)`
/// (static): a point inside the base face near the oriented edge projects onto the
/// face with a parallel normal; whether the normals are the same.
fn are_faces_coincide_in_area(
    base_face: &Shape,
    the_face: &Shape,
    the_edge: &Shape,
    all_edges: &List<Shape>,
) -> Result<Option<bool>, Failure> {
    let reverse = match the_edge.orientation() {
        Orientation::Forward => false,
        Orientation::Reversed => true,
        _ => return Ok(None),
    };
    let (base, face, an_edge) = (
        topods::face(base_face)?,
        topods::face(the_face)?,
        topods::edge(the_edge)?,
    );
    let Some((pcref, pf, pl)) = pcurve_or_make(&an_edge, &base, None)? else {
        return Ok(None);
    };
    let t = 0.456789;
    let (pt, mut d1) = pcref
        .get()
        .ok_or_else(|| Failure::new("null p-curve"))?
        .d1((1. - t) * pf + t * pl)?;
    if d1.magnitude() < gp::resolution() {
        return Ok(None);
    }
    if reverse {
        d1.reverse();
    }
    let a_lin = Line2::new(pt, Direction2::new(-d1.y(), d1.x())?);
    let mut max_dist = BRepTool::face_tolerance(&base)?.max(BRepTool::face_tolerance(&face)?);
    let Some(Some(p_lin_min)) = nearest_inside(&a_lin, &an_edge, &base, all_edges, &mut max_dist)?
    else {
        return Ok(None);
    };
    let ap2d = elclib::value(p_lin_min * t, &a_lin);
    let bas = BRepAdaptorSurface::with_face(&base, true)?;
    let d = bas.d1(ap2d.x(), ap2d.y())?;
    let mut mag = d.d1u.crossed(d.d1v).magnitude();
    if mag < gp::resolution() {
        return Ok(None);
    }
    if base.orientation() == Orientation::Reversed {
        mag = -mag;
    }
    let n_base = d.d1u.crossed(d.d1v) / mag;
    let a_surf = BRepTool::surface(&face)?;
    let (umin, umax, vmin, vmax) = uv_bounds(&face)?;
    let a_proj = ProjectPointOnSurf::with_bounds(
        d.point,
        &a_surf,
        (umin, umax, vmin, vmax),
        Default::default(),
    )?;
    if a_proj.nb_points()? == 0 || a_proj.lower_distance()? > max_dist {
        return Ok(None);
    }
    let (u, v) = a_proj.lower_distance_parameters()?;
    let d2 = a_surf
        .get()
        .ok_or_else(|| Failure::new("null surface"))?
        .d1(u, v)?;
    let mut mag = d2.d1u.crossed(d2.d1v).magnitude();
    if mag < gp::resolution() {
        return Ok(None);
    }
    if face.orientation() == Orientation::Reversed {
        mag = -mag;
    }
    let dot = n_base.dot(d2.d1u.crossed(d2.d1v) / mag);
    if dot.abs() < 0.9999 {
        return Ok(None);
    }
    Ok(Some(dot > 0.))
}

/// Whether one of the faces is on the edge (known in the state map, else checked and
/// recorded).
fn faces_on(
    base_face: &Shape,
    faces: &List<Shape>,
    edges: &[Shape],
    all_edges: &List<Shape>,
    states: &mut DataMap<Shape, i32, ShapeMapHasher>,
) -> Result<bool, Failure> {
    let mut to_check = Vec::new();
    for f in faces.iter() {
        match states.find(f) {
            Ok(s) if *s != 0 => return Ok(true),
            Ok(_) => {}
            Err(_) => to_check.push(f.clone()),
        }
    }
    for f in to_check {
        let mut ok = None;
        for e in edges {
            ok = are_faces_coincide_in_area(base_face, &f, e, all_edges)?;
            if ok.is_some() {
                break;
            }
        }
        let on = ok == Some(true);
        states.bind(f, i32::from(on));
        if on {
            return Ok(true);
        }
    }
    Ok(false)
}

impl TopOpeBRepBuildBuilder {
    /// The first non-degenerated edge of the face.
    fn check_edge(face: &Face) -> Result<Option<Edge>, Failure> {
        let mut m: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        topexp::map_shapes_of_type(face.as_ref(), ShapeEnum::Edge, &mut m)?;
        for k in 1..=m.extent() {
            let e = topods::edge(m.find_key(k)?)?;
            if !BRepTool::degenerated(&e)? {
                return Ok(Some(e));
            }
        }
        Ok(None)
    }

    /// `AddONPatchesSFS(G1, SFS)`: each ON face of the first shape paired with an ON
    /// face of the second of the same edges and normal: the first one added oriented
    /// as its ancestor, both as the ON splits of their ancestors;
    /// `b.add_on_patches_sfs(&g, &mut sfs)?`.
    pub fn add_on_patches_sfs(
        &mut self,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let scal_min = 0.999847695;
        let n = self.my_on_faces_map.len();
        for i in 1..=n {
            let anc1 = self.my_on_faces_map.find_from_index(i)?.clone();
            if self.hds()?.ds().ancestor_rank(&anc1) != 1 {
                continue;
            }
            let face1 = topods::face(self.my_on_faces_map.find_key(i)?)?;
            let Some(chk) = Self::check_edge(&face1)? else {
                continue;
            };
            let bac1 = BRepAdaptorCurve2d::with_edge_on_face(&chk, &face1)?;
            let par_t = 0.456321;
            let par = bac1.first_parameter()? * (1. - par_t) + bac1.last_parameter()? * par_t;
            let p2d = bac1.d0(par)?;
            let Some(n1) = face_normal(&face1, p2d.x(), p2d.y())? else {
                continue;
            };
            let mut me1: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
            topexp::map_shapes_of_type(face1.as_ref(), ShapeEnum::Edge, &mut me1)?;
            let mut ok = true;
            let mut j = i + 1;
            while j <= n && ok {
                let anc2 = self.my_on_faces_map.find_from_index(j)?.clone();
                j += 1;
                if self.hds()?.ds().ancestor_rank(&anc2) != 2 {
                    continue;
                }
                let face2 = topods::face(self.my_on_faces_map.find_key(j - 1)?)?;
                let mut me2: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
                topexp::map_shapes_of_type(face2.as_ref(), ShapeEnum::Edge, &mut me2)?;
                if me1.extent() != me2.extent()
                    || !(1..=me2.extent()).all(|k| me2.find_key(k).is_ok_and(|e| me1.contains(e)))
                {
                    continue;
                }
                let p2d2 = BRepAdaptorCurve2d::with_edge_on_face(&chk, &face2)?.d0(par)?;
                let Some(n2) = face_normal(&face2, p2d2.x(), p2d2.y())? else {
                    ok = false;
                    continue;
                };
                if n1.dot(n2) < scal_min {
                    ok = false;
                    continue;
                }
                let neworif = Self::orient(anc1.orientation(), g1.is_to_reverse1()?);
                sfs.add_start_element(&face1.oriented(neworif))?;
                self.mark_split(&anc1, State::On, true);
                self.change_split(&anc1, State::On)
                    .append(face1.clone().into_shape());
                self.mark_split(&anc2, State::On, true);
                self.change_split(&anc2, State::On)
                    .append(face2.into_shape());
            }
        }
        Ok(())
    }

    /// The ON edges of the base face in the wire edge set: the ON parts of its section
    /// edges on a coincident face of the other shape (FillOnPatches step 1).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn on_parts_on_faces(
        &self,
        an_edges_on: &List<Shape>,
        a_base_face: &Shape,
        avoid: &IndexedMap<Shape>,
        rank_opp: i32,
        all_edges: &List<Shape>,
        states: &mut DataMap<Shape, i32, ShapeMapHasher>,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let mut a_map_on: Map<Shape, ShapeMapHasher> = Map::default();
        for e in an_edges_on.iter() {
            if !self.my_on_elem_map.contains(e)
                && !self.my_on_elem_map.contains(&e.reversed())
                && !avoid.contains(e)
            {
                a_map_on.add(e.clone());
            }
        }
        let mut anc: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
        if !a_map_on.is_empty() {
            self.fill_sec_edge_ancestor_map(rank_opp, &a_map_on, &mut anc)?;
        }
        if anc.is_empty() {
            return Ok(());
        }
        for e in an_edges_on.iter() {
            let Ok(anc_e) = anc.find(e) else {
                continue;
            };
            if avoid.contains(e) {
                continue;
            }
            let faces =
                fdscnx_edge_connexity_shape_index(anc_e, &self.my_data_structure, rank_opp)?;
            let edges = if matches!(
                e.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                vec![e.clone()]
            } else {
                vec![
                    e.oriented(Orientation::Forward),
                    e.oriented(Orientation::Reversed),
                ]
            };
            if faces_on(a_base_face, &faces, &edges, all_edges, states)? {
                wes.add_start_element(e)?;
            }
        }
        Ok(())
    }

    /// The ON elements (edges of a rank, wires) of the base face in the wire edge set
    /// when on a coincident face touching their ancestor (FillOnPatches step 2).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn on_elems_on_faces(
        &self,
        a_base_face: &Shape,
        avoid: &IndexedMap<Shape>,
        rank_bf: i32,
        rank_opp: i32,
        has_wires: bool,
        all_edges: &List<Shape>,
        states: &mut DataMap<Shape, i32, ShapeMapHasher>,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let mut a_map_on1: Map<Shape, ShapeMapHasher> = Map::default();
        for i in 1..=self.my_on_elem_map.extent() {
            let e = self.my_on_elem_map.find_key(i)?;
            if e.shape_type()? != ShapeEnum::Wire && !avoid.contains(e) {
                a_map_on1.add(e.clone());
            }
        }
        let mut anc: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
        if !a_map_on1.is_empty() {
            self.fill_sec_edge_ancestor_map(rank_bf, &a_map_on1, &mut anc)?;
        }
        if !has_wires && anc.is_empty() {
            return Ok(());
        }
        for i in 1..=self.my_on_elem_map.extent() {
            let ae = self.my_on_elem_map.find_key(i)?.clone();
            let mut pair = None;
            if ae.shape_type()? == ShapeEnum::Wire {
                for e in ShapeIterator::new(&ae, true, true) {
                    if avoid.contains(&e) {
                        break;
                    }
                    if !BRepTool::degenerated(&topods::edge(&e)?)? {
                        pair = Some((e.clone(), e));
                        break;
                    }
                }
            } else if let Ok(a) = anc.find(&ae).cloned()
                && !avoid.contains(&ae)
            {
                pair = Some((ae.clone(), a));
            }
            let Some((an_edge, anc_e)) = pair else {
                continue;
            };
            let cnx = fdscnx_face_edge_connex_faces(a_base_face, &anc_e, &self.my_data_structure)?;
            let Some(cnx_f) = cnx.first() else {
                continue;
            };
            let mut faces = List::default();
            self.find_faces_touching_edge(cnx_f, &anc_e, rank_opp, &mut faces)?;
            if faces_on(a_base_face, &faces, &[an_edge], all_edges, states)? {
                if ae.shape_type()? == ShapeEnum::Wire {
                    wes.add_shape(&ae)?
                } else {
                    wes.add_start_element(&ae)?
                }
            }
        }
        Ok(())
    }
}
