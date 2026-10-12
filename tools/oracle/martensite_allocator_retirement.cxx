// Valid Heap allocator owner loss: nodes survive and the current compatible resource frees them.
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_Sequence.hxx>
#include <iostream>
class CurrentHeapProbe : public NCollection_HeapAllocator {
public:
  CurrentHeapProbe(NCollection_Sequence<int>& sequence, bool& retired)
      : mySequence(sequence), myRetired(retired) {}
  void Free(void* pointer) override {
    std::cout << "current-free " << mySequence.Size() << ' ' << GetRefCount()
              << ' ' << myRetired << '\n';
    NCollection_HeapAllocator::Free(pointer);
  }
private:
  NCollection_Sequence<int>& mySequence;
  bool& myRetired;
};
class RetiringHeapProbe : public NCollection_HeapAllocator {
public:
  RetiringHeapProbe(NCollection_Sequence<int>& sequence,
                    const Handle(CurrentHeapProbe)& current, bool& retired)
      : mySequence(sequence), myCurrent(current), myRetired(retired) {}
  void* Allocate(const size_t size) override {
    std::cout << "allocate " << mySequence.Size() << ' ' << GetRefCount() << '\n';
    mySequence.Clear(myCurrent);
    std::cout << "replaced " << mySequence.Size() << ' ' << GetRefCount()
              << ' ' << myCurrent->GetRefCount() << '\n';
    return NCollection_HeapAllocator::Allocate(size);
  }
  ~RetiringHeapProbe() override {
    myRetired = true;
    std::cout << "retired " << mySequence.Size() << ' ' << GetRefCount()
              << ' ' << myCurrent->GetRefCount() << '\n';
  }
private:
  NCollection_Sequence<int>& mySequence;
  const Handle(CurrentHeapProbe)& myCurrent;
  bool& myRetired;
};
int main() {
  bool retired = false;
  NCollection_Sequence<int> sequence;
  Handle(CurrentHeapProbe) current = new CurrentHeapProbe(sequence, retired);
  Handle(RetiringHeapProbe) original = new RetiringHeapProbe(sequence, current, retired);
  sequence.Clear(original);
  sequence.Append(19);
  original.Nullify();
  std::cout << "after-owner-loss " << sequence.Size() << ' ' << retired << '\n';
  sequence.Clear();
  std::cout << "cleared " << sequence.Size() << ' ' << retired << '\n';
  return 0;
}
