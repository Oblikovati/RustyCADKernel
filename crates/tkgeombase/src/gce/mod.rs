// Rust translation of ModelingData/TKGeomBase/gce at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce: construction of the gp elementary geometry.
//!
//! Each `gce_Make*` class is a struct built by one named constructor per native
//! overload. The algorithms that derive from gce_Root implement [`Root`] and
//! return their result through `value()` (StdFail_NotDone when the construction
//! failed), `operator()` and the `TryFrom` conversion that replaces the native
//! conversion operator; the transformation makers always succeed. A native
//! constructor that can raise through gp returns `Result`.
mod circ;
mod circ2d;
mod cone;
mod cylinder;
mod dir;
mod elips;
mod hypr;
mod lin;
mod parab;
mod pln;
mod root;
mod transformations;

pub use circ::MakeCirc;
pub(crate) use circ::nearest_extremum_middle;
pub use circ2d::MakeCirc2d;
pub use cone::MakeCone;
pub use cylinder::MakeCylinder;
pub use dir::{MakeDir, MakeDir2d};
pub use elips::{MakeElips, MakeElips2d};
pub use hypr::{MakeHypr, MakeHypr2d};
pub use lin::{MakeLin, MakeLin2d};
pub use parab::{MakeParab, MakeParab2d};
pub use pln::MakePln;
pub(crate) use root::check_done;
pub use root::{ErrorType, Root};
pub use transformations::{
    MakeMirror, MakeMirror2d, MakeRotation, MakeRotation2d, MakeScale, MakeScale2d,
    MakeTranslation, MakeTranslation2d,
};

/// The members that every gce_Root algorithm shares: `Status` through [`Root`],
/// `Value()`, `Operator()` and the conversion operator to the gp result.
macro_rules! gce_result {
    ($maker:ident, $result:ty, $native:literal) => {
        impl $crate::gce::Root for $maker {
            fn status(&self) -> $crate::gce::ErrorType {
                self.the_error
            }
        }
        impl $maker {
            #[doc = concat!("`", $native, "::Value`: the constructed result, or StdFail_NotDone ")]
            #[doc = "when the construction failed; `maker.value()?`."]
            pub fn value(&self) -> Result<$result, rustycad_tkernel::standard::Failure> {
                $crate::gce::check_done(self.the_error, concat!($native, "::Value()"))?;
                Ok(self.the_result)
            }
            #[doc = concat!("`", $native, "::Operator`: the same result as `value`; `maker.operator()?`.")]
            pub fn operator(&self) -> Result<$result, rustycad_tkernel::standard::Failure> {
                self.value()
            }
        }
        #[doc = concat!("The conversion operator of `", $native, "`; `Type::try_from(&maker)?`.")]
        impl TryFrom<&$maker> for $result {
            type Error = rustycad_tkernel::standard::Failure;
            fn try_from(maker: &$maker) -> Result<Self, Self::Error> {
                maker.operator()
            }
        }
    };
}
pub(crate) use gce_result;
