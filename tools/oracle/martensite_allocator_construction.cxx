// Real protected construction, original allocator aliases and the static const handle reference.
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
class ProtectedBaseProbe : public NCollection_BaseAllocator {
public:
  ProtectedBaseProbe() = default;
};
int main() {
  ProtectedBaseProbe stack;
  std::cout << "stack " << stack.GetRefCount() << '\n';
  Handle(ProtectedBaseProbe) owner = new ProtectedBaseProbe;
  std::cout << "owned " << owner->GetRefCount() << '\n';
  Handle(NCollection_BaseAllocator) source = owner;
  std::cout << "source " << source->GetRefCount() << ' ' << (source.get() == owner.get()) << '\n';
  source.Nullify();
  std::cout << "released " << owner->GetRefCount() << '\n';
  const Handle(NCollection_BaseAllocator)& original = NCollection_BaseAllocator::CommonBaseAllocator();
  const int before = original->GetRefCount();
  const Handle(NCollection_BaseAllocator)& repeated = NCollection_BaseAllocator::CommonBaseAllocator();
  std::cout << "common " << (&original == &repeated) << ' ' << (original->GetRefCount() == before) << '\n';
  return 0;
}
