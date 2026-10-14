// Free throws after sequence node destruction: retain valid callback reads, never reuse corrupt nodes.
#define main picked_slot_success_main
#include "martensite_picked_slot_retention.cxx"
#undef main
class PickedSlotThrowingAllocator : public PickedSlotAllocator {
public:
  int policy = 0;
  void Free(void* pointer) override {
    if (!armed) { NCollection_BaseAllocator::Free(pointer); return; }
    ++calls; controller->emit(1);
    if (policy == 1) throw Standard_ConstructionError("Free policy=1; expected success");
    throw std::logic_error("Free policy=2; expected success");
  }
};
static void picked_slot_reset_failure(int policy) {
  auto scene = picked_slot_scene();
  occ::handle<PickedSlotThrowingAllocator> allocator = new PickedSlotThrowingAllocator;
  allocator->policy = policy;
  auto* controller = new PickedSlotController(allocator.get(), scene.child.get());
  allocator->controller = controller;
  controller->ChangeInputBuffer(AIS_ViewInputBufferType_GL).ZoomActions.Clear(allocator);
  controller->baseline = scene.child->GetRefCount();
  picked_slot_frame(*controller, scene, 0, 0);
  // Native Clear has ended a node lifetime before Free throws. Do not destroy/retry that sequence.
  // This process-scoped leak is fixture-only; a fresh controller proves independent reuse below.
}
int main() {
  picked_slot_reset_failure(1);
  picked_slot_reset_failure(2);
  picked_slot_case(0);
  return 0;
}
