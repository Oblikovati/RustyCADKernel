// Rust translation of ModelingAlgorithms/TKGeomAlgo/GccAna (GccAna_NoSolution.hxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GccAna: analytic 2d constraint solvers. They build lines and circles
//! tangent to, passing through or centred on points, lines and circles (qualified by
//! GccEnt), and the bisecting loci (GccInt) of points, lines and circles.
//!
//! Every solver keeps the native solution order, tolerance tests and fixed-size
//! solution arrays: a solution is addressed by its native number (1-based), and an
//! index past an array raises Standard_OutOfRange as the native bounds check does.
//! Each native constructor overload is a named Rust constructor returning `Result`,
//! since the native ones raise (GccEnt_BadQualifier, Standard_NegativeValue...);
//! output references become returned tuples.
mod circ2d_2tan_on;
mod circ2d_2tan_rad;
mod circ2d_3tan;
mod circ2d_bisec;
mod circ2d_tan_cen;
mod circ2d_tan_on_rad;
mod circ_lin2d_bisec;
mod circ_pnt2d_bisec;
mod lin2d_2tan;
mod lin2d_bisec;
mod lin2d_tan_obl;
mod lin_pnt2d_bisec;
mod pnt2d_bisec;
pub(crate) mod solutions;

pub use circ_lin2d_bisec::CircLin2dBisec;
pub use circ_pnt2d_bisec::CircPnt2dBisec;
pub use circ2d_2tan_on::Circ2d2TanOn;
pub use circ2d_2tan_rad::Circ2d2TanRad;
pub use circ2d_3tan::Circ2d3Tan;
pub use circ2d_bisec::Circ2dBisec;
pub use circ2d_tan_cen::Circ2dTanCen;
pub use circ2d_tan_on_rad::Circ2dTanOnRad;
pub use lin_pnt2d_bisec::LinPnt2dBisec;
pub use lin2d_2tan::Lin2d2Tan;
pub use lin2d_bisec::Lin2dBisec;
pub use lin2d_tan_obl::Lin2dTanObl;
pub use pnt2d_bisec::Pnt2dBisec;

use rustycad_tkernel::standard::Failure;

/// `GccAna_NoSolution` (a Standard_Failure natively), raised when a solution is read
/// from a solver that has none; `no_solution("GccAna_Pnt2dBisec::ThisSolution: coincident points")`.
pub fn no_solution(message: &str) -> Failure {
    Failure::with_type("GccAna_NoSolution", message, "")
}

/// The accepted-qualifier test that opens every solver taking qualified arguments:
/// GccEnt_BadQualifier unless the qualifier is enclosed, enclosing, outside or
/// unqualified (that is, anything but GccEnt_noqualifier).
pub(crate) fn check_qualifier(
    position: crate::gcc_ent::Position,
    member: &str,
) -> Result<(), Failure> {
    use crate::gcc_ent::Position;
    if position == Position::NoQualifier {
        return Err(crate::gcc_ent::bad_qualifier(&format!(
            "{member}: qualifier GccEnt_noqualifier; expected enclosed, enclosing, outside or unqualified"
        )));
    }
    Ok(())
}

/// The accepted-qualifier test of a line argument: GccEnt_BadQualifier unless the
/// qualifier is enclosed, outside or unqualified (a line encloses nothing).
pub(crate) fn check_line_position(
    position: crate::gcc_ent::Position,
    member: &str,
) -> Result<(), Failure> {
    use crate::gcc_ent::Position;
    if matches!(
        position,
        Position::Enclosed | Position::Outside | Position::Unqualified
    ) {
        return Ok(());
    }
    Err(crate::gcc_ent::bad_qualifier(&format!(
        "{member}: line qualifier {position:?}; expected enclosed, outside or unqualified"
    )))
}

/// `occ::handle<GccInt_Bisec> h = new T(...)`: a bisecting locus behind the
/// abstract handle the bisector algorithms return.
pub(crate) fn bisector<T>(value: T) -> rustycad_tkernel::standard::Handle<dyn crate::gcc_int::Bisec>
where
    T: crate::gcc_int::Bisec + rustycad_tkernel::standard::RuntimeTyped,
{
    rustycad_tkernel::standard::Handle::new(value).cast()
}
