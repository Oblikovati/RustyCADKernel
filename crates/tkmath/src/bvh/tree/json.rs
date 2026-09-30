// Rust translation of FoundationClasses/TKMath/BVH/BVH_Tree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_TreeBase::DumpJson` / `DumpNode` with the keys the dump macros derive
//! from the stringified field expressions.
use super::BvhTree;
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::{
    Failure,
    dump::{self, DumpOutput, DumpStream},
};
use std::io;

fn io_failure(error: io::Error) -> Failure {
    Failure::new(&format!("BVH tree dump: stream write failed: {error}"))
}

/// `OCCT_DUMP_FIELD_VALUE_STRING(stream, className)`.
fn dump_class_name(output: &mut impl DumpOutput, name: &str) -> io::Result<()> {
    dump::add_values_separator(output)?;
    output.write_fragment(format!("\"className\": \"{name}\"").as_bytes())
}

/// `Standard_Dump::DumpKeyToClass`: `"key": {nested}`.
fn dump_key_to_class(output: &mut impl DumpOutput, key: &str, nested: &str) -> io::Result<()> {
    dump::add_values_separator(output)?;
    output.write_fragment(format!("\"{key}\": {{{nested}}}").as_bytes())
}

impl<V: BvhVector, A> BvhTree<V, A> {
    /// Dumps the class name (`BVH_TreeBaseTransient`, the RTTI name the template
    /// inherits), depth, length and every node; `tree.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> Result<(), Failure> {
        self.dump_header(output).map_err(io_failure)?;
        for node in 0..self.length() {
            self.dump_node(node, output, depth)?;
        }
        Ok(())
    }

    fn dump_header(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump_class_name(output, "BVH_TreeBaseTransient")?;
        dump::dump_integer(output, "Depth", self.depth)?;
        dump::dump_integer(output, "Length", self.length())
    }

    /// Dumps one node record; the node box is nested as `Pointer` unless `depth`
    /// is zero. Keys keep upstream's call text, e.g. `BegPrimitive(theNodeIndex)`.
    /// Example: `tree.dump_node(0, &mut stream, -1)?`.
    pub fn dump_node(
        &self,
        node: i32,
        output: &mut impl DumpOutput,
        depth: i32,
    ) -> Result<(), Failure> {
        let bounding = V::to_bnd_box(self.min_point(node)?, self.max_point(node)?);
        dump_class_name(output, "BVH_TreeNode").map_err(io_failure)?;
        dump::dump_integer(output, "theNodeIndex", node).map_err(io_failure)?;
        if depth != 0 {
            let mut nested = DumpStream::default();
            bounding.dump_json(&mut nested).map_err(io_failure)?;
            dump_key_to_class(output, "Pointer", nested.text()).map_err(io_failure)?;
        }
        let fields = [
            ("BegPrimitive(theNodeIndex)", self.beg_primitive(node)?),
            ("EndPrimitive(theNodeIndex)", self.end_primitive(node)?),
            ("Level(theNodeIndex)", self.level(node)?),
            ("IsOuter(theNodeIndex)", i32::from(self.is_outer(node)?)),
        ];
        for (key, value) in fields {
            dump::dump_integer(output, key, value).map_err(io_failure)?;
        }
        Ok(())
    }
}
