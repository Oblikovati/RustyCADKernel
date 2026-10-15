// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc.hxx and .cxx (the GUIDs) and
// XCAFDoc_ColorType.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc`: the GUIDs of the XDE references and flags, and `XCAFDoc_ColorType`.
use rustycad_tkernel::standard::Guid;
use rustycad_tklcaf::guid_from_text;

/// `XCAFDoc_ColorType`: the kind of color of a shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDocColorType {
    /// `XCAFDoc_ColorGen`: the generic color.
    #[default]
    ColorGen = 0,
    /// `XCAFDoc_ColorSurf`: the color of the surfaces.
    ColorSurf = 1,
    /// `XCAFDoc_ColorCurv`: the color of the curves.
    ColorCurv = 2,
}

impl crate::NativeEnum for XCAFDocColorType {
    const ALL: &'static [Self] = &[Self::ColorGen, Self::ColorSurf, Self::ColorCurv];
}

/// `XCAFDoc`: a class of static members only.
pub struct XCAFDoc;

/// Declares the `XCAFDoc` GUID getters: `name => "guid text"` with their native name.
macro_rules! xcaf_doc_guids {
    ($($(#[$doc:meta])* $name:ident => $text:literal;)*) => {
        impl XCAFDoc {
            $(
                $(#[$doc])*
                pub fn $name() -> Guid {
                    const ID: Guid = guid_from_text($text);
                    ID
                }
            )*
        }
    };
}

xcaf_doc_guids! {
    /// `AssemblyGUID`: the flag of an assembly label.
    assembly_guid => "5b896b00-3adf-11d4-b9b7-0060b0ee281b";
    /// `ShapeRefGUID`: the tree node of a component referring to its shape.
    shape_ref_guid => "5b896afe-3adf-11d4-b9b7-0060b0ee281b";
    /// `DimTolRefGUID`: the tree node of a shape referring to its dimension or tolerance.
    dim_tol_ref_guid => "efd212e9-6dfd-11d4-b9c8-0060b0ee281b";
    /// `DimensionRefFirstGUID`: the graph node of the first shape of a dimension.
    dimension_ref_first_guid => "efd212e3-6dfd-11d4-b9c8-0060b0ee281b";
    /// `DimensionRefSecondGUID`: the graph node of the second shape of a dimension.
    dimension_ref_second_guid => "efd212e0-6dfd-11d4-b9c8-0060b0ee281b";
    /// `GeomToleranceRefGUID`: the graph node of a shape referring to its tolerance.
    geom_tolerance_ref_guid => "efd213e3-6dfd-11d4-b9c8-0060b0ee281b";
    /// `DatumRefGUID`: the reference of a shape to its datum.
    datum_ref_guid => "efd212e2-6dfd-11d4-b9c8-0060b0ee281b";
    /// `DatumTolRefGUID`: the graph node of a datum referring to its tolerances.
    datum_tol_ref_guid => "efd212e7-6dfd-11d4-b9c8-0060b0ee281b";
    /// `LayerRefGUID`: the graph node of a shape referring to its layers.
    layer_ref_guid => "efd212e8-6dfd-11d4-b9c8-0060b0ee281b";
    /// `MaterialRefGUID`: the tree node of a shape referring to its material.
    material_ref_guid => "efd212f7-6dfd-11d4-b9c8-0060b0ee281b";
    /// `VisMaterialRefGUID`: the reference of a shape to its visualization material.
    vis_material_ref_guid => "936F4070-5369-405D-A7AD-2AC76C860EC8";
    /// `NoteRefGUID`: the reference of an annotated item to its note.
    note_ref_guid => "F3599E50-F84A-493e-8D1B-1284E79322F1";
    /// `InvisibleGUID`: the flag of an invisible shape or layer.
    invisible_guid => "5b896aff-3adf-11d4-b9b7-0060b0ee281b";
    /// `ColorByLayerGUID`: the flag of a shape colored by its layer.
    color_by_layer_guid => "279e8c1e-70af-4130-b626-9cc52a537db8";
    /// `ExternRefGUID`: the external reference of a shape.
    extern_ref_guid => "6b896b01-3adf-11d4-b9b7-0060b0ee281b";
    /// `SHUORefGUID`: the graph node of a specified higher usage occurrence.
    shuo_ref_guid => "efd212ea-6dfd-11d4-b9c8-0060b0ee281b";
    /// `ViewRefGUID`: the reference of a view.
    view_ref_guid => "efd213e5-6dfd-11d4-b9c8-0060b0ee281b";
    /// `ViewRefShapeGUID`: the graph node of a view referring to its shapes.
    view_ref_shape_guid => "efd213e6-6dfd-11d4-b9c8-0060b0ee281b";
    /// `ViewRefGDTGUID`: the graph node of a view referring to its GD&T.
    view_ref_gdt_guid => "efd213e7-6dfd-11d4-b9c8-0060b0ee281b";
    /// `ViewRefPlaneGUID`: the graph node of a view referring to its clipping planes.
    view_ref_plane_guid => "efd213e9-6dfd-11d4-b9c8-0060b0ee281b";
    /// `ViewRefNoteGUID`: the graph node of a view referring to its notes.
    view_ref_note_guid => "C814ACC6-43AC-4812-9B2A-4E9A2A549354";
    /// `ViewRefAnnotationGUID`: the graph node of a view referring to its annotations.
    view_ref_annotation_guid => "A2B5BA42-DD00-43f5-8882-4B5F8E76B9D2";
    /// `LockGUID`: the flag of a locked label.
    lock_guid => "efd213eb-6dfd-11d4-b9c8-0060b0ee281b";
}

impl XCAFDoc {
    /// `ColorRefGUID(type)`: the tree node of a shape referring to its color of that kind.
    ///
    /// ```
    /// use rustycad_tkxcaf::xcaf_doc::{XCAFDoc, XCAFDocColorType};
    /// assert_ne!(XCAFDoc::color_ref_guid(XCAFDocColorType::ColorGen),
    ///     XCAFDoc::color_ref_guid(XCAFDocColorType::ColorSurf));
    /// ```
    pub fn color_ref_guid(kind: XCAFDocColorType) -> Guid {
        const GENERIC: Guid = guid_from_text("efd212e4-6dfd-11d4-b9c8-0060b0ee281b");
        const SURFACE: Guid = guid_from_text("efd212e5-6dfd-11d4-b9c8-0060b0ee281b");
        const CURVE: Guid = guid_from_text("efd212e6-6dfd-11d4-b9c8-0060b0ee281b");
        match kind {
            XCAFDocColorType::ColorGen => GENERIC,
            XCAFDocColorType::ColorSurf => SURFACE,
            XCAFDocColorType::ColorCurv => CURVE,
        }
    }
}
