#include <NCollection_DynamicArray.hxx>
#include <iostream>
using LegacyArray = NCollection_DynamicArray<int>;

void fill_cursor_array(LegacyArray& array)
{
  for (int value : {10, 20, 30, 40}) array.Append(value);
}

void captured_end()
{
  LegacyArray array(2);
  fill_cursor_array(array);
  LegacyArray::ConstIterator cursor(array);
  for (int value = 50; value < 80; ++value) array.Append(value);
  std::cout << "snapshot";
  while (cursor.More()) { std::cout << ':' << cursor.Value(); cursor.Next(); }
  std::cout << ':' << cursor.More() << ':' << (cursor.operator->() == nullptr);
  cursor.Initialize(array);
  std::cout << ':' << cursor[33] << '\n';
}

void mutable_conversion()
{
  LegacyArray array(2), other(2);
  fill_cursor_array(array);
  fill_cursor_array(other);
  LegacyArray::Iterator cursor(array);
  cursor.ChangeValue() = 99;
  LegacyArray::const_iterator constant(cursor);
  std::cout << "mutable:" << constant.Value() << ':' << (cursor == constant)
            << ':' << (constant != other.cbegin()) << ':' << (constant.operator->() == &array.Value(0)) << '\n';
}

void prefix_postfix()
{
  LegacyArray array(2);
  fill_cursor_array(array);
  auto cursor = array.cbegin();
  std::cout << "steps:" << (cursor++).Value() << ':' << (++cursor).Value()
            << ':' << (cursor--).Value() << ':' << (--cursor).Value();
  auto end = array.cend();
  std::cout << ':' << (--end).Value();
  end += 20;
  std::cout << ':' << (--end).Value() << '\n';
}

void offset_positions()
{
  LegacyArray array(2);
  fill_cursor_array(array);
  const auto begin = array.cbegin();
  auto cursor = begin + 3;
  std::cout << "offsets:" << (cursor - begin) << ':' << cursor[-2] << ':' << (cursor - 2).Value();
  std::cout << ':' << (cursor -= 2).Value() << ':' << (cursor += -1).Value()
            << ':' << ((begin + -1) - begin) << '\n';
  cursor += 3;
  std::cout << "ordering:" << (cursor > begin) << ':' << (cursor >= begin)
            << ':' << (begin < cursor) << ':' << (begin <= cursor) << '\n';
}

void assignment_positions()
{
  LegacyArray array(2);
  fill_cursor_array(array);
  auto mutable_cursor = array.begin();
  LegacyArray::const_iterator constant;
  auto& converted = (constant = mutable_cursor);
  const auto copied = constant;
  array.Append(50);
  auto& assigned = (constant = copied);
  auto& self_assigned = (constant = constant);
  std::cout << "assignment:" << (&converted == &constant) << ':' << !(copied + 4).More()
            << ':' << (&assigned == &constant) << ':' << (&self_assigned == &constant)
            << ':' << (constant == mutable_cursor) << '\n';
}

void default_and_owner_positions()
{
  LegacyArray empty, first(2), second(2);
  LegacyArray::ConstIterator detached;
  LegacyArray::ConstIterator another_detached;
  auto empty_end = empty.cend();
  std::cout << "default:" << detached.More() << ':' << (detached == another_detached)
            << ':' << (--empty_end).More() << '\n';
  fill_cursor_array(first);
  fill_cursor_array(second);
  auto left = first.cbegin();
  auto right = second.begin();
  std::cout << "owners:" << (left == right) << ':' << (left - right)
            << ':' << (left < right) << ':' << (left > right)
            << ':' << (left <= right) << ':' << (left >= right) << '\n';
}

int main()
{
  captured_end();
  mutable_conversion();
  prefix_postfix();
  offset_positions();
  assignment_positions();
  default_and_owner_positions();
}
