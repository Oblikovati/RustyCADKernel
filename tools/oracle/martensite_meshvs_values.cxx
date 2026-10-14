// OCCT pinned 3d097a0: every exported MeshVS declaration value.
#include <MeshVS_DrawerAttribute.hxx>
#include <MeshVS_DisplayModeFlags.hxx>
#include <MeshVS_EntityType.hxx>
#include <iostream>
static void DrawerAttributesFirst() {
  std::cout << "MeshVS_DA_InteriorStyle " << int(MeshVS_DA_InteriorStyle) << "\n";
  std::cout << "MeshVS_DA_InteriorColor " << int(MeshVS_DA_InteriorColor) << "\n";
  std::cout << "MeshVS_DA_BackInteriorColor " << int(MeshVS_DA_BackInteriorColor) << "\n";
  std::cout << "MeshVS_DA_EdgeColor " << int(MeshVS_DA_EdgeColor) << "\n";
  std::cout << "MeshVS_DA_EdgeType " << int(MeshVS_DA_EdgeType) << "\n";
  std::cout << "MeshVS_DA_EdgeWidth " << int(MeshVS_DA_EdgeWidth) << "\n";
  std::cout << "MeshVS_DA_HatchStyle " << int(MeshVS_DA_HatchStyle) << "\n";
  std::cout << "MeshVS_DA_FrontMaterial " << int(MeshVS_DA_FrontMaterial) << "\n";
  std::cout << "MeshVS_DA_BackMaterial " << int(MeshVS_DA_BackMaterial) << "\n";
  std::cout << "MeshVS_DA_BeamType " << int(MeshVS_DA_BeamType) << "\n";
  std::cout << "MeshVS_DA_BeamWidth " << int(MeshVS_DA_BeamWidth) << "\n";
  std::cout << "MeshVS_DA_BeamColor " << int(MeshVS_DA_BeamColor) << "\n";
  std::cout << "MeshVS_DA_MarkerType " << int(MeshVS_DA_MarkerType) << "\n";
  std::cout << "MeshVS_DA_MarkerColor " << int(MeshVS_DA_MarkerColor) << "\n";
  std::cout << "MeshVS_DA_MarkerScale " << int(MeshVS_DA_MarkerScale) << "\n";
  std::cout << "MeshVS_DA_TextColor " << int(MeshVS_DA_TextColor) << "\n";
}
static void DrawerAttributesSecond() {
  std::cout << "MeshVS_DA_TextHeight " << int(MeshVS_DA_TextHeight) << "\n";
  std::cout << "MeshVS_DA_TextFont " << int(MeshVS_DA_TextFont) << "\n";
  std::cout << "MeshVS_DA_TextExpansionFactor " << int(MeshVS_DA_TextExpansionFactor) << "\n";
  std::cout << "MeshVS_DA_TextSpace " << int(MeshVS_DA_TextSpace) << "\n";
  std::cout << "MeshVS_DA_TextStyle " << int(MeshVS_DA_TextStyle) << "\n";
  std::cout << "MeshVS_DA_TextDisplayType " << int(MeshVS_DA_TextDisplayType) << "\n";
  std::cout << "MeshVS_DA_TextTexFont " << int(MeshVS_DA_TextTexFont) << "\n";
  std::cout << "MeshVS_DA_TextFontAspect " << int(MeshVS_DA_TextFontAspect) << "\n";
  std::cout << "MeshVS_DA_VectorColor " << int(MeshVS_DA_VectorColor) << "\n";
  std::cout << "MeshVS_DA_VectorMaxLength " << int(MeshVS_DA_VectorMaxLength) << "\n";
  std::cout << "MeshVS_DA_VectorArrowPart " << int(MeshVS_DA_VectorArrowPart) << "\n";
  std::cout << "MeshVS_DA_IsAllowOverlapped " << int(MeshVS_DA_IsAllowOverlapped) << "\n";
  std::cout << "MeshVS_DA_Reflection " << int(MeshVS_DA_Reflection) << "\n";
  std::cout << "MeshVS_DA_ColorReflection " << int(MeshVS_DA_ColorReflection) << "\n";
  std::cout << "MeshVS_DA_ShrinkCoeff " << int(MeshVS_DA_ShrinkCoeff) << "\n";
  std::cout << "MeshVS_DA_MaxFaceNodes " << int(MeshVS_DA_MaxFaceNodes) << "\n";
}
static void DrawerAttributesLast() {
  std::cout << "MeshVS_DA_ComputeTime " << int(MeshVS_DA_ComputeTime) << "\n";
  std::cout << "MeshVS_DA_ComputeSelectionTime " << int(MeshVS_DA_ComputeSelectionTime) << "\n";
  std::cout << "MeshVS_DA_DisplayNodes " << int(MeshVS_DA_DisplayNodes) << "\n";
  std::cout << "MeshVS_DA_SelectableAuto " << int(MeshVS_DA_SelectableAuto) << "\n";
  std::cout << "MeshVS_DA_ShowEdges " << int(MeshVS_DA_ShowEdges) << "\n";
  std::cout << "MeshVS_DA_SmoothShading " << int(MeshVS_DA_SmoothShading) << "\n";
  std::cout << "MeshVS_DA_SupressBackFaces " << int(MeshVS_DA_SupressBackFaces) << "\n";
  std::cout << "MeshVS_DA_User " << int(MeshVS_DA_User) << "\n";
}
static void DisplayMasks() {
  std::cout << "MeshVS_DMF_WireFrame " << int(MeshVS_DMF_WireFrame) << "\n";
  std::cout << "MeshVS_DMF_Shading " << int(MeshVS_DMF_Shading) << "\n";
  std::cout << "MeshVS_DMF_Shrink " << int(MeshVS_DMF_Shrink) << "\n";
  std::cout << "MeshVS_DMF_OCCMask " << int(MeshVS_DMF_OCCMask) << "\n";
  std::cout << "MeshVS_DMF_VectorDataPrs " << int(MeshVS_DMF_VectorDataPrs) << "\n";
  std::cout << "MeshVS_DMF_NodalColorDataPrs " << int(MeshVS_DMF_NodalColorDataPrs) << "\n";
  std::cout << "MeshVS_DMF_ElementalColorDataPrs " << int(MeshVS_DMF_ElementalColorDataPrs) << "\n";
  std::cout << "MeshVS_DMF_TextDataPrs " << int(MeshVS_DMF_TextDataPrs) << "\n";
  std::cout << "MeshVS_DMF_EntitiesWithData " << int(MeshVS_DMF_EntitiesWithData) << "\n";
  std::cout << "MeshVS_DMF_DeformedPrsWireFrame " << int(MeshVS_DMF_DeformedPrsWireFrame) << "\n";
  std::cout << "MeshVS_DMF_DeformedPrsShading " << int(MeshVS_DMF_DeformedPrsShading) << "\n";
  std::cout << "MeshVS_DMF_DeformedPrsShrink " << int(MeshVS_DMF_DeformedPrsShrink) << "\n";
  std::cout << "MeshVS_DMF_DeformedMask " << int(MeshVS_DMF_DeformedMask) << "\n";
  std::cout << "MeshVS_DMF_SelectionPrs " << int(MeshVS_DMF_SelectionPrs) << "\n";
  std::cout << "MeshVS_DMF_HilightPrs " << int(MeshVS_DMF_HilightPrs) << "\n";
  std::cout << "MeshVS_DMF_User " << int(MeshVS_DMF_User) << "\n";
}
static void EntityMasks() {
  std::cout << "MeshVS_ET_NONE " << int(MeshVS_ET_NONE) << "\n";
  std::cout << "MeshVS_ET_Node " << int(MeshVS_ET_Node) << "\n";
  std::cout << "MeshVS_ET_0D " << int(MeshVS_ET_0D) << "\n";
  std::cout << "MeshVS_ET_Link " << int(MeshVS_ET_Link) << "\n";
  std::cout << "MeshVS_ET_Face " << int(MeshVS_ET_Face) << "\n";
  std::cout << "MeshVS_ET_Volume " << int(MeshVS_ET_Volume) << "\n";
  std::cout << "MeshVS_ET_Element " << int(MeshVS_ET_Element) << "\n";
  std::cout << "MeshVS_ET_All " << int(MeshVS_ET_All) << "\n";
}
int main() {
  DrawerAttributesFirst();
  DrawerAttributesSecond();
  DrawerAttributesLast();
  DisplayMasks();
  EntityMasks();
}
