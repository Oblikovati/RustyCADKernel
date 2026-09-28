//! gp_Trsf2d and gp_GTrsf2d requests.
use super::{ax, dir, mat, pair, read_mat, show};
use crate::elementary_probe::{Fields, attempt, end, flag, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Direction2, Direction3, GeneralTransform2, Line2, Point2, Point3, Transform2,
    Transform3, Vector2, Xy,
};

struct TransformSpec {
    kind: i32,
    p: [f64; 6],
    scale: Option<f64>,
    translation: Option<Xy>,
}
fn read_spec(fields: &mut Fields) -> TransformSpec {
    let kind = fields.int();
    let p = std::array::from_fn(|_| fields.real());
    let (scale_kind, scale) = (fields.int(), fields.real());
    let (translation_kind, translation) = (fields.int(), fields.xy());
    TransformSpec {
        kind,
        p,
        scale: (scale_kind != 0).then_some(scale),
        translation: (translation_kind != 0).then_some(translation),
    }
}
fn build(spec: &TransformSpec) -> Result<Transform2, Failure> {
    let p = spec.p;
    let (a, b) = (Point2::new(p[0], p[1]), Point2::new(p[2], p[3]));
    let mut t = match spec.kind {
        1 => Transform2::translation_vector(Vector2::from(a.xy())),
        2 => Transform2::translation_between(a, b),
        3 => Transform2::rotation(a, p[2]),
        4 => Transform2::mirror_point(a),
        5 => Transform2::mirror_axis(Axis2d::new(a, Direction2::new(p[2], p[3])?)),
        6 => Transform2::scaling(a, p[2]),
        7 => Transform2::transformation_to(Axis2d::new(a, Direction2::new(p[2], p[3])?)),
        8 => between_axes(p)?,
        9 => values_transform(p)?,
        10 => Transform2::from(rotation3(p)?),
        11 => Transform2::from(Transform3::scaling(Point3::new(p[0], p[1], p[2]), p[3])?),
        _ => Transform2::default(),
    };
    if let Some(scale) = spec.scale {
        t.set_scale_factor(scale);
    }
    if let Some(offset) = spec.translation {
        t.set_translation_part(Vector2::from(offset));
    }
    Ok(t)
}
fn between_axes(p: [f64; 6]) -> Result<Transform2, Failure> {
    let from = Axis2d::new(Point2::new(p[0], p[1]), Direction2::new(p[2], p[3])?);
    let to = Axis2d::new(
        Point2::new(p[4], p[5]),
        Direction2::new(p[3] - p[5], p[2] + p[4])?,
    );
    Ok(Transform2::transformation_between(from, to))
}
fn values_transform(p: [f64; 6]) -> Result<Transform2, Failure> {
    let mut t = Transform2::default();
    t.set_values(p[0], p[1], p[2], p[3], p[4], p[5])?;
    Ok(t)
}
fn rotation3(p: [f64; 6]) -> Result<Transform3, Failure> {
    let axis = Axis1::new(
        Point3::new(p[0], p[1], 0.0),
        Direction3::new(p[2], p[3], p[4])?,
    );
    Transform3::rotation(axis, p[5])
}
/// Mirrors `GP2_TRY(t=gp2_trsf(r))`: the spec is read in full before anything is built, and a
/// failed build leaves an identity after printing the exception type.
fn read_transform(fields: &mut Fields) -> Transform2 {
    let spec = read_spec(fields);
    let mut t = Transform2::default();
    attempt(|| {
        t = build(&spec)?;
        Ok(())
    });
    t
}
fn trsf_state(t: Transform2) {
    print!("{} ", t.form() as i32);
    real(t.scale_factor());
    flag(t.is_negative());
    for row in 1..=2 {
        for col in 1..=3 {
            real(t.value(row, col).unwrap());
        }
    }
    mat(t.vectorial_part());
    mat(t.matrix());
    real(t.rotation_part());
    pair(t.translation_part());
}
fn trsf_objects(t: Transform2, p: Point2, d: Xy) {
    pair(p.transformed(t).xy());
    pair(t.apply_coordinates(p.xy()));
    let (x, y) = t.transforms_coordinates(p.x(), p.y());
    real(x);
    real(y);
    pair(Vector2::from(d).transformed(t).xy());
    attempt(|| {
        let direction = Direction2::from_xy(d)?;
        dir(direction.transformed(t));
        ax(Axis2d::new(p, direction).transformed(t));
        ax(Line2::new(p, direction).transformed(t).position());
        Ok(())
    });
}
pub fn trsf_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (a, b) = (read_transform(&mut fields), read_transform(&mut fields));
    let (p, d, n) = (fields.point2(), fields.xy(), fields.int());
    [a, b, a.multiplied(b), a * b]
        .into_iter()
        .for_each(trsf_state);
    let mut c = a;
    c.pre_multiply(b);
    trsf_state(c);
    c = a;
    c *= b;
    trsf_state(c);
    show(a.inverted(), trsf_state);
    show(a.powered(n), trsf_state);
    c = a;
    attempt(|| c.invert());
    trsf_state(c);
    c = a;
    attempt(|| c.power(n));
    trsf_state(c);
    trsf_objects(a, p, d);
    end();
}

fn read_general(fields: &mut Fields) -> GeneralTransform2 {
    let kind = fields.int();
    let t = read_transform(fields);
    let (m, v) = (read_mat(fields), fields.xy());
    let (axis_point, axis_direction, ratio) = (fields.point2(), fields.xy(), fields.real());
    let (row, col, value) = (fields.int(), fields.int(), fields.real());
    let mut g = GeneralTransform2::from(t);
    match kind {
        1 => g = GeneralTransform2::new(m, v),
        2 => attempt(|| {
            g.set_affinity(
                Axis2d::new(axis_point, Direction2::from_xy(axis_direction)?),
                ratio,
            );
            Ok(())
        }),
        3 => attempt(|| g.set_value(row, col, value)),
        4 => g.set_translation_part(v),
        5 => {
            g = GeneralTransform2::new(m, v);
            g.set_trsf2d(t);
        }
        6 => g.set_vectorial_part(m),
        _ => {}
    }
    g
}
fn gtrsf_state(g: GeneralTransform2) {
    print!("{} ", g.form() as i32);
    flag(g.is_negative());
    flag(g.is_singular());
    for row in 1..=2 {
        for col in 1..=3 {
            real(g.value(row, col).unwrap());
        }
    }
    mat(g.vectorial_part());
    pair(g.translation_part());
    show(g.trsf2d(), trsf_state);
}
pub fn gtrsf_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (a, b) = (read_general(&mut fields), read_general(&mut fields));
    let (p, n) = (fields.xy(), fields.int());
    gtrsf_state(a);
    gtrsf_state(b);
    show(a.multiplied(b), gtrsf_state);
    show(a * b, gtrsf_state);
    let mut c = a;
    attempt(|| c.pre_multiply(b));
    gtrsf_state(c);
    c = a;
    attempt(|| c.multiply(b));
    gtrsf_state(c);
    show(a.inverted(), gtrsf_state);
    show(a.powered(n), gtrsf_state);
    pair(a.transformed(p));
    let mut q = p;
    a.transforms(&mut q);
    pair(q);
    let (x, y) = a.transforms_coordinates(p.x(), p.y());
    real(x);
    real(y);
    end();
}
