//! The registration of the XCAFDoc derived attributes (`IMPLEMENT_DERIVED_ATTRIBUTE` and
//! `IMPLEMENT_DERIVED_ATTRIBUTE_WITH_TYPE`, run when the native library loads). Rust has no
//! load-time initializer, so the constructor of each derived attribute calls
//! [`register_derived_attributes`], which registers all of them once.
use super::{
    XCAFDocArea, XCAFDocClippingPlaneTool, XCAFDocColorTool, XCAFDocDimTolTool, XCAFDocDimension,
    XCAFDocDocumentTool, XCAFDocGeomTolerance, XCAFDocLayerTool, XCAFDocLengthUnit,
    XCAFDocMaterialTool, XCAFDocNoteBalloon, XCAFDocNotesTool, XCAFDocShapeTool, XCAFDocView,
    XCAFDocViewTool, XCAFDocVolume,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfDerivedAttribute};
use std::sync::atomic::{AtomicBool, Ordering};

/// A derived attribute: its constructor and its `"xcaf"` type name, if any.
type Registration = (fn() -> Handle<dyn TdfAttribute>, Option<&'static str>);

/// The derived attributes in the order of the library's source files.
const REGISTRATIONS: [Registration; 16] = [
    (|| Handle::new(XCAFDocArea::new()).cast(), Some("Area")),
    (
        || Handle::new(XCAFDocClippingPlaneTool::new()).cast(),
        Some("ClippingPlaneTool"),
    ),
    (
        || Handle::new(XCAFDocColorTool::new()).cast(),
        Some("ColorTool"),
    ),
    (
        || Handle::new(XCAFDocDimTolTool::new()).cast(),
        Some("DimTolTool"),
    ),
    (|| Handle::new(XCAFDocDimension::new()).cast(), None),
    (
        || Handle::new(XCAFDocDocumentTool::new()).cast(),
        Some("DocumentTool"),
    ),
    (|| Handle::new(XCAFDocGeomTolerance::new()).cast(), None),
    (
        || Handle::new(XCAFDocLayerTool::new()).cast(),
        Some("LayerTool"),
    ),
    (
        || Handle::new(XCAFDocLengthUnit::new()).cast(),
        Some("LengthUnit"),
    ),
    (
        || Handle::new(XCAFDocMaterialTool::new()).cast(),
        Some("MaterialTool"),
    ),
    (|| Handle::new(XCAFDocNoteBalloon::new()).cast(), None),
    (|| Handle::new(XCAFDocNotesTool::new()).cast(), None),
    (
        || Handle::new(XCAFDocShapeTool::new()).cast(),
        Some("ShapeTool"),
    ),
    (|| Handle::new(XCAFDocView::new()).cast(), None),
    (
        || Handle::new(XCAFDocViewTool::new()).cast(),
        Some("ViewTool"),
    ),
    (|| Handle::new(XCAFDocVolume::new()).cast(), Some("Volume")),
];

/// Registers the derived attributes of XCAFDoc once; the namespaced ones carry their
/// `"xcaf"` type names.
pub fn register_derived_attributes() {
    // A flag, not a `Once`: the registry builds its prototypes with these constructors,
    // which call back here while the registration may still be running.
    static REGISTERED: AtomicBool = AtomicBool::new(false);
    if REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    for (create, type_name) in REGISTRATIONS {
        let namespace = type_name.map(|_| "xcaf");
        TdfDerivedAttribute::register(create, namespace, type_name);
    }
}
