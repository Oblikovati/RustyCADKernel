// Actual native live presentation Sequence traversal; a named group fake appends one retained mode3.
#include "martensite_textured_shape_state.hxx"
#include "tkservice_probe_null_driver.cxx"
#include <Prs3d_Drawer.hxx>
#include <PrsMgr_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <V3d_Viewer.hxx>
class NativeLivePresentationShape : public NativeTexturedState {
public:
  const occ::handle<Graphic3d_AspectFillArea3d>& finalAspect() const { return myAspect; }
};
class NativeLiveModePresentation : public PrsMgr_Presentation {
public:
  NativeLiveModePresentation(const occ::handle<PrsMgr_PresentationManager>& owner,
                            const occ::handle<NativeLivePresentationShape>& shape)
    : PrsMgr_Presentation(owner,shape,3) {}
};
struct NativeLivePresentationTarget {
  occ::handle<NativeLivePresentationShape> shape;
  occ::handle<PrsMgr_Presentation> next;
  bool appended = false;
  int groupCount = 0;
};
class NativeLivePresentationGroup : public SvGroup {
  NativeLivePresentationTarget& target;
public:
  NativeLivePresentationGroup(const occ::handle<Graphic3d_Structure>& prs,NativeLivePresentationTarget& value)
    : SvGroup(prs),target(value) {}
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override {
    SvGroup::SetGroupPrimitivesAspect(aspect);
    if (target.appended) return;
    target.appended = true;
    target.shape->Presentations().Append(target.next);
  }
};
class NativeLivePresentationStructure : public SvCStructure {
  NativeLivePresentationTarget& target;
public:
  NativeLivePresentationStructure(const occ::handle<Graphic3d_StructureManager>& owner,NativeLivePresentationTarget& value)
    : SvCStructure(owner),target(value) {}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override {
    const occ::handle<Graphic3d_Group> group = target.groupCount++ == 0
      ? static_cast<Graphic3d_Group*>(new NativeLivePresentationGroup(prs,target))
      : static_cast<Graphic3d_Group*>(new SvGroup(prs));
    myGroups.Append(group); return group;
  }
};
class NativeLivePresentationDriver : public SvDriver {
  NativeLivePresentationTarget& target;
public:
  NativeLivePresentationDriver(NativeLivePresentationTarget& value) : target(value) {}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& owner) override {
    return new NativeLivePresentationStructure(owner,target);
  }
};
static void change_live_presentation_appearance(NativeLivePresentationShape& object,int operation) {
  if (operation == 0) { object.SetColor(Quantity_Color(Quantity_NOC_RED)); return; }
  if (operation == 1) { object.SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold)); return; }
  object.UnsetMaterial();
}
static void prepare_live_presentation_shape(NativeLivePresentationShape& object,int operation) {
  object.Attributes()->SetupOwnDefaults(); object.SetTextureMapOff();
  const occ::handle<Prs3d_Drawer> defaults = new Prs3d_Drawer; defaults->SetupOwnDefaults();
  object.Attributes()->SetLink(defaults);
  if (operation == 2) object.SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
}
static void print_live_presentation_result(const NativeLivePresentationTarget& target,int operation,
                                         const occ::handle<PrsMgr_Presentation>& first) {
  const auto a = first->Groups().Value(1)->Aspects(); const auto b = target.next->Groups().Value(1)->Aspects();
  std::cout << "MARTENSITE_APPENDED_PRESENTATION " << operation << ' ' << target.shape->Presentations().Size()
    << ' ' << (a == b) << ' ' << (b == target.shape->finalAspect()) << '\n';
}
static void native_live_presentation_case(int operation) {
  NativeLivePresentationTarget target; target.shape = new NativeLivePresentationShape;
  prepare_live_presentation_shape(*target.shape,operation);
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new NativeLivePresentationDriver(target));
  const occ::handle<PrsMgr_PresentationManager> owner = new PrsMgr_PresentationManager(viewer->StructureManager());
  const occ::handle<PrsMgr_Presentation> first = new NativeLiveModePresentation(owner,target.shape);
  target.next = new NativeLiveModePresentation(owner,target.shape);
  first->NewGroup(); target.next->NewGroup(); target.shape->Presentations().Append(first);
  change_live_presentation_appearance(*target.shape,operation);
  print_live_presentation_result(target,operation,first);
}
int main() {
  for (int operation = 0; operation < 3; ++operation) {
    native_live_presentation_case(operation);
  }
}
