//! Composite JSON dumps preserve the original child sequence and native member names.
use super::{Select3DSensitiveGroup, Select3DSensitiveGroupSource, select3d_set_dump_json};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;
impl Select3DSensitiveGroup {
    pub(super) fn dump_group_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // Application wrappers retain their virtual base-state queries through this common writer.
        select3d_group_dump_json(self, stream, depth)
    }
    pub(super) fn dump_group_children(
        &self,
        stream: &mut DumpStream,
        depth: i32,
    ) -> io::Result<()> {
        for child in self.entities().iter().filter_map(|handle| handle.get()) {
            field_dumped(stream, depth, "anEntity.get()", &|s, d| {
                child.dump_json(s, d)
            })?;
        }
        Ok(())
    }
}

/// Writes native group diagnostics while preserving the source's virtual base queries.
/// Example: `select3d_group_dump_json(&application_group,&mut stream,-1)?;`.
pub fn select3d_group_dump_json<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Select3D_SensitiveGroup")?;
    base_class(stream, depth, &|nested, nested_depth| {
        select3d_set_dump_json(source, nested, nested_depth)
    })?;
    let group = source.sensitive_group();
    group.dump_group_children(stream, depth)?;
    dump_group_tail(group, stream, depth)
}
fn dump_group_tail(
    group: &Select3DSensitiveGroup,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    field_integer(stream, "myMustMatchAll", i32::from(group.must_match_all()))?;
    field_integer(
        stream,
        "myToCheckOverlapAll",
        i32::from(group.to_check_overlap_all()),
    )?;
    let bounds = group.state.read().bnd_box;
    field_dumped(stream, depth, "&myBndBox", &|nested, _| {
        bounds.dump_json(nested)
    })
}
