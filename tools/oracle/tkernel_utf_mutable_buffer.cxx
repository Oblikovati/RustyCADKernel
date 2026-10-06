#include <NCollection_UtfIterator.hxx>
#include <iostream>

template <typename T>
void emit(const char* name, const NCollection_UtfIterator<T>& cursor, const T* base)
{
  std::cout << name << ' ' << static_cast<unsigned int>(*cursor) << ' '
            << cursor.Index() << ' ' << (cursor.BufferHere() - base) << ' '
            << (cursor.BufferNext() - base) << '\n';
}
int main()
{
  unsigned char bytes[] = {0xc3, 0xa9, 'b', 0, 0, 0, 0, 0};
  NCollection_UtfIterator<unsigned char> cursor(bytes);
  emit("initial", cursor, bytes);
  cursor.ChangeBufferHere()[0] = 'x';
  cursor.ChangeBufferHere()[1] = 'y';
  emit("cached", cursor, bytes);
  ++cursor;
  emit("cached_next", cursor, bytes);
  cursor.ChangeBufferHere()[1] = 'z';
  ++cursor;
  emit("edited_next", cursor, bytes);
  cursor.Init(bytes);
  emit("rewound", cursor, bytes);
  char16_t halves[] = {0xd83d, 0xde00, 'q', 0, 0};
  NCollection_UtfIterator<char16_t> wide(halves);
  emit("wide_initial", wide, halves);
  wide.ChangeBufferHere()[1] = 'r';
  emit("wide_cached", wide, halves);
  wide.Init(halves);
  emit("wide_rewound", wide, halves);
  ++wide;
  emit("wide_next", wide, halves);
  NCollection_UtfIterator<char> empty(nullptr);
  std::cout << "null " << (empty.ChangeBufferHere() == nullptr) << ' '
            << static_cast<unsigned int>(*empty) << ' ' << empty.Index() << '\n';
}
