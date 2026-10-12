#include <Graphic3d_PresentationAttributes.hxx>
// Real native XOR/ordinary sequence; distinct aspects preserve per-draw colors.
static occ::handle<Graphic3d_AspectText3d> native_blend_style(Aspect_TypeOfDisplayText display,
                                                           float red, float green, float blue, float alpha) {
  const auto style = new Graphic3d_AspectText3d(Quantity_NOC_WHITE, "DejaVu Sans", 1, 0, Aspect_TOST_NORMAL, display);
  style->SetColor(Quantity_ColorRGBA(Quantity_Color(red, green, blue, Quantity_TOC_RGB), alpha));
  return style;
}
static void native_blend_extras(const occ::handle<Graphic3d_Group>& group,
                               const occ::handle<Graphic3d_Text>& text, const std::string& mode) {
  if (mode == "blend-double") group->AddText(text);
  if (mode == "blend-colored-overlap") {
    group->SetPrimitivesAspect(native_blend_style(Aspect_TODT_BLEND, 0.7f, 0.2f, 0.4f, 1.f)); group->AddText(text);
  }
  if (mode == "blend-normal-after") {
    group->SetPrimitivesAspect(native_blend_style(Aspect_TODT_NORMAL, 0.18f, 0.4f, 0.7f, 1.f)); group->AddText(text);
  }
}
static void native_blend_highlight(const occ::handle<Graphic3d_Structure>& structure,
                                    const std::string& mode) {
  if (mode != "blend-highlight") return;
  const occ::handle<Graphic3d_PresentationAttributes> highlight = new Graphic3d_PresentationAttributes;
  highlight->SetColor(Quantity_Color(0.18, 0.4, 0.7, Quantity_TOC_RGB)); structure->Highlight(highlight);
}
static occ::handle<Graphic3d_Structure> native_blend_fixture(const occ::handle<V3d_View>& view,
                                                           const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure = new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group = structure->NewGroup();
  const occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  subtitle_format(text, "subtitle-basic");
  const bool colored = mode == "blend-colored" || mode == "blend-colored-alpha" || mode == "blend-colored-overlap" || mode == "blend-palette";
  const float alpha = mode == "blend-alpha-zero" ? 0.f : mode == "blend-colored-alpha" ? 0.4f : 1.f;
  group->SetPrimitivesAspect(native_blend_style(Aspect_TODT_BLEND, colored ? 0.18f : 1.f, colored ? 0.4f : 1.f, colored ? 0.7f : 1.f, alpha));
  group->AddText(text); native_blend_extras(group, text, mode);
  if (mode == "blend-palette") view->SetBackgroundColor(Quantity_Color(0.03, 0.2, 0.5, Quantity_TOC_RGB));
  structure->Display();
  native_blend_highlight(structure, mode);
  return structure;
}
