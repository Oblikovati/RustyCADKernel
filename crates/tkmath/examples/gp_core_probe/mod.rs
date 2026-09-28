//! gp_Vec, gp_Dir, gp_Pnt, gp_Lin, gp_Pln, gp_Ax2/gp_Ax3 and NCollection_Lerp/gp_TrsfNLerp
//! oracle counterpart; prints the same token stream as tools/oracle/tkmath_gp_core_probe.cxx.
use crate::elementary_probe::{attempt, flag, text};
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{Line3, Plane3};
mod directions;
mod frames;
mod lerp;
mod lines;
mod planes;
mod vectors;
pub use directions::{gp3_dir_case, gp3_pnt_case};
pub use frames::gp3_frame_case;
pub use lerp::gp3_lerp_case;
pub use lines::gp3_lin_case;
pub use planes::gp3_pln_case;
pub use vectors::gp3_vec_case;

pub fn line(value: Line3) {
    crate::elementary_probe::ax1(value.position());
}
pub fn plane(value: Plane3) {
    crate::elementary_probe::ax3(value.position());
}
/// Native `gpc_json`: dump, then parse the dump into a default value and print flag, cursor
/// and the parsed value, or the raised exception type.
pub fn json<T: Default + Copy>(
    value: T,
    dump: fn(&T, &mut DumpStream) -> std::io::Result<()>,
    parse: fn(&mut T, &str, &mut i32) -> Result<bool, Failure>,
    print: fn(T),
) {
    let mut stream = DumpStream::default();
    dump(&value, &mut stream).unwrap();
    text(stream.text());
    let mut parsed = T::default();
    let mut position: i32 = 1;
    attempt(|| {
        flag(parse(&mut parsed, stream.text(), &mut position)?);
        print!("{position} ");
        print(parsed);
        Ok(())
    });
}
