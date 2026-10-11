// Later geometry checks actual native Subtitle depth write behavior, including alpha0 and annotation.
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
static void append_native_subtitle_occluder(const occ::handle<Graphic3d_Group>& group,
                                           const occ::handle<V3d_View>& view, const std::string& mode) {
  if (mode.find("-depth") == std::string::npos) return;
  const occ::handle<Graphic3d_ArrayOfTriangles> triangles = new Graphic3d_ArrayOfTriangles(6);
  const float square[6][3] = {{-20,-20,-1},{20,-20,-1},{20,20,-1},{-20,-20,-1},{20,20,-1},{-20,20,-1}};
  for (const auto& point : square) triangles->AddVertex(point[0], point[1], point[2]);
  const occ::handle<Graphic3d_AspectFillArea3d> green = new Graphic3d_AspectFillArea3d;
  green->SetInteriorStyle(Aspect_IS_SOLID);
  green->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  green->SetInteriorColor(Quantity_NOC_GREEN); green->SetPolygonOffsets(Aspect_POM_Off, 0, 0);
  group->SetPrimitivesAspect(green); group->AddPrimitiveArray(triangles);
  if (mode == "subtitle-zero-alpha-no-depth-write") {
    auto settings = view->Viewer()->ZLayerSettings(Graphic3d_ZLayerId_Default);
    settings.SetEnableDepthWrite(false);
    view->Viewer()->SetZLayerSettings(Graphic3d_ZLayerId_Default, settings);
  }
}
