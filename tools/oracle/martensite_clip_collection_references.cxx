// Live node/handle references use actual native protected collection storage.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <Standard_NoSuchObject.hxx>
#include <iostream>

class ClipReferenceSequence : public Graphic3d_SequenceOfHClipPlane {
public:
  void ReplaceFirst(const occ::handle<Graphic3d_ClipPlane>& replacement) {
    myItems.ChangeFirst() = replacement;
  }
};
static void write_counts(const char* phase, const Graphic3d_SequenceOfHClipPlane& sequence,
                         const Graphic3d_ClipPlane& first, const Graphic3d_ClipPlane& second) {
  std::cout << phase << ' ' << sequence.Size() << ' ' << sequence.GetRefCount()
            << ' ' << first.GetRefCount() << ' ' << second.GetRefCount() << '\n';
}
static void null_and_empty() {
  occ::handle<ClipReferenceSequence> sequence = new ClipReferenceSequence();
  occ::handle<Graphic3d_ClipPlane> null;
  bool first = sequence->Append(null);
  bool duplicate = sequence->Append(null);
  std::cout << "null " << first << ' ' << duplicate << ' ' << sequence->Size()
            << ' ' << sequence->First().IsNull() << '\n';
  sequence->Remove(null);
  try { sequence->First(); }
  catch (const Standard_NoSuchObject&) {
    std::cout << "empty " << sequence->IsEmpty() << ' ' << sequence->Size() << '\n';
  }
}
static void live_aliases() {
  occ::handle<ClipReferenceSequence> sequence = new ClipReferenceSequence();
  occ::handle<Graphic3d_ClipPlane> first = new Graphic3d_ClipPlane();
  occ::handle<Graphic3d_ClipPlane> second = new Graphic3d_ClipPlane();
  sequence->Append(first);
  const occ::handle<Graphic3d_ClipPlane>& firstAlias = sequence->First();
  Graphic3d_SequenceOfHClipPlane::Iterator iterator(*sequence);
  const occ::handle<Graphic3d_ClipPlane>& cursorAlias = iterator.Value();
  write_counts("borrow", *sequence, *first, *second);
  sequence->ReplaceFirst(second);
  std::cout << "alias " << (firstAlias == second) << ' ' << (cursorAlias == second)
            << ' ' << (&firstAlias == &cursorAlias) << '\n';
  write_counts("replace", *sequence, *first, *second);
}
static void write_cursor_edit(const Graphic3d_SequenceOfHClipPlane& sequence,
                              const Graphic3d_SequenceOfHClipPlane::Iterator& iterator,
                              const Graphic3d_ClipPlane& first,
                              const Graphic3d_ClipPlane& second,
                              const occ::handle<Graphic3d_ClipPlane>& third) {
  std::cout << "edit " << (iterator.Value() == third) << ' ' << sequence.Size()
            << ' ' << first.GetRefCount() << ' ' << second.GetRefCount()
            << ' ' << third->GetRefCount() << '\n';
}
static void live_cursor_edits() {
  occ::handle<ClipReferenceSequence> sequence = new ClipReferenceSequence();
  occ::handle<Graphic3d_ClipPlane> first = new Graphic3d_ClipPlane();
  occ::handle<Graphic3d_ClipPlane> second = new Graphic3d_ClipPlane();
  occ::handle<Graphic3d_ClipPlane> third = new Graphic3d_ClipPlane();
  sequence->Append(first); sequence->Append(second);
  Graphic3d_SequenceOfHClipPlane::Iterator iterator(*sequence);
  sequence->Remove(second); sequence->Append(third);
  iterator.Next();
  write_cursor_edit(*sequence, iterator, *first, *second, third);
  sequence->Remove(iterator);
  std::cout << "remove-cursor " << iterator.More() << ' ' << sequence->Size()
            << ' ' << third->GetRefCount() << '\n';
}
static void iterator_null_init() {
  occ::handle<Graphic3d_SequenceOfHClipPlane> sequence = new Graphic3d_SequenceOfHClipPlane();
  sequence->Append(new Graphic3d_ClipPlane());
  Graphic3d_SequenceOfHClipPlane::Iterator iterator(sequence);
  std::cout << "iterator-borrow " << iterator.More() << ' ' << sequence->GetRefCount()
            << ' ' << iterator.Value()->GetRefCount() << '\n';
  occ::handle<Graphic3d_SequenceOfHClipPlane> null;
  iterator.Init(null);
  std::cout << "iterator-null " << iterator.More() << ' ' << sequence->GetRefCount() << '\n';
  sequence->Remove(iterator);
  std::cout << "remove-end " << iterator.More() << ' ' << sequence->Size() << '\n';
}
int main() {
  null_and_empty();
  live_aliases();
  live_cursor_edits();
  iterator_null_init();
}
