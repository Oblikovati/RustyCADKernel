// Actual native updateAttributes traverses a valid sequence append from a virtual group callback.
#include "martensite_textured_shape_state.hxx"
#include "tkservice_probe_null_driver.cxx"
#include <Prs3d_Drawer.hxx>
#include <Prs3d_Presentation.hxx>
#include <V3d_Viewer.hxx>
class NativeAppendingGroup : public SvGroup {
  bool appendOnce = true;
public:
  NativeAppendingGroup(const occ::handle<Graphic3d_Structure>& prs) : SvGroup(prs) {}
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override {
    SvGroup::SetGroupPrimitivesAspect(aspect);
    if (appendOnce) { appendOnce = false; Structure()->NewGroup(); }
  }
};
class NativeAppendingStructure : public SvCStructure {
public:
  NativeAppendingStructure(const occ::handle<Graphic3d_StructureManager>& manager) : SvCStructure(manager) {}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override {
    occ::handle<Graphic3d_Group> group = myGroups.IsEmpty()
      ? static_cast<Graphic3d_Group*>(new NativeAppendingGroup(prs))
      : static_cast<Graphic3d_Group*>(new SvGroup(prs));
    myGroups.Append(group);
    return group;
  }
};
class NativeAppendingDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {
    return new NativeAppendingStructure(manager);
  }
};
class NativeAppendingShape : public NativeTexturedState {
public:
  void update(const occ::handle<Prs3d_Presentation>& prs) { updateAttributes(prs); }
  const occ::handle<Graphic3d_AspectFillArea3d>& aspect() const { return myAspect; }
};
int main() {
  NativeAppendingShape shape;
  shape.Attributes()->SetupOwnDefaults(); shape.SetTextureMapOff();
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new NativeAppendingDriver());
  const occ::handle<Prs3d_Presentation> prs = new Prs3d_Presentation(viewer->StructureManager());
  prs->NewGroup(); shape.update(prs);
  std::cout << "MARTENSITE_APPENDED_GROUP " << prs->Groups().Size()
    << ' ' << (prs->Groups().Value(1)->Aspects() == shape.aspect())
    << ' ' << (prs->Groups().Value(2)->Aspects() == shape.aspect()) << '\n';
}
