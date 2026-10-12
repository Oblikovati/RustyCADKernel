// Real appearance overrides on genuine PrsMgr mode presentations; null driver isolates GPU I/O.
#include "martensite_textured_shape_state.hxx"
#include "tkservice_probe_null_driver.cxx"
#include <Prs3d_Drawer.hxx>
#include <PrsMgr_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <V3d_Viewer.hxx>
class NativeAppearancePresentation : public PrsMgr_Presentation {
public:
  NativeAppearancePresentation(const occ::handle<PrsMgr_PresentationManager>& manager,
                               const occ::handle<PrsMgr_PresentableObject>& object, int mode)
    : PrsMgr_Presentation(manager, object, mode) {}
};
static void print_appearance_color(const char* label,
                                  const occ::handle<NativeTexturedState>& object,
                                  const occ::handle<PrsMgr_Presentation>& prs) {
  const auto aspect = prs->Groups().Value(1)->Aspects();
  const auto color = aspect->InteriorColor();
  std::cout << "MARTENSITE_APPEARANCE " << label << ' ' << object->HasColor()
    << ' ' << object->HasMaterial() << ' ' << color.Red() << ' ' << color.Green()
    << ' ' << color.Blue() << '\n';
}
static void run_native_appearance_changes(const occ::handle<NativeTexturedState>& object,
                                          const occ::handle<PrsMgr_Presentation>& prs) {
  object->SetColor(Quantity_Color(Quantity_NOC_RED)); print_appearance_color("red", object, prs);
  const auto old = prs->Groups().Value(1)->Aspects();
  object->UnsetColor(); print_appearance_color("unset", object, prs);
  std::cout << "MARTENSITE_SAME_ASPECT " << (old == prs->Groups().Value(1)->Aspects()) << '\n';
  object->SetColor(Quantity_Color(Quantity_NOC_RED));
  object->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  print_appearance_color("material", object, prs);
  const auto previousMaterial = prs->Groups().Value(1)->Aspects();
  object->UnsetMaterial(); print_appearance_color("unsetMaterial", object, prs);
  std::cout << "MARTENSITE_MATERIAL_ASPECT_CHANGED "
    << (previousMaterial != prs->Groups().Value(1)->Aspects()) << '\n';
}
int main() {
  const occ::handle<NativeTexturedState> object = new NativeTexturedState();
  const occ::handle<Prs3d_Drawer> linked = new Prs3d_Drawer();
  linked->SetupOwnDefaults(); object->Attributes()->SetLink(linked);
  object->SetTextureMapOff();
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  const occ::handle<PrsMgr_PresentationManager> manager = new PrsMgr_PresentationManager(viewer->StructureManager());
  const occ::handle<PrsMgr_Presentation> prs = new NativeAppearancePresentation(manager, object, 3);
  object->Presentations().Append(prs); prs->NewGroup();
  run_native_appearance_changes(object, prs);
}
