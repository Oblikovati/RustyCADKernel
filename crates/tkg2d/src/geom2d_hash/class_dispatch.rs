// Rust translation of the dispatch shared by Geom2dHash_CurveHasher.cxx, GeomHash_CurveHasher.cxx and GeomHash_SurfaceHasher.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact-type dispatch of the polymorphic hashers: the null and identity cases,
//! the down cast on an exact dynamic type, and the fallback for unknown classes.
use rustycad_tkernel::standard::c_string::CStringHasher;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};

/// The typed handle when the dynamic type of `handle` is exactly `T`
/// (`aType == STANDARD_TYPE(T)`, then `occ::down_cast<T>`);
/// `exact_class::<Geom2dLine, _>(&curve)`.
pub fn exact_class<T: RuntimeTyped, S: ?Sized + RuntimeTyped>(
    handle: &Handle<S>,
) -> Option<Handle<T>> {
    handle
        .is_instance(T::static_type())
        .then(|| handle.cast::<T>())
}

/// The part of every polymorphic comparison that precedes the typed dispatch: two
/// null handles are equal, one null handle is not, the same object is, and objects
/// of different dynamic types are not; `None` when the typed hashers must decide;
/// `decided_equality(&a, &b)`.
pub fn decided_equality<S: ?Sized + RuntimeTyped>(
    first: &Handle<S>,
    second: &Handle<S>,
) -> Option<bool> {
    let (Some(first_type), Some(second_type)) = (first.dynamic_type(), second.dynamic_type())
    else {
        return Some(first.is_null() && second.is_null());
    };
    if first.object_address() == second.object_address() {
        return Some(true);
    }
    if first_type.rust_type() != second_type.rust_type() {
        return Some(false);
    }
    None
}

/// The hash of an object that no typed hasher handles: the Standard_CStringHasher
/// hash of its type name, 0 for a null handle; `unknown_class_hash(&curve)`.
pub fn unknown_class_hash<S: ?Sized + RuntimeTyped>(handle: &Handle<S>) -> usize {
    handle
        .dynamic_type()
        .map_or(0, |kind| CStringHasher.hash_code(kind.name().as_bytes()))
}

/// Returns the typed hasher result for the first listed class that is the exact
/// dynamic type of the handle (of both handles for `equal`), in the listed order, which
/// is the native dispatch order; falls through when no class matches.
///
/// Exported for the GeomHash polymorphic hashers (TKG3d);
/// `hasher_class_dispatch!(hash curve, comp, tol; Geom2dLine => Geom2dHashLineHasher)`.
#[macro_export]
macro_rules! hasher_class_dispatch {
    (hash $handle:expr, $comp:expr, $tol:expr; $($class:ty => $hasher:ident),+ $(,)?) => {
        $(if let Some(typed) =
            $crate::geom2d_hash::class_dispatch::exact_class::<$class, _>($handle)
        {
            return $hasher::new($comp, $tol).hash(&typed);
        })+
    };
    (equal $first:expr, $second:expr, $comp:expr, $tol:expr;
     $($class:ty => $hasher:ident),+ $(,)?) => {
        $(if let (Some(a), Some(b)) = (
            $crate::geom2d_hash::class_dispatch::exact_class::<$class, _>($first),
            $crate::geom2d_hash::class_dispatch::exact_class::<$class, _>($second),
        ) {
            return $hasher::new($comp, $tol).equal(&a, &b);
        })+
    };
}
