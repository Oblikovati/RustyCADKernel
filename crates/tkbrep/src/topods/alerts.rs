// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_AlertAttribute.hxx/.cxx and
// TopoDS_AlertWithShape.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Message alerts and attributes that carry a shape.
//!
//! The Message port shares alerts and attributes as `Arc<dyn Alert>` and
//! `Arc<dyn Attribute>` (never null); their mutable fields sit behind locks.
use super::Shape;
use rustycad_tkernel::message::{
    Alert, AlertExtended, AlertHandle, Attribute, AttributeBase, AttributeStream, BaseAttribute,
    Gravity, Messenger, PrinterToReport,
};
use rustycad_tkernel::message_runtime_type;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // No member panics while it holds the lock, so a poisoned value is consistent.
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// `TopoDS_AlertAttribute`: a `Message_AttributeStream` whose stream is the JSON dump
/// of a shape, reported by the `Messenger << Shape` operator.
///
/// ```
/// use rustycad_tkbrep::topods::{AlertAttribute, Shape};
/// use rustycad_tkernel::message::Attribute;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let attribute = AlertAttribute::new(&Shape::default(), &AsciiString::default());
/// assert!(attribute.get_shape().is_null());
/// assert_eq!(attribute.dynamic_type().name(), "TopoDS_AlertAttribute");
/// ```
pub struct AlertAttribute {
    stream: AttributeStream,
    shape: Shape,
}
message_runtime_type!(AlertAttribute, "TopoDS_AlertAttribute", AttributeStream);

impl AlertAttribute {
    /// `TopoDS_AlertAttribute(theShape, theName)`: stores the shape and its full JSON
    /// dump as the attribute stream; `AlertAttribute::new(&shape, &name)`.
    pub fn new(shape: &Shape, name: &AsciiString) -> Self {
        let mut text = DumpStream::default();
        // An in-memory dump stream cannot fail.
        let _ = shape.dump_json(&mut text, -1);
        Self {
            stream: AttributeStream::new(text.bytes(), name),
            shape: shape.clone(),
        }
    }

    /// `GetShape()`: the contained shape; `attribute.get_shape()`.
    pub fn get_shape(&self) -> &Shape {
        &self.shape
    }

    /// The inherited `Message_AttributeStream::Stream()` text; `attribute.stream()`.
    pub fn stream(&self) -> Vec<u8> {
        self.stream.stream()
    }

    /// `Send(theMessenger, theShape)` (and `operator<<`): adds, to the report of each
    /// printer of `messenger` that is a `Message_PrinterToReport`, an Info alert holding
    /// a new attribute for `shape`; `AlertAttribute::send(&messenger, &shape)`.
    pub fn send(messenger: &Messenger, shape: &Shape) {
        let printers = messenger.printers().clone();
        for printer in printers {
            if !printer
                .dynamic_type()
                .is_subtype(PrinterToReport::static_type())
            {
                continue;
            }
            let any: Arc<dyn Any + Send + Sync> = printer;
            let Ok(to_report) = any.downcast::<PrinterToReport>() else {
                continue;
            };
            let attribute = Arc::new(Self::new(shape, &AsciiString::default()));
            AlertExtended::add_alert(&to_report.report(), Some(attribute), Gravity::Info);
        }
    }
}

impl Attribute for AlertAttribute {
    fn attribute_base(&self) -> &AttributeBase {
        self.stream.attribute_base()
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// The class name, the plain `Message_Attribute` dump as the base class (not the
    /// stream, as natively), then `"Shape": {...}`.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        // In-memory dump streams cannot fail.
        let _ = dump::dump_class_begin(output, Self::static_type().name());
        if depth == 0 {
            return;
        }
        let _ = dump::add_values_separator(output);
        BaseAttribute::new(&self.get_name()).dump_json(output, depth - 1);
        let mut field = DumpStream::default();
        let _ = self.shape.dump_json(&mut field, depth - 1);
        let _ = dump::dump_key_to_class(output, "Shape", field.bytes());
    }
}

/// `TopoDS_AlertWithShape`: an alert that carries a shape and never merges.
///
/// ```
/// use rustycad_tkbrep::topods::{AlertWithShape, Shape};
/// use rustycad_tkernel::message::Alert;
/// let alert = AlertWithShape::new(&Shape::default());
/// assert!(!alert.supports_merge());
/// ```
pub struct AlertWithShape {
    shape: Mutex<Shape>,
}
message_runtime_type!(
    AlertWithShape,
    "TopoDS_AlertWithShape",
    rustycad_tkernel::message::BaseAlert
);

impl AlertWithShape {
    /// `TopoDS_AlertWithShape(theShape)`; `AlertWithShape::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        Self {
            shape: Mutex::new(shape.clone()),
        }
    }

    /// `GetShape()`: a copy of the contained shape; `alert.get_shape()`.
    pub fn get_shape(&self) -> Shape {
        locked(&self.shape).clone()
    }

    /// `SetShape(theShape)`; `alert.set_shape(&shape)`.
    pub fn set_shape(&self, shape: &Shape) {
        *locked(&self.shape) = shape.clone();
    }
}

impl Alert for AlertWithShape {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// `SupportsMerge()`: false.
    fn supports_merge(&self) -> bool {
        false
    }
    /// `Merge(theTarget)`: false (the native shape comparison is commented out).
    fn merge(&self, _target: &AlertHandle) -> bool {
        false
    }
}

/// `DEFINE_ALERT_WITH_SHAPE(Alert)`: declares an alert class derived from
/// `TopoDS_AlertWithShape`, with its own runtime type.
///
/// ```
/// use rustycad_tkbrep::define_alert_with_shape;
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::message::Alert;
/// define_alert_with_shape!(AlertCustom, "AlertCustom");
/// let alert = AlertCustom::new(&Shape::default());
/// assert_eq!(alert.dynamic_type().name(), "AlertCustom");
/// assert!(!alert.supports_merge());
/// ```
#[macro_export]
macro_rules! define_alert_with_shape {
    ($alert:ident, $name:literal) => {
        #[doc = concat!("Alert with shape `", $name, "`.")]
        pub struct $alert {
            parent: $crate::topods::AlertWithShape,
        }
        rustycad_tkernel::message_runtime_type!($alert, $name, $crate::topods::AlertWithShape);
        impl $alert {
            /// The native constructor `Alert(theShape)`.
            pub fn new(shape: &$crate::topods::Shape) -> Self {
                Self {
                    parent: $crate::topods::AlertWithShape::new(shape),
                }
            }
        }
        impl std::ops::Deref for $alert {
            type Target = $crate::topods::AlertWithShape;
            fn deref(&self) -> &Self::Target {
                &self.parent
            }
        }
        impl rustycad_tkernel::message::Alert for $alert {
            fn dynamic_type(&self) -> &'static rustycad_tkernel::standard::RuntimeType {
                <Self as rustycad_tkernel::standard::RuntimeTyped>::static_type()
            }
            fn supports_merge(&self) -> bool {
                rustycad_tkernel::message::Alert::supports_merge(&self.parent)
            }
            fn merge(&self, target: &rustycad_tkernel::message::AlertHandle) -> bool {
                rustycad_tkernel::message::Alert::merge(&self.parent, target)
            }
        }
    };
}
