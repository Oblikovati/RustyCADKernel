// SPDX-License-Identifier: GPL-2.0-only
//! Native MeshVS keys and bit masks at OCCT 3d097a0; arbitrary integer drawer keys stay valid.
/// Native display-mode mask, including combinations; `let mode:MeshVSDisplayModeFlags=MESHVS_DMF_SHADING`.
pub type MeshVSDisplayModeFlags = i32;
/// Mesh entity mask admits native unnamed combinations; `MeshVSEntityType(0x09)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct MeshVSEntityType(pub i32);
/// Native appearance attribute key; `MeshVSDrawerAttribute::InteriorColor as i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MeshVSDrawerAttribute {
    /// Native `MeshVS_DA_InteriorStyle` appearance key.
    InteriorStyle = 0,
    /// Native `MeshVS_DA_InteriorColor` appearance key.
    InteriorColor = 1,
    /// Native `MeshVS_DA_BackInteriorColor` appearance key.
    BackInteriorColor = 2,
    /// Native `MeshVS_DA_EdgeColor` appearance key.
    EdgeColor = 3,
    /// Native `MeshVS_DA_EdgeType` appearance key.
    EdgeType = 4,
    /// Native `MeshVS_DA_EdgeWidth` appearance key.
    EdgeWidth = 5,
    /// Native `MeshVS_DA_HatchStyle` appearance key.
    HatchStyle = 6,
    /// Native `MeshVS_DA_FrontMaterial` appearance key.
    FrontMaterial = 7,
    /// Native `MeshVS_DA_BackMaterial` appearance key.
    BackMaterial = 8,
    /// Native `MeshVS_DA_BeamType` appearance key.
    BeamType = 9,
    /// Native `MeshVS_DA_BeamWidth` appearance key.
    BeamWidth = 10,
    /// Native `MeshVS_DA_BeamColor` appearance key.
    BeamColor = 11,
    /// Native `MeshVS_DA_MarkerType` appearance key.
    MarkerType = 12,
    /// Native `MeshVS_DA_MarkerColor` appearance key.
    MarkerColor = 13,
    /// Native `MeshVS_DA_MarkerScale` appearance key.
    MarkerScale = 14,
    /// Native `MeshVS_DA_TextColor` appearance key.
    TextColor = 15,
    /// Native `MeshVS_DA_TextHeight` appearance key.
    TextHeight = 16,
    /// Native `MeshVS_DA_TextFont` appearance key.
    TextFont = 17,
    /// Native `MeshVS_DA_TextExpansionFactor` appearance key.
    TextExpansionFactor = 18,
    /// Native `MeshVS_DA_TextSpace` appearance key.
    TextSpace = 19,
    /// Native `MeshVS_DA_TextStyle` appearance key.
    TextStyle = 20,
    /// Native `MeshVS_DA_TextDisplayType` appearance key.
    TextDisplayType = 21,
    /// Native `MeshVS_DA_TextTexFont` appearance key.
    TextTexFont = 22,
    /// Native `MeshVS_DA_TextFontAspect` appearance key.
    TextFontAspect = 23,
    /// Native `MeshVS_DA_VectorColor` appearance key.
    VectorColor = 24,
    /// Native `MeshVS_DA_VectorMaxLength` appearance key.
    VectorMaxLength = 25,
    /// Native `MeshVS_DA_VectorArrowPart` appearance key.
    VectorArrowPart = 26,
    /// Native `MeshVS_DA_IsAllowOverlapped` appearance key.
    IsAllowOverlapped = 27,
    /// Native `MeshVS_DA_Reflection` appearance key.
    Reflection = 28,
    /// Native `MeshVS_DA_ColorReflection` appearance key.
    ColorReflection = 29,
    /// Native `MeshVS_DA_ShrinkCoeff` appearance key.
    ShrinkCoeff = 30,
    /// Native `MeshVS_DA_MaxFaceNodes` appearance key.
    MaxFaceNodes = 31,
    /// Native `MeshVS_DA_ComputeTime` appearance key.
    ComputeTime = 32,
    /// Native `MeshVS_DA_ComputeSelectionTime` appearance key.
    ComputeSelectionTime = 33,
    /// Native `MeshVS_DA_DisplayNodes` appearance key.
    DisplayNodes = 34,
    /// Native `MeshVS_DA_SelectableAuto` appearance key.
    SelectableAuto = 35,
    /// Native `MeshVS_DA_ShowEdges` appearance key.
    ShowEdges = 36,
    /// Native `MeshVS_DA_SmoothShading` appearance key.
    SmoothShading = 37,
    /// Native `MeshVS_DA_SupressBackFaces` appearance key.
    SupressBackFaces = 38,
    /// Native `MeshVS_DA_User` appearance key.
    User = 39,
}
// Native TextExpansionFactor/TextSpace/TextDisplayType have no effect; retain their keys.
/// Native `WireFrame` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_WIRE_FRAME: MeshVSDisplayModeFlags = 0x0001;
/// Native `Shading` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_SHADING: MeshVSDisplayModeFlags = 0x0002;
/// Native `Shrink` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_SHRINK: MeshVSDisplayModeFlags = 0x0003;
/// Native `VectorDataPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_VECTOR_DATA_PRS: MeshVSDisplayModeFlags = 0x0004;
/// Native `NodalColorDataPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_NODAL_COLOR_DATA_PRS: MeshVSDisplayModeFlags = 0x0008;
/// Native `ElementalColorDataPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS: MeshVSDisplayModeFlags = 0x0010;
/// Native `TextDataPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_TEXT_DATA_PRS: MeshVSDisplayModeFlags = 0x0020;
/// Native `EntitiesWithData` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_ENTITIES_WITH_DATA: MeshVSDisplayModeFlags = 0x0040;
/// Native `DeformedPrsWireFrame` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_DEFORMED_PRS_WIRE_FRAME: MeshVSDisplayModeFlags = 0x0080;
/// Native `DeformedPrsShading` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_DEFORMED_PRS_SHADING: MeshVSDisplayModeFlags = 0x0100;
/// Native `DeformedPrsShrink` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_DEFORMED_PRS_SHRINK: MeshVSDisplayModeFlags = 0x0180;
/// Native `SelectionPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_SELECTION_PRS: MeshVSDisplayModeFlags = 0x0200;
/// Native `HilightPrs` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_HILIGHT_PRS: MeshVSDisplayModeFlags = 0x0400;
/// Native `User` display mode; combine integer masks as upstream.
pub const MESHVS_DMF_USER: MeshVSDisplayModeFlags = 0x0800;
/// Mask selecting native Cascade modes; `mode & MESHVS_DMF_OCC_MASK`.
pub const MESHVS_DMF_OCC_MASK: MeshVSDisplayModeFlags =
    MESHVS_DMF_WIRE_FRAME | MESHVS_DMF_SHADING | MESHVS_DMF_SHRINK;
/// Mask selecting native deformation modes; `mode & MESHVS_DMF_DEFORMED_MASK`.
pub const MESHVS_DMF_DEFORMED_MASK: MeshVSDisplayModeFlags = MESHVS_DMF_DEFORMED_PRS_WIRE_FRAME
    | MESHVS_DMF_DEFORMED_PRS_SHADING
    | MESHVS_DMF_DEFORMED_PRS_SHRINK;
/// Native entity mask `NONE`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_NONE: MeshVSEntityType = MeshVSEntityType(0);
/// Native entity mask `NODE`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_NODE: MeshVSEntityType = MeshVSEntityType(1);
/// Native entity mask `0D`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_0D: MeshVSEntityType = MeshVSEntityType(2);
/// Native entity mask `LINK`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_LINK: MeshVSEntityType = MeshVSEntityType(4);
/// Native entity mask `FACE`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_FACE: MeshVSEntityType = MeshVSEntityType(8);
/// Native entity mask `VOLUME`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_VOLUME: MeshVSEntityType = MeshVSEntityType(16);
/// Native entity mask `ELEMENT`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_ELEMENT: MeshVSEntityType = MeshVSEntityType(30);
/// Native entity mask `ALL`; arbitrary unnamed combinations remain representable.
pub const MESHVS_ET_ALL: MeshVSEntityType = MeshVSEntityType(31);
