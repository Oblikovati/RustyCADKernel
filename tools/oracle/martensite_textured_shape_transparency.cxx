// Actual AIS_Shape inheritance on AIS_TexturedShape: facing and ownership, OCCT 3d097a0.
#include "martensite_textured_shape_state.hxx"
#include <Prs3d_Drawer.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Standard_Failure.hxx>
static occ::handle<NativeTexturedState> transparency_object() {
  const occ::handle<NativeTexturedState> object = new NativeTexturedState();
  const occ::handle<Prs3d_Drawer> linked = new Prs3d_Drawer();
  linked->SetupOwnDefaults(); object->Attributes()->SetLink(linked);
  return object;
}
static void print_transparency(const char* label, const occ::handle<NativeTexturedState>& object) {
  const auto drawer = object->Attributes();
  const auto shading = drawer->ShadingAspect();
  std::cout << "MARTENSITE_TRANSPARENCY " << label << ' ' << drawer->HasOwnShadingAspect()
    << ' ' << drawer->Transparency() << ' ' << shading->Transparency(Aspect_TOFM_FRONT_SIDE)
    << ' ' << shading->Transparency(Aspect_TOFM_BACK_SIDE) << '\n';
}
static void check_transparency_facing() {
  for (int side = 0; side != 3; ++side) {
    const auto object = transparency_object();
    object->SetCurrentFacingModel(static_cast<Aspect_TypeOfFacingModel>(side));
    object->SetTransparency(0.375);
    print_transparency((std::string("side") + std::to_string(side)).c_str(), object);
    object->UnsetTransparency();
    print_transparency((std::string("unset") + std::to_string(side)).c_str(), object);
  }
}
static void check_retained_transparency() {
  const auto object = transparency_object();
  object->SetColor(Quantity_Color(Quantity_NOC_RED));
  object->SetCurrentFacingModel(Aspect_TOFM_FRONT_SIDE); object->SetTransparency(0.375);
  object->UnsetTransparency(); print_transparency("color", object);
  const auto failure = transparency_object();
  try { failure->SetTransparency(1.25); } catch (const Standard_Failure&) {}
  print_transparency("failure", failure);
}
static void check_transparency_retention_matrix() {
  const auto material = transparency_object();
  material->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  material->SetTransparency(0.375); material->UnsetTransparency();
  print_transparency("material", material);
  const auto texture = transparency_object();
  texture->Attributes()->SetupOwnShadingAspect();
  texture->Attributes()->ShadingAspect()->Aspect()->SetTextureMapOn(true);
  texture->SetTransparency(0.375); texture->UnsetTransparency();
  print_transparency("texture", texture);
  const auto empty = transparency_object();
  empty->UnsetTransparency(); print_transparency("empty", empty);
}
int main() {
  std::cout << std::setprecision(17);
  check_transparency_facing();
  check_retained_transparency();
  check_transparency_retention_matrix();
}
