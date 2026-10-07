//! ShapeProcess and ShapeProcessAPI probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shape_process.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, real, text};
use super::sh_common::shapes;
use super::wire_data::sh_shape;
use rustycad_tkernel::message::{Messenger, ProgressRange};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkshhealing::shape_process::{
    ShapeProcess, ShapeProcessContext, ShapeProcessOperLibrary, ShapeProcessOperation,
    ShapeProcessOperationsFlags, ShapeProcessShapeContext,
};
use rustycad_tkshhealing::shape_process_api::ShapeProcessAPIApplySequence;
use std::sync::Arc;

const SP_PARAMS: [(&str, &str); 33] = [
    ("seq.exec.op", "FixShape"),
    ("seq.exec.op", "DirectFaces,SameParameter;SetTolerance"),
    ("seq.exec.op", "SplitAngle ToBezier"),
    ("seq.exec.op", "Unknown,FixShape"),
    ("seq.exec.op", "BSplineRestriction"),
    (
        "seq.exec.op",
        "SplitContinuity SplitClosedFaces SplitClosedEdges",
    ),
    (
        "seq.exec.op",
        "FixWireGaps FixFaceSize DropSmallSolids DropSmallEdges SplitCommonVertex",
    ),
    (
        "seq.exec.op",
        "ElementaryToRevolution SweptToElementary SurfaceToBSpline",
    ),
    ("seq.FixShape.Tolerance3d", "1.e-4"),
    ("seq.FixShape.MaxTolerance3d", "1."),
    ("seq.SetTolerance.Mode", "1"),
    ("seq.SetTolerance.Value", "& seq.Base"),
    ("seq.Base", "0.001"),
    ("seq.SplitAngle.Angle", "1.5"),
    ("seq.BSplineRestriction.RequiredDegree", "3"),
    ("seq.BSplineRestriction.PlaneMode", "1"),
    ("seq.BSplineRestriction.Continuity3d", "c2"),
    ("seq.SetTolerance.Regularity", "0.01"),
    ("seq.ToBezier.SurfaceMode", "1"),
    ("seq.ToBezier.Curve3dMode", "1"),
    ("seq.FixShape.FixTailMode", "1"),
    ("seq.SetTolerance.Ratio", "2"),
    ("Blank", "  "),
    ("Word", " c1 "),
    ("Int", "12abc"),
    ("seq.SurfaceToBSpline.OffsetMode", "0"),
    ("FixShape.Tolerance3d", "1e-3"),
    ("SplitAngle.Angle", "1.0"),
    ("seq.Alias", "&Missing"),
    ("seq.Real", "2.5e-1"),
    ("ToBezier.SurfaceMode", "1"),
    ("ToBezier.Curve3dMode", "1"),
    ("ToBezier.Curve2dMode", "1"),
];
const SP_SCOPES: [&str; 3] = ["seq", "FixShape", "SetTolerance"];
const SP_NAMES: [&str; 10] = [
    "exec.op",
    "FixShape.Tolerance3d",
    "Base",
    "Blank",
    "Word",
    "Int",
    "SetTolerance.Value",
    "Missing",
    "Alias",
    "Real",
];

fn set_param(context: &ShapeProcessContext, k: i32) {
    let (name, value) = SP_PARAMS[k as usize];
    if let Some(resource) = context.resource_manager() {
        resource.write().set_resource(name, value.as_bytes());
    }
}

fn query(context: &ShapeProcessShapeContext, op: i32, name: &str) {
    match op {
        3 => flag(context.is_param_set(name)),
        4 => attempt(|| {
            let value = context.get_real(name)?;
            flag(value.is_some());
            value.into_iter().for_each(real);
            Ok(())
        }),
        5 => attempt(|| {
            let value = context.get_integer(name)?;
            flag(value.is_some());
            value.into_iter().for_each(int);
            Ok(())
        }),
        6 => {
            let value = context.get_boolean(name);
            flag(value.is_some());
            value.into_iter().for_each(flag);
        }
        7 => {
            let value = context.get_string(name);
            flag(value.is_some());
            value.into_iter().for_each(|v| text(v.as_bytes()));
        }
        8 => text(context.string_val(name, "def").as_bytes()),
        _ => {
            let value = context.get_continuity(name);
            flag(value.is_some());
            value.into_iter().for_each(|v| int(v as i32));
        }
    }
}

/// `sp_context nops (op)...`.
pub fn context_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let context = ShapeProcessShapeContext::new("", "").expect("empty resource context");
    for _ in 0..r.integer() {
        match r.integer() {
            0 => set_param(&context, r.integer()),
            1 => context.set_scope(SP_SCOPES[r.integer() as usize]),
            2 => context.un_set_scope(),
            op => query(&context, op, SP_NAMES[r.integer() as usize]),
        }
    }
    end();
}

fn digest(context: &ShapeProcessShapeContext) -> Result<(), Failure> {
    shape(&context.result())?;
    int(context.map().extent() as i32);
    let messages = context.messages();
    int(messages
        .get()
        .map_or(-1, |m| m.map_shape(|map| map.extent() as i32)));
    Ok(())
}

fn silent() -> Arc<Messenger> {
    let messenger = Messenger::new();
    messenger.printers_mut().clear();
    Arc::new(messenger)
}

fn perform(context: &Handle<ShapeProcessShapeContext>, mode: i32) -> Result<(), Failure> {
    let base: Handle<ShapeProcessContext> = context.clone().cast();
    let range = ProgressRange::default();
    if mode >= 0 {
        flag(ShapeProcess::perform_sequence(&base, "seq", &range)?);
        return Ok(());
    }
    let mut flags = ShapeProcessOperationsFlags::default();
    for operation in ShapeProcessOperation::ALL {
        if ((-mode) >> operation as i32) & 1 != 0 {
            flags.set(operation);
        }
    }
    flag(ShapeProcess::perform_operations(&base, flags, &range)?);
    Ok(())
}

/// `sp_process (prelude) program s mode trace nparams (k)...`.
pub fn process_case(values: &[f64]) {
    ShapeProcessOperLibrary::init().expect("operator library");
    let mut r = Reader::new(values);
    let t = shapes(&mut r).expect("probe shape program");
    let s = sh_shape(&t, r.integer());
    let (mode, trace) = (r.integer(), r.integer());
    let context = Handle::new(ShapeProcessShapeContext::with_shape(&s, "", "").expect("context"));
    context.set_messenger(Some(silent()));
    context.set_trace_level(trace);
    for _ in 0..r.integer() {
        set_param(&context, r.integer());
    }
    attempt(|| perform(&context, mode));
    digest(&context).expect("context digest");
    end();
}

/// `sp_apply (prelude) program s nparams (k)...`.
pub fn apply_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = shapes(&mut r).expect("probe shape program");
    let s = sh_shape(&t, r.integer());
    let mut api = ShapeProcessAPIApplySequence::new("", "seq").expect("apply sequence");
    let context = api.context();
    context.set_messenger(Some(silent()));
    for _ in 0..r.integer() {
        set_param(&context, r.integer());
    }
    attempt(|| {
        let range = ProgressRange::default();
        shape(&api.prepare_shape(&s, false, rustycad_tkg3d::top_abs::ShapeEnum::Shape, &range)?)
    });
    digest(&context).expect("context digest");
    end();
}
