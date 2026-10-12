// Valid delete-this during Allocate: after last-owner loss, the body uses only captured locals.
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. No retired-object read follows Nullify.
#define main clip_argument_baseline_main
#include "martensite_clip_allocator_argument.cxx"
#undef main
class ClipActiveLossAllocator : public NCollection_BaseAllocator {
public:
  ClipAllocatorSequence* sequence;
  occ::handle<NCollection_BaseAllocator>* external;
  occ::handle<Graphic3d_ClipPlane>* argument;
  const occ::handle<Graphic3d_ClipPlane>* replacement;
  ~ClipActiveLossAllocator() override {
    std::cout << "destroy " << GetRefCount() << '\n';
    *argument = *replacement;
  }
  void* Allocate(size_t size) override {
    void* pointer = NCollection_BaseAllocator::Allocate(size);
    auto* currentSequence = sequence;
    auto* currentOwner = external;
    std::cout << "allocate " << GetRefCount() << '\n';
    currentSequence->UseAllocator(NCollection_BaseAllocator::CommonBaseAllocator());
    currentOwner->Nullify();
    // Both locals refer outside the retired object; no member is accessed here.
    std::cout << "allocate-return\n";
    return pointer;
  }
};
static void configureActiveLoss(ClipActiveLossAllocator* allocator,
    ClipAllocatorSequence& sequence, occ::handle<NCollection_BaseAllocator>& owner,
    occ::handle<Graphic3d_ClipPlane>& argument,
    const occ::handle<Graphic3d_ClipPlane>& replacement) {
  allocator->sequence = &sequence;
  allocator->external = &owner;
  allocator->argument = &argument;
  allocator->replacement = &replacement;
  sequence.UseAllocator(owner);
}
int main() {
  ClipAllocatorSequence sequence;
  occ::handle<Graphic3d_ClipPlane> original = new Graphic3d_ClipPlane;
  occ::handle<Graphic3d_ClipPlane> replacement = new Graphic3d_ClipPlane;
  occ::handle<Graphic3d_ClipPlane> argument = original;
  occ::handle<NCollection_BaseAllocator> owner = new ClipActiveLossAllocator;
  configureActiveLoss(static_cast<ClipActiveLossAllocator*>(owner.get()),
                      sequence, owner, argument, replacement);
  bool appended = sequence.Append(argument);
  std::cout << "append " << appended << ' ' << (sequence.First() == replacement) << ' '
            << original->GetRefCount() << ' ' << replacement->GetRefCount() << '\n';
  sequence.Clear();
  return 0;
}
