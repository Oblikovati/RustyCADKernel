//! BRepExtrema probe operations of the proximity group, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepextrema_prox.cxx: triangle sets, the overlap tool,
//! self-intersection, the proximity distance and value tools, shape proximity and the
//! vertex inspector, on the meshes of shapes read by `tp_shape`.
use super::brep_extrema::{id, sub};
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use super::shape::{TpShape, tp_shape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::{BvhArray3d, BvhVec3d, array};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaOverlapMap, BRepExtremaOverlapTool, BRepExtremaProxPntStatus,
    BRepExtremaProximityDistTool, BRepExtremaProximityInputs, BRepExtremaProximityValueTool,
    BRepExtremaSelfIntersection, BRepExtremaShapeProximity, BRepExtremaTriangleSet,
    BRepExtremaVertexInspector,
};

fn vec3(v: &BvhVec3d) {
    real(v.x);
    real(v.y);
    real(v.z);
}

/// `tp_bxp_list`: `n (kind index)...`.
fn list(r: &mut Reader, s: &TpShape) -> Result<Vec<Shape>, Failure> {
    let n = r.integer();
    (0..n).map(|_| sub(r, s)).collect()
}

/// `tp_bxp_map`.
fn overlap_map(m: &BRepExtremaOverlapMap) {
    int(m.extent() as i32);
    for (key, set) in m.items() {
        int(*key);
        int(set.extent());
        for k in set.iter() {
            int(k);
        }
    }
}

/// `tp_bxp_faces_before`.
fn faces_before(list: &[Shape], id: i32) -> Result<bool, Failure> {
    for i in 0..=id {
        match list.get(i as usize) {
            Some(shape) if shape.shape_type()? == ShapeEnum::Face => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}

/// The triangles of `tp_bxp_set`.
fn set_triangles(set: &BRepExtremaTriangleSet, list: &[Shape]) -> Result<(), Failure> {
    for i in 0..set.size() {
        let b = set.element_box(i)?;
        vec3(b.corner_min());
        vec3(b.corner_max());
        for axis in 0..3 {
            real(set.center(i, axis)?);
        }
        for v in set.get_vertices(i)? {
            vec3(&v);
        }
        for k in set.get_vtx_indices(i)? {
            int(k);
        }
        int(set.get_face_id(i)?);
        if faces_before(list, set.get_face_id(i)?)? {
            int(set.get_trg_idx_in_shape(i)?);
        }
    }
    Ok(())
}

/// `tp_bxp_set`.
fn print_set(set: &BRepExtremaTriangleSet, list: &[Shape]) -> Result<(), Failure> {
    int(set.size());
    let nv = array::size(&set.vertices());
    int(nv);
    let all = set.bounding_box()?;
    vec3(all.corner_min());
    vec3(all.corner_max());
    set_triangles(set, list)?;
    for i in 0..nv {
        int(set.get_shape_id_of_vtx(i)?);
        int(set.get_vtx_idx_in_shape(i)?);
    }
    let tree = set.bvh()?;
    int(tree.length());
    for i in 0..tree.length() {
        let info = array::value(tree.node_info_buffer(), i)?;
        int(info.x);
        int(info.y);
        int(info.z);
        vec3(tree.min_point(i)?);
        vec3(tree.max_point(i)?);
    }
    Ok(())
}

/// `bxp_tset`: `shape list mode [list2]`.
pub fn tset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let l = list(&mut r, &s)?;
        let mode = r.integer();
        match mode {
            0 => print_set(&BRepExtremaTriangleSet::with_shapes(&l)?, &l)?,
            1 => {
                let l2 = list(&mut r, &s)?;
                let set = BRepExtremaTriangleSet::new();
                flag(set.init(&l)?);
                print_set(&set, &l)?;
                flag(set.init(&l2)?);
                print_set(&set, &l2)?;
            }
            _ => {
                let set = BRepExtremaTriangleSet::new();
                flag(set.init(&l)?);
                if set.size() > 1 {
                    set.swap(0, 1)?;
                }
                print_set(&set, &l)?;
                set.clear();
                int(set.size());
                int(array::size(&set.vertices()));
            }
        }
        Ok(())
    });
    end();
}

/// `bxp_overlap`: `shape1 list1 shape2 list2 n (tolerance)...`.
pub fn overlap_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let l1 = list(&mut r, &a)?;
        let b = tp_shape(&mut r)?;
        let l2 = list(&mut r, &b)?;
        let s1 = Handle::new(BRepExtremaTriangleSet::with_shapes(&l1)?);
        let s2 = match l2.is_empty() {
            true => s1.clone(),
            false => Handle::new(BRepExtremaTriangleSet::with_shapes(&l2)?),
        };
        let mut tool = BRepExtremaOverlapTool::with_sets(&s1, &s2);
        flag(tool.is_done());
        for _ in 0..r.integer() {
            let tol = r.real();
            attempt(|| tool.perform(tol));
            flag(tool.is_done());
            overlap_map(tool.overlap_sub_shapes1());
            overlap_map(tool.overlap_sub_shapes2());
            tool.mark_dirty();
            flag(tool.is_done());
        }
        Ok(())
    });
    end();
}

/// The pairs `(i, j)`, `i < j`, of the first `npairs` of `bxp_self`.
fn triangle_pairs(size: i32, npairs: i32) -> Vec<(i32, i32)> {
    let mut pairs = Vec::new();
    for i in 0..size {
        for j in i + 1..size {
            if pairs.len() as i32 >= npairs {
                return pairs;
            }
            pairs.push((i, j));
        }
    }
    pairs
}

/// `bxp_self`: `shape tolerance mode npairs`.
pub fn self_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let (tol, mode, npairs) = (r.real(), r.integer(), r.integer());
        let mut t = if mode == 0 {
            BRepExtremaSelfIntersection::with_shape(&s.shape, tol)?
        } else {
            let mut t = BRepExtremaSelfIntersection::new(tol);
            flag(t.load_shape(&s.shape)?);
            t
        };
        real(t.tolerance());
        flag(t.is_done());
        attempt(|| t.perform());
        flag(t.is_done());
        overlap_map(t.overlap_elements());
        if mode == 1 {
            attempt(|| t.perform());
            overlap_map(t.overlap_elements());
            t.set_tolerance(tol + 0.25);
            real(t.tolerance());
        }
        let size = t.element_set().size();
        int(size);
        for i in 0..(t.overlap_elements().extent() as i32).min(3) {
            id(&s, &s, &t.get_sub_shape(i)?.into_shape())?;
        }
        for (i, j) in triangle_pairs(size, npairs) {
            attempt(|| t.pre_check_elements(i, j).map(|result| int(result as i32)));
        }
        Ok(())
    });
    end();
}

/// A vector `x y z` of the request.
fn read_vec(r: &mut Reader) -> BvhVec3d {
    let (x, y) = (r.real(), r.real());
    BvhVec3d::new(x, y, r.real())
}

/// `bxp_shared`: `n (shared v1 v2 w1 w2)... m (v0 v1 v2 w2)...`.
pub fn shared_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = BRepExtremaSelfIntersection::new(0.0);
    for _ in 0..r.integer() {
        let v: [BvhVec3d; 5] = std::array::from_fn(|_| read_vec(&mut r));
        attempt(|| {
            let result = t.is_regular_shared_vertex(&v[0], [&v[1], &v[2]], [&v[3], &v[4]])?;
            int(result as i32);
            Ok(())
        });
    }
    for _ in 0..r.integer() {
        let v: [BvhVec3d; 4] = std::array::from_fn(|_| read_vec(&mut r));
        int(t.is_regular_shared_edge(&v[0], &v[1], &v[2], &v[3]) as i32);
    }
    end();
}

/// The status of a request integer.
fn status(value: i32) -> BRepExtremaProxPntStatus {
    match value {
        0 => BRepExtremaProxPntStatus::Border,
        1 => BRepExtremaProxPntStatus::Middle,
        _ => BRepExtremaProxPntStatus::Unknown,
    }
}

/// `bxp_pdist`: `shape1 list1 shape2 list2 nsamples n (x y z status)... mode`.
pub fn pdist_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let l1 = list(&mut r, &a)?;
        let b = tp_shape(&mut r)?;
        let l2 = list(&mut r, &b)?;
        let ns = r.integer();
        let (mut add, mut st) = (BvhArray3d::default(), Vec::new());
        for _ in 0..r.integer() {
            array::append(&mut add, read_vec(&mut r));
            st.push(status(r.integer()));
        }
        let mode = r.integer();
        let s1 = Handle::new(BRepExtremaTriangleSet::with_shapes(&l1)?);
        let s2 = Handle::new(BRepExtremaTriangleSet::with_shapes(&l2)?);
        let mut t = if mode == 0 {
            let inputs = BRepExtremaProximityInputs {
                set1: &s1,
                nb_samples1: ns,
                add_vertices1: &add,
                add_status1: &st,
                set2: &s2,
                shape_list1: &l1,
                shape_list2: &l2,
            };
            BRepExtremaProximityDistTool::with_inputs(&inputs)
        } else {
            let mut t = BRepExtremaProximityDistTool::new();
            real(t.proximity_distance());
            t.load_triangle_sets(&s1, &s2);
            t.load_shape_lists(&l1, &l2);
            t.load_additional_points_first_set(&add, &st);
            t
        };
        attempt(|| t.perform());
        flag(t.is_done());
        real(t.proximity_distance());
        let (p1, p2) = t.proximity_points();
        vec3(&p1);
        vec3(&p2);
        let (q1, q2) = t.proximity_points_status();
        int(q1 as i32);
        int(q2 as i32);
        Ok(())
    });
    end();
}

/// `bxp_border`: `shape face`.
pub fn border_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = topods::face(&sub(&mut r, &s)?)?;
        let (tr, _) = BRepTool::triangulation(&f, mesh_purpose::NONE)?;
        let triangulation = tr
            .get()
            .ok_or_else(|| Failure::new("bxp_border: no triangulation"))?;
        for i in 1..=triangulation.nb_nodes() {
            attempt(|| BRepExtremaProximityDistTool::is_node_on_border(i, &tr).map(flag));
        }
        for t in 1..=triangulation.nb_triangles() {
            for j in 0..3 {
                for k in 0..3 {
                    attempt(|| {
                        BRepExtremaProximityDistTool::is_edge_on_border(t, j, k, &tr).map(flag)
                    });
                }
            }
        }
        Ok(())
    });
    end();
}

/// `tp_bxp_pval_state`.
fn pval_state(t: &BRepExtremaProximityValueTool) {
    flag(t.is_done());
    real(t.distance());
    let (p1, p2) = t.proximity_points();
    pnt(p1);
    pnt(p2);
    // Natively the statuses are uninitialized until a computation is done.
    if t.is_done() {
        let (q1, q2) = t.proximity_points_status();
        int(q1 as i32);
        int(q2 as i32);
    }
}

/// `bxp_pval`: `shape1 list1 shape2 list2 ns1 ns2 mode`.
pub fn pval_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let l1 = list(&mut r, &a)?;
        let b = tp_shape(&mut r)?;
        let l2 = list(&mut r, &b)?;
        let (ns1, ns2, mode) = (r.integer(), r.integer(), r.integer());
        let s1 = Handle::new(BRepExtremaTriangleSet::with_shapes(&l1)?);
        let s2 = Handle::new(BRepExtremaTriangleSet::with_shapes(&l2)?);
        let mut tol: f64 = -1.0;
        if mode == 0 {
            let mut t = BRepExtremaProximityValueTool::with_sets(&s1, &s2, &l1, &l2)?;
            t.set_nb_sample_points(ns1, ns2);
            attempt(|| t.perform(&mut tol));
            real(tol);
            pval_state(&t);
            return Ok(());
        }
        let mut t = BRepExtremaProximityValueTool::new();
        pval_state(&t);
        attempt(|| t.load_shape_lists(&l1, &l2));
        t.load_triangle_sets(&s1, &s2);
        t.set_nb_sample_points(ns1, ns2);
        if mode == 1 {
            for _ in 0..2 {
                attempt(|| t.perform(&mut tol));
                real(tol);
                pval_state(&t);
                t.mark_dirty();
            }
        }
        pval_state(&t);
        Ok(())
    });
    end();
}

/// The state line of `bxp_prox`; the statuses only after an infinite-tolerance computation
/// (natively they are uninitialized otherwise).
fn prox_state(p: &BRepExtremaShapeProximity, inf: bool) {
    flag(p.is_done());
    real(p.proximity());
    real(p.tolerance());
    pnt(*p.proximity_point1());
    pnt(*p.proximity_point2());
    if inf && p.is_done() {
        int(p.prox_pnt_status1() as i32);
        int(p.prox_pnt_status2() as i32);
    }
    overlap_map(p.overlap_sub_shapes1());
    overlap_map(p.overlap_sub_shapes2());
    int(p.element_set1().size());
    int(p.element_set2().size());
}

/// `bxp_prox`: `shape1 shape2 tolerance ns1 ns2 mode`.
pub fn prox_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let b = tp_shape(&mut r)?;
        let (tol, ns1, ns2, mode) = (r.real(), r.integer(), r.integer(), r.integer());
        let inf = tol == rustycad_tkernel::precision::infinite();
        let mut p = if mode == 0 {
            BRepExtremaShapeProximity::with_shapes(&a.shape, &b.shape, tol)?
        } else {
            let mut p = BRepExtremaShapeProximity::new(tol);
            flag(p.load_shape1(&a.shape)?);
            flag(p.load_shape2(&b.shape)?);
            p
        };
        p.set_nb_samples1(ns1);
        p.set_nb_samples2(ns2);
        attempt(|| p.perform());
        prox_state(&p, inf);
        attempt(|| p.perform());
        prox_state(&p, inf);
        if mode == 2 {
            flag(p.load_shape1(&a.shape)?);
            attempt(|| p.perform());
            prox_state(&p, inf);
        }
        attempt(|| id(&a, &b, &p.get_sub_shape1(0)?));
        attempt(|| id(&a, &b, &p.get_sub_shape2(0)?));
        Ok(())
    });
    end();
}

/// `bxp_insp`: `tol n (x y z)... m (cx cy cz ntargets target...)...`.
pub fn insp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut insp = BRepExtremaVertexInspector::new();
    flag(insp.is_need_add());
    let tol = r.real();
    if tol >= 0.0 {
        insp.set_tol(tol);
    }
    for _ in 0..r.integer() {
        let p = r.xyz();
        for k in 0..3 {
            real(BRepExtremaVertexInspector::coord(k, &p));
        }
        xyz(BRepExtremaVertexInspector::shift(&p, tol));
        insp.add(p);
    }
    for _ in 0..r.integer() {
        insp.set_current(r.xyz());
        flag(insp.is_need_add());
        for _ in 0..r.integer() {
            let action = insp.inspect_target(r.integer());
            int(action as i32);
            flag(insp.is_need_add());
        }
    }
    end();
}
