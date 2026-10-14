//! `Aspect_DisplayConnection`: the connection to the window system (an X11 display on
//! Linux), shared by the windows and the graphic driver.
//!
//! On Linux the native library is built with `HAVE_XLIB`: the connection opens the display
//! named by `DISPLAY` (or the given name) through the Xlib wrapper and interns the
//! `WM_DELETE_WINDOW` atom. On the other platforms the native `#else` branch only stores
//! the given display address.
#[cfg(target_os = "linux")]
use super::exceptions::display_connection_definition_error;
use super::{AspectFbConfig, AspectXAtom};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::HashMap;
use std::sync::Arc;
#[path = "display_native_calls.rs"]
mod native_calls;
use native_calls::{NativeDisplayCalls, XlibDisplayCalls};

/// The locked fields of the connection.
#[derive(Debug, Default)]
struct ConnectionState {
    /// `myDisplay`: the `Display*` address, 0 for none.
    display: usize,
    /// `myDefVisualInfo`: the `XVisualInfo*` address, 0 for none.
    default_visual_info: usize,
    /// `myDefFBConfig`.
    default_fb_config: AspectFbConfig,
    /// `myAtoms`.
    atoms: HashMap<AspectXAtom, u64>,
    /// `myDisplayName`.
    display_name: AsciiString,
    /// `myIsOwnDisplay`: the connection opened the display and closes it.
    is_own_display: bool,
}

/// A window-system connection; `AspectDisplayConnection::new()?` opens `$DISPLAY`.
pub struct AspectDisplayConnection {
    transient: StandardTransient,
    state: SharedState<ConnectionState>,
    native: Arc<dyn NativeDisplayCalls>,
}
rustycad_tkernel::impl_transient!(
    AspectDisplayConnection,
    "Aspect_DisplayConnection",
    StandardTransient,
    transient
);

impl AspectDisplayConnection {
    fn with_name(display_name: AsciiString) -> Self {
        Self::with_native(display_name, Arc::new(XlibDisplayCalls))
    }
    fn with_native(display_name: AsciiString, native: Arc<dyn NativeDisplayCalls>) -> Self {
        let state = ConnectionState {
            display_name,
            ..Default::default()
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
            native,
        }
    }
    /// `Aspect_DisplayConnection()`: opens the display named by the `DISPLAY` environment
    /// variable; `Aspect_DisplayConnectionDefinitionError` when it cannot be reached (Linux).
    ///
    /// Example: `let display = AspectDisplayConnection::new()?;`.
    pub fn new() -> Result<Self, Failure> {
        let name = std::env::var("DISPLAY").unwrap_or_default();
        Self::with_display_name(&AsciiString::from(name.as_str()))
    }
    /// `Aspect_DisplayConnection(theDisplayName)`: opens the named display.
    ///
    /// Example: `AspectDisplayConnection::with_display_name(&AsciiString::from(":0"))?`.
    pub fn with_display_name(display_name: &AsciiString) -> Result<Self, Failure> {
        let connection = Self::with_name(display_name.clone());
        connection.init(0)?;
        Ok(connection)
    }
    /// `Aspect_DisplayConnection(Aspect_XDisplay*)`: wraps an existing display (the
    /// caller keeps ownership); `AspectDisplayConnection::with_display(address)?`.
    pub fn with_display(display: usize) -> Result<Self, Failure> {
        let connection = Self::with_name(AsciiString::default());
        connection.init(display)?;
        Ok(connection)
    }
    /// `GetDisplayAspect`: the display address, 0 for none; `connection.get_display_aspect()`.
    pub fn get_display_aspect(&self) -> usize {
        self.state.read().display
    }
    /// True when the connection opened (and closes) the display; `connection.is_own_display()`.
    pub fn is_own_display(&self) -> bool {
        self.state.read().is_own_display
    }
    /// `GetAtom`: the interned atom; `Standard_NoSuchObject` when it was not interned, as
    /// `NCollection_DataMap::Find` raises. Example: `connection.get_atom(AspectXAtom::DeleteWindow)?`.
    pub fn get_atom(&self, atom: AspectXAtom) -> Result<u64, Failure> {
        self.state.read().atoms.get(&atom).copied().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::no_such_object(&format!(
                "NCollection_DataMap::Find: atom {atom:?} is not interned"
            ))
        })
    }
    /// The display name; `connection.get_display_name()`.
    pub fn get_display_name(&self) -> AsciiString {
        self.state.read().display_name.clone()
    }
    /// The default `XVisualInfo*` address, 0 for none; `connection.get_default_visual_info()`.
    pub fn get_default_visual_info(&self) -> usize {
        self.state.read().default_visual_info
    }
    /// The default frame-buffer configuration; `connection.get_default_fb_config()`.
    pub fn get_default_fb_config(&self) -> AspectFbConfig {
        self.state.read().default_fb_config
    }
    /// Replaces the default visual (freeing the previous one with `XFree`) and FB config.
    ///
    /// Example: `connection.set_default_visual_info(visual, fb_config)`.
    pub fn set_default_visual_info(&self, visual: usize, fb_config: AspectFbConfig) {
        let mut state = self.state.write();
        if state.default_visual_info != 0 {
            self.native.free_visual(state.default_visual_info);
        }
        state.default_visual_info = visual;
        state.default_fb_config = fb_config;
    }
    /// `Init(theDisplay)`: closes an owned display, then wraps `display` or (when 0) opens
    /// the named one. Example: `connection.init(0)?`.
    pub fn init(&self, display: usize) -> Result<(), Failure> {
        let mut state = self.state.write();
        platform::init(&mut state, display, self.native.as_ref())
    }
}

impl Drop for AspectDisplayConnection {
    fn drop(&mut self) {
        let state = self.state.read();
        if state.default_visual_info != 0 {
            self.native.free_visual(state.default_visual_info);
        }
        if state.display != 0 && state.is_own_display {
            self.native.close(state.display);
        }
    }
}

/// The `HAVE_XLIB` branch; ordering is pinned by the owned unleased fixtures.
#[cfg(target_os = "linux")]
mod platform {
    use super::{
        AspectXAtom, ConnectionState, NativeDisplayCalls, display_connection_definition_error,
    };
    use rustycad_tkernel::standard::Failure;
    pub(super) fn init(
        state: &mut ConnectionState,
        display: usize,
        native: &dyn NativeDisplayCalls,
    ) -> Result<(), Failure> {
        prepare_display(state, display, native);
        let atom = (state.display != 0)
            .then(|| native.delete_atom(state.display))
            .flatten();
        let Some(atom) = atom else {
            return Err(connection_error(state));
        };
        state.is_own_display = display == 0;
        state.atoms.insert(AspectXAtom::DeleteWindow, atom);
        Ok(())
    }
    fn prepare_display(
        state: &mut ConnectionState,
        display: usize,
        native: &dyn NativeDisplayCalls,
    ) {
        if state.display != 0 && state.is_own_display {
            native.close(state.display);
        }
        state.is_own_display = false;
        state.atoms.clear();
        state.display = if display != 0 {
            display
        } else {
            native.open(&state.display_name)
        };
    }
    fn connection_error(state: &ConnectionState) -> Failure {
        let name = String::from_utf8_lossy(state.display_name.as_bytes());
        display_connection_definition_error(&format!("Can not connect to the server \"{name}\""))
    }
}

/// The native branch without Xlib: the connection only stores the given display.
#[cfg(not(target_os = "linux"))]
mod platform {
    use super::{ConnectionState, NativeDisplayCalls};
    use rustycad_tkernel::standard::Failure;

    pub(super) fn init(
        state: &mut ConnectionState,
        display: usize,
        _native: &dyn NativeDisplayCalls,
    ) -> Result<(), Failure> {
        state.display = display;
        state.is_own_display = display == 0;
        Ok(())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn an_unreachable_display_raises_the_definition_error() {
        let Err(failure) = AspectDisplayConnection::with_display_name(&AsciiString::from(":9999"))
        else {
            panic!("display :9999 must not exist");
        };
        assert_eq!(
            failure.exception_type(),
            "Aspect_DisplayConnectionDefinitionError"
        );
        assert_eq!(failure.message(), "Can not connect to the server \":9999\"");
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "display_connection_baseline.rs"]
mod baseline;
