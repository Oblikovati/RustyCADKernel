// Last-owner callbacks inspect the real collection, not an owning snapshot.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <iostream>

class CollectionRetirementSequence : public Graphic3d_SequenceOfHClipPlane {
public:
  void AppendItemDirect(const occ::handle<Graphic3d_ClipPlane>& plane) {
    myItems.Append(plane);
  }
};
struct CollectionRetirementAppend {
  const occ::handle<Graphic3d_ClipPlane>* plane = nullptr;
  int ordinal = 0;
  CollectionRetirementSequence* direct = nullptr;
};
static CollectionRetirementAppend THE_APPEND;

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
    if (THE_APPEND.plane != nullptr && myOrdinal == THE_APPEND.ordinal) {
      append_callback();
    }
  }
private:
  void append_callback() {
    if (THE_APPEND.direct != nullptr) {
      THE_APPEND.direct->AppendItemDirect(*THE_APPEND.plane);
      return;
    }
    myCollection.Append(*THE_APPEND.plane);
  }
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
static void append_during_clear(int ordinal) {
  Graphic3d_SequenceOfHClipPlane collection;
  occ::handle<Graphic3d_ClipPlane> appended = new Graphic3d_ClipPlane();
  THE_APPEND = {&appended, ordinal};
  append_retiring(collection, "append-clear", 1);
  append_retiring(collection, "append-clear", 2);
  collection.Clear();
  THE_APPEND = {};
  std::cout << "append-end " << ordinal << ' ' << collection.Size() << ' '
            << appended->GetRefCount() << '\n';
}
static void append_protected_during_last_clear() {
  CollectionRetirementSequence collection;
  occ::handle<Graphic3d_ClipPlane> appended = new Graphic3d_ClipPlane();
  THE_APPEND = {&appended, 2, &collection};
  append_retiring(collection, "protected-append", 1);
  append_retiring(collection, "protected-append", 2);
  collection.Clear();
  THE_APPEND = {};
  std::cout << "protected-end " << collection.Size() << ' '
            << appended->GetRefCount() << '\n';
}
int main() {
  clear_ordered();
  remove_ordered();
  append_during_clear(1);
  append_protected_during_last_clear();
}
