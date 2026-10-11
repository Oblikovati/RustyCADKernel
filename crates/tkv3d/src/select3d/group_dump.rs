//! Composite JSON dumps preserve the original child sequence and native member names.
use super::{Select3DSensitiveGroup, select3d_set_dump_json};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;
impl Select3DSensitiveGroup {
    pub(super) fn dump_group_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveGroup")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        self.dump_group_children(stream, depth)?;
        field_integer(stream, "myMustMatchAll", i32::from(self.must_match_all()))?;
        field_integer(
            stream,
            "myToCheckOverlapAll",
            i32::from(self.to_check_overlap_all()),
        )?;
        let bounds = self.state.read().bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
    fn dump_group_children(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        for child in self.entities().iter().filter_map(|handle| handle.get()) {
            field_dumped(stream, depth, "anEntity.get()", &|s, d| {
                child.dump_json(s, d)
            })?;
        }
        Ok(())
    }
}
