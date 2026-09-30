// Rust translation of ModelingData/TKGeomBase/GCE2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GCE2d: the deprecated names of the 2D GC algorithms.
//!
//! Since OCCT 8.0.0 every GCE2d header is a deprecated alias of the GC class that
//! replaced it (`using GCE2d_MakeCircle = GC_MakeCircle2d;`), so each alias is a
//! deprecated type alias here. `GCE2d_Root` aliases GC_Root, which the port shares
//! with gce_Root as the [`Root`] trait; a trait re-export cannot carry the
//! deprecation, so its documentation carries it.
//!
//! `#[allow(deprecated)] let circle = gce2d::MakeCircle::from_frame(frame, 1.0)?;`

/// GCE2d_Root, deprecated since OCCT 8.0.0: use GC_Root ([`crate::gc::Root`]).
pub use crate::gc::Root;

/// Declares the deprecated aliases, each with the native deprecation message.
macro_rules! deprecated_aliases {
    ($($alias:ident = $target:ident, $native:literal, $message:literal;)*) => {
        $(
            #[doc = concat!("`", $native, "`: ", $message)]
            #[deprecated(note = $message)]
            pub type $alias = crate::gc::$target;
        )*
    };
}

deprecated_aliases! {
    MakeArcOfCircle = MakeArcOfCircle2d,
        "GCE2d_MakeArcOfCircle",
        "GCE2d_MakeArcOfCircle is deprecated since OCCT 8.0.0. Use GC_MakeArcOfCircle2d instead.";
    MakeArcOfEllipse = MakeArcOfEllipse2d,
        "GCE2d_MakeArcOfEllipse",
        "GCE2d_MakeArcOfEllipse is deprecated since OCCT 8.0.0. Use GC_MakeArcOfEllipse2d instead.";
    MakeArcOfHyperbola = MakeArcOfHyperbola2d,
        "GCE2d_MakeArcOfHyperbola",
        "GCE2d_MakeArcOfHyperbola is deprecated since OCCT 8.0.0. Use GC_MakeArcOfHyperbola2d instead.";
    MakeArcOfParabola = MakeArcOfParabola2d,
        "GCE2d_MakeArcOfParabola",
        "GCE2d_MakeArcOfParabola is deprecated since OCCT 8.0.0. Use GC_MakeArcOfParabola2d instead.";
    MakeCircle = MakeCircle2d,
        "GCE2d_MakeCircle",
        "GCE2d_MakeCircle is deprecated since OCCT 8.0.0. Use GC_MakeCircle2d instead.";
    MakeEllipse = MakeEllipse2d,
        "GCE2d_MakeEllipse",
        "GCE2d_MakeEllipse is deprecated since OCCT 8.0.0. Use GC_MakeEllipse2d instead.";
    MakeHyperbola = MakeHyperbola2d,
        "GCE2d_MakeHyperbola",
        "GCE2d_MakeHyperbola is deprecated since OCCT 8.0.0. Use GC_MakeHyperbola2d instead.";
    MakeLine = MakeLine2d,
        "GCE2d_MakeLine",
        "GCE2d_MakeLine is deprecated since OCCT 8.0.0. Use GC_MakeLine2d instead.";
    MakeMirror = MakeMirror2d,
        "GCE2d_MakeMirror",
        "GCE2d_MakeMirror is deprecated since OCCT 8.0.0. Use GC_MakeMirror2d instead.";
    MakeParabola = MakeParabola2d,
        "GCE2d_MakeParabola",
        "GCE2d_MakeParabola is deprecated since OCCT 8.0.0. Use GC_MakeParabola2d instead.";
    MakeRotation = MakeRotation2d,
        "GCE2d_MakeRotation",
        "GCE2d_MakeRotation is deprecated since OCCT 8.0.0. Use GC_MakeRotation2d instead.";
    MakeScale = MakeScale2d,
        "GCE2d_MakeScale",
        "GCE2d_MakeScale is deprecated since OCCT 8.0.0. Use GC_MakeScale2d instead.";
    MakeSegment = MakeSegment2d,
        "GCE2d_MakeSegment",
        "GCE2d_MakeSegment is deprecated since OCCT 8.0.0. Use GC_MakeSegment2d instead.";
    MakeTranslation = MakeTranslation2d,
        "GCE2d_MakeTranslation",
        "GCE2d_MakeTranslation is deprecated since OCCT 8.0.0. Use GC_MakeTranslation2d instead.";
}
