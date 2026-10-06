// OCCT is the oracle; each line is reproduced from live Rust cursor operations.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <type_traits>
#include <utility>
struct Item { int value; };
using Array = NCollection_DynamicArray<Item>;
static_assert(std::is_same_v<decltype(std::declval<const Array::iterator>().Value()), Item&>);
static_assert(std::is_same_v<decltype(std::declval<const Array::Iterator>().Value()), const Item&>);
static_assert(std::is_same_v<decltype(std::declval<const Array::const_iterator>().Value()), const Item&>);
static_assert(!std::is_constructible_v<Array::iterator, Array::const_iterator>);
static Array fixture()
{
  Array array(2);
  for (int value : {10, 20, 30, 40, 50}) array.Append(Item{value});
  return array;
}
static void mutableAccess(Array& array)
{
  const Array::iterator cursor(array, 1);
  cursor.Value().value = 21;
  cursor->value += 2;
  cursor[2].value = 44;
  cursor[-1].value = 11;
  std::cout << "mutable " << array[0].value << ' ' << array[1].value << ' ' << array[3].value << '\n';
  std::cout << "arrow " << (cursor.operator->() == &array[1]) << ' '
            << (Array::iterator(array, array.Size()).operator->() == nullptr) << '\n';
}
static void arithmetic(Array& array)
{
  Array::iterator cursor(array);
  auto position = 3 + cursor;
  position -= 1;
  position += -1;
  auto later = position + 3;
  auto back = later - 2;
  std::cout << "arithmetic " << position->value << ' ' << later->value << ' ' << back->value << ' '
            << (later - position) << '\n';
  const Array& constant = array;
  Array::const_iterator start(constant), end(constant, constant.Size());
  std::cout << "const_arithmetic " << (2 + start)->value << ' ' << (end - 1)->value << ' '
            << (end - cursor) << ' ' << (cursor - end) << '\n';
}
static void constructors(Array& array)
{
  Array::iterator cursor(array, 2);
  Array::const_iterator converted(cursor), assigned;
  const bool assignment = &(assigned = cursor) == &assigned;
  Array::iterator copied(cursor), copyAssigned;
  const bool copiedAssignment = &(copyAssigned = copied) == &copyAssigned;
  array.Append(Item{60});
  std::cout << "conversion " << converted->value << ' ' << assignment << ' ' << (assigned == cursor)
            << ' ' << copiedAssignment << ' ' << (copyAssigned == cursor) << '\n';
  std::cout << "captured " << (converted + 3).More() << ' ' << (assigned + 3).More()
            << ' ' << (copied + 3).More() << '\n';
  Array::const_iterator fresh(array);
  std::cout << "fresh " << fresh[5].value << ' ' << fresh[1].value << '\n';
}
static void emptyPointers()
{
  Array array;
  const Array& constant = array;
  Array::iterator mutableCursor(array), defaultMutable;
  Array::const_iterator constCursor(constant), defaultConst;
  std::cout << "empty " << (mutableCursor.operator->() == nullptr) << ' '
            << (constCursor.operator->() == nullptr) << ' ' << (defaultMutable == defaultConst) << '\n';
}
int main()
{
  Array array = fixture();
  mutableAccess(array);
  arithmetic(array);
  constructors(array);
  emptyPointers();
}
