// Last-owner callbacks inspect the real collection, not an owning snapshot.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <iostream>

class CollectionRetiringPlane : public Graphic3d_ClipPlane {
  DEFINE_STANDARD_RTTI_INLINE(CollectionRetiringPlane, Graphic3d_ClipPlane)
public:
  CollectionRetiringPlane(Graphic3d_SequenceOfHClipPlane& collection,
                         const char* phase, int ordinal)
      : myCollection(collection), myPhase(phase), myOrdinal(ordinal) {}
  ~CollectionRetiringPlane() override {
    std::cout << myPhase << ' ' << myOrdinal << ' ' << myCollection.Size()
              << ' ' << myCollection.IsEmpty() << ' ' << GetRefCount();
    // First remains valid only until the first node has been destroyed. Later
    // callbacks read metadata alone rather than dereferencing its freed node.
    if (myOrdinal == 1) {
      const occ::handle<Graphic3d_ClipPlane>& first = myCollection.First();
      std::cout << ' ' << (first.get() == this) << ' ' << first->GetRefCount();
    }
    std::cout << '\n';
  }
private:
  Graphic3d_SequenceOfHClipPlane& myCollection;
  const char* myPhase;
  int myOrdinal;
};

static void append_retiring(Graphic3d_SequenceOfHClipPlane& collection,
                           const char* phase, int ordinal) {
  occ::handle<Graphic3d_ClipPlane> plane =
      new CollectionRetiringPlane(collection, phase, ordinal);
  collection.Append(plane);
}
static void clear_ordered() {
  Graphic3d_SequenceOfHClipPlane collection;
  append_retiring(collection, "clear", 1);
  append_retiring(collection, "clear", 2);
  append_retiring(collection, "clear", 3);
  collection.Clear();
  std::cout << "clear-end " << collection.Size() << ' ' << collection.IsEmpty() << '\n';
}
static void remove_ordered() {
  Graphic3d_SequenceOfHClipPlane collection;
  append_retiring(collection, "remove", 1);
  append_retiring(collection, "remove", 2);
  Graphic3d_SequenceOfHClipPlane::Iterator cursor(collection);
  collection.Remove(cursor);
  std::cout << "remove-end " << collection.Size() << ' ' << cursor.More() << '\n';
  collection.Clear();
}
int main() {
  clear_ordered();
  remove_ordered();
}
