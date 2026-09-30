//! BRepBuilderAPI modification probe operations (ModifyShape, Transform, GTransform, Copy,
//! NurbsConvert, Collect, FindPlane), the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbuilderapi_modify.cxx (the request layout is
//! documented there).
use super::brep_builder_api_makers::made;
use super::brep_lib_package::{guarded, sub_shapes};
use super::brep_lib_shapes::{mesh, shape_record};
use super::common::{Reader, dir, flag, int, pnt};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, CopyModification};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, GeneralTransform3, Matrix3, Point3, Transform3, Vector3,
};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICollect, BRepBuilderAPICopy, BRepBuilderAPIFindPlane, BRepBuilderAPIGTransform,
    BRepBuilderAPIMakeShape, BRepBuilderAPIModifyShape, BRepBuilderAPINurbsConvert,
    BRepBuilderAPITransform, ShapeHistoryMap,
};

/// The `ModifiedShape` member each probed class declares (virtual natively).
trait ModifiedShapeProbe: BRepBuilderAPIMakeShape {
    fn image_of(&self, shape: &Shape) -> Result<Shape, Failure>;
}
macro_rules! modified_shape_probe {
    ($($t:ty),*) => {
        $(impl ModifiedShapeProbe for $t {
            fn image_of(&self, shape: &Shape) -> Result<Shape, Failure> {
                self.modified_shape(shape)
            }
        })*
    };
}
modified_shape_probe!(
    BRepBuilderAPIModifyShape,
    BRepBuilderAPITransform,
    BRepBuilderAPIGTransform,
    BRepBuilderAPICopy,
    BRepBuilderAPINurbsConvert
);

/// `tp_bba_trsf`.
fn trsf(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let mut t = Transform3::default();
    match r.integer() {
        0 => {
            let p = r.pnt();
            t.set_translation(Vector3::new(p.x(), p.y(), p.z()));
        }
        1 => {
            let (p, d) = (r.pnt(), r.dir());
            t.set_rotation(Axis1::new(p, d), r.real())?;
        }
        2 => {
            let p = r.pnt();
            t.set_scale(p, r.real())?;
        }
        3 => t.set_mirror_point(r.pnt()),
        4 => {
            let p = r.pnt();
            t.set_mirror_axis(Axis1::new(p, r.dir()));
        }
        5 => {
            let p = r.pnt();
            t.set_mirror_plane(Axis2::new(p, r.dir())?);
        }
        6 => {
            let p = r.pnt();
            t.set_transformation_to(Axis3::from_normal(p, r.dir())?);
        }
        7 => {
            let a = trsf(r)?;
            let b = trsf(r)?;
            t = a.multiplied(b);
        }
        _ => {}
    }
    Ok(t)
}

/// `tp_bba_gtrsf`.
fn gtrsf(r: &mut Reader<'_>) -> Result<GeneralTransform3, Failure> {
    let mut g = GeneralTransform3::default();
    match r.integer() {
        0 => g = GeneralTransform3::from(trsf(r)?),
        1 => {
            let (p, d) = (r.pnt(), r.dir());
            g.set_affinity_axis(Axis1::new(p, d), r.real());
        }
        2 => {
            let (p, d) = (r.pnt(), r.dir());
            g.set_affinity_plane(Axis2::new(p, d)?, r.real());
        }
        _ => {
            let mut m = Matrix3::default();
            for i in 1..=3 {
                for j in 1..=3 {
                    m.set_value(i, j, r.real())?;
                }
            }
            g.set_vectorial_part(m);
            g.set_translation_part(r.pnt().xyz());
        }
    }
    Ok(g)
}

/// `tp_bba_all`: the sub-shapes in TopExp::MapShapes order (the shape first).
fn all(s: &Shape) -> IndexedMapOfShape {
    let mut m = IndexedMapOfShape::default();
    if !s.is_null() {
        topexp::map_shapes(s, &mut m, true, true);
    }
    m
}

/// `tp_bba_image`.
fn image(image: &Shape, out: &IndexedMapOfShape) {
    if image.is_null() {
        int(-1);
        return;
    }
    int(out.find_index(image) as i32);
    int(image.orientation() as i32);
}

/// Runs a fallible print, printing the exception type on failure (GA_TRY).
fn guard(body: impl FnOnce() -> Result<(), Failure>) {
    super::common::attempt(body);
}

/// `tp_bba_images`.
fn images<A: ModifiedShapeProbe>(
    a: &mut A,
    input: &IndexedMapOfShape,
    out: &IndexedMapOfShape,
) -> Result<(), Failure> {
    int(input.extent() as i32);
    for i in 1..=input.extent() {
        let s = input.find_key(i)?.clone();
        guard(|| {
            image(&a.image_of(&s)?, out);
            Ok(())
        });
        guard(|| {
            let list = a.modified(&s)?;
            int(list.extent());
            list.iter().for_each(|m| image(m, out));
            Ok(())
        });
        guard(|| {
            int(a.generated(&s)?.extent());
            flag(a.is_deleted(&s)?);
            Ok(())
        });
    }
    let mut foreign = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut foreign, Point3::new(1., 2., 3.), 1e-7)?;
    guard(|| {
        image(&a.image_of(&foreign)?, out);
        Ok(())
    });
    Ok(())
}

/// `tp_bba_shared`.
fn shared<A: ModifiedShapeProbe>(a: &A, input: &Shape) -> Result<(), Failure> {
    let faces = sub_shapes(input, ShapeEnum::Face)?;
    for i in 1..=faces.extent() {
        let f = faces.find_key(i)?;
        guard(|| {
            let im = a.image_of(f)?;
            let (s1, _) = BRepTool::surface_with_location(&topods::face(f)?)?;
            let (s2, _) = BRepTool::surface_with_location(&topods::face(&im)?)?;
            flag(s1.object_address() == s2.object_address());
            Ok(())
        });
    }
    let edges = sub_shapes(input, ShapeEnum::Edge)?;
    for i in 1..=edges.extent() {
        let e = edges.find_key(i)?;
        guard(|| {
            let im = a.image_of(e)?;
            let c1 = BRepTool::curve_with_location(&topods::edge(e)?)?.curve;
            let c2 = BRepTool::curve_with_location(&topods::edge(&im)?)?.curve;
            flag(c1.object_address() == c2.object_address());
            Ok(())
        });
    }
    Ok(())
}

/// `tp_bba_meshes`.
fn meshes(s: &Shape) -> Result<(), Failure> {
    let faces = sub_shapes(s, ShapeEnum::Face)?;
    for i in 1..=faces.extent() {
        let (t, l) =
            BRepTool::triangulation(&topods::face(faces.find_key(i)?)?, mesh_purpose::NONE)?;
        let Some(t) = t.get() else {
            int(-1);
            continue;
        };
        int(t.nb_nodes());
        pnt(t.node(1)?.transformed(l.transformation()));
    }
    Ok(())
}

/// `tp_bba_modified`.
fn modified<A: ModifiedShapeProbe>(a: &mut A, input: &Shape) -> Result<(), Failure> {
    made(a, true);
    if !a.is_done() {
        return Ok(());
    }
    let result = a.shape()?.clone();
    let out = all(&result);
    images(a, &all(input), &out)?;
    shared(a, input)?;
    meshes(&result)
}

/// `tp_bba_meshed_record`.
fn meshed_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let s = shape_record(r)?;
    let n = r.integer();
    if n > 0 {
        mesh(&s, n, false)?;
    }
    Ok(s)
}

/// `bba_transform`.
pub fn transform_case(values: &[f64]) {
    guarded(values, |r| {
        let s = meshed_record(r)?;
        let t = trsf(r)?;
        let (g, m) = (r.flag(), r.flag());
        let mode = r.integer();
        let mut a = if mode == 0 {
            BRepBuilderAPITransform::with_shape(&s, t, g, m)?
        } else {
            let mut a = BRepBuilderAPITransform::new(t);
            a.perform(&s, g, m)?;
            if mode == 2 {
                a.perform(&s, !g, m)?;
            }
            a
        };
        modified(&mut a, &s)
    });
}

/// `bba_gtransform`.
pub fn gtransform_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let t = gtrsf(r)?;
        let (c, mode) = (r.flag(), r.integer());
        let mut a = if mode == 1 {
            let mut a = BRepBuilderAPIGTransform::new(t);
            a.perform(&s, c)?;
            a
        } else {
            let mut a = BRepBuilderAPIGTransform::with_shape(&s, t, c)?;
            if mode == 2 {
                a.perform(&s, c)?;
            }
            a
        };
        modified(&mut a, &s)
    });
}

/// `bba_copy`.
pub fn copy_case(values: &[f64]) {
    guarded(values, |r| {
        let s = meshed_record(r)?;
        let (g, m) = (r.flag(), r.flag());
        let mode = r.integer();
        let mut a = if mode == 1 {
            let mut a = BRepBuilderAPICopy::new();
            a.perform(&s, g, m)?;
            a
        } else {
            let mut a = BRepBuilderAPICopy::with_shape(&s, g, m)?;
            if mode == 2 {
                a.perform(&s, g, m)?;
            }
            a
        };
        modified(&mut a, &s)
    });
}

/// `bba_nurbs`.
pub fn nurbs_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (c, mode) = (r.flag(), r.integer());
        let mut a = if mode == 1 {
            let mut a = BRepBuilderAPINurbsConvert::new();
            a.perform(&s, c)?;
            a
        } else {
            let mut a = BRepBuilderAPINurbsConvert::with_shape(&s, c)?;
            if mode == 2 {
                a.perform(&s, c)?;
            }
            a
        };
        modified(&mut a, &s)
    });
}

/// `tp_bba_modify_mode`.
fn modify_mode(mode: i32, s: &Shape) -> Result<BRepBuilderAPIModifyShape, Failure> {
    let m: Handle<dyn BRepToolsModification> =
        Handle::new(CopyModification::new(true, false)).cast();
    Ok(match mode {
        0 => {
            let mut a = BRepBuilderAPIModifyShape::with_shape(s)?;
            a.do_modif_modification(&m)?;
            a
        }
        1 | 4 | 6 => {
            let mut a = BRepBuilderAPIModifyShape::with_modification(&m);
            let null = Shape::default();
            a.do_modif_shape(if mode == 6 { &null } else { s })?;
            a
        }
        2 => BRepBuilderAPIModifyShape::with_shape_modification(s, &m)?,
        3 => {
            let mut a = BRepBuilderAPIModifyShape::new();
            a.do_modif_shape_modification(s, &m)?;
            a
        }
        5 => {
            let mut a = BRepBuilderAPIModifyShape::new();
            a.do_modif_shape(s)?;
            a
        }
        _ => BRepBuilderAPIModifyShape::with_shape(s)?,
    })
}

/// `bba_modify`.
pub fn modify_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let mode = r.integer();
        let mut a = modify_mode(mode, &s)?;
        if mode == 4 {
            let first = a.shape()?.clone();
            a.do_modif_shape(&s)?;
            flag(first.is_equal(a.shape()?));
        }
        if mode == 7 {
            flag(a.is_done());
            guard(|| {
                image(&a.modified_shape(&s)?, &IndexedMapOfShape::default());
                Ok(())
            });
        }
        modified(&mut a, &s)
    });
}

/// `bba_findplane`.
pub fn findplane_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let f = BRepBuilderAPIFindPlane::from_shape(&s, r.real())?;
        flag(f.found());
        if let Some(plane) = f.plane().get() {
            let a = plane.pln().position();
            pnt(a.location());
            dir(a.direction());
            dir(a.x_direction());
        }
        Ok(())
    });
}

/// One algorithm step of `bba_collect` (ops 0 to 3): the algorithm run on `cur`.
fn collect_algorithm(op: i32, cur: &Shape) -> Result<Box<dyn BRepBuilderAPIMakeShape>, Failure> {
    let mut t = Transform3::default();
    t.set_translation(Vector3::new(1., 2., 3.));
    Ok(match op {
        0 => Box::new(BRepBuilderAPITransform::with_shape(cur, t, true, false)?),
        1 => Box::new(BRepBuilderAPINurbsConvert::with_shape(cur, false)?),
        2 => Box::new(BRepBuilderAPICopy::with_shape(cur, true, false)?),
        _ => Box::new(BRepBuilderAPITransform::with_shape(cur, t, false, false)?),
    })
}

/// `tp_bba_collect_step`.
fn collect_step(
    op: i32,
    r: &mut Reader<'_>,
    c: &mut BRepBuilderAPICollect,
    cur: &mut Shape,
) -> Result<(), Failure> {
    if op == 4 || op == 5 {
        let m = all(cur);
        let a = m.find_key(r.integer() as usize)?.clone();
        let b = m.find_key(r.integer() as usize)?.clone();
        return if op == 4 {
            c.add_generated(&a, &b)
        } else {
            c.add_modif(&a, &b)
        };
    }
    if op == 6 {
        return c.filter(cur);
    }
    let mut a = collect_algorithm(op, cur)?;
    c.add(cur, a.as_mut())?;
    *cur = a.shape()?.clone();
    Ok(())
}

/// `tp_bba_collect_list`.
fn collect_list(h: &ShapeHistoryMap, key: &Shape, out: &IndexedMapOfShape) {
    let l = h.seek(key);
    flag(l.is_some());
    let Some(l) = l else { return };
    int(l.extent());
    l.iter().for_each(|s| image(s, out));
}

/// `bba_collect`.
pub fn collect_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let mut cur = s.clone();
        let mut c = BRepBuilderAPICollect::new();
        let n = r.integer();
        for _ in 0..n {
            let op = r.integer();
            collect_step(op, r, &mut c, &mut cur)?;
        }
        let (input, out) = (all(&s), all(&cur));
        for i in 1..=input.extent() {
            let key = input.find_key(i)?;
            collect_list(c.modification(), key, &out);
            collect_list(c.generated(), key, &out);
        }
        Ok(())
    });
}
