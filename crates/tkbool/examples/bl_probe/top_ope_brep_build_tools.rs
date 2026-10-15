//! TopOpeBRepBuild_Tools probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_topopebrepbuild_tools.cxx (the records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int, real, xyz};
use super::shape::shape;
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use rustycad_tkbool::top_ope_brep_build::{DataMapOfShapeState, TopOpeBRepBuildTools};
use rustycad_tkbool::top_ope_brep_ds::MapOfShapeWithState;
use rustycad_tkbool::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Vector3;

/// The program of a request, `None` (printed) when it fails.
fn program(r: &mut Reader<'_>) -> Option<TpShapes> {
    tp_shapes(r).map_err(failed).ok()
}

/// A vector as its coordinates.
fn vec(v: Vector3) {
    xyz(*v.xyz());
}

/// `tb_tools_geom program face edge edge2`.
pub fn geom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (fi, ei, e2i) = (r.integer(), r.integer(), r.integer());
    let shapes = (|| Ok::<_, Failure>((at(&t, fi)?, at(&t, ei)?, at(&t, e2i)?)))();
    let (f, e, e2) = match shapes {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    attempt(|| {
        vec(TopOpeBRepBuildTools::get_normal_to_face_on_edge(
            &topods::face(&f)?,
            &topods::edge(&e)?,
        )?);
        Ok(())
    });
    attempt(|| {
        vec(TopOpeBRepBuildTools::get_normal_in_nearest_point(
            &topods::face(&f)?,
            &topods::edge(&e)?,
        )?);
        Ok(())
    });
    attempt(|| {
        let (ok, v) = TopOpeBRepBuildTools::get_tangent_to_edge(&topods::edge(&e)?)?;
        flag(ok);
        vec(v);
        Ok(())
    });
    attempt(|| {
        let (ok, v) = TopOpeBRepBuildTools::get_tangent_to_edge_edge(
            &topods::face(&f)?,
            &topods::edge(&e)?,
            &topods::edge(&e2)?,
        )?;
        flag(ok);
        vec(v);
        Ok(())
    });
    attempt(|| {
        flag(TopOpeBRepBuildTools::is_deg_edges_the_same(&e, &e2)?);
        Ok(())
    });
    attempt(|| {
        flag(TopOpeBRepBuildTools::check_face_closed2d(&topods::face(
            &f,
        )?)?);
        Ok(())
    });
    attempt(|| shape(&TopOpeBRepBuildTools::normalize_face(&f)?));
    end();
}

/// `bl_tb_dump_sws`.
fn dump_sws(t: &TpShapes, m: &MapOfShapeWithState) -> Result<(), Failure> {
    int(m.extent() as i32);
    for i in 1..=m.extent() {
        int(tp_same_index(t, m.find_key(i)?));
        let sws = m.find_from_index(i)?;
        int(sws.state() as i32);
        flag(sws.is_splitted());
    }
    Ok(())
}

/// The PropagateState part of `tb_tools_states`.
fn propagate(t: &TpShapes, s: &Shape, rf: &Shape, avoid: &MapOfShape) -> Result<(), Failure> {
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    sc.set_reference(rf)?;
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, ShapeEnum::Face, &mut faces)?;
    let mut spl = DataMapOfShapeState::default();
    spl.bind(faces.find_key(1)?.clone(), State::In);
    let mut rest = IndexedMapOfShape::default();
    for f in faces.iter().skip(1) {
        rest.add(f.clone());
    }
    let mut m = MapOfShapeWithState::default();
    TopOpeBRepBuildTools::propagate_state(
        &spl,
        &rest,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        &mut sc,
        &mut m,
        avoid,
    )?;
    TopOpeBRepBuildTools::propagate_state_for_wires(&rest, &mut m)?;
    dump_sws(t, &m)
}

/// `tb_tools_states program shape ref avoid`.
pub fn states_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (si, ri, av) = (r.integer(), r.integer(), r.integer());
    let shapes = (|| {
        Ok::<_, Failure>((
            at(&t, si)?,
            at(&t, ri)?,
            if av >= 0 { Some(at(&t, av)?) } else { None },
        ))
    })();
    let (s, rf, avs) = match shapes {
        Ok(x) => x,
        Err(error) => return failed(error),
    };
    let mut avoid = MapOfShape::default();
    if let Some(a) = avs {
        avoid.add(a);
    }
    attempt(|| {
        let mut sc = TopOpeBRepToolShapeClassifier::new();
        sc.set_reference(&rf)?;
        let mut m = MapOfShapeWithState::default();
        int(TopOpeBRepBuildTools::find_state_through_vertex(&s, &mut sc, &mut m, &avoid)? as i32);
        dump_sws(&t, &m)
    });
    attempt(|| propagate(&t, &s, &rf, &avoid));
    end();
}

/// `tb_tools_tolerances program shape maxtol`.
pub fn tolerances_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (si, maxtol) = (r.integer(), r.real());
    let s = match at(&t, si) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    attempt(|| TopOpeBRepBuildTools::correct_tolerances(&s, maxtol));
    let (mut vs, mut es) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    let maps = topexp::map_shapes_of_type(&s, ShapeEnum::Vertex, &mut vs)
        .and_then(|()| topexp::map_shapes_of_type(&s, ShapeEnum::Edge, &mut es));
    if let Err(error) = maps {
        return failed(error);
    }
    int(vs.extent() as i32);
    for v in vs.iter() {
        real(
            topods::vertex(v)
                .and_then(|v| BRepTool::vertex_tolerance(&v))
                .unwrap_or(f64::NAN),
        );
    }
    int(es.extent() as i32);
    for e in es.iter() {
        real(
            topods::edge(e)
                .and_then(|e| BRepTool::edge_tolerance(&e))
                .unwrap_or(f64::NAN),
        );
    }
    end();
}
