//! `Dump`: one `printf` line per multipoint.
use super::ApproxIntMultiLine;
use crate::approx_int::line::ApproxIntLine;
use crate::int_patch::dump::{signed_fixed, sink_failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkmath::gp::{Point2, Point3};
use std::fmt::Write;

impl<L: ApproxIntLine> ApproxIntMultiLine<L> {
    /// `Dump()`: the index, the 3d point and both 2d points of each multipoint, as the
    /// native `printf("%4d  [%+10.20f ...]")` writes them to the standard output;
    /// `line.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        let mut arr1: Array1<'static, Point3> = arrays::one_based(1)?;
        let mut arr2: Array1<'static, Point2> = arrays::one_based(2)?;
        for ind in self.first_point()..=self.last_point() {
            self.value(ind, &mut arr1, &mut arr2)?;
            let (p, a, b) = (
                arrays::at(&arr1, 1)?,
                arrays::at(&arr2, 1)?,
                arrays::at(&arr2, 2)?,
            );
            writeln!(
                out,
                "{ind:4}  [{} {} {}]  [{} {}]  [{} {}]",
                signed_fixed(p.x()),
                signed_fixed(p.y()),
                signed_fixed(p.z()),
                signed_fixed(a.x()),
                signed_fixed(a.y()),
                signed_fixed(b.x()),
                signed_fixed(b.y()),
            )
            .map_err(sink_failure)?;
        }
        Ok(())
    }
}
