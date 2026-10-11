// Actual OpenGl_Text Subtitle formatter rectangle capture for Martensite; OCCT3d097a0.
#include "martensite_text_culling_probe.hxx"
#include "martensite_text_capture_camera.hxx"
#include "martensite_text_capture_image.hxx"
#include <Graphic3d_Group.hxx>
#include <Graphic3d_Structure.hxx>
#include <Graphic3d_Text.hxx>
#include <Image_PixMap.hxx>
#include <TopLoc_Datum3D.hxx>
#include <fstream>
#include <string>
#include "martensite_subtitle_occlusion.hxx"
static void subtitle_format(const occ::handle<Graphic3d_Text>& text, const std::string& mode) {
  text->SetText("AV"); text->SetPosition(gp_Pnt(0, 0, 0));
  if (mode == "subtitle-spaces") text->SetText("AV   ");
  if (mode == "subtitle-tabs") text->SetText("A\tV");
  if (mode == "subtitle-multiline") text->SetText("AV\nA ");
  if (mode == "subtitle-centered") text->SetHorizontalAlignment(Graphic3d_HTA_CENTER);
  if (mode == "subtitle-right" || mode == "subtitle-right-back-culled") text->SetHorizontalAlignment(Graphic3d_HTA_RIGHT);
  if (mode == "subtitle-top") text->SetVerticalAlignment(Graphic3d_VTA_TOP);
  if (mode == "subtitle-bottom") text->SetVerticalAlignment(Graphic3d_VTA_BOTTOM);
  if (mode == "subtitle-empty") text->SetText("");
  if (mode == "subtitle-whitespace") text->SetText(" \t ");
}
static void subtitle_style(const occ::handle<Graphic3d_AspectText3d>& style, const std::string& mode) {
  style->SetColorSubTitle(Quantity_ColorRGBA(Quantity_Color(Quantity_NOC_RED), 1.f));
  if (mode.find("translucent") != std::string::npos || mode.rfind("subtitle-zero-alpha", 0) == 0)
    style->SetColorSubTitle(Quantity_ColorRGBA(Quantity_Color(Quantity_NOC_RED), mode.rfind("subtitle-zero-alpha", 0) == 0 ? 0.f : 0.4f));
  if ((mode == "subtitle-annotation" || mode == "subtitle-annotation-depth")) style->SetStyle(Aspect_TOST_ANNOTATION);
  if (mode == "subtitle-opaque-translucent") style->SetAlphaMode(Graphic3d_AlphaMode_Opaque);
  if (mode == "subtitle-offset") style->SetPolygonOffsets(Aspect_POM_Fill, 2.f, 3.f);
  if (mode == "subtitle-zoomable") style->SetTextZoomable(true);
  if (mode == "subtitle-back-culled" || mode == "subtitle-right-back-culled" || mode == "subtitle-view-double")
    style->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_BackCulled);
  if (mode == "subtitle-front-culled") style->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_FrontCulled);
}
static void subtitle_plane(const occ::handle<Graphic3d_Text>& text, const std::string& mode) {
  if (mode.find("culled") == std::string::npos && mode != "subtitle-mirror" && mode != "subtitle-view-double") return;
  const int normal = mode == "subtitle-view-double" ? -1 : 1;
  text->SetOrientation(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, normal), gp_Dir(1, 0, 0)));
  text->SetOwnAnchorPoint(true);
}
static occ::handle<Graphic3d_Structure> subtitle_structure(const occ::handle<V3d_View>& view, const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure = new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group = structure->NewGroup();
  const occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  const occ::handle<Graphic3d_AspectText3d> style = new Graphic3d_AspectText3d(Quantity_NOC_WHITE, "DejaVu Sans", 1, 0, Aspect_TOST_NORMAL, Aspect_TODT_SUBTITLE);
  subtitle_format(text, mode); subtitle_style(style, mode); subtitle_plane(text, mode);
  group->SetPrimitivesAspect(style); group->AddText(text);
  if (mode == "subtitle-mirror") {
    gp_Trsf mirror; mirror.SetMirror(gp_Pnt(0, 0, 0));
    structure->SetTransformation(new TopLoc_Datum3D(mirror));
  }
  append_native_subtitle_occluder(group, view, mode);
  structure->Display();
  return structure;
}
int main(int argc, char** argv) {
  if (argc != 3 && argc != 4) throw Standard_Failure("native Subtitle arguments; expected mode prefix [rgba|depth]");
  auto probe = create_policy_probe(); prepare_native_text_capture_view(probe.view);
  const std::string mode = argv[1];
  if (mode == "subtitle-resolution") probe.view->ChangeRenderingParams().Resolution = 144;
  if (mode == "subtitle-view-double") probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  const auto structure = subtitle_structure(probe.view, mode);
  if (argc == 3) capture_native_text_view(probe.view, argv[2]);
  else if (std::string(argv[3]) == "rgba") capture_native_text_buffer(probe.view, argv[2], Graphic3d_BT_RGBA);
  else if (std::string(argv[3]) == "depth") capture_native_text_buffer(probe.view, argv[2], Graphic3d_BT_Depth);
  else throw Standard_Failure("native Subtitle buffer; expected rgba or depth");
  probe.view->Remove();
}
