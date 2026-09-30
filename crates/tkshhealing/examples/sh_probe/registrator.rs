//! ShapeExtend_BasicMsgRegistrator and ShapeExtend_MsgRegistrator probe operations, the
//! Rust twin of tools/oracle/tkshhealing_probe_registrator.cxx (which documents the request).
use super::common::{Reader, end, flag, int, text};
use super::shapes::{TpShapes, tp_shapes};
use super::wire_data::sh_shape;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkshhealing::shape_extend::{
    BasicMsgRegistrator, ShapeExtendBasicMsgRegistrator, ShapeExtendMsgRegistrator, TransientKey,
};

/// `sh_rg_message`: the message whose text is `m<k>`.
fn message(k: i32) -> Msg {
    let mut msg = Msg::new();
    msg.set_text(format!("m{k}").as_bytes());
    msg
}

fn gravity(value: i32) -> Gravity {
    [
        Gravity::Trace,
        Gravity::Info,
        Gravity::Warning,
        Gravity::Alarm,
        Gravity::Fail,
    ][value as usize]
}

/// `sh_rg_list`: the count and the texts of a message list.
fn list(messages: &List<Msg>) {
    int(messages.extent());
    messages
        .iter()
        .for_each(|m| text(m.value().to_utf8().as_bytes()));
}

fn maps(t: &TpShapes, reg: &ShapeExtendMsgRegistrator, objects: &[Handle<StandardTransient>]) {
    reg.map_transient(|map| {
        int(map.len() as i32);
        for object in objects {
            let found = map.seek(&TransientKey(object.clone()));
            flag(found.is_some());
            found.into_iter().for_each(list);
        }
    });
    reg.map_shape(|map| {
        int(map.len() as i32);
        for s in &t.shapes {
            let found = map.seek(s);
            flag(found.is_some());
            found.into_iter().for_each(list);
        }
    });
}

fn send(
    r: &mut Reader<'_>,
    t: &TpShapes,
    reg: &dyn BasicMsgRegistrator,
    objects: &[Handle<StandardTransient>],
) {
    match r.integer() {
        0 => {
            let o = r.integer();
            let object =
                usize::try_from(o).map_or_else(|_| Handle::default(), |o| objects[o].clone());
            let (m, g) = (r.integer(), r.integer());
            reg.send_transient(&object, &message(m), gravity(g));
        }
        1 => {
            let s = sh_shape(t, r.integer());
            let (m, g) = (r.integer(), r.integer());
            reg.send_shape(&s, &message(m), gravity(g));
        }
        _ => {
            let m = r.integer();
            reg.send(&message(m), gravity(r.integer()));
        }
    }
}

/// `sh_registrator program kind nobj nops (op)...`.
pub fn registrator_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let reg: Handle<dyn BasicMsgRegistrator> = if r.integer() == 0 {
        Handle::new(ShapeExtendBasicMsgRegistrator::new()).cast()
    } else {
        Handle::new(ShapeExtendMsgRegistrator::new()).cast()
    };
    let objects: Vec<Handle<StandardTransient>> = (0..r.integer())
        .map(|_| Handle::new(StandardTransient::default()))
        .collect();
    for _ in 0..r.integer() {
        send(&mut r, &t, &*reg, &objects);
    }
    text(reg.dynamic_type().expect("registrator").name().as_bytes());
    flag(reg.is_kind_name("ShapeExtend_BasicMsgRegistrator"));
    flag(reg.is_kind_name("ShapeExtend_MsgRegistrator"));
    let keeper = reg.cast::<ShapeExtendMsgRegistrator>();
    if let Some(keeper) = keeper.get() {
        maps(&t, keeper, &objects);
    }
    end();
}
