// Actual OpenGl_Text::Set2D oracle; Graphic3d_Group::AddText never sets this native flag.
#include <OpenGl_Group.hxx>
#include <OpenGl_Text.hxx>
struct NativePixelTextFixture {
  occ::handle<Graphic3d_Structure> structure;
  occ::handle<Graphic3d_Text> parameters;
  occ::handle<Graphic3d_AspectText3d> style;
  OpenGl_Text* element;
};
static NativePixelTextFixture native_pixel_text_fixture(const occ::handle<V3d_View>& view,
                                                       const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure = new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group = occ::handle<OpenGl_Group>::DownCast(structure->NewGroup());
  const occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  const occ::handle<Graphic3d_AspectText3d> style = new Graphic3d_AspectText3d(Quantity_NOC_WHITE, "DejaVu Sans", 1, 0, Aspect_TOST_NORMAL, Aspect_TODT_BLEND);
  subtitle_format(text, mode); subtitle_style(style, mode); subtitle_plane(text, mode);
  text->SetPosition(gp_Pnt(-20, 20, 5));
  auto* element = new OpenGl_Text(text); element->Set2D(true);
  group->SetPrimitivesAspect(style); group->AddElement(element);
  group->SetMinMaxValues(-20, 20, 5, -20, 20, 5);
  native_fixture_placement(structure, mode);
  if (mode == "subtitle-mirror") { gp_Trsf mirror; mirror.SetMirror(gp_Pnt(0, 0, 0)); structure->SetTransformation(new TopLoc_Datum3D(mirror)); }
  structure->Display();
  return {structure, text, style, element};
}
static void configure_pixel_reuse(NativePixelTextFixture& fixture,
                                  const occ::handle<OpenGl_Context>& context, bool reset) {
  fixture.parameters->SetText("V");
  fixture.parameters->SetPosition(gp_Pnt(12, -8, 99));
  fixture.parameters->SetHorizontalAlignment(Graphic3d_HTA_RIGHT);
  // OpenGl_Text retains its original glyph VBOs until Reset (OCCT3d097a0).
  if (reset) fixture.element->Reset(context);
  fixture.style->SetColor(Quantity_Color(0.18, 0.4, 0.7, Quantity_TOC_RGB));
}

static void prepare_native_pixel_view(const occ::handle<V3d_View>& view, const std::string& mode) {
  view->Camera()->SetScale(128); view->Camera()->SetZRange(-100, 100);
  if (mode == "subtitle-resolution") view->ChangeRenderingParams().Resolution = 144;
  if (mode == "subtitle-view-double") view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_DoubleSided);
}
