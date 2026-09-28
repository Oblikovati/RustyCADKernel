//! The gce package probe; the counterpart of tools/oracle/tkgeombase_probe_gce.cxx,
//! split by the kind of result.
mod curves;
mod surfaces;
mod transformations;
pub use curves::*;
pub use surfaces::*;
pub use transformations::*;

/// gb_gce_root inside GB_TRY: status, IsDone, IsError, Value() and Operator(), or the
/// exception that the constructor raised.
macro_rules! gce_root {
    ($print:path, $maker:expr) => {
        crate::gb_probe::common::attempt(|| {
            let maker = $maker?;
            crate::gb_probe::common::int(maker.status() as i32);
            crate::gb_probe::common::flag(maker.is_done());
            crate::gb_probe::common::flag(maker.is_error());
            crate::gb_probe::common::show(maker.value(), $print);
            crate::gb_probe::common::show(maker.operator(), $print);
            Ok(())
        })
    };
}

/// gb_gce_trsf inside GB_TRY: Value() and Operator() of a transformation maker.
macro_rules! gce_trsf {
    ($print:path, $maker:expr) => {
        crate::gb_probe::common::attempt(|| {
            let maker = $maker?;
            $print(maker.value());
            $print(maker.operator());
            Ok(())
        })
    };
}

/// A maker that cannot raise, in the shape of the fallible ones.
pub(super) fn ok<T>(value: T) -> Result<T, rustycad_tkernel::standard::Failure> {
    Ok(value)
}
pub(super) use {gce_root, gce_trsf};
