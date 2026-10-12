// Genuine parent width propagation and linked-default restoration at OCCT 3d097a0.
#include "martensite_textured_shape_state.hxx"
#include <Prs3d_Drawer.hxx>
#include <Prs3d_LineAspect.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Standard_Failure.hxx>
#include <array>
static std::array<occ::handle<Prs3d_LineAspect>,6> width_lines(const occ::handle<Prs3d_Drawer>& drawer) {
  return {drawer->LineAspect(), drawer->WireAspect(), drawer->FreeBoundaryAspect(),
    drawer->UnFreeBoundaryAspect(), drawer->SeenLineAspect(), drawer->FaceBoundaryAspect()};
}
static occ::handle<NativeTexturedState> width_object(bool linked) {
  const occ::handle<NativeTexturedState> object = new NativeTexturedState();
  object->Attributes()->SetupOwnDefaults();
  if (!linked) return object;
  const occ::handle<Prs3d_Drawer> defaults = new Prs3d_Drawer(); defaults->SetupOwnDefaults();
  int index = 0;
  for (const auto& line : width_lines(defaults)) line->SetWidth(++index);
  object->Attributes()->SetLink(defaults); return object;
}
static void print_width(const char* label, const occ::handle<NativeTexturedState>& object) {
  std::cout << "MARTENSITE_WIDTH " << label << ' ' << object->HasWidth() << ' ' << object->Width()
    << ' ' << object->Attributes()->HasOwnLineAspect();
  for (const auto& line : width_lines(object->Attributes())) std::cout << ' ' << line->Aspect()->Width();
  std::cout << '\n';
}
static void width_color_case(bool linked) {
  const auto object = width_object(linked);
  object->SetColor(Quantity_Color(Quantity_NOC_RED)); object->SetWidth(2.75);
  print_width(linked ? "color" : "unlinked", object);
  object->UnsetWidth(); print_width(linked ? "colorUnset" : "unlinkedUnset", object);
}
static void width_failure_case() {
  const auto object = width_object(true);
  try { object->SetWidth(-2.0); } catch (const Standard_Failure&) {}
  print_width("failure", object);
}
int main() {
  const auto object = width_object(true);
  object->SetWidth(2.75); print_width("plain", object);
  object->UnsetWidth(); print_width("plainUnset", object);
  object->UnsetWidth(); print_width("plainNoop", object);
  width_color_case(true); width_color_case(false); width_failure_case();
}
