//! ShapeCustom modification probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_modification.cxx (which documents the request).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, text};
use super::registrator::{gravity, list, message};
use super::sh_common::{curve, curve2d, location, shapes, surface, trsf};
use super::shapes::TpShapes;
use super::wire_data::{sh_edge, sh_face, sh_shape};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, NewCurve, NewSurface};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_custom::{
    ShapeCustomConvertToRevolution, ShapeCustomDirectModification, ShapeCustomModification,
    ShapeCustomTrsfModification,
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
        _ => flag(cm.get().is_some_and(|cm| cm.msg_registrator().is_null())),
    }
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
