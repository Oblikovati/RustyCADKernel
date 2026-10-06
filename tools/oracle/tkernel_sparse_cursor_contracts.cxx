#include <NCollection_SparseArray.hxx>
#include <iostream>

void initialize_cursor_fixture(NCollection_SparseArray<int>& array)
{
  array.SetValue(9, 90);
  array.SetValue(0, 10);
  array.SetValue(5, 50);
}

void const_cursor_states(const NCollection_SparseArray<int>& array)
{
  NCollection_SparseArray<int>::ConstIterator cursor;
  std::cout << "default_const:" << cursor.More() << ':' << cursor.Key();
  cursor.Next();
  cursor.Restart();
  std::cout << ':' << cursor.More() << ':' << cursor.Key() << '\n';
  cursor.Init(array);
  std::cout << "const:";
  while (cursor.More()) {
    std::cout << cursor.Key() << '=' << cursor.Value() << ',';
    cursor.Next();
  }
  cursor.Next();
  std::cout << ':' << cursor.More() << ':' << cursor.Key() << '\n';
  cursor.Restart();
  std::cout << "restart_const:" << cursor.Key() << ':' << cursor() << '\n';
}

void end_cursor_states(const NCollection_SparseArray<int>& array)
{
  auto cursor = array.cend();
  std::cout << "end_const:" << cursor.Key() << ':' << cursor.More();
  cursor.Restart();
  std::cout << ':' << cursor.Key() << ':' << *cursor << '\n';
}

void mutable_cursor_states(NCollection_SparseArray<int>& array)
{
  NCollection_SparseArray<int>::Iterator cursor;
  std::cout << "default_mut:" << cursor.More() << ':' << cursor.Key();
  cursor.Next();
  cursor.Restart();
  std::cout << ':' << cursor.More() << ':' << cursor.Key() << '\n';
  cursor.Init(array);
  while (cursor.More()) {
    cursor.ChangeValue() += int(cursor.Key());
    cursor.Next();
  }
  std::cout << "mut:" << array.Value(0) << ':' << array.Value(5) << ':' << array.Value(9)
            << ':' << cursor.Key() << ':' << cursor.More() << '\n';
  cursor.Restart();
  std::cout << "restart_mut:" << cursor.Key() << ':' << cursor.Value() << '\n';
}

void empty_cursor_states()
{
  NCollection_SparseArray<int> array(4);
  array.SetValue(9, 90);
  array.UnsetValue(9);
  auto cursor = array.begin();
  std::cout << "empty:" << array.Capacity() << ':' << cursor.More() << ':' << cursor.Key();
  cursor.Restart();
  std::cout << ':' << cursor.Key() << '\n';
}

void reinitialized_cursor_states(const NCollection_SparseArray<int>& first)
{
  NCollection_SparseArray<int> other(2);
  other.SetValue(2, 20);
  NCollection_SparseArray<int>::ConstIterator cursor(first);
  cursor.Next();
  cursor.Init(other);
  std::cout << "reinit:" << cursor.Key() << ':' << cursor.Value();
  cursor.Next();
  std::cout << ':' << cursor.Key() << ':' << cursor.More() << '\n';
}

void copied_cursor_states(const NCollection_SparseArray<int>& array)
{
  auto first = array.cbegin();
  auto second = first;
  const bool same_item = &*first == &*second;
  ++second;
  std::cout << "copy:" << first.Key() << ':' << second.Key() << ':' << same_item << '\n';
  NCollection_SparseArray<int> other(4);
  initialize_cursor_fixture(other);
  NCollection_SparseArray<int>::const_iterator detached;
  std::cout << "equality:" << (first == array.cbegin()) << ':' << (first == other.cbegin())
            << ':' << (array.cend() == other.cend()) << ':' << (detached == array.cend()) << '\n';
}

void incremented_cursor_states(const NCollection_SparseArray<int>& array)
{
  auto cursor = array.cbegin();
  auto previous = cursor++;
  std::cout << "increment:" << previous.Key() << ':' << cursor.Key();
  auto& returned = ++cursor;
  std::cout << ':' << (&returned == &cursor) << ':' << cursor.Key() << '\n';
}

void converted_cursor_states(NCollection_SparseArray<int>& array)
{
  auto cursor = array.begin();
  NCollection_SparseArray<int>::const_iterator view(cursor);
  std::cout << "snapshot:" << view.Key() << ':' << *view;
  ++cursor;
  *cursor = 55;
  view = cursor;
  std::cout << ':' << view.Key() << ':' << *view << '\n';
}

void indexed_cursor_states(NCollection_SparseArray<int>& array)
{
  for (size_t start : {0, 3, 6, 16, 20}) {
    NCollection_SparseArray<int>::const_iterator constant(array, start);
    NCollection_SparseArray<int>::iterator mutable_cursor(array, start);
    std::cout << "indexed:" << start << ':' << constant.Key() << ':' << constant.More()
              << ':' << mutable_cursor.Key() << ':' << mutable_cursor.More() << '\n';
  }
  NCollection_SparseArray<int>::const_iterator past_end(array, 20);
  ++past_end;
  std::cout << "past_end:" << past_end.Key();
  past_end.Restart();
  std::cout << ':' << past_end.Key() << '\n';
}

void pointed_cursor_states(NCollection_SparseArray<int>& array)
{
  NCollection_SparseArray<int>::const_iterator constant(array, 3);
  const bool same_address = constant.operator->() == &array.Value(5);
  NCollection_SparseArray<int>::iterator cursor(array, 3);
  *cursor.operator->() = 555;
  std::cout << "pointer:" << same_address << ':' << *cursor;
  ++cursor;
  std::cout << ':' << cursor.Key() << '\n';
}

void assigned_cursor_states(NCollection_SparseArray<int>& array)
{
  auto destination = array.cbegin();
  const auto ending = array.cend();
  auto& returned = (destination = ending);
  std::cout << "assign:" << (&returned == &destination) << ':' << destination.Key()
            << ':' << destination.More();
  destination = destination;
  destination = array.cbegin();
  ++destination;
  std::cout << ':' << destination.Key() << ':' << array.cbegin().Key() << '\n';
  auto mutable_cursor = array.begin();
  ++mutable_cursor;
  auto& converted = (destination = mutable_cursor);
  std::cout << "assign_mut:" << (&converted == &destination) << ':' << destination.Key()
            << ':' << *destination << ':' << mutable_cursor.Key() << '\n';
}

void advanced_cursor_states(NCollection_SparseArray<int>& array)
{
  copied_cursor_states(array);
  incremented_cursor_states(array);
  initialize_cursor_fixture(array);
  converted_cursor_states(array);
  indexed_cursor_states(array);
  initialize_cursor_fixture(array);
  pointed_cursor_states(array);
  initialize_cursor_fixture(array);
  assigned_cursor_states(array);
}

int main()
{
  NCollection_SparseArray<int> array(4);
  initialize_cursor_fixture(array);
  const_cursor_states(array);
  end_cursor_states(array);
  mutable_cursor_states(array);
  empty_cursor_states();
  reinitialized_cursor_states(array);
  advanced_cursor_states(array);
}
