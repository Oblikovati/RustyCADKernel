#include <NCollection_Sequence.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <limits>
#include <utility>

using Sequence = NCollection_Sequence<int>;

template <class Operation> bool rejected(Operation operation)
{
  try { operation(); }
  catch (const Standard_OutOfRange&) { return true; }
  return false;
}

Sequence sequence()
{
  Sequence values;
  values.Append(10); values.Append(20); values.Append(30);
  return values;
}

void signed_rejections()
{
  Sequence values = sequence();
  int failures = 0;
  failures += rejected([&] { values.Value(-1); });
  failures += rejected([&] { values.ChangeValue(-1); });
  failures += rejected([&] { values.SetValue(-1, 99); });
  failures += rejected([&] { values.Exchange(-1, 1); });
  failures += rejected([&] { values.Exchange(1, -1); });
  failures += rejected([&] { values.Remove(-1); });
  failures += rejected([&] { values.Remove(-1, 2); });
  failures += rejected([&] { values.Remove(1, -1); });
  failures += rejected([&] { values.InsertBefore(-1, 99); });
  failures += rejected([&] { values.InsertAfter(-1, 99); });
  std::cout << "negative:" << failures << ':' << values.Size() << ':' << values.First()
            << ':' << values.Last() << '\n';
}

void signed_positive()
{
  Sequence values = sequence();
  values.ChangeValue(1) = 11;
  values.SetValue(2, 22); values.Exchange(1, 2);
  values.InsertBefore(1, 1); values.InsertAfter(0, 0);
  values.Remove(1); values.Remove(2, 3);
  std::cout << "positive:" << values.Size() << ':' << values.First() << ':' << values.Last()
            << ':' << rejected([&] { values.Value(0); })
            << ':' << rejected([&] { values.ChangeValue(3); }) << '\n';
}

void emplacement_validation()
{
  Sequence values;
  values.Append(10);
  int failures = rejected([&] { values.EmplaceBefore(-1, 20); });
  failures += rejected([&] { values.EmplaceAfter(-1, 20); });
  failures += rejected([&] { values.EmplaceBefore(0, 20); });
  failures += rejected([&] { values.EmplaceAfter(2, 20); });
  int& head = values.EmplaceAfter(0, 20);
  int& tail = values.EmplaceBefore(3, 20);
  std::cout << "emplace:" << failures << ':' << head << ':' << tail << ':' << values.Size()
            << ':' << values.Value(2) << '\n';
}

void split_order()
{
  occ::handle<NCollection_BaseAllocator> first = new NCollection_IncAllocator(128);
  occ::handle<NCollection_BaseAllocator> second = new NCollection_IncAllocator(128);
  Sequence values(first), tail(second);
  values.Append(10); tail.Append(99);
  const bool negative = rejected([&] { values.Split(-1, tail); });
  std::cout << "split-negative:" << negative << ':' << tail.First()
            << ':' << (tail.Allocator() == second) << '\n';
  const bool zero = rejected([&] { values.Split(0, tail); });
  std::cout << "split-zero:" << zero << ':' << tail.IsEmpty()
            << ':' << (tail.Allocator() == first) << '\n';
  values.Split(1, tail);
  std::cout << "split-valid:" << values.IsEmpty() << ':' << tail.First() << '\n';
}

void empty_splice()
{
  Sequence values, empty;
  values.Append(10);
  int failures = rejected([&] { values.InsertAfter(-1, empty); });
  failures += rejected([&] { values.InsertBefore(-1, empty); });
  failures += rejected([&] { values.InsertBefore(0, empty); });
  values.InsertAfter(std::numeric_limits<int>::max(), empty);
  values.InsertBefore(std::numeric_limits<int>::max(), empty);
  std::cout << "empty-splice:" << failures << ':' << values.Size() << ':' << values.First() << '\n';
}

void assignment_contract()
{
  occ::handle<NCollection_BaseAllocator> first = new NCollection_IncAllocator(128);
  occ::handle<NCollection_BaseAllocator> second = new NCollection_IncAllocator(128);
  Sequence source(first), target(second);
  source.Append(10);
  const int* pointer = &source.First();
  const bool returns_copy = &target.Assign(source) == &target;
  std::cout << "copy:" << returns_copy << ':' << (target.Allocator() == second)
            << ':' << (&target.First() != pointer) << '\n';
  const bool returns_move = &(target = std::move(source)) == &target;
  std::cout << "move:" << returns_move << ':' << (target.Allocator() == first)
            << ':' << (source.Allocator() == first) << ':' << source.IsEmpty()
            << ':' << (&target.First() == pointer) << '\n';
}

void self_branches()
{
  Sequence values = sequence();
  const int* pointer = &values.First();
  const bool copy = &values.Assign(values) == &values;
  const bool move = &(values = std::move(values)) == &values;
  values.Append(values); values.Prepend(values);
  values.InsertAfter(std::numeric_limits<size_t>::max(), values);
  values.InsertBefore(std::numeric_limits<size_t>::max(), values);
  const bool zero = rejected([&] { values.InsertBefore(size_t(0), values); });
  std::cout << "self:" << copy << ':' << move << ':' << zero << ':' << values.Size()
            << ':' << (&values.First() == pointer) << '\n';
  const bool split = rejected([&] { values.Split(size_t(1), values); });
  std::cout << "self-split:" << split << ':' << values.IsEmpty() << '\n';
}

void cursor_steps()
{
  Sequence values = sequence();
  Sequence::Iterator cursor(values);
  auto first = cursor++;
  const int initial = first.Value(), second = cursor.Value();
  const bool prefix = &(++cursor) == &cursor;
  auto last = cursor--;
  const int old_last = last.Value(), new_second = cursor.Value();
  const bool retreat = &(--cursor) == &cursor;
  std::cout << "cursor:" << initial << ':' << second << ':' << prefix << ':' << old_last
            << ':' << new_second << ':' << retreat << ':' << cursor.Value() << '\n';
  cursor.Initialize(values, false);
  Sequence::Iterator copy;
  cursor.CopyTo(copy);
  std::cout << "cursor-copy:" << copy.IsEqual(cursor) << ':' << (--copy).Value() << '\n';
}

void iterator_end_identity()
{
  Sequence values = sequence(), other;
  other.Append(99);
  Sequence::Iterator cursor(values, false), copy;
  const bool assigned = &(copy = cursor) == &copy;
  const bool self = &(copy = copy) == &copy;
  const int last = (--copy).Value();
  ++cursor;
  std::cout << "cursor-assign:" << assigned << ':' << self << ':' << last
            << ':' << (cursor == other.end()) << ':' << (--cursor).Value() << '\n';
  auto position = values.cbegin(), clone = position;
  const bool same = position == clone;
  ++position;
  std::cout << "iterator:" << same << ':' << (position != clone)
            << ':' << (values.cend() == other.cend()) << '\n';
}

int main()
{
  signed_rejections(); signed_positive(); emplacement_validation(); split_order();
  empty_splice(); assignment_contract(); self_branches(); cursor_steps(); iterator_end_identity();
}
