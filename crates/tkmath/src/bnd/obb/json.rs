// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::OrientedBoundingBox;
use crate::gp::Xyz;
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use std::io;

impl OrientedBoundingBox {
    /// `DumpJson`: class name, the nested center and axes (skipped at depth 0), the three
    /// half sizes and the axis-aligned flag as 0/1; `obb.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "Bnd_OBB")?;
        let nested = [
            ("Center", self.center),
            ("Axes[0]", self.axes[0]),
            ("Axes[1]", self.axes[1]),
            ("Axes[2]", self.axes[2]),
        ];
        for (key, value) in nested {
            dump_nested_xyz(output, depth, key, value)?;
        }
        for (index, size) in self.half_sizes.iter().enumerate() {
            dump::dump_real(output, &format!("HDims[{index}]"), *size)?;
        }
        dump::dump_integer(output, "IsAABox", i32::from(self.is_aa_box))
    }
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED`: the field's own dump wrapped as `"key": {...}`.
fn dump_nested_xyz(
    output: &mut impl DumpOutput,
    depth: i32,
    key: &str,
    value: Xyz,
) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    let mut nested = DumpStream::default();
    value.dump_json(&mut nested)?;
    dump::dump_key_to_class(output, key, nested.text())
}
