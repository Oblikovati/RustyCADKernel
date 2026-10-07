//! ShapeCustom modification probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_modification.cxx (which documents the request).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, text};
use super::registrator::{gravity, list, message};
use super::sh_common::{curve, curve2d, location, shapes, surface, trsf};
use super::shapes::TpShapes;
use super::wire_data::{sh_edge, sh_face, sh_shape};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, NewCurve, NewSurface};
use rustycad_tkbrep::topods;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_custom::{
    ShapeCustom, ShapeCustomBSplineRestriction, ShapeCustomBSplineRestrictionSettings,
    ShapeCustomContext, ShapeCustomConvertToBSpline, ShapeCustomConvertToRevolution,
    ShapeCustomDirectModification, ShapeCustomModification, ShapeCustomRestrictionParameters,
    ShapeCustomRestrictionRequest, ShapeCustomSweptToElementary, ShapeCustomTrsfModification,
};
use rustycad_tkshhealing::shape_extend::{
    BasicMsgRegistrator, ShapeExtend, ShapeExtendMsgRegistrator,
};

type Modification = Handle<dyn BRepToolsModification>;

/// `sc_modification`: the modification of the request.
fn modification(r: &mut Reader<'_>) -> Result<Modification, Failure> {
    Ok(match r.integer() {
        0 => {
            let m = ShapeCustomTrsfModification::new(trsf(r)?);
            m.trsf_modification().set_copy_mesh(r.flag());
            Handle::new(m).cast()
        }
        1 => Handle::new(ShapeCustomDirectModification::new()).cast(),
        3 => {
            let m = ShapeCustomConvertToBSpline::new();
            m.set_extrusion_mode(r.flag());
            m.set_revolution_mode(r.flag());
            m.set_offset_mode(r.flag());
            m.set_plane_mode(r.flag());
            Handle::new(m).cast()
        }
        4 => Handle::new(ShapeCustomSweptToElementary::new()).cast(),
        5 => bspline_restriction(r),
        _ => Handle::new(ShapeCustomConvertToRevolution::new()).cast(),
    })
}

fn new_surface(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let f = sh_face(t, r.integer())?;
    let mut result = NewSurface {
        tolerance: -1.0,
        rev_wires: true,
        ..NewSurface::default()
    };
    show(m.new_surface(&f, &mut result), flag);
    surface(&result.surface);
    location(&result.location);
    real(result.tolerance);
    flag(result.rev_wires);
    flag(result.rev_face);
    Ok(())
}

fn new_curve(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let e = sh_edge(t, r.integer())?;
    let mut result = NewCurve {
        tolerance: -1.0,
        ..NewCurve::default()
    };
    show(m.new_curve(&e, &mut result), flag);
    curve(&result.curve);
    location(&result.location);
    real(result.tolerance);
    Ok(())
}

fn new_point(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let v = topods::vertex(&sh_shape(t, r.integer()))?;
    let (mut p, mut tol) = (Point3::new(7.0, 7.0, 7.0), -1.0);
    show(m.new_point(&v, &mut p, &mut tol), flag);
    pnt(p);
    real(tol);
    Ok(())
}

fn new_curve2d(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let (e, f) = (sh_edge(t, r.integer())?, sh_face(t, r.integer())?);
    let (ne, nf) = (sh_edge(t, r.integer())?, sh_face(t, r.integer())?);
    let (mut c, mut tol) = (Handle::default(), -1.0);
    show(m.new_curve2d(&e, &f, &ne, &nf, &mut c, &mut tol), flag);
    curve2d(&c);
    real(tol);
    Ok(())
}

fn new_parameter(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let v = topods::vertex(&sh_shape(t, r.integer()))?;
    let e = sh_edge(t, r.integer())?;
    let (mut p, mut tol) = (0.5, -1.0);
    show(m.new_parameter(&v, &e, &mut p, &mut tol), flag);
    real(p);
    real(tol);
    Ok(())
}

fn continuity(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
) -> Result<(), Failure> {
    let e = sh_edge(t, r.integer())?;
    let (f1, f2) = (sh_face(t, r.integer())?, sh_face(t, r.integer())?);
    show(
        m.continuity(&e, &f1, &f2, &e, &f1, &f2)
            .map(|value| value as i32),
        int,
    );
    Ok(())
}

fn member(
    r: &mut Reader<'_>,
    t: &TpShapes,
    m: &dyn BRepToolsModification,
    op: i32,
) -> Result<(), Failure> {
    match op {
        0 => new_surface(r, t, m),
        1 => new_curve(r, t, m),
        2 => new_point(r, t, m),
        3 => new_curve2d(r, t, m),
        4 => new_parameter(r, t, m),
        _ => continuity(r, t, m),
    }
}

/// `sc_registrator`: the messages kept for each shape of the table.
fn registrator(t: &TpShapes, cm: &Handle<dyn ShapeCustomModification>) {
    let reg = cm.get().map_or_else(Handle::default, |cm| {
        cm.msg_registrator().cast::<ShapeExtendMsgRegistrator>()
    });
    flag(!reg.is_null());
    let Some(reg) = reg.get() else { return };
    reg.map_shape(|map| {
        int(map.len() as i32);
        for s in &t.shapes {
            let found = map.seek(s);
            flag(found.is_some());
            found.into_iter().for_each(list);
        }
    });
}

fn other(r: &mut Reader<'_>, t: &TpShapes, m: &Modification, op: i32) {
    let cm = m.cast::<dyn ShapeCustomModification>();
    match op {
        6 => {
            let s = sh_shape(t, r.integer());
            attempt(|| {
                let modifier = Modifier::with_modification(&s, m)?;
                flag(modifier.is_done());
                shape(&modifier.modified_shape(&s)?)
            });
        }
        7 => registrator(t, &cm),
        8 => {
            text(m.dynamic_type().map_or("null", |t| t.name()).as_bytes());
            flag(m.is_kind_name("ShapeCustom_Modification"));
            flag(m.is_kind_name("BRepTools_TrsfModification"));
        }
        9 => {
            let s = sh_shape(t, r.integer());
            let k = r.integer();
            let g = gravity(r.integer());
            if let Some(cm) = cm.get() {
                cm.send_msg(&s, &message(k), g);
            }
        }
        11 => restriction_errors(m),
        12 => {
            let k = r.integer();
            let s = sh_shape(t, r.integer());
            attempt(|| shape(&package_function(k, &s)?));
        }
        13 => {
            let s = sh_shape(t, r.integer());
            attempt(|| apply_with_reshape(&s, m));
        }
        _ => flag(cm.get().is_some_and(|cm| cm.msg_registrator().is_null())),
    }
}

/// Kind 5: a ShapeCustom_BSplineRestriction of the settings and restriction fields.
fn bspline_restriction(r: &mut Reader<'_>) -> Modification {
    let settings = ShapeCustomBSplineRestrictionSettings {
        approx_surface: r.flag(),
        approx_curve_3d: r.flag(),
        approx_curve_2d: r.flag(),
        tol_3d: r.real(),
        tol_2d: r.real(),
        continuity_3d: continuity_of(r.integer()),
        continuity_2d: continuity_of(r.integer()),
        max_degree: r.integer(),
        nb_max_seg: r.integer(),
        degree_priority: r.flag(),
        rational: r.flag(),
    };
    let m = ShapeCustomBSplineRestriction::with_settings(settings);
    let parameters = m.restriction_parameters();
    for _ in 0..r.integer() {
        let field = r.integer();
        super::shape_custom::restriction_set(&parameters, field, r.integer());
    }
    Handle::new(m).cast()
}

fn continuity_of(value: i32) -> Continuity {
    [
        Continuity::C0,
        Continuity::G1,
        Continuity::C1,
        Continuity::G2,
        Continuity::C2,
        Continuity::C3,
        Continuity::CN,
    ][value as usize]
}

/// Op 12: the ShapeCustom package function `k` on `s`.
fn package_function(k: i32, s: &Shape) -> Result<Shape, Failure> {
    match k {
        0 => ShapeCustom::direct_faces(s),
        1 => ShapeCustom::scale_shape(s, 2.5),
        2 => {
            let parameters = Handle::new(ShapeCustomRestrictionParameters::new());
            parameters.set_convert_plane(true);
            let request = ShapeCustomRestrictionRequest {
                tol_3d: 1e-3,
                tol_2d: 1e-6,
                max_degree: 3,
                max_nb_segment: 2,
                continuity_3d: Continuity::C1,
                continuity_2d: Continuity::C1,
                degree: true,
                rational: false,
                parameters,
            };
            ShapeCustom::bspline_restriction(s, &request)
        }
        3 => ShapeCustom::convert_to_revolution(s),
        4 => ShapeCustom::swept_to_elementary(s),
        _ => ShapeCustom::convert_to_bspline(s, k == 5, k == 5, k == 5, k == 5),
    }
}

/// Op 13: ApplyModifier with a new reshape, the context size and the reshape image.
fn apply_with_reshape(s: &Shape, m: &Modification) -> Result<(), Failure> {
    let mut context = ShapeCustomContext::default();
    let mut modifier = Modifier::new(false);
    let reshape = Handle::new(ShapeBuildReShape::new());
    let range = ProgressRange::default();
    shape(&ShapeCustom::apply_modifier(
        s,
        m,
        &mut context,
        &mut modifier,
        &range,
        &reshape,
    )?)?;
    int(context.extent() as i32);
    shape(&ReShape::apply(&*reshape, s, ShapeEnum::Shape)?)
}

/// Op 11: MaxErrors, NbOfSpan and the error accessors of a B-spline restriction.
fn restriction_errors(m: &Modification) {
    let b = m.cast::<ShapeCustomBSplineRestriction>();
    let Some(b) = b.get() else { return };
    let (surface, c3d, c2d) = b.max_errors();
    [surface, c3d, c2d].into_iter().for_each(real);
    int(b.nb_of_span());
    [b.surface_error(), b.curve_3d_error(), b.curve_2d_error()]
        .into_iter()
        .for_each(real);
}

/// `sc_modif (prelude) program kind [(trsf) copymesh] reg nops (op)...`.
pub fn modif_case(values: &[f64]) {
    ShapeExtend::init().expect("Shape Healing messages");
    let mut r = Reader::new(values);
    let t = shapes(&mut r).expect("probe shape program");
    let m = modification(&mut r).expect("probe modification record");
    let cm = m.cast::<dyn ShapeCustomModification>();
    if r.flag()
        && let Some(cm) = cm.get()
    {
        cm.set_msg_registrator(
            &Handle::new(ShapeExtendMsgRegistrator::new()).cast::<dyn BasicMsgRegistrator>(),
        );
    }
    for _ in 0..r.integer() {
        let op = r.integer();
        if op <= 5 {
            let stored = m.get().expect("probe modification");
            member(&mut r, &t, stored, op).expect("probe shape index");
        } else {
            other(&mut r, &t, &m, op);
        }
    }
    end();
}
