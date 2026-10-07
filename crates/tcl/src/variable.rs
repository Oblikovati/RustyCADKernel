//! Variable access: `Tcl_GetVar`, `Tcl_SetVar`, `Tcl_SetVar2`, `Tcl_UnsetVar`.
use crate::text::{from_c_pointer, to_c_string};
use crate::{InterpHandle, TclError, sys};
use std::ffi::c_int;

impl InterpHandle {
    /// `Tcl_GetVar`: the value of `name` (`a` or `a(key)`), copied; `None` when unset.
    ///
    /// ```
    /// let interp = rustycad_tcl::Interp::new();
    /// interp.set_var("x", "1", 0).unwrap();
    /// assert_eq!(interp.get_var("x", 0).unwrap().as_deref(), Some("1"));
    /// ```
    pub fn get_var(&self, name: &str, flags: c_int) -> Result<Option<String>, TclError> {
        let name = to_c_string(name)?;
        // SAFETY: live interpreter and C string; the value is copied before any other
        // call can invalidate it.
        Ok(unsafe { from_c_pointer(sys::Tcl_GetVar(self.as_ptr(), name.as_ptr(), flags)) })
    }

    /// `Tcl_SetVar`: sets `name` and returns the new value, or `None` when Tcl refused
    /// (a trace error, or an array name); with `TCL_LEAVE_ERR_MSG` the reason is the result.
    pub fn set_var(
        &self,
        name: &str,
        value: &str,
        flags: c_int,
    ) -> Result<Option<String>, TclError> {
        let (name, value) = (to_c_string(name)?, to_c_string(value)?);
        // SAFETY: live interpreter and C strings; the returned value is copied at once.
        let stored =
            unsafe { sys::Tcl_SetVar(self.as_ptr(), name.as_ptr(), value.as_ptr(), flags) };
        // SAFETY: Tcl returns NULL or the variable's live value.
        Ok(unsafe { from_c_pointer(stored) })
    }

    /// `Tcl_SetVar2`: sets element `element` of array `array` (or scalar `array` when
    /// `element` is `None`); returns the new value or `None` on refusal.
    pub fn set_var2(
        &self,
        array: &str,
        element: Option<&str>,
        value: &str,
        flags: c_int,
    ) -> Result<Option<String>, TclError> {
        let (array, value) = (to_c_string(array)?, to_c_string(value)?);
        let element = element.map(to_c_string).transpose()?;
        let element_pointer = element
            .as_ref()
            .map_or(std::ptr::null(), |text| text.as_ptr());
        // SAFETY: live interpreter; every pointer is null (allowed for part2) or a live C string.
        let stored = unsafe {
            sys::Tcl_SetVar2(
                self.as_ptr(),
                array.as_ptr(),
                element_pointer,
                value.as_ptr(),
                flags,
            )
        };
        // SAFETY: Tcl returns NULL or the variable's live value, copied at once.
        Ok(unsafe { from_c_pointer(stored) })
    }

    /// `Tcl_UnsetVar`: true when the variable existed and was removed.
    pub fn unset_var(&self, name: &str, flags: c_int) -> Result<bool, TclError> {
        let name = to_c_string(name)?;
        // SAFETY: live interpreter and C string.
        Ok(unsafe { sys::Tcl_UnsetVar(self.as_ptr(), name.as_ptr(), flags) } == sys::TCL_OK)
    }
}
