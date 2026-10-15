//! BRepOffset_MakeOffset probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffset_make_offset.cxx (the records are documented there).
use super::common::{Reader, attempt, flag, int};
use super::tool::{finish, primitive};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset::{
    BRepOffsetMakeOffset, BRepOffsetMakeSimpleOffset, BRepOffsetMode, MakeOffsetOptions,
};
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// A value rounded to 1e-6.
pub(super) fn round6(v: f64) -> i64 {
    (v * 1e6).round() as i64
}

/// The sub-shapes of a type.
fn mapped(s: &Shape, t: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, t, &mut m)?;
    Ok(m)
}

/// The order-independent digest of a result.
pub(super) fn digest(s: &Shape) -> Result<(), Failure> {
    flag(s.is_null());
    if s.is_null() {
        return Ok(());
    }
    int(s.shape_type()? as i32);
    for t in [
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        int(mapped(s, t)?.extent() as i32);
    }
    let fm = mapped(s, ShapeEnum::Face)?;
    let mut faces = Vec::new();
    for i in 1..=fm.extent() {
        let f = topods::face(fm.find_key(i)?)?;
        let mut p = GProps::new();
        BRepGProp::surface_properties(f.as_ref(), &mut p, false, false)?;
        let a = BRepAdaptorSurface::with_face(&f, false)?;
        faces.push(vec![
            a.get_type()? as i64,
            f.orientation() as i64,
            round6(p.mass()),
            mapped(f.as_ref(), ShapeEnum::Edge)?.extent() as i64,
        ]);
    }
    faces.sort();
    for r in &faces {
        for v in r {
            print!("{v} ");
        }
    }
    let vm = mapped(s, ShapeEnum::Vertex)?;
    let mut pts = Vec::new();
    for i in 1..=vm.extent() {
        let p = BRepTool::pnt(&topods::vertex(vm.find_key(i)?)?)?;
        pts.push(vec![round6(p.x()), round6(p.y()), round6(p.z())]);
    }
    pts.sort();
    for r in &pts {
        for v in r {
            print!("{v} ");
        }
    }
    let mut vp = GProps::new();
    BRepGProp::volume_properties(s, &mut vp, false, false, false)?;
    print!("{} ", round6(vp.mass()));
    Ok(())
}

/// The shell of faces of a primitive.
fn shell_of(p: &Shape, idx: &[usize]) -> Result<Shape, Failure> {
    let b = rustycad_tkbrep::brep::BRepBuilder::default();
    let mut sh = rustycad_tkbrep::topods::Shell::default();
    b.make_shell(&mut sh);
    let mut sh = sh.into_shape();
    let faces: Vec<Shape> = Explorer::new(p, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    for &i in idx {
        if let Some(f) = faces.get(i) {
            b.add(&mut sh, f)?;
        }
    }
    Ok(sh)
}

/// The count and types of a history list.
fn list(l: &rustycad_tkernel::ncollection::List<Shape>) -> Result<(), Failure> {
    int(l.len() as i32);
    for s in l.iter() {
        int(s.shape_type()? as i32);
    }
    Ok(())
}

/// The mode of its enumeration value.
fn mode_of(v: i32) -> BRepOffsetMode {
    match v {
        1 => BRepOffsetMode::Pipe,
        2 => BRepOffsetMode::RectoVerso,
        _ => BRepOffsetMode::Skin,
    }
}

/// `om_offset`.
pub fn offset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let sub = r.integer();
        let removed = r.integer();
        let off = r.real();
        let join = if r.flag() {
            JoinType::Intersection
        } else {
            JoinType::Arc
        };
        let intersection = r.flag();
        let thickening = r.flag();
        let remove_int_edges = r.flag();
        let mode = mode_of(r.integer());
        let src = match sub {
            -1 => p.clone(),
            -2 => shell_of(&p, &[0, 2])?,
            s if s < 0 => shell_of(&p, &[0, 2, 4])?,
            s => Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)?
                .nth(s as usize)
                .unwrap_or_default(),
        };
        let mut m = BRepOffsetMakeOffset::new();
        let options = MakeOffsetOptions {
            mode,
            intersection,
            self_inter: false,
            join,
            thickening,
            remove_int_edges,
        };
        m.initialize(&src, off, 1e-7, options);
        let range = ProgressRange::default();
        if removed >= 0 {
            let f = Explorer::new(&src, ShapeEnum::Face, ShapeEnum::Shape)?
                .nth(removed as usize)
                .unwrap_or_default();
            m.add_face(&topods::face(&f)?);
            m.make_thick_solid(&range)?;
        } else {
            m.make_offset_shape(&range)?;
        }
        int(m.error() as i32);
        flag(m.is_done());
        digest(&m.shape().clone())?;
        for t in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex] {
            for s in Explorer::new(&src, t, ShapeEnum::Shape)? {
                attempt(|| m.generated(&s).and_then(list));
                attempt(|| m.modified(&s).and_then(list));
                attempt(|| m.is_deleted(&s).map(flag));
            }
        }
        Ok(())
    })());
}

/// `om_face_offsets prim removed offset join inter n (face offset)...`.
pub fn face_offsets_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let removed = r.integer();
        let off = r.real();
        let join = if r.flag() {
            JoinType::Intersection
        } else {
            JoinType::Arc
        };
        let intersection = r.flag();
        let mut m = BRepOffsetMakeOffset::new();
        let options = MakeOffsetOptions {
            mode: BRepOffsetMode::Skin,
            intersection,
            self_inter: false,
            join,
            thickening: false,
            remove_int_edges: false,
        };
        m.initialize(&p, off, 1e-7, options);
        let face_of = |index: i32| -> Result<topods::Face, Failure> {
            topods::face(
                &Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)?
                    .nth(index as usize)
                    .unwrap_or_default(),
            )
        };
        for _ in 0..r.integer() {
            let f = face_of(r.integer())?;
            m.set_offset_on_face(&f, r.real());
        }
        let range = ProgressRange::default();
        if removed >= 0 {
            m.add_face(&face_of(removed)?);
            m.make_thick_solid(&range)?;
        } else {
            m.make_offset_shape(&range)?;
        }
        int(m.error() as i32);
        flag(m.is_done());
        digest(&m.shape().clone())?;
        for t in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex] {
            for s in Explorer::new(&p, t, ShapeEnum::Shape)? {
                attempt(|| m.generated(&s).and_then(list));
                attempt(|| m.modified(&s).and_then(list));
                attempt(|| m.is_deleted(&s).map(flag));
            }
        }
        Ok(())
    })());
}

/// `om_simple`.
pub fn simple_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let sub = r.integer();
        let off = r.real();
        let solid = r.flag();
        let src = match sub {
            -1 => p.clone(),
            -2 => shell_of(&p, &[0, 2])?,
            s if s < 0 => shell_of(&p, &[0, 2, 4])?,
            s => Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)?
                .nth(s as usize)
                .unwrap_or_default(),
        };
        let mut m = BRepOffsetMakeSimpleOffset::new();
        m.initialize(&src, off);
        m.set_build_solid_flag(solid);
        m.perform()?;
        int(m.error() as i32);
        flag(m.is_done());
        if m.is_done() {
            digest(&m.result_shape().clone())?;
        }
        Ok(())
    })());
}
