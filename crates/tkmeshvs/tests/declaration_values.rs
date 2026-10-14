// SPDX-License-Identifier: GPL-2.0-only
//! All values replay pinned native declarations; OCCT is never invoked by Rust tests.
use martensite_tkmeshvs::flags::*;
const PINNED_DECLARATION_VALUES: &[(&str, i32)] = &[
    (
        "MeshVS_DA_InteriorStyle",
        MeshVSDrawerAttribute::InteriorStyle as i32,
    ),
    (
        "MeshVS_DA_InteriorColor",
        MeshVSDrawerAttribute::InteriorColor as i32,
    ),
    (
        "MeshVS_DA_BackInteriorColor",
        MeshVSDrawerAttribute::BackInteriorColor as i32,
    ),
    (
        "MeshVS_DA_EdgeColor",
        MeshVSDrawerAttribute::EdgeColor as i32,
    ),
    ("MeshVS_DA_EdgeType", MeshVSDrawerAttribute::EdgeType as i32),
    (
        "MeshVS_DA_EdgeWidth",
        MeshVSDrawerAttribute::EdgeWidth as i32,
    ),
    (
        "MeshVS_DA_HatchStyle",
        MeshVSDrawerAttribute::HatchStyle as i32,
    ),
    (
        "MeshVS_DA_FrontMaterial",
        MeshVSDrawerAttribute::FrontMaterial as i32,
    ),
    (
        "MeshVS_DA_BackMaterial",
        MeshVSDrawerAttribute::BackMaterial as i32,
    ),
    ("MeshVS_DA_BeamType", MeshVSDrawerAttribute::BeamType as i32),
    (
        "MeshVS_DA_BeamWidth",
        MeshVSDrawerAttribute::BeamWidth as i32,
    ),
    (
        "MeshVS_DA_BeamColor",
        MeshVSDrawerAttribute::BeamColor as i32,
    ),
    (
        "MeshVS_DA_MarkerType",
        MeshVSDrawerAttribute::MarkerType as i32,
    ),
    (
        "MeshVS_DA_MarkerColor",
        MeshVSDrawerAttribute::MarkerColor as i32,
    ),
    (
        "MeshVS_DA_MarkerScale",
        MeshVSDrawerAttribute::MarkerScale as i32,
    ),
    (
        "MeshVS_DA_TextColor",
        MeshVSDrawerAttribute::TextColor as i32,
    ),
    (
        "MeshVS_DA_TextHeight",
        MeshVSDrawerAttribute::TextHeight as i32,
    ),
    ("MeshVS_DA_TextFont", MeshVSDrawerAttribute::TextFont as i32),
    (
        "MeshVS_DA_TextExpansionFactor",
        MeshVSDrawerAttribute::TextExpansionFactor as i32,
    ),
    (
        "MeshVS_DA_TextSpace",
        MeshVSDrawerAttribute::TextSpace as i32,
    ),
    (
        "MeshVS_DA_TextStyle",
        MeshVSDrawerAttribute::TextStyle as i32,
    ),
    (
        "MeshVS_DA_TextDisplayType",
        MeshVSDrawerAttribute::TextDisplayType as i32,
    ),
    (
        "MeshVS_DA_TextTexFont",
        MeshVSDrawerAttribute::TextTexFont as i32,
    ),
    (
        "MeshVS_DA_TextFontAspect",
        MeshVSDrawerAttribute::TextFontAspect as i32,
    ),
    (
        "MeshVS_DA_VectorColor",
        MeshVSDrawerAttribute::VectorColor as i32,
    ),
    (
        "MeshVS_DA_VectorMaxLength",
        MeshVSDrawerAttribute::VectorMaxLength as i32,
    ),
    (
        "MeshVS_DA_VectorArrowPart",
        MeshVSDrawerAttribute::VectorArrowPart as i32,
    ),
    (
        "MeshVS_DA_IsAllowOverlapped",
        MeshVSDrawerAttribute::IsAllowOverlapped as i32,
    ),
    (
        "MeshVS_DA_Reflection",
        MeshVSDrawerAttribute::Reflection as i32,
    ),
    (
        "MeshVS_DA_ColorReflection",
        MeshVSDrawerAttribute::ColorReflection as i32,
    ),
    (
        "MeshVS_DA_ShrinkCoeff",
        MeshVSDrawerAttribute::ShrinkCoeff as i32,
    ),
    (
        "MeshVS_DA_MaxFaceNodes",
        MeshVSDrawerAttribute::MaxFaceNodes as i32,
    ),
    (
        "MeshVS_DA_ComputeTime",
        MeshVSDrawerAttribute::ComputeTime as i32,
    ),
    (
        "MeshVS_DA_ComputeSelectionTime",
        MeshVSDrawerAttribute::ComputeSelectionTime as i32,
    ),
    (
        "MeshVS_DA_DisplayNodes",
        MeshVSDrawerAttribute::DisplayNodes as i32,
    ),
    (
        "MeshVS_DA_SelectableAuto",
        MeshVSDrawerAttribute::SelectableAuto as i32,
    ),
    (
        "MeshVS_DA_ShowEdges",
        MeshVSDrawerAttribute::ShowEdges as i32,
    ),
    (
        "MeshVS_DA_SmoothShading",
        MeshVSDrawerAttribute::SmoothShading as i32,
    ),
    (
        "MeshVS_DA_SupressBackFaces",
        MeshVSDrawerAttribute::SupressBackFaces as i32,
    ),
    ("MeshVS_DA_User", MeshVSDrawerAttribute::User as i32),
    ("MeshVS_DMF_WireFrame", MESHVS_DMF_WIRE_FRAME),
    ("MeshVS_DMF_Shading", MESHVS_DMF_SHADING),
    ("MeshVS_DMF_Shrink", MESHVS_DMF_SHRINK),
    ("MeshVS_DMF_OCCMask", MESHVS_DMF_OCC_MASK),
    ("MeshVS_DMF_VectorDataPrs", MESHVS_DMF_VECTOR_DATA_PRS),
    (
        "MeshVS_DMF_NodalColorDataPrs",
        MESHVS_DMF_NODAL_COLOR_DATA_PRS,
    ),
    (
        "MeshVS_DMF_ElementalColorDataPrs",
        MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS,
    ),
    ("MeshVS_DMF_TextDataPrs", MESHVS_DMF_TEXT_DATA_PRS),
    ("MeshVS_DMF_EntitiesWithData", MESHVS_DMF_ENTITIES_WITH_DATA),
    (
        "MeshVS_DMF_DeformedPrsWireFrame",
        MESHVS_DMF_DEFORMED_PRS_WIRE_FRAME,
    ),
    (
        "MeshVS_DMF_DeformedPrsShading",
        MESHVS_DMF_DEFORMED_PRS_SHADING,
    ),
    (
        "MeshVS_DMF_DeformedPrsShrink",
        MESHVS_DMF_DEFORMED_PRS_SHRINK,
    ),
    ("MeshVS_DMF_DeformedMask", MESHVS_DMF_DEFORMED_MASK),
    ("MeshVS_DMF_SelectionPrs", MESHVS_DMF_SELECTION_PRS),
    ("MeshVS_DMF_HilightPrs", MESHVS_DMF_HILIGHT_PRS),
    ("MeshVS_DMF_User", MESHVS_DMF_USER),
    ("MeshVS_ET_NONE", MESHVS_ET_NONE.0),
    ("MeshVS_ET_Node", MESHVS_ET_NODE.0),
    ("MeshVS_ET_0D", MESHVS_ET_0D.0),
    ("MeshVS_ET_Link", MESHVS_ET_LINK.0),
    ("MeshVS_ET_Face", MESHVS_ET_FACE.0),
    ("MeshVS_ET_Volume", MESHVS_ET_VOLUME.0),
    ("MeshVS_ET_Element", MESHVS_ET_ELEMENT.0),
    ("MeshVS_ET_All", MESHVS_ET_ALL.0),
];
#[test]
fn every_exported_meshvs_value_matches_pinned_native_declarations() {
    let rows = PINNED_DECLARATION_VALUES
        .iter()
        .map(|(name, value)| format!("{name} {value}\n"))
        .collect::<String>();
    assert_eq!(PINNED_DECLARATION_VALUES.len(), 64);
    assert_eq!(rows, include_str!("fixtures/declaration-values-native.txt"));
}
