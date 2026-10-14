// AIS_ViewController 3d097a0: actual local aPickedView survives OnSubviewChanged and Reset.
#include "martensite_picked_slot_scene.hxx"
#include <Standard_ConstructionError.hxx>
#include <AIS_AnimationCamera.hxx>
#include <iostream>
#include <stdexcept>
class PickedSlotController;
class PickedSlotAllocator : public NCollection_BaseAllocator {
public:
  PickedSlotController* controller = nullptr;
  bool armed = false;
  int calls = 0;
  void Free(void* pointer) override;
};
class PickedSlotController : public AIS_ViewController {
public:
  PickedSlotAllocator* allocator;
  V3d_View* child;
  const occ::handle<V3d_View>* alias = nullptr;
  const occ::handle<V3d_View>* second_alias = nullptr;
  int policy = 0, frame = 0, baseline = 0;
  explicit PickedSlotController(PickedSlotAllocator* alloc, V3d_View* selected)
    : allocator(alloc), child(selected) { ViewAnimation()->Stop(); }
  void OnSubviewChanged(const occ::handle<AIS_InteractiveContext>&,
      const occ::handle<V3d_View>& original, const occ::handle<V3d_View>& picked) override {
    alias = &picked; second_alias = &picked;
    emit(0, original.get());
    if (policy == 1) throw Standard_ConstructionError("picked callback policy=1; expected success");
    if (policy == 2) throw std::logic_error("picked callback policy=2; expected success");
  }
  void emit(int event, const V3d_View* original = nullptr) const {
    std::cout << policy << ' ' << frame << ' ' << event << ' ' << (alias == second_alias)
      << ' ' << (alias->get() == child) << ' ' << ((*alias)->GetRefCount() - baseline)
      << ' ' << child->View()->IsActive() << ' ' << child->View()->IsSubview()
      << ' ' << InputBuffer(AIS_ViewInputBufferType_GL).ZoomActions.Length()
      << ' ' << toAskNextFrame() << ' ' << (original != child) << '\n';
  }
protected:
  void handleViewRedraw(const occ::handle<AIS_InteractiveContext>& context,
      const occ::handle<V3d_View>& view) override {
    AIS_ViewController::handleViewRedraw(context, view);
    auto& input = ChangeInputBuffer(AIS_ViewInputBufferType_GL);
    input.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(10,20),.5));
    input.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(30,40),-.25));
    setAskNextFrame(true); allocator->armed = true;
  }
};
void PickedSlotAllocator::Free(void* pointer) {
  if (armed) { ++calls; controller->emit(1); }
  NCollection_BaseAllocator::Free(pointer);
}
static int picked_slot_apply(PickedSlotController& controller, const PickedSlotScene& scene) {
  try { controller.HandleViewEvents(scene.context, scene.parent); return 0; }
  catch (const Standard_ConstructionError&) { return 1; }
  catch (const std::logic_error&) { return 2; }
}
static void picked_slot_frame(PickedSlotController& controller, const PickedSlotScene& scene,
    int frame, int policy) {
  controller.frame = frame; controller.policy = policy; controller.alias = nullptr;
  controller.second_alias = nullptr; controller.allocator->calls = 0;
  controller.allocator->armed = false;
  auto& input = controller.ChangeInputBuffer(AIS_ViewInputBufferType_GL);
  input.Selection.Tool = AIS_ViewSelectionTool_Picking;
  input.Selection.Points.Append(NCollection_Vec2<int>(1,1));
  const int failure = picked_slot_apply(controller, scene);
  // The captured native local is now dangling; observe only independently owned child and input metadata.
  controller.alias = controller.second_alias = nullptr;
  std::cout << policy << ' ' << frame << " 2 " << failure << ' ' << controller.allocator->calls
    << ' ' << (scene.child->GetRefCount() - controller.baseline)
    << ' ' << scene.parent->View()->IsActive() << ' ' << scene.child->View()->IsActive()
    << ' ' << input.ZoomActions.Length() << ' ' << controller.toAskNextFrame() << " 1\n";
}
static void picked_slot_case(int policy) {
  auto scene = picked_slot_scene();
  occ::handle<PickedSlotAllocator> allocator = new PickedSlotAllocator;
  PickedSlotController controller(allocator.get(), scene.child.get());
  allocator->controller = &controller;
  controller.ChangeInputBuffer(AIS_ViewInputBufferType_GL).ZoomActions.Clear(allocator);
  controller.baseline = scene.child->GetRefCount();
  picked_slot_frame(controller, scene, 0, policy);
  picked_slot_frame(controller, scene, 1, 0);
  allocator->armed = false; picked_slot_scene_close(scene);
}
int main() {
  for (int policy = 0; policy < 3; ++policy) picked_slot_case(policy);
  return 0;
}
