use super::ShapeFixFixSmallFace;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::{FixRoot, ShapeFixFace};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

pub(super) fn fix_spot_face(fixer: &ShapeFixFixSmallFace) -> Result<Shape, Failure> {
    let input = fixer.input.read().clone();
    if !spot_phase_shape(&input)? {
        return Ok(input);
    }
    let done = check_faces(fixer, &input)?;
    let reshaped = apply_context(fixer, &input)?;
    *fixer.input.write() = reshaped.clone();
    if done && reshaped.is_null() {
        return Ok(reshaped);
    }
    let result = finish_spot_phase(fixer, &reshaped, done)?;
    *fixer.result.write() = result.clone();
    Ok(result)
}

pub(super) fn replace_vertices_in_case_of_spot(
    fixer: &ShapeFixFixSmallFace,
    face: &mut Face,
    _tolerance: f64,
) -> Result<bool, Failure> {
    let context = context(fixer)?;
    replace_spot_vertices(&context, face)?;
    Ok(true)
}

pub(super) fn remove_faces_in_case_of_spot(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
) -> Result<bool, Failure> {
    remove_spot_face(fixer, &context(fixer)?, face)?;
    Ok(true)
}

fn spot_phase_shape(shape: &Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Compound
            | ShapeEnum::CompSolid
            | ShapeEnum::Solid
            | ShapeEnum::Shell
            | ShapeEnum::Face
    ))
}

fn check_faces(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<bool, Failure> {
    let context = context(fixer)?;
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    let mut done = false;
    while faces.more() {
        done |= check_candidate(fixer, &context, faces.current())?;
        faces.next()?;
    }
    Ok(done)
}

fn check_candidate(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    source: &Shape,
) -> Result<bool, Failure> {
    let applied = ReShape::apply(context.get().unwrap(), source, ShapeEnum::Shape)?;
    if applied.is_null() {
        return Ok(false);
    }
    let mut face = topods::face(&applied)?;
    let is_spot = fixer
        .analyzer
        .write()
        .check_spot_face(&face, fixer.precision())?;
    if !is_spot {
        return Ok(false);
    }
    replace_spot_vertices(context, &mut face)?;
    remove_spot_face(fixer, context, &face)?;
    *fixer.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done1);
    Ok(true)
}

fn replace_spot_vertices(
    context: &Handle<ShapeBuildReShape>,
    face: &mut Face,
) -> Result<(), Failure> {
    let applied = ReShape::apply(
        context.get().unwrap(),
        &face.clone().into_shape(),
        ShapeEnum::Shape,
    )?;
    *face = topods::face(&applied)?;
    if !has_direct_wire(face)? {
        return Ok(());
    }
    let (points, max_tolerance) = face_vertex_samples(face)?;
    let (point, max_deviation) = average_point_and_deviation(&points);
    replace_face_vertices(context, face, point, max_deviation + max_tolerance / 2.0)
}

fn has_direct_wire(face: &Face) -> Result<bool, Failure> {
    let face_shape = face.clone().into_shape();
    let mut children = ShapeIterator::new(&face_shape, false, true);
    while children.more() {
        if children.value()?.shape_type()? == ShapeEnum::Wire {
            return Ok(true);
        }
        children.next();
    }
    Ok(false)
}

fn face_vertex_samples(face: &Face) -> Result<(Vec<Point3>, f64), Failure> {
    let mut points = Vec::new();
    let mut max_tolerance: f64 = 0.0;
    let mut vertices = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while vertices.more() {
        let vertex = topods::vertex(vertices.current())?;
        points.push(BRepTool::pnt(&vertex)?);
        let tolerance = BRepTool::vertex_tolerance(&vertex)?;
        if tolerance > max_tolerance {
            max_tolerance = tolerance;
        }
        vertices.next()?;
    }
    Ok((points, max_tolerance))
}

fn average_point_and_deviation(points: &[Point3]) -> (Point3, f64) {
    let mut sum = [0.0; 3];
    for point in points {
        for (total, coordinate) in sum.iter_mut().zip(point.coordinates()) {
            *total += coordinate;
        }
    }
    let divisor = if points.len() > 1 {
        points.len() as f64
    } else {
        1.0
    };
    let average = Point3::new(sum[0] / divisor, sum[1] / divisor, sum[2] / divisor);
    let mut max_deviation = 0.0;
    for point in points {
        let deviation = average.distance(point);
        if deviation > max_deviation {
            max_deviation = deviation;
        }
    }
    max_deviation *= 1.00001;
    (average, max_deviation)
}

fn replace_face_vertices(
    context: &Handle<ShapeBuildReShape>,
    face: &Face,
    point: Point3,
    tolerance: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut shared = rustycad_tkbrep::topods::Vertex::default();
    builder.make_vertex_at(&mut shared, point, tolerance)?;
    let mut vertices = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while vertices.more() {
        let old = topods::vertex(vertices.current())?;
        let replacement = shared.oriented(old.orientation());
        ReShape::replace(context.get().unwrap(), &old.into_shape(), &replacement)?;
        vertices.next()?;
    }
    Ok(())
}

fn remove_spot_face(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    face: &Face,
) -> Result<(), Failure> {
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        ReShape::remove(context.get().unwrap(), edges.current())?;
        edges.next()?;
    }
    ReShape::remove(context.get().unwrap(), &face.clone().into_shape())?;
    let message = Msg::from_key(&AsciiString::from("FixAdvFace.FixSpotFace.MSG0"));
    fixer.send_warning(&face.clone().into_shape(), &message);
    Ok(())
}

fn apply_context(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<Shape, Failure> {
    let context = context(fixer)?;
    ReShape::apply(context.get().unwrap(), shape, ShapeEnum::Shape)
}

fn finish_spot_phase(
    fixer: &ShapeFixFixSmallFace,
    shape: &Shape,
    done: bool,
) -> Result<Shape, Failure> {
    if !done || shape.is_null() {
        return Ok(shape.clone());
    }
    fix_shape_faces(fixer, shape)
}

pub(super) fn fix_shape(fixer: &ShapeFixFixSmallFace) -> Result<Shape, Failure> {
    let shape = fixer.input.read().clone();
    fix_shape_faces(fixer, &shape)
}

fn fix_shape_faces(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<Shape, Failure> {
    let context = context(fixer)?;
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        fix_one_face(&context, faces.current())?;
        faces.next()?;
    }
    ReShape::apply(context.get().unwrap(), shape, ShapeEnum::Shape)
}

pub(super) fn fix_face(fixer: &ShapeFixFixSmallFace, face: &Face) -> Result<Face, Failure> {
    let fixer_face = ShapeFixFace::default();
    fixer_face.set_context(&context(fixer)?);
    fixer_face.init(face.clone().into_shape());
    fixer_face.perform()?;
    fixer_face.face()
}

fn fix_one_face(context: &Handle<ShapeBuildReShape>, shape: &Shape) -> Result<(), Failure> {
    let applied = ReShape::apply(context.get().unwrap(), shape, ShapeEnum::Shape)?;
    let face = topods::face(&applied)?;
    let fixer = ShapeFixFace::default();
    fixer.set_context(context);
    fixer.init(face.into_shape());
    fixer.perform()?;
    ReShape::replace(context.get().unwrap(), &applied, &fixer.result())
}

fn context(fixer: &ShapeFixFixSmallFace) -> Result<Handle<ShapeBuildReShape>, Failure> {
    let context = fixer.context();
    if context.get().is_some() {
        return Ok(context);
    }
    Err(Failure::new(
        "ShapeFix_FixSmallFace requires Init(shape) before fixing a face",
    ))
}
