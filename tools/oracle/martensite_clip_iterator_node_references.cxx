// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: actual inherited node/link aliases.
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <iostream>
using ClipItems = NCollection_Sequence<occ::handle<Graphic3d_ClipPlane>>;
struct NativeNodePlanes : Graphic3d_SequenceOfHClipPlane
{
  ClipItems& Items() { return myItems; }
};
struct NativeNodeCursor : Graphic3d_SequenceOfHClipPlane::Iterator
{
  explicit NativeNodeCursor(NativeNodePlanes& planes) : Iterator(planes) {}
  NCollection_SeqNode*& CurrentField() { return myCurrent; }
  NCollection_SeqNode*& PreviousField() { return myPrevious; }
};

static void inspectPrefixes(NativeNodeCursor& cursor, ClipItems& items)
{
  NCollection_SeqNode* first = cursor.MutableNode();
  auto end = items.end();
  NCollection_SeqNode* last = end.MutablePreviousNode();
  std::cout << "prefix " << (first == cursor.Node()) << ' '
            << (first == cursor.CurrentNode()) << ' ' << (first->Previous() == nullptr) << '\n';
  NCollection_SeqNode* successor = first->Next();
  first->SetNext(last);
  cursor.Next();
  std::cout << "successor " << (cursor.Node() == last) << ' '
            << (cursor.PreviousNode() == first) << '\n';
  first->SetNext(successor);
  first->SetPrevious(last);
  std::cout << "predecessor " << (first->Previous() == last) << '\n';
  first->SetPrevious(nullptr);
}

static void inspectCursorFields(NativeNodeCursor& cursor, NativeNodePlanes& planes)
{
  cursor.Init(planes);
  auto*& current = cursor.CurrentField();
  auto*& repeated = cursor.CurrentField();
  auto*& previous = cursor.PreviousField();
  NCollection_SeqNode* first = current;
  cursor.Next();
  std::cout << "fields " << (&current == &repeated) << ' ' << (current != first)
            << ' ' << (previous == first) << '\n';
  auto copied = static_cast<ClipItems::iterator>(cursor);
  copied.Next();
  std::cout << "copy " << (copied.CurrentNode() != current) << ' '
            << (cursor.CurrentNode() == current) << '\n';
  current = previous;
  std::cout << "rebind " << (cursor.CurrentNode() == first) << '\n';
}

static void inspectConstConversion(NativeNodeCursor& cursor)
{
  ClipItems::const_iterator readonly(cursor);
  ClipItems::iterator writable(readonly);
  std::cout << "const " << (readonly.Node() == writable.MutableNode()) << ' '
            << (readonly.PreviousNode() == writable.MutablePreviousNode()) << ' '
            << (&readonly.Value() == &writable.ChangeValue()) << '\n';
  writable.Next();
  readonly = writable;
  std::cout << "const-assign " << (readonly == writable) << '\n';
}

static void inspectFieldInitialization(NativeNodeCursor& cursor, NativeNodePlanes& planes)
{
  auto*& current = cursor.CurrentField();
  NCollection_SeqNode** originalField = &current;
  cursor.Init(planes);
  std::cout << "field-init " << (&cursor.CurrentField() == originalField) << ' '
            << (current == planes.Items().begin().MutableNode()) << '\n';
  NativeNodeCursor source(planes);
  source.Next();
  cursor = source;
  std::cout << "field-assign " << (&cursor.CurrentField() == originalField) << ' '
            << (current == source.CurrentNode()) << '\n';
  cursor.Init(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  std::cout << "field-null " << (current == nullptr) << ' '
            << (&cursor.CurrentField() == originalField) << '\n';
}

static void inspectForeignNode(NativeNodeCursor& cursor)
{
  NativeNodePlanes foreign;
  foreign.Append(new Graphic3d_ClipPlane);
  NativeNodeCursor source(foreign);
  cursor.CurrentField() = source.MutableNode();
  std::cout << "foreign " << (cursor.Node() == source.Node()) << ' '
            << (&cursor.Value() == &source.Value()) << ' ' << cursor.Value()->GetRefCount() << '\n';
  cursor.CurrentField() = nullptr;
}

int main()
{
  NativeNodePlanes planes;
  planes.Append(new Graphic3d_ClipPlane);
  planes.Append(new Graphic3d_ClipPlane);
  planes.Append(new Graphic3d_ClipPlane);
  NativeNodeCursor cursor(planes);
  inspectPrefixes(cursor, planes.Items());
  inspectCursorFields(cursor, planes);
  inspectConstConversion(cursor);
  inspectFieldInitialization(cursor, planes);
  inspectForeignNode(cursor);
  return 0;
}
