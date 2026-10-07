//! BRepAlgo_NormalProjection and BRepProj_Projection probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepproj.cxx (the records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_algo::BRepAlgoNormalProjection;
use rustycad_tkbool::brep_proj::BRepProjProjection;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Point3};

/// `bl_program_index`: the index of a shape among the program shapes, -1 for none.
fn program_index(t: &TpShapes, s: &Shape) -> i32 {
    t.shapes
        .iter()
        .position(|p| p.is_same(s))
        .map_or(-1, |i| i as i32)
}

/// The edges of the projection.
fn projected_edges(s: &Shape) -> Vec<Shape> {
    Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)
        .map(|e| e.collect())
        .unwrap_or_default()
}

/// The projection of the request, its edges added; the edges.
fn projection_of(
    r: &mut Reader<'_>,
    t: &TpShapes,
) -> Result<(BRepAlgoNormalProjection, Vec<Shape>), Failure> {
    let mut np = BRepAlgoNormalProjection::with_shape(&at(t, r.integer())?);
    let mut edges = Vec::new();
    for _ in 0..r.integer() {
        let e = at(t, r.integer())?;
        np.add(&e)?;
        edges.push(e);
    }
    np.set_max_distance(r.real());
    np.compute3d(r.flag());
    np.set_limit(r.flag());
    Ok((np, edges))
}

/// `ba_normal_projection program shape n (edge)... maxdist with3d limit`.
pub fn normal_projection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (np, edges) =
        match projection_of(&mut r, &t).and_then(|(mut np, e)| np.build().map(|()| (np, e))) {
            Ok(v) => v,
            Err(error) => return failed(error),
        };
    flag(np.is_done());
    attempt(|| shape(np.projection()));
    for e in projected_edges(np.projection()) {
        attempt(|| {
            np.ancestor(&topods::edge(&e)?)
                .map(|a| int(program_index(&t, a)))
        });
        attempt(|| {
            np.couple(&topods::edge(&e)?)
                .map(|c| int(program_index(&t, c)))
        });
    }
    for e in &edges {
        attempt(|| np.generated(e).map(|g| int(g.len() as i32)));
    }
    attempt(|| {
        let mut wires: List<Shape> = List::default();
        flag(np.build_wire(&mut wires)?);
        int(wires.len() as i32);
        Ok(())
    });
    end();
}

/// `bp_projection program wire shape mode x y z`.
pub fn projection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (wi, si, mode) = (r.integer(), r.integer(), r.integer());
    let (x, y, z) = (r.real(), r.real(), r.real());
    let built = (|| {
        let (w, s) = (at(&t, wi)?, at(&t, si)?);
        if mode == 0 {
            BRepProjProjection::along(&w, &s, Direction3::new(x, y, z)?)
        } else {
            BRepProjProjection::from_point(&w, &s, Point3::new(x, y, z))
        }
    })();
    let mut p = match built {
        Ok(p) => p,
        Err(error) => return failed(error),
    };
    flag(p.is_done());
    attempt(|| shape(p.shape().as_ref()));
    let mut n = 0;
    p.init();
    while p.more() {
        n += 1;
        p.next();
    }
    int(n);
    p.init();
    while p.more() {
        attempt(|| shape(p.current()?.as_ref()));
        p.next();
    }
    end();
}
