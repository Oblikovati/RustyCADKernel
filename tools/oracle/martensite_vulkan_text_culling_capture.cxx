// Real native glyph culling capture for Martensite; OCCT 3d097a0.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include <Graphic3d_Group.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_Structure.hxx>
#include <Graphic3d_Text.hxx>
#include <Image_PixMap.hxx>
#include <TopLoc_Datum3D.hxx>
#include <fstream>
#include <string>
static void culling_orientation(const occ::handle<Graphic3d_Text>& text, const std::string& mode) {
  if (mode.rfind("billboard", 0) == 0) return;
  const bool back = mode.rfind("back", 0) == 0 || mode.find("-back-") != std::string::npos
    || mode.rfind("auto", 0) == 0 || mode == "view-double-back" || mode == "view-back-double";
  text->SetOrientation(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, back ? -1 : 1), gp_Dir(1, 0, 0)));
  text->SetOwnAnchorPoint(mode.rfind("own", 0) == 0);
}
static Graphic3d_TypeOfBackfacingModel culling_mode(const std::string& mode) {
  if (mode.size() >= 5 && mode.substr(mode.size() - 5) == "front") return Graphic3d_TypeOfBackfacingModel_FrontCulled;
  if (mode.size() >= 6 && mode.substr(mode.size() - 6) == "double") return Graphic3d_TypeOfBackfacingModel_DoubleSided;
  if (mode.rfind("auto", 0) == 0) return Graphic3d_TypeOfBackfacingModel_Auto;
  return Graphic3d_TypeOfBackfacingModel_BackCulled;
}
static void culling_style(const occ::handle<Graphic3d_AspectText3d>& style, const std::string& mode) {
  style->SetFaceCulling(culling_mode(mode));
  const auto alpha = mode.find("opaque") != std::string::npos ? Graphic3d_AlphaMode_Opaque
    : mode.find("mask-back") != std::string::npos ? Graphic3d_AlphaMode_Mask : Graphic3d_AlphaMode_MaskBlend;
  style->SetAlphaMode(alpha, 0.285f);
}
// Displaying afterward refreshes bounds; retain the upstream empty displayed-group lifecycle separately.
static void displayed_empty_group(const occ::handle<Graphic3d_Structure>& structure,
                                 const occ::handle<Graphic3d_Group>& group) {
  const occ::handle<Graphic3d_ArrayOfTriangles> triangle = new Graphic3d_ArrayOfTriangles(3, 3);
  triangle->AddVertex(-8.f, -8.f, 0.f); triangle->AddVertex(8.f, -8.f, 0.f); triangle->AddVertex(0.f, 8.f, 0.f);
  triangle->AddEdge(1); triangle->AddEdge(2); triangle->AddEdge(3);
  group->SetGroupPrimitivesAspect(new Graphic3d_AspectFillArea3d);
  group->AddPrimitiveArray(triangle);
  structure->Display(); group->Clear(true);
}
static void culling_mirror(const occ::handle<Graphic3d_Structure>& structure) {
  gp_Trsf mirror; mirror.SetMirror(gp_Pnt(0, 0, 0));
  structure->SetTransformation(new TopLoc_Datum3D(mirror));
}
static occ::handle<Graphic3d_Structure> culling_scene(const occ::handle<V3d_View>& view, const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure = new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group = structure->NewGroup();
  const occ::handle<Graphic3d_AspectText3d> style = new Graphic3d_AspectText3d(Quantity_NOC_WHITE, "DejaVu Sans", 1, 0, Aspect_TOST_NORMAL, Aspect_TODT_NORMAL);
  const occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  text->SetText("AV"); text->SetPosition(gp_Pnt(0, 0, 0));
  text->SetHorizontalAlignment(Graphic3d_HTA_CENTER); text->SetVerticalAlignment(Graphic3d_VTA_CENTER);
  const bool liveBefore = mode == "live-mirror-before-back";
  const bool freshBefore = mode == "fresh-mirror-before-back";
  culling_orientation(text, (liveBefore || freshBefore) ? "back-back" : mode); culling_style(style, mode);
  if (liveBefore) displayed_empty_group(structure, group);
  if (liveBefore || freshBefore) culling_mirror(structure);
  group->SetClosed(mode.rfind("auto-closed", 0) == 0);
  group->SetPrimitivesAspect(style); group->AddText(text);
  if (mode.rfind("mirror", 0) == 0) culling_mirror(structure);
  if (!liveBefore) structure->Display();
  return structure;
}
int main(int argc, char** argv) {
  if (argc != 3) throw Standard_Failure("native culling arguments; expected mode prefix");
  auto probe = create_policy_probe();
  prepare_native_text_capture_view(probe.view);
  const std::string mode = argv[1];
  if (mode == "view-double-back") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  if (mode == "view-back-double") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_BackCulled);
  if (mode == "view-front-double") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_FrontCulled);
  const auto structure = culling_scene(probe.view, mode);
  capture_native_text_view(probe.view, argv[2]);
  probe.view->Remove();
}
