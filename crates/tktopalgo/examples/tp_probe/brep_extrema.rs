//! BRepExtrema probe operations of the distance group, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepextrema.cxx: DistShapeShape, DistanceSS and Poly on
//! shapes read by `tp_shape`; sub-shapes picked by `kind index`, supports printed as their
//! type, their index (by TShape) in each record and their orientation.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real};
use super::shape::{TpShape, tp_shape};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaDistShapeShape, BRepExtremaDistanceSS, BRepExtremaPoly, BRepExtremaSolutionElem,
};

/// `tp_bxe_tindex`: the index (by TShape) of `x` among the sub-shapes of its type.
pub fn tindex(s: &TpShape, x: &Shape) -> Result<i32, Failure> {
    let same = |y: &Shape| y.tshape().object_address() == x.tshape().object_address();
    let found = match x.shape_type()? {
        ShapeEnum::Vertex => s.vertices.iter().position(|v| same(v)),
        ShapeEnum::Edge => s.edges.iter().position(|e| same(e)),
        ShapeEnum::Face => s.faces.iter().position(|f| same(f)),
        _ => None,
    };
    Ok(found.map_or(-1, |i| i as i32))
}

/// `tp_bxe_id`: -1 for a null shape, else type, index in each record and orientation.
pub fn id(a: &TpShape, b: &TpShape, x: &Shape) -> Result<(), Failure> {
    if x.is_null() {
        int(-1);
        return Ok(());
    }
    int(x.shape_type()? as i32);
    int(tindex(a, x)?);
    int(tindex(b, x)?);
    int(x.orientation() as i32);
    Ok(())
}

/// `tp_bxe_sub`: `kind index`.
pub fn sub(r: &mut Reader, s: &TpShape) -> Result<Shape, Failure> {
    let (kind, i) = (r.integer(), r.integer() as usize);
    let at = |shape: Option<Shape>| {
        shape.ok_or_else(|| Failure::out_of_range("tp_bxe_sub: sub-shape index"))
    };
    match kind {
        1 => at(s.vertices.get(i).map(|v| v.clone().into_shape())),
        2 => at(s.edges.get(i).map(|e| e.clone().into_shape())),
        3 => at(s.faces.get(i).map(|f| f.clone().into_shape())),
        5..=7 => {
            let t = match kind {
                5 => ShapeEnum::Face,
                6 => ShapeEnum::Edge,
                _ => ShapeEnum::Vertex,
            };
            let mut found = Explorer::new(&s.shape, t, ShapeEnum::Shape)?.skip(i);
            Ok(found.next().unwrap_or_default())
        }
        _ => Ok(s.shape.clone()),
    }
}

/// `tp_bxe_vertex`: `x y z tol` (BRepBuilderAPI_MakeVertex for a tolerance of 0).
pub fn vertex(r: &mut Reader) -> Result<Vertex, Failure> {
    let p = r.pnt();
    let tol = r.real();
    if tol <= 0.0 {
        return BRepBuilderAPIMakeVertex::new(p)?.vertex();
    }
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, p, tol)?;
    Ok(v)
}

/// `tp_bxe_flag`.
pub fn ext_flag(f: i32) -> ExtFlag {
    match f {
        0 => ExtFlag::Min,
        1 => ExtFlag::Max,
        _ => ExtFlag::MinMax,
    }
}

/// `tp_bxe_algo`.
pub fn ext_algo(a: i32) -> ExtAlgo {
    if a == 0 { ExtAlgo::Grad } else { ExtAlgo::Tree }
}

/// `tp_bxd_solution`.
fn solution(a: &TpShape, b: &TpShape, d: &BRepExtremaDistShapeShape, n: i32) {
    attempt(|| d.support_type_shape1(n).map(|k| int(k as i32)));
    attempt(|| d.support_type_shape2(n).map(|k| int(k as i32)));
    attempt(|| d.point_on_shape1(n).map(pnt));
    attempt(|| d.point_on_shape2(n).map(pnt));
    attempt(|| id(a, b, &d.support_on_shape1(n)?));
    attempt(|| id(a, b, &d.support_on_shape2(n)?));
    attempt(|| d.par_on_edge_s1(n).map(real));
    attempt(|| d.par_on_edge_s2(n).map(real));
    attempt(|| {
        d.par_on_face_s1(n)
            .map(|(u, v)| (real(u), real(v)))
            .map(|_| ())
    });
    attempt(|| {
        d.par_on_face_s2(n)
            .map(|(u, v)| (real(u), real(v)))
            .map(|_| ())
    });
}

/// `tp_bxd_state`.
fn state(a: &TpShape, b: &TpShape, d: &BRepExtremaDistShapeShape) {
    flag(d.is_done());
    int(d.nb_solution());
    attempt(|| d.value().map(real));
    flag(d.inner_solution());
    flag(d.is_multi_thread());
    for i in 1..=d.nb_solution().min(40) {
        solution(a, b, d, i);
    }
    solution(a, b, d, 0);
    solution(a, b, d, d.nb_solution() + 1);
}

/// The performed distance of `bxd_dist` for the mode, or `None` when a constructor raised.
fn build_dist(
    (a, b): (&TpShape, &TpShape),
    defl: f64,
    mode: i32,
    (mt, f, al): (bool, ExtFlag, ExtAlgo),
) -> Option<BRepExtremaDistShapeShape> {
    let range = ProgressRange::default();
    let built = match mode {
        0 => BRepExtremaDistShapeShape::with_deflection(&a.shape, &b.shape, defl, (f, al), &range),
        2 => BRepExtremaDistShapeShape::with_shapes(&a.shape, &b.shape, (f, al)),
        _ => {
            let mut d = BRepExtremaDistShapeShape::new();
            d.set_multi_thread(mt);
            d.set_flag(f);
            d.set_algo(al);
            d.set_deflection(defl);
            attempt(|| d.load_s1(&a.shape));
            attempt(|| d.load_s2(&b.shape));
            attempt(|| d.perform(&range).map(flag));
            Ok(d)
        }
    };
    match built {
        Ok(d) => Some(d),
        Err(error) => {
            print!("{} ", error.exception_type());
            None
        }
    }
}

/// `bxd_dist`: `shape1 shape2 deflection mode multithread flag algo again`.
pub fn dist_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let b = tp_shape(&mut r)?;
        let (defl, mode, mt) = (r.real(), r.integer(), r.flag());
        let (f, al, again) = (ext_flag(r.integer()), ext_algo(r.integer()), r.flag());
        let Some(mut d) = build_dist((&a, &b), defl, mode, (mt, f, al)) else {
            return Ok(());
        };
        state(&a, &b, &d);
        if again {
            let range = ProgressRange::default();
            attempt(|| d.perform(&range).map(flag));
            state(&a, &b, &d);
            attempt(|| d.load_s1(&b.shape));
            attempt(|| d.load_s2(&a.shape));
            attempt(|| d.perform(&range).map(flag));
            state(&b, &a, &d);
        }
        Ok(())
    });
    end();
}

/// Dump of a distance into the captured text.
fn dump(d: &BRepExtremaDistShapeShape, console: &mut String) -> Result<(), Failure> {
    let mut bytes: Vec<u8> = Vec::new();
    let result = d.dump(&mut bytes);
    console.push_str(&String::from_utf8_lossy(&bytes));
    result
}

/// `bxd_dump`: `shape1 shape2 deflection`.
pub fn dump_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let b = tp_shape(&mut r)?;
        let defl = r.real();
        let options = (ExtFlag::MinMax, ExtAlgo::Grad);
        let range = ProgressRange::default();
        let d =
            BRepExtremaDistShapeShape::with_deflection(&a.shape, &b.shape, defl, options, &range)?;
        captured(|console| dump(&d, console));
        let empty = BRepExtremaDistShapeShape::new();
        captured(|console| dump(&empty, console));
        Ok(())
    });
    end();
}

/// One solution of DistanceSS.
fn ss_solution(a: &TpShape, b: &TpShape, s: &BRepExtremaSolutionElem) -> Result<(), Failure> {
    real(s.dist());
    pnt(*s.point());
    int(s.support_kind() as i32);
    id(a, b, s.vertex())?;
    id(a, b, s.edge())?;
    id(a, b, s.face())?;
    real(s.edge_parameter());
    let (u, v) = s.face_parameter();
    real(u);
    real(v);
    Ok(())
}

/// `bxd_ss`: `shape1 sub1 shape2 sub2 boxes dstref deflection flag algo`.
pub fn ss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let s1 = sub(&mut r, &a)?;
        let b = tp_shape(&mut r)?;
        let s2 = sub(&mut r, &b)?;
        let (boxes, dst, defl) = (r.flag(), r.real(), r.real());
        let options = (ext_flag(r.integer()), ext_algo(r.integer()));
        let (mut b1, mut b2) = (BoundingBox3::default(), BoundingBox3::default());
        if boxes {
            BRepBndLib::add(&s1, &mut b1, true)?;
            BRepBndLib::add(&s2, &mut b2, true)?;
        }
        let ss = BRepExtremaDistanceSS::new(&s1, &s2, (&b1, &b2), dst, defl, options)?;
        flag(ss.is_done());
        real(ss.dist_value());
        int(ss.seq1_value().len() as i32);
        int(ss.seq2_value().len() as i32);
        for s in ss.seq1_value().iter().chain(ss.seq2_value()) {
            ss_solution(&a, &b, s)?;
        }
        Ok(())
    });
    end();
}

/// `bxd_poly`: `shape1 shape2`.
pub fn poly_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let b = tp_shape(&mut r)?;
        let (found, nearest) = BRepExtremaPoly::distance(&a.shape, &b.shape)?;
        flag(found);
        pnt(nearest.p1);
        pnt(nearest.p2);
        real(nearest.dist);
        Ok(())
    });
    end();
}
