//! Each native getter reads the original drawer argument; values are genuine value copies.
use crate::{MeshVSDrawer, MeshVSDrawerAttribute, MeshVSDrawerSource};
use rustycad_tkernel::standard::{Failure, HandleReference, exceptions::program_error};
pub(super) struct DrawerValues<'a> {
    pub(super) drawer: &'a dyn HandleReference<dyn MeshVSDrawerSource>,
    pub(super) defaults: bool,
}
impl DrawerValues<'_> {
    pub(super) fn new(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        defaults: bool,
    ) -> DrawerValues<'_> {
        DrawerValues { drawer, defaults }
    }

    pub(super) fn read<T>(
        &self,
        key: MeshVSDrawerAttribute,
        mut output: T,
        getter: fn(&MeshVSDrawer, i32, &mut T) -> bool,
    ) -> Result<Option<T>, Failure> {
        let observed = self.drawer.observe();
        let source = observed.get().ok_or_else(|| {
            program_error("MeshVS_Tool drawer=null/expired: expected a live original drawer slot")
        })?;
        let found = getter(source.drawer_base(), key as i32, &mut output);
        Ok((found || self.defaults).then_some(output))
    }
}
pub(super) fn native_enum<T>(
    value: i32,
    converted: Option<T>,
    expected: &str,
) -> Result<T, Failure> {
    converted.ok_or_else(|| {
        program_error(&format!(
            "MeshVS_Tool enum value={value}: expected native {expected} enumeration range"
        ))
    })
}
macro_rules! required_value {
    ($value:expr) => {
        match $value? {
            Some(value) => value,
            None => return Ok(None),
        }
    };
}
pub(super) use required_value;
