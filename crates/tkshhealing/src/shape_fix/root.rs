// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_Root.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_Root`: the context, precision, tolerance limits and message registrator
//! shared by the fixing tools (docs/shhealing-design.md section 4).
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{BasicMsgRegistrator, ShapeExtendBasicMsgRegistrator};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};

/// The fields of a `ShapeFix_Root` (the private ones and the protected `myShape`).
#[derive(Clone)]
pub struct FixRootFields {
    /// `myShape` (protected): the shape the messages without a shape go to.
    pub shape: Shape,
    /// `myContext`: the re-shaper that records the changes (null: none).
    pub context: Handle<ShapeBuildReShape>,
    /// `myMsgReg`: the message registrator.
    pub msg_reg: Handle<dyn BasicMsgRegistrator>,
    /// `myPrecision`.
    pub precision: f64,
    /// `myMinTol`.
    pub min_tol: f64,
    /// `myMaxTol`.
    pub max_tol: f64,
}

/// The `ShapeFix_Root` part of a fixing tool, in one shared state.
pub struct FixRootData {
    fields: SharedState<FixRootFields>,
}

impl Default for FixRootData {
    /// The fields of `ShapeFix_Root()`: no context, a basic message registrator, and the
    /// precision and both tolerance limits at `Precision::Confusion()`.
    fn default() -> Self {
        let registrator = Handle::new(ShapeExtendBasicMsgRegistrator::new());
        let confusion = precision::confusion();
        Self {
            fields: SharedState::new(FixRootFields {
                shape: Shape::default(),
                context: Handle::default(),
                msg_reg: registrator.cast::<dyn BasicMsgRegistrator>(),
                precision: confusion,
                min_tol: confusion,
                max_tol: confusion,
            }),
        }
    }
}

impl FixRootData {
    /// Lends the fields for reading; `data.with(|f| f.precision)`.
    pub fn with<R>(&self, read: impl FnOnce(&FixRootFields) -> R) -> R {
        read(&self.fields.read())
    }

    /// Lends the fields for writing; `data.with_mut(|f| f.shape = shape)`.
    pub fn with_mut<R>(&self, edit: impl FnOnce(&mut FixRootFields) -> R) -> R {
        edit(&mut self.fields.write())
    }
}

/// `ShapeFix_Root`: the virtual interface of the fixing tools. The native virtual
/// setters (`Set`, `SetContext`, `SetMsgRegistrator`, `SetPrecision`, `SetMinTolerance`,
/// `SetMaxTolerance`) are provided methods that a derived tool may override; the other
/// members are the native non-virtual ones.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixRoot};
/// let root = ShapeFixRoot::new();
/// root.set_precision(1e-3);
/// assert_eq!(root.max_tolerance(), 1e-3);
/// assert_eq!(root.limit_tolerance(1.0), 1e-3);
/// ```
pub trait FixRoot: Transient {
    /// The `ShapeFix_Root` fields of the tool.
    fn fix_root_data(&self) -> &FixRootData;

    /// `Set(Root)` (virtual): copies the context, registrator, precision, limits and shape
    /// of another tool; `tool.set(&*other)`.
    fn set(&self, root: &dyn FixRoot) {
        let copy = root.fix_root_data().with(FixRootFields::clone);
        self.fix_root_data().with_mut(|fields| *fields = copy);
    }

    /// `SetContext(context)` (virtual); `tool.set_context(&reshape)`.
    fn set_context(&self, context: &Handle<ShapeBuildReShape>) {
        self.fix_root_data()
            .with_mut(|fields| fields.context = context.clone());
    }

    /// `Context()`; `tool.context()`.
    fn context(&self) -> Handle<ShapeBuildReShape> {
        self.fix_root_data().with(|fields| fields.context.clone())
    }

    /// `SetMsgRegistrator(msgreg)` (virtual); `tool.set_msg_registrator(&registrator)`.
    fn set_msg_registrator(&self, msgreg: &Handle<dyn BasicMsgRegistrator>) {
        self.fix_root_data()
            .with_mut(|fields| fields.msg_reg = msgreg.clone());
    }

    /// `MsgRegistrator()`; `tool.msg_registrator()`.
    fn msg_registrator(&self) -> Handle<dyn BasicMsgRegistrator> {
        self.fix_root_data().with(|fields| fields.msg_reg.clone())
    }

    /// `SetPrecision(preci)` (virtual): the precision, raising the maximal tolerance to it
    /// and lowering the minimal one to it; `tool.set_precision(1e-3)`.
    fn set_precision(&self, preci: f64) {
        self.fix_root_data().with_mut(|fields| {
            fields.precision = preci;
            if fields.max_tol < preci {
                fields.max_tol = preci;
            }
            if fields.min_tol > preci {
                fields.min_tol = preci;
            }
        });
    }

    /// `Precision()`; `tool.precision()`.
    fn precision(&self) -> f64 {
        self.fix_root_data().with(|fields| fields.precision)
    }

    /// `SetMinTolerance(mintol)` (virtual); `tool.set_min_tolerance(1e-7)`.
    fn set_min_tolerance(&self, mintol: f64) {
        self.fix_root_data()
            .with_mut(|fields| fields.min_tol = mintol);
    }

    /// `MinTolerance()`; `tool.min_tolerance()`.
    fn min_tolerance(&self) -> f64 {
        self.fix_root_data().with(|fields| fields.min_tol)
    }

    /// `SetMaxTolerance(maxtol)` (virtual); `tool.set_max_tolerance(1.0)`.
    fn set_max_tolerance(&self, maxtol: f64) {
        self.fix_root_data()
            .with_mut(|fields| fields.max_tol = maxtol);
    }

    /// `MaxTolerance()`; `tool.max_tolerance()`.
    fn max_tolerance(&self) -> f64 {
        self.fix_root_data().with(|fields| fields.max_tol)
    }

    /// `LimitTolerance(toler)`: `toler` limited by the maximal tolerance ("only maximal
    /// restriction implemented"); `tool.limit_tolerance(0.5)`.
    fn limit_tolerance(&self, toler: f64) -> f64 {
        self.max_tolerance().min(toler)
    }

    /// `SendMsg(shape, message, gravity)`: gives the message to the registrator, if any;
    /// `tool.send_msg(&shape, &message, Gravity::Info)`.
    fn send_msg(&self, shape: &Shape, message: &Msg, gravity: Gravity) {
        let registrator = self.msg_registrator();
        if let Some(reg) = registrator.get() {
            reg.send_shape(shape, message, gravity);
        }
    }

    /// `SendMsg(message, gravity)`: [`FixRoot::send_msg`] on the tool's shape;
    /// `tool.send_msg_on_shape(&message, Gravity::Info)`.
    fn send_msg_on_shape(&self, message: &Msg, gravity: Gravity) {
        let shape = self.fix_root_data().with(|fields| fields.shape.clone());
        self.send_msg(&shape, message, gravity);
    }

    /// `SendWarning(shape, message)`: [`FixRoot::send_msg`] with `Message_Warning`;
    /// `tool.send_warning(&shape, &message)`.
    fn send_warning(&self, shape: &Shape, message: &Msg) {
        self.send_msg(shape, message, Gravity::Warning);
    }

    /// `SendWarning(message)`: [`FixRoot::send_warning`] on the tool's shape;
    /// `tool.send_warning_on_shape(&message)`.
    fn send_warning_on_shape(&self, message: &Msg) {
        self.send_msg_on_shape(message, Gravity::Warning);
    }

    /// `SendFail(shape, message)`: [`FixRoot::send_msg`] with `Message_Fail`;
    /// `tool.send_fail(&shape, &message)`.
    fn send_fail(&self, shape: &Shape, message: &Msg) {
        self.send_msg(shape, message, Gravity::Fail);
    }

    /// `SendFail(message)`: [`FixRoot::send_fail`] on the tool's shape;
    /// `tool.send_fail_on_shape(&message)`.
    fn send_fail_on_shape(&self, message: &Msg) {
        self.send_msg_on_shape(message, Gravity::Fail);
    }
}
impl_transient_interface!(dyn FixRoot, "ShapeFix_Root", StandardTransient);

/// The protected static `NeedFix(flag, def)`: a three-position mode (-1 default, 0 off,
/// above 0 on) decides whether a fix runs; the default takes `def`;
/// `need_fix(-1, true)`.
pub fn need_fix(flag: i32, def: bool) -> bool {
    if flag < 0 { def } else { flag > 0 }
}

/// The base class `ShapeFix_Root` itself (no fixing member of its own).
#[derive(Default)]
pub struct ShapeFixRoot {
    parent: StandardTransient,
    root: FixRootData,
}
impl_transient!(
    ShapeFixRoot,
    "ShapeFix_Root",
    StandardTransient,
    parent,
    interfaces: [dyn FixRoot]
);

impl ShapeFixRoot {
    /// `ShapeFix_Root()`: no context, a basic registrator, `Precision::Confusion()` for
    /// the precision and both limits; `ShapeFixRoot::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl FixRoot for ShapeFixRoot {
    fn fix_root_data(&self) -> &FixRootData {
        &self.root
    }
}
