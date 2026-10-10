//! Native dump field order, including empty separators and ignored depth.
use super::SelectMgrSelectableObjectSet;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::json_dump::{field_integer, field_pointer, field_string};
use std::io;
impl SelectMgrSelectableObjectSet {
    /// Dump subset membership and rebuild flags; depth is ignored; `set.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        for (index, subset) in self.subsets.iter().enumerate() {
            let state = subset.read();
            field_integer(stream, "aSubsetIdx", index as i32)?;
            field_integer(stream, "IsDirty", i32::from(state.dirty))?;
            for object in state.objects.iter() {
                field_pointer(stream, "SelectableObject", object.object_address() as usize)?;
            }
            field_string(stream, "separator", b"")?;
        }
        let size = self.view.read().size;
        field_integer(stream, "myLastWinSize.x()", size[0])?;
        field_integer(stream, "myLastWinSize.y()", size[1])
    }
}
