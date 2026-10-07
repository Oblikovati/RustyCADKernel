//! `Draw_Drawable3D` and `Draw_Drawable2D`: the objects DRAW variables hold.
//!
//! A drawable is shared through [`DrawableHandle`] (`occ::handle<Draw_Drawable3D>`), so
//! its common state (screen bounds, name, visibility, protection) lives in a
//! [`Drawable3DBase`] with interior mutability. Each drawable has a process-unique id that
//! plays the role of the C++ object address (the client data of DRAW's variable traces).
use super::DrawInterpretor;
use super::display::DrawDisplay;
use super::ostream::DrawOStream;
use rustycad_tkernel::standard::Failure;
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Implements [`Drawable3D::base`] (on a field named `base`) and
/// [`Drawable3D::type_name`] inside an `impl Drawable3D` block.
macro_rules! drawable_identity {
    ($name:literal) => {
        fn base(&self) -> &$crate::draw::Drawable3DBase {
            &self.base
        }

        fn type_name(&self) -> &'static str {
            $name
        }
    };
}
pub(crate) use drawable_identity;

/// A shared drawable (`occ::handle<Draw_Drawable3D>`).
pub type DrawableHandle = Rc<dyn Drawable3D>;

/// The common state of every drawable (the private members of `Draw_Drawable3D`).
#[derive(Debug)]
pub struct Drawable3DBase {
    id: u64,
    bounds: Cell<[f64; 4]>,
    name: RefCell<Option<String>>,
    visible: Cell<bool>,
    protected: Cell<bool>,
}

impl Default for Drawable3DBase {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawable3DBase {
    /// The protected constructor: empty bounds at 0, no name, not visible, not protected.
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            bounds: Cell::new([0.0; 4]),
            name: RefCell::new(None),
            visible: Cell::new(false),
            protected: Cell::new(false),
        }
    }

    /// The identity of the object (stands for its C++ address).
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// `Draw_Drawable3D`: a displayable, nameable value of a DRAW variable.
///
/// Implementors provide [`Drawable3D::base`], [`Drawable3D::type_name`] and
/// [`Drawable3D::draw_on`]; the other methods have the C++ defaults.
pub trait Drawable3D: Any {
    /// The embedded common state.
    fn base(&self) -> &Drawable3DBase;

    /// `DynamicType()->Name()`: the C++ class name (`Draw_Number`, ...).
    fn type_name(&self) -> &'static str;

    /// `DrawOn`: draws the object on `display`. The C++ may throw while it evaluates
    /// geometry; the failure aborts the drawing and reaches the command.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure>;

    /// `PickReject`: true when the pick point `(x, y)` with tolerance `prec` is outside
    /// the screen bounds.
    fn pick_reject(&self, x: f64, y: f64, prec: f64) -> bool {
        let [x_min, x_max, y_min, y_max] = self.base().bounds.get();
        (x + prec < x_min) || (x - prec > x_max) || (y + prec < y_min) || (y - prec > y_max)
    }

    /// `Copy`: the copy for `copy`/`renamevar`; by default the object itself (`this`).
    /// Copying geometry may fail, as the C++ `Copy` may throw.
    fn copy(&self, this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        Ok(Rc::clone(this))
    }

    /// `Dump`: the textual dump; by default the screen bounds on two lines. Printing
    /// geometry may fail, as the C++ `Dump` may throw.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let [x_min, x_max, y_min, y_max] = self.base().bounds.get();
        stream
            .put_real(x_min)
            .put_str(" ")
            .put_real(x_max)
            .put_str("\n");
        stream
            .put_real(y_min)
            .put_str(" ")
            .put_real(y_max)
            .put_str("\n");
        Ok(())
    }

    /// `Save`: writes the object for `save`; by default `Standard_NotImplemented`.
    fn save(&self, _stream: &mut DrawOStream) -> Result<(), Failure> {
        let message = "Draw_Drawable3D::Save() should be redefined in sub-class";
        Err(Failure::with_message("Standard_NotImplemented", message))
    }

    /// `Whatis`: appends the kind of the object to the interpreter result.
    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("drawable 3d")?;
        Ok(())
    }

    /// `Is3D`: true for 3D objects (`Draw_Drawable2D` answers false).
    fn is_3d(&self) -> bool {
        true
    }

    /// `IsDisplayable`: false for objects that are never drawn (numbers, ...).
    fn is_displayable(&self) -> bool {
        true
    }

    /// The embedded base-class part of type `type_id`, for `occ::down_cast` to a base
    /// class: a drawable that derives from another drawable class (such as
    /// `DrawTrSurf_BSplineCurve` from `DrawTrSurf_Curve`) embeds it and answers here
    /// with [`kind_of`]. `None` by default (no drawable base class).
    fn ancestor(&self, _type_id: TypeId) -> Option<&dyn Any> {
        None
    }

    // The non-virtual members of `Draw_Drawable3D`, provided over the base state.

    /// `SetBounds`.
    fn set_bounds(&self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
        self.base().bounds.set([x_min, x_max, y_min, y_max]);
    }

    /// `Bounds`: `[x_min, x_max, y_min, y_max]`.
    fn bounds(&self) -> [f64; 4] {
        self.base().bounds.get()
    }

    /// `Visible()`.
    fn visible(&self) -> bool {
        self.base().visible.get()
    }

    /// `Visible(bool)`.
    fn set_visible(&self, visible: bool) {
        self.base().visible.set(visible);
    }

    /// `Protected()`.
    fn protected(&self) -> bool {
        self.base().protected.get()
    }

    /// `Protected(bool)`.
    fn set_protected(&self, protected: bool) {
        self.base().protected.set(protected);
    }

    /// `Name()`: the variable name, `None` before the object was stored.
    fn name(&self) -> Option<String> {
        self.base().name.borrow().clone()
    }

    /// `Name(const char*)` (virtual): records the variable name.
    fn set_name(&self, name: Option<&str>) {
        *self.base().name.borrow_mut() = name.map(str::to_string);
    }
}

impl dyn Drawable3D {
    /// `occ::down_cast` to the exact class: the concrete object when it is a `T`.
    pub fn downcast_ref<T: Drawable3D>(&self) -> Option<&T> {
        let any: &dyn Any = self;
        any.downcast_ref::<T>()
    }

    /// `occ::down_cast` to a class or one of its subclasses (`IsKind`): the object
    /// itself when it is a `T`, otherwise its embedded `T` part (which may be an
    /// abstract base part that is not a drawable on its own).
    pub fn downcast_kind<T: Any>(&self) -> Option<&T> {
        let any: &dyn Any = self;
        any.downcast_ref::<T>()
            .or_else(|| self.ancestor(TypeId::of::<T>())?.downcast_ref::<T>())
    }
}

/// `drawable` as `type_id` when it is that class or derives from it: the
/// [`Drawable3D::ancestor`] answer of a drawable that embeds a base drawable `part`.
pub fn kind_of<T: Drawable3D>(part: &T, type_id: TypeId) -> Option<&dyn Any> {
    if TypeId::of::<T>() == type_id {
        return Some(part);
    }
    part.ancestor(type_id)
}

/// `occ::down_cast` on a handle: the concrete handle when the object is a `T`.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawNumber, DrawableHandle, downcast_handle};
/// use std::rc::Rc;
/// let handle: DrawableHandle = Rc::new(DrawNumber::new(1.0));
/// assert!(downcast_handle::<DrawNumber>(&handle).is_some());
/// ```
pub fn downcast_handle<T: Drawable3D>(handle: &DrawableHandle) -> Option<Rc<T>> {
    let any: Rc<dyn Any> = handle.clone();
    any.downcast::<T>().ok()
}
