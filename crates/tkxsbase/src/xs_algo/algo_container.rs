// Rust port of DataExchange/TKXSBase/XSAlgo/XSAlgo_AlgoContainer (.hxx, .cxx) and XSAlgo
// (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSAlgo_AlgoContainer`: the shape processing services of the exchange formats, by
//! resource file and sequence; `XSAlgo`: the process-wide container and the statics.
use super::XSAlgoShapeProcessor;
use crate::interface::InterfaceStatic;
use crate::transfer::{TransferFinderProcess, TransferTransientProcess};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::ProgressRange,
    standard::{Failure, Handle, StandardTransient, Transient},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::{
    shape_algo::ShapeAlgo,
    shape_extend::ShapeExtendMsgRegistrator,
    shape_fix::FixRoot,
    shape_process::{
        ShapeProcess, ShapeProcessContext, ShapeProcessOperLibrary, ShapeProcessShapeContext,
    },
};
use std::sync::{OnceLock, RwLock};

/// The arguments of `ProcessShape` (the native has nine parameters).
pub struct XSAlgoProcessShapeArgs<'a> {
    /// `thePrec`.
    pub prec: f64,
    /// `theMaxTol`.
    pub max_tol: f64,
    /// `thePrscfile`: the static naming the resource file.
    pub rscfile: &'a str,
    /// `thePseq`: the static naming the sequence (or the sequence itself).
    pub pseq: &'a str,
    /// `theNonManifold` (false by default).
    pub non_manifold: bool,
    /// `theDetailingLevel` (`TopAbs_VERTEX` by default).
    pub detailing_level: ShapeEnum,
}

/// The virtual members of `XSAlgo_AlgoContainer`, with the native bodies.
pub trait XSAlgoAlgoContainerOps: Transient {
    /// `PrepareForTransfer`: `XSAlgo_ShapeProcessor::PrepareForTransfer`.
    fn prepare_for_transfer(&self) {
        XSAlgoShapeProcessor::prepare_for_transfer();
    }
    /// `ProcessShape(...)`: the shape processed by the sequence of the resource file (a
    /// default ShapeFix for a "read." sequence the file does not define); `info` keeps
    /// the shape context.
    fn process_shape(
        &self,
        shape: &Shape,
        args: &XSAlgoProcessShapeArgs<'_>,
        info: &mut Handle<StandardTransient>,
        progress: &ProgressRange<'_>,
    ) -> Result<Shape, Failure> {
        process_shape(shape, args, info, progress)
    }
    /// `CheckPCurve(theEdge, theFace, thePrecision, theIsSeam)`.
    fn check_pcurve(
        &self,
        edge: &Edge,
        face: &Face,
        precision: f64,
        is_seam: bool,
    ) -> Result<bool, Failure> {
        XSAlgoShapeProcessor::check_pcurve(edge, face, precision, is_seam)
    }
    /// `MergeTransferInfo(TP, info, startTPitem = 1)`: the modifications of the context
    /// `info` into the shape binders of `tp`.
    fn merge_transfer_info(
        &self,
        tp: &Handle<TransferTransientProcess>,
        info: &Handle<StandardTransient>,
        start_tp_item: i32,
    ) -> Result<(), Failure> {
        let context = info.cast::<ShapeProcessShapeContext>();
        let Some(live) = context.get() else {
            return Ok(());
        };
        let (map, msg) = (live.map(), live.messages());
        if map.is_empty() && msg.get().is_none_or(|m| m.map_shape(|s| s.is_empty())) {
            return Ok(());
        }
        merge_into_transient(tp, &map, start_tp_item, &msg)
    }
    /// `MergeTransferInfo(FP, info)`: the modifications of the context `info` into `fp`.
    fn merge_transfer_info_finder(
        &self,
        fp: &Handle<TransferFinderProcess>,
        info: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let context = info.cast::<ShapeProcessShapeContext>();
        let Some(live) = context.get() else {
            return Ok(());
        };
        XSAlgoShapeProcessor::merge_shape_transfer_info_finder(fp, &live.map(), &live.messages())
    }
}
impl_transient_interface!(
    dyn XSAlgoAlgoContainerOps,
    "XSAlgo_AlgoContainer",
    StandardTransient
);

/// The container `MergeTransferInfo(TP, ...)`: as the processor's, from item `start`.
fn merge_into_transient(
    tp: &Handle<TransferTransientProcess>,
    map: &rustycad_tkshhealing::shape_process::ShapeProcessShapeMap,
    start: i32,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<(), Failure> {
    if map.is_empty() {
        // native: the loop still runs for the messages; with no modification only the
        // messages of the shape binders are added
        return super::merge::add_binder_messages(tp, start, messages);
    }
    XSAlgoShapeProcessor::merge_shape_transfer_info(tp, map, start, messages)
}

/// `XSAlgo_AlgoContainer`.
///
/// ```
/// use rustycad_tkxsbase::xs_algo::{XSAlgoAlgoContainer, XSAlgoAlgoContainerOps};
/// use rustycad_tkbrep::topods::{Edge, Face};
/// let container = XSAlgoAlgoContainer::new();
/// assert!(container.check_pcurve(&Edge::default(), &Face::default(), 1e-7, false).is_err());
/// ```
#[derive(Default)]
pub struct XSAlgoAlgoContainer {
    transient: StandardTransient,
}
impl_transient!(XSAlgoAlgoContainer, "XSAlgo_AlgoContainer", StandardTransient, transient, interfaces: [dyn XSAlgoAlgoContainerOps]);

impl XSAlgoAlgoContainer {
    /// `XSAlgo_AlgoContainer()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl XSAlgoAlgoContainerOps for XSAlgoAlgoContainer {}

/// The context of `ProcessShape`: `info` when it is a shape context, else a new one of
/// the resource file (filled from the statics when the file is missing).
fn shape_context(
    shape: &Shape,
    args: &XSAlgoProcessShapeArgs<'_>,
    info: &Handle<StandardTransient>,
) -> Result<Handle<ShapeProcessShapeContext>, Failure> {
    let given = info.cast::<ShapeProcessShapeContext>();
    if given.is_some() {
        return Ok(given);
    }
    let rscfile = InterfaceStatic::c_val(args.rscfile);
    let context = ShapeProcessShapeContext::with_shape(
        shape,
        &String::from_utf8_lossy(rscfile.as_bytes()),
        "",
    )?;
    if let Some(resource) = context.resource_manager() {
        let mut manager = resource.write();
        if !manager.is_initialized() {
            InterfaceStatic::fill_map(manager.get_map(true));
        }
    }
    context.set_detalisation(args.detailing_level);
    Ok(Handle::new(context))
}

/// `XSAlgo_AlgoContainer::ProcessShape`.
fn process_shape(
    shape: &Shape,
    args: &XSAlgoProcessShapeArgs<'_>,
    info: &mut Handle<StandardTransient>,
    progress: &ProgressRange<'_>,
) -> Result<Shape, Failure> {
    if shape.is_null() {
        return Ok(shape.clone());
    }
    let context = shape_context(shape, args, info)?;
    let live = context.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_AlgoContainer::ProcessShape: no context",
        )
    })?;
    live.set_non_manifold(args.non_manifold);
    *info = context.cast();
    let seq = String::from_utf8_lossy(InterfaceStatic::c_val(args.pseq).as_bytes()).into_owned();
    let resource = live.resource_manager().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_AlgoContainer::ProcessShape: no resource manager",
        )
    })?;
    let exec_op = format!("{seq}.exec.op");
    if !resource.read().find(&exec_op) {
        if args.pseq.starts_with("read.") {
            default_fix(shape, args, live, progress);
            return Ok(live.result());
        }
        if args.pseq.starts_with("write.") {
            resource.write().set_resource(&exec_op, b"DirectFaces");
        }
    }
    resource
        .write()
        .set_resource_real("Runtime.Tolerance", args.prec);
    resource
        .write()
        .set_resource_real("Runtime.MaxTolerance", args.max_tol);
    if !ShapeProcess::perform_sequence(&context.cast::<ShapeProcessContext>(), &seq, progress)? {
        return Ok(shape.clone());
    }
    Ok(live.result())
}

/// The default ShapeFix of a "read." sequence the resource file does not define; a
/// failure is caught and ignored, as natively.
fn default_fix(
    shape: &Shape,
    args: &XSAlgoProcessShapeArgs<'_>,
    context: &ShapeProcessShapeContext,
    progress: &ProgressRange<'_>,
) {
    let run = || -> Result<(), Failure> {
        let msg = Handle::new(ShapeExtendMsgRegistrator::new());
        let sfs = ShapeAlgo::algo_container()
            .get()
            .map(|c| c.tool_container())
            .and_then(|t| t.get().map(|t| t.fix_shape()))
            .unwrap_or_default();
        let fixer = sfs.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSAlgo_AlgoContainer: no ShapeFix_Shape",
            )
        })?;
        fixer.init(shape.clone());
        fixer.set_msg_registrator(&msg.cast());
        fixer.set_precision(args.prec);
        fixer.set_max_tolerance(args.max_tol);
        fixer
            .fix_face_tool()
            .fix_wire_tool()
            .set_fix_same_parameter_mode(0);
        fixer.fix_solid_tool().set_create_open_solid_mode(false);
        fixer.perform_with_progress(progress)?;
        let result = fixer.shape();
        if !result.is_null() && result != *shape {
            context.record_modification_reshape(&fixer.context(), &msg)?;
            context.set_result(&result);
        }
        Ok(())
    };
    let _ = run();
}

/// The process-wide container of `XSAlgo` (null before `Init`).
fn container_slot() -> &'static RwLock<Handle<dyn XSAlgoAlgoContainerOps>> {
    static CONTAINER: OnceLock<RwLock<Handle<dyn XSAlgoAlgoContainerOps>>> = OnceLock::new();
    CONTAINER.get_or_init(|| RwLock::new(Handle::default()))
}

/// `XSAlgo`: the process-wide algorithm container and the exchange statics.
pub struct XSAlgo;

impl XSAlgo {
    /// `Init`: once, the ShapeAlgo container, a new algorithm container, the standard
    /// statics, "read.stdsameparameter.mode" (Off) and "xstep.cascade.unit" (MM), and the
    /// shape processing operators.
    pub fn init() -> Result<(), Failure> {
        static INIT: OnceLock<()> = OnceLock::new();
        if INIT.set(()).is_err() {
            return Ok(());
        }
        ShapeAlgo::init()?;
        Self::set_algo_container(&Handle::new(XSAlgoAlgoContainer::new()).cast());
        InterfaceStatic::standards()?;
        init_statics()?;
        ShapeProcessOperLibrary::init()
    }
    /// `SetAlgoContainer(aContainer)`.
    pub fn set_algo_container(container: &Handle<dyn XSAlgoAlgoContainerOps>) {
        *container_slot()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = container.clone();
    }
    /// `AlgoContainer`.
    pub fn algo_container() -> Handle<dyn XSAlgoAlgoContainerOps> {
        container_slot()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// The statics `XSAlgo::Init` defines.
fn init_statics() -> Result<(), Failure> {
    let mode = "read.stdsameparameter.mode";
    InterfaceStatic::init_char("XSTEP", mode, b'e', "")?;
    for spec in ["ematch 0", "eval Off", "eval On"] {
        InterfaceStatic::init_char("XSTEP", mode, b'&', spec)?;
    }
    InterfaceStatic::set_i_val(mode, 0)?;
    let unit = "xstep.cascade.unit";
    InterfaceStatic::init_char("XSTEP", unit, b'e', "")?;
    let specs = [
        "enum 1",
        "eval INCH",
        "eval MM",
        "eval ??",
        "eval FT",
        "eval MI",
        "eval M",
        "eval KM",
        "eval MIL",
        "eval UM",
        "eval CM",
        "eval UIN",
    ];
    for spec in specs {
        InterfaceStatic::init_char("XSTEP", unit, b'&', spec)?;
    }
    InterfaceStatic::set_c_val(unit, "MM")?;
    Ok(())
}
