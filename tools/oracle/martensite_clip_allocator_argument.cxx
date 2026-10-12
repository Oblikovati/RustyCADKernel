// A real native allocator callback retargets Append's caller-owned handle slot.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>

class ClipAllocatorSequence : public Graphic3d_SequenceOfHClipPlane {
public:
  void UseAllocator(const occ::handle<NCollection_BaseAllocator>& allocator) {
    myItems.Clear(allocator);
  }
};
class ClipArgumentAllocator : public NCollection_BaseAllocator {
public:
  ClipArgumentAllocator(ClipAllocatorSequence& sequence,
                        occ::handle<Graphic3d_ClipPlane>& argument,
                        const occ::handle<Graphic3d_ClipPlane>& original,
                        const occ::handle<Graphic3d_ClipPlane>& replacement)
      : mySequence(sequence), myArgument(argument), myOriginal(original),
        myReplacement(replacement) {}
  void* Allocate(const size_t size) override {
    write_counts("allocate-before");
    myArgument = myReplacement;
    write_counts("allocate-after");
    return NCollection_BaseAllocator::Allocate(size);
  }
  void Free(void* pointer) override {
    write_counts("free-before");
    NCollection_BaseAllocator::Free(pointer);
  }
private:
  void write_counts(const char* phase) const {
    std::cout << phase << ' ' << mySequence.Size() << ' ' << GetRefCount()
              << ' ' << myOriginal->GetRefCount() << ' '
              << myReplacement->GetRefCount() << '\n';
  }
  ClipAllocatorSequence& mySequence;
  occ::handle<Graphic3d_ClipPlane>& myArgument;
  const occ::handle<Graphic3d_ClipPlane>& myOriginal;
  const occ::handle<Graphic3d_ClipPlane>& myReplacement;
};
int main() {
  occ::handle<Graphic3d_ClipPlane> original = new Graphic3d_ClipPlane();
  occ::handle<Graphic3d_ClipPlane> replacement = new Graphic3d_ClipPlane();
  occ::handle<Graphic3d_ClipPlane> argument = original;
  ClipAllocatorSequence sequence;
  occ::handle<ClipArgumentAllocator> allocator =
      new ClipArgumentAllocator(sequence, argument, original, replacement);
  sequence.UseAllocator(allocator);
  bool appended = sequence.Append(argument);
  std::cout << "append " << appended << ' ' << (sequence.First() == replacement)
            << ' ' << (argument == replacement) << ' ' << original->GetRefCount()
            << ' ' << replacement->GetRefCount() << '\n';
  sequence.Clear();
  std::cout << "clear " << sequence.Size() << ' ' << allocator->GetRefCount()
            << ' ' << original->GetRefCount() << ' ' << replacement->GetRefCount() << '\n';
}
