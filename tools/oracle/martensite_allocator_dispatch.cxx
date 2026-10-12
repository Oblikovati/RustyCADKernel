// Pinned OCCT derived allocator virtual dispatch/count oracle; no renderer dependency.
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
class AllocatorProbe : public NCollection_BaseAllocator {
public:
  void* Allocate(const size_t size) override {
    std::cout << "allocate " << size << " " << GetRefCount() << "\n";
    return NCollection_BaseAllocator::Allocate(size);
  }
  void* AllocateOptimal(const size_t size) override {
    std::cout << "optimal " << size << " " << GetRefCount() << "\n";
    return NCollection_BaseAllocator::AllocateOptimal(size);
  }
  void Free(void* pointer) override {
    std::cout << "free " << GetRefCount() << "\n";
    NCollection_BaseAllocator::Free(pointer);
  }
};
int main() {
  Handle(AllocatorProbe) owner = new AllocatorProbe;
  Handle(NCollection_BaseAllocator) resource = owner;
  void* pointer = resource->AllocateOptimal(16);
  resource->Free(pointer);
  unsigned long long* payload = new (resource->Allocate(sizeof(unsigned long long))) unsigned long long(29);
  if (owner->GetRefCount() != 2 || *payload != 29) return 1;
  resource->Free(payload);
  return 0;
}
