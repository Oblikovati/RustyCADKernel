// Const selector references remain bound to the original selection manager, OCCT 3d097a0.
#include "martensite_camera_primitives_oracle_shared.cxx"
#include <AIS_InteractiveContext.hxx>
#include <SelectMgr_SelectionManager.hxx>
#include <StdSelect_ViewerSelector3d.hxx>
class SelectorReferenceContext : public AIS_InteractiveContext {
public:
  SelectorReferenceContext(const occ::handle<V3d_Viewer>& viewer)
    : AIS_InteractiveContext(viewer)
  {
  }
  void replace_manager(const occ::handle<SelectMgr_SelectionManager>& manager) {
    // Preserve actual protected-field ownership; both possible owners remain externally retained.
    mgrSelector=manager;
  }
};
static void print_selector_reference(int phase,const occ::handle<SelectorReferenceContext>& context,
    const occ::handle<SelectMgr_SelectionManager>& first,
    const occ::handle<SelectMgr_SelectionManager>& second,
    const occ::handle<StdSelect_ViewerSelector3d>& reference) {
  std::cout<<phase<<' '<<(&reference==&first->Selector())<<' '
    <<(reference==first->Selector())<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<' '
    <<reference->GetRefCount()<<' '<<second->Selector()->GetRefCount()<<' '
    <<(context->SelectionManager()==first)<<'\n';
}
int main() {
  auto scene=primitive_scene(0);
  occ::handle<SelectorReferenceContext> context=new SelectorReferenceContext(scene.viewer);
  occ::handle<SelectMgr_SelectionManager> first=context->SelectionManager();
  occ::handle<StdSelect_ViewerSelector3d> selector=new StdSelect_ViewerSelector3d();
  occ::handle<SelectMgr_SelectionManager> second=new SelectMgr_SelectionManager(selector);
  const auto& reference=context->MainSelector();
  print_selector_reference(0,context,first,second,reference);
  context->replace_manager(second);print_selector_reference(1,context,first,second,reference);
  context->replace_manager({});print_selector_reference(2,context,first,second,reference);
  context->replace_manager(first);print_selector_reference(3,context,first,second,reference);
  return 0;
}
