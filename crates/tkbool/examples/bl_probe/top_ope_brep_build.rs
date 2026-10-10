//! TopOpeBRepBuild face chain probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_topopebrepbuild.cxx (the records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int};
use super::shape::shape;
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use rustycad_tkbool::top_ope_brep_build::{
    ShapeToShapeMap, TopOpeBRepBuildBlockBuilder, TopOpeBRepBuildBuilder,
    TopOpeBRepBuildCompositeClassifier, TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildWireEdgeClassifier, TopOpeBRepBuildWireEdgeSet, TopOpeBRepBuildWireToFace,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use rustycad_tkbool::top_ope_brep_tool::TopOpeBRepToolOutCurveType;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The program of a request, `None` (printed) when it fails.
fn program(r: &mut Reader<'_>) -> Option<TpShapes> {
    tp_shapes(r).map_err(failed).ok()
}

/// A state as its TopAbs_State value.
fn put_state(st: State) -> Result<(), Failure> {
    int(st as i32);
    Ok(())
}

/// The count and each face of a list.
fn faces(lf: &List<Shape>) -> Result<(), Failure> {
    int(lf.len() as i32);
    lf.iter().try_for_each(shape)
}

/// `tb_wire_to_face program face n wire...`.
pub fn wire_to_face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let face_index = r.integer();
    let n = r.integer();
    let wires: Vec<i32> = (0..n).map(|_| r.integer()).collect();
    attempt(|| {
        let mut wtf = TopOpeBRepBuildWireToFace::new();
        for w in &wires {
            wtf.add_wire(&topods::wire(&at(&t, *w)?)?);
        }
        let mut lf = List::default();
        wtf.make_faces(&topods::face(&at(&t, face_index)?)?, &mut lf)?;
        faces(&lf)
    });
    end();
}

/// The edges of the current new wire of `bl_tb_dump_face_builder`.
fn dump_new_wire(fb: &mut TopOpeBRepBuildFaceBuilder, t: &TpShapes) -> Result<(), Failure> {
    int(fb.init_edge()?);
    while fb.more_edge() {
        int(tp_same_index(t, fb.edge()?));
        int(fb.edge()?.orientation() as i32);
        fb.next_edge()?;
    }
    int(-3);
    Ok(())
}

/// `bl_tb_dump_face_builder`.
fn dump_face_builder(fb: &mut TopOpeBRepBuildFaceBuilder, t: &TpShapes) -> Result<(), Failure> {
    int(fb.init_face());
    while fb.more_face() {
        int(fb.init_wire());
        while fb.more_wire() {
            let old = fb.is_old_wire()?;
            flag(old);
            if old {
                int(tp_same_index(t, &fb.old_wire()?));
            } else {
                dump_new_wire(fb, t)?;
            }
            fb.next_wire();
        }
        fb.next_face();
    }
    Ok(())
}

/// The wire edge set of `tb_face_builder`.
fn wire_edge_set(
    r: &mut Reader<'_>,
    t: &TpShapes,
    f: &Shape,
) -> Result<TopOpeBRepBuildWireEdgeSet, Failure> {
    let mut wes = TopOpeBRepBuildWireEdgeSet::new(f)?;
    let n = r.integer();
    for _ in 0..n {
        let kind = r.integer();
        let s = at(t, r.integer())?;
        attempt(|| match kind {
            0 => wes.add_shape(&s),
            1 => wes.add_start_element(&s),
            _ => wes.add_element(&s),
        });
    }
    Ok(wes)
}

/// The detections of `tb_face_builder`.
fn detections(fb: &mut TopOpeBRepBuildFaceBuilder, t: &TpShapes) {
    attempt(|| {
        let mut me: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        fb.detect_pseudo_internal_edge(&mut me)?;
        int(me.len() as i32);
        (1..=me.len()).try_for_each(|i| {
            int(tp_same_index(t, me.find_key(i)?));
            Ok(())
        })
    });
    attempt(|| {
        let (mut vv, mut v1e) = (ShapeToShapeMap::default(), ShapeToShapeMap::default());
        fb.detect_unclosed_wire(&mut vv, &mut v1e)?;
        int(vv.len() as i32);
        int(v1e.len() as i32);
        Ok(())
    });
}

/// `tb_face_builder program face force n (kind shape)...`.
pub fn face_builder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let f = match at(&t, r.integer()) {
        Ok(f) => f,
        Err(error) => return failed(error),
    };
    let force = r.flag();
    let mut fb = TopOpeBRepBuildFaceBuilder::new();
    let built =
        wire_edge_set(&mut r, &t, &f).and_then(|mut wes| fb.init_face_builder(&mut wes, &f, force));
    if let Err(error) = built {
        return failed(error);
    }
    attempt(|| dump_face_builder(&mut fb, &t));
    detections(&mut fb, &t);
    attempt(|| dump_face_builder(&mut fb, &t));
    attempt(|| {
        let bt = TopOpeBRepDSBuildTool::with_type(TopOpeBRepToolOutCurveType::Approx);
        let mut lf = List::default();
        TopOpeBRepBuildBuilder::new(&bt).make_faces(&f, &mut fb, &mut lf)?;
        faces(&lf)
    });
    end();
}

/// The edges of a shape.
fn edges_of(s: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

/// `tb_wec program face s1 s2`.
pub fn wec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (fi, i1, i2) = (r.integer(), r.integer(), r.integer());
    let bb = TopOpeBRepBuildBlockBuilder::new();
    let run = || -> Result<(), Failure> {
        let (f, s1, s2) = (at(&t, fi)?, at(&t, i1)?, at(&t, i2)?);
        let mut wec = TopOpeBRepBuildWireEdgeClassifier::new(&f, &bb)?;
        attempt(|| put_state(wec.compare_shapes(&s1, &s2)?));
        attempt(|| {
            let first = edges_of(&s1)?.into_iter().next().ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "TopExp_Explorer::Current: no edge",
                    "TopExp_Explorer",
                )
            })?;
            put_state(wec.compare_element_to_shape(&first, &s2)?)
        });
        attempt(|| {
            wec.reset_shape(&s1)?;
            for e in edges_of(&s2)? {
                flag(wec.compare_element(&e)?);
            }
            put_state(wec.state())
        });
        Ok(())
    };
    attempt(run);
    end();
}
