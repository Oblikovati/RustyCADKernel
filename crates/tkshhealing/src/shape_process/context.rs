// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcess/ShapeProcess_Context.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeProcess_Context`: the resource file of a shape-processing sequence, read under a
//! stack of scopes, and the messenger and trace level of the operators.
mod resource_cache;
mod values;

use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Message, Messenger};
use rustycad_tkernel::resource::ResourceManager;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

/// The shared resource manager of a context (the native `occ::handle<Resource_Manager>`).
pub type ShapeProcessResource = Arc<SharedState<ResourceManager>>;

/// The private fields of `ShapeProcess_Context`.
struct ContextFields {
    resource: Option<ShapeProcessResource>,
    scope: Option<Vec<AsciiString>>,
    messenger: Arc<Messenger>,
    trace_level: i32,
}

/// `ShapeProcess_Context`: parameters are looked up in the resource manager under the
/// current scope (`scope.param`), with `&name` aliases for real and integer values.
///
/// ```
/// use rustycad_tkshhealing::shape_process::ShapeProcessContext;
/// let context = ShapeProcessContext::new();
/// assert!(!context.is_param_set("Tolerance"));
/// assert_eq!(context.real_val("Tolerance", 0.5).unwrap(), 0.5);
/// assert_eq!(context.trace_level(), 1);
/// ```
pub struct ShapeProcessContext {
    parent: StandardTransient,
    fields: SharedState<ContextFields>,
}
impl_transient!(
    ShapeProcessContext,
    "ShapeProcess_Context",
    StandardTransient,
    parent
);

impl Default for ShapeProcessContext {
    /// The native default constructor: no resource manager, the default messenger, trace
    /// level 1.
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(ContextFields {
                resource: None,
                scope: None,
                messenger: Message::default_messenger().clone(),
                trace_level: 1,
            }),
        }
    }
}

impl ShapeProcessContext {
    /// `ShapeProcess_Context()`; `ShapeProcessContext::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeProcess_Context(file, scope)`: `Init(file, scope)`, then the default messenger
    /// and trace level 1; `ShapeProcessContext::with_file("DE", "read")?`.
    pub fn with_file(file: &str, scope: &str) -> Result<Self, Failure> {
        let context = Self::new();
        context.init(file, scope)?;
        Ok(context)
    }

    /// `Init(file, scope)`: the scopes are cleared, the resource manager of `file` loaded
    /// (an empty one for an empty name) and `scope` pushed when not empty; always true
    /// ("NOT IMPLEMENTED"); `context.init("DE", "read")?`.
    pub fn init(&self, file: &str, scope: &str) -> Result<bool, Failure> {
        self.fields.write().scope = None;
        let resource = if file.is_empty() {
            Arc::new(SharedState::new(ResourceManager::empty(
                rustycad_tkernel::resource::ResourceHost::native(),
            )))
        } else {
            self.load_resource_manager(file)?
        };
        self.fields.write().resource = Some(resource);
        if !scope.is_empty() {
            self.set_scope(scope);
        }
        Ok(true)
    }

    /// `ResourceManager()`: the current manager, `None` for the native null handle.
    pub fn resource_manager(&self) -> Option<ShapeProcessResource> {
        self.fields.read().resource.clone()
    }

    /// `SetScope(scope)`: pushes `scope` appended (with a dot) to the current one.
    /// Example: `context.set_scope("FixShape")`.
    pub fn set_scope(&self, scope: &str) {
        let mut fields = self.fields.write();
        let stack = fields.scope.get_or_insert_with(Vec::new);
        let mut name = match stack.last() {
            Some(current) => {
                let mut name = current.clone();
                name.assign_cat(".");
                name
            }
            None => AsciiString::default(),
        };
        name.assign_cat(scope);
        stack.push(name);
    }

    /// `UnSetScope()`: pops the current scope, if any; `context.un_set_scope()`.
    pub fn un_set_scope(&self) {
        if let Some(stack) = self.fields.write().scope.as_mut() {
            stack.pop();
        }
    }

    /// `SetMessenger(messenger)`: `None` (the native null handle) restores the default
    /// messenger; `context.set_messenger(Some(messenger))`.
    pub fn set_messenger(&self, messenger: Option<Arc<Messenger>>) {
        self.fields.write().messenger =
            messenger.unwrap_or_else(|| Message::default_messenger().clone());
    }

    /// `Messenger()`; `context.messenger()`.
    pub fn messenger(&self) -> Arc<Messenger> {
        self.fields.read().messenger.clone()
    }

    /// `SetTraceLevel(tracelev)`; `context.set_trace_level(2)`.
    pub fn set_trace_level(&self, level: i32) {
        self.fields.write().trace_level = level;
    }

    /// `TraceLevel()`; `context.trace_level()`.
    pub fn trace_level(&self) -> i32 {
        self.fields.read().trace_level
    }

    /// The static `MakeName(scope, param)`: `param` under the current scope.
    fn scoped_name(&self, param: &str) -> AsciiString {
        let fields = self.fields.read();
        let Some(current) = fields.scope.as_ref().and_then(|stack| stack.last()) else {
            return AsciiString::from(param);
        };
        let mut name = current.clone();
        name.assign_cat(".");
        name.assign_cat(param);
        name
    }
}

/// The resource name of an ASCII string (the native `ToCString()`).
fn key(name: &AsciiString) -> String {
    String::from_utf8_lossy(name.as_bytes()).into_owned()
}
