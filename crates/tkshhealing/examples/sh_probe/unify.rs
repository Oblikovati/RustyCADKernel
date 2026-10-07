//! ShapeUpgrade_UnifySameDomain and ShapeFix_FaceConnect probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_unify.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis,
    BRepPrimAPIMakeSphere,
};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire, BRepBuilderAPISewing,
};

fn sewn(shapes: &[Shape], tolerance: f64) -> Result<Shape, Failure> {
    let mut sewing = BRepBuilderAPISewing::new(tolerance, true, true, true, false);
    for s in shapes {
        sewing.add(s)?;
    }
    sewing.perform(&ProgressRange::default())?;
    Ok(sewing.sewed_shape().clone())
}

fn polygons(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let mut faces = Vec::new();
    for _ in 0..r.integer() {
        let mut polygon = BRepBuilderAPIMakePolygon::new();
        for _ in 0..r.integer() {
            polygon.add_point(r.pnt())?;
        }
        polygon.close()?;
        faces.push(
            BRepBuilderAPIMakeFace::from_wire(&polygon.wire()?, true)?
                .face()?
                .into_shape(),
        );
    }
    sewn(&faces, r.real())
}

fn piece(maker: &mut dyn BRepPrimAPIMakeOneAxis, shell: bool) -> Result<Shape, Failure> {
    if shell {
        return Ok(maker.shell()?.into_shape());
    }
    Ok(maker.face()?.into_shape())
}

fn lateral(kind: i32, axes: Axis2, angle: f64, p: &[f64], shell: bool) -> Result<Shape, Failure> {
    match kind {
        0 => piece(
            &mut BRepPrimAPIMakeCylinder::with_axes_angle(axes, p[0], p[1], angle)?,
            shell,
        ),
        1 => piece(
            &mut BRepPrimAPIMakeCone::with_axes_angle(axes, p[0], p[1], p[2], angle)?,
            shell,
        ),
        _ => piece(
            &mut BRepPrimAPIMakeSphere::with_axis_angle(axes, p[0], angle)?,
            shell,
        ),
    }
}

fn revolved(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let kind = r.integer();
    let angle = r.real();
    let count = r.integer();
    let shell = r.flag();
    let n = match kind {
        0 => 2,
        1 => 3,
        _ => 1,
    };
    let params: Vec<f64> = (0..n).map(|_| r.real()).collect();
    let mut faces = Vec::new();
    for k in 0..count {
        let a = f64::from(k) * angle;
        let x = Direction3::new(a.cos(), a.sin(), 0.)?;
        let axes =
            Axis2::with_x_direction(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?, x)?;
        faces.push(lateral(kind, axes, angle, &params, shell)?);
    }
    sewn(&faces, 1e-6)
}

fn edge(r: &mut Reader<'_>) -> Result<Edge, Failure> {
    match r.integer() {
        0 => {
            let a = r.pnt();
            BRepBuilderAPIMakeEdge::from_points(a, r.pnt())?.edge()
        }
        1 => {
            let c = r.pnt();
            let (radius, a1) = (r.real(), r.real());
            let circle = GeomCircle::new(Axis2::new(c, Direction3::new(0., 0., 1.)?)?, radius)?;
            let curve: Handle<dyn GeomCurve> = Handle::new(circle).cast();
            BRepBuilderAPIMakeEdge::from_curve_params(&curve, a1, r.real())?.edge()
        }
        _ => {
            let poles: Vec<Point3> = (0..r.integer()).map(|_| r.pnt()).collect();
            let poles = Array1::from_shared(&poles, 1);
            let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles)?).cast();
            BRepBuilderAPIMakeEdge::from_curve(&curve)?.edge()
        }
    }
}

fn wire(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let mut maker = BRepBuilderAPIMakeWire::new();
    for _ in 0..r.integer() {
        maker.add_edge(&edge(r)?)?;
    }
    Ok(maker.wire()?.into_shape())
}

fn input(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    match r.integer() {
        0 => polygons(r),
        1 => revolved(r),
        2 => wire(r),
        4 => {
            let w = rustycad_tkbrep::topods::wire(&wire(r)?)?;
            Ok(BRepBuilderAPIMakeFace::from_wire(&w, true)?
                .face()?
                .into_shape())
        }
        _ => {
            let (dx, dy, dz) = (r.real(), r.real(), r.real());
            let mut maker = BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)?;
            maker.build(&ProgressRange::default())?;
            Ok(maker.shape()?.clone())
        }
    }
}

fn all_shapes(s: &Shape) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes(s, &mut map, true, true);
    map
}

fn history(input: &Shape, result: &Shape, h: &Handle<History>) -> Result<(), Failure> {
    let h = h
        .get()
        .ok_or_else(|| Failure::with_type("Standard_NullObject", "probe: null history", ""))?;
    let (inputs, outputs) = (all_shapes(input), all_shapes(result));
    int(inputs.extent() as i32);
    for i in 1..=inputs.extent() {
        let s = inputs.find_key(i)?;
        if !History::is_supported_type(s)? {
            int(-3);
            continue;
        }
        flag(h.is_removed(s)?);
        let images = h.modified(s)?;
        int(images.len() as i32);
        for image in &images {
            let k = outputs.find_index(image);
            int(if k == 0 { -2 } else { k as i32 });
        }
    }
    Ok(())
}

fn keep_shapes(
    unifier: &ShapeUpgradeUnifySameDomain,
    s: &Shape,
    keep: &[(i32, i32)],
) -> Result<(), Failure> {
    let (mut edges, mut vertices) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes_of_type(s, ShapeEnum::Edge, &mut edges)?;
    topexp::map_shapes_of_type(s, ShapeEnum::Vertex, &mut vertices)?;
    for &(kind, index) in keep {
        let map = if kind == 0 { &edges } else { &vertices };
        if index >= 1 && index as usize <= map.extent() {
            unifier.keep_shape(map.find_key(index as usize)?)?;
        }
    }
    Ok(())
}

/// `sh_unify (input) init ue uf concat allow safe lintol angtol nkeep (kind index)...`.
pub fn unify_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut s = Shape::default();
    attempt(|| {
        s = input(&mut r)?;
        Ok(())
    });
    let (init, ue, uf, concat, allow, safe) =
        (r.flag(), r.flag(), r.flag(), r.flag(), r.flag(), r.flag());
    let (lin, ang) = (r.real(), r.real());
    let keep: Vec<(i32, i32)> = (0..r.integer())
        .map(|_| (r.integer(), r.integer()))
        .collect();
    if s.is_null() {
        end();
        return;
    }
    let unifier = if init {
        ShapeUpgradeUnifySameDomain::empty()
    } else {
        ShapeUpgradeUnifySameDomain::new(&s, ue, uf, concat)
    };
    if init {
        unifier
            .initialize(&s, ue, uf, concat)
            .expect("probe initialize");
    }
    unifier.allow_internal_edges(allow);
    unifier.set_safe_input_mode(safe);
    unifier.set_linear_tolerance(lin);
    unifier.set_angular_tolerance(ang);
    keep_shapes(&unifier, &s, &keep).expect("probe keep shapes");
    attempt(|| {
        unifier.build()?;
        shape(&unifier.shape())?;
        history(&s, &unifier.shape(), &unifier.history())
    });
    end();
}

/// `sh_face_connect n (m (point)*m)*n npairs (i j)* sewtol fixtol`.
pub fn face_connect_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let mut shell = rustycad_tkbrep::topods::Shell::default();
    builder.make_shell(&mut shell);
    let mut faces = Vec::new();
    for _ in 0..r.integer() {
        let mut polygon = BRepBuilderAPIMakePolygon::new();
        for _ in 0..r.integer() {
            polygon.add_point(r.pnt()).expect("probe polygon");
        }
        polygon.close().expect("probe polygon");
        let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().expect("probe wire"), true)
            .and_then(|m| m.face())
            .expect("probe face");
        builder.add(&mut shell, &face).expect("probe shell");
        faces.push(face);
    }
    let mut connector = rustycad_tkshhealing::shape_fix::ShapeFixFaceConnect::new();
    for _ in 0..r.integer() {
        let (i, j) = (r.integer() as usize, r.integer() as usize);
        flag(connector.add(&faces[i], &faces[j]));
    }
    let (sew, fix) = (r.real(), r.real());
    attempt(|| {
        let result = connector.build(&shell, sew, fix)?;
        shape(result.as_ref())?;
        shape(connector.build(&result, sew, fix)?.as_ref())
    });
    end();
}
