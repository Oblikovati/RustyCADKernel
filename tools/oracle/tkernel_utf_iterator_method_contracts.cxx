#include <NCollection_UtfIterator.hxx>
#include <iostream>

static void nullState() {
  NCollection_UtfIterator<char> iterator(nullptr);
  std::cout << "null " << (iterator.BufferHere() == nullptr) << ' '
            << (iterator.BufferNext() == nullptr) << ' ' << unsigned(*iterator)
            << ' ' << iterator.Index() << ' ' << iterator.IsValid() << '\n';
  iterator.Init("ab");
  ++iterator;
  iterator.Init(nullptr);
  std::cout << "reset " << (iterator.BufferHere() == nullptr) << ' '
            << (iterator.BufferNext() == nullptr) << ' ' << unsigned(*iterator)
            << ' ' << iterator.Index() << '\n';
}

static void sliceIdentity() {
  const char bytes[] = "a\xc3\xa9z";
  NCollection_UtfIterator<char> complete(bytes);
  ++complete;
  NCollection_UtfIterator<char> suffix(bytes + 1);
  std::cout << "identity " << (complete == suffix) << ' ' << complete.Index() << ' '
            << suffix.Index() << ' ' << (complete.BufferHere() - bytes) << ' '
            << (complete.BufferNext() - bytes) << ' ' << unsigned(*complete) << '\n';
}

static void wideIdentity() {
  const char16_t units[] = {0xd83d, 0xde00, 0x41, 0};
  NCollection_UtfIterator<char16_t> iterator(units);
  const auto before = iterator++;
  NCollection_UtfIterator<char16_t> suffix(units + 2);
  std::cout << "wide " << (before.BufferNext() - units) << ' ' << unsigned(*before)
            << ' ' << (iterator.BufferHere() - units) << ' ' << (iterator == suffix) << '\n';
}

static void outputPosition() {
  const char32_t scalar[] = {0x1f600, 0};
  NCollection_UtfIterator<char32_t> iterator(scalar);
  unsigned char output[] = {7, 7, 7, 7, 7, 7};
  const auto remaining = iterator.GetUtf8(output);
  std::cout << "output " << (remaining - output);
  for (unsigned char byte : output) { std::cout << ' ' << unsigned(byte); }
  std::cout << '\n';
  const char32_t illegal[] = {0x110000, 0};
  iterator.Init(illegal);
  char16_t wide[] = {9, 9};
  std::cout << "invalid " << (iterator.GetUtf(wide) - wide) << ' '
            << unsigned(wide[0]) << ' ' << unsigned(wide[1]) << '\n';
}

int main() {
  nullState();
  sliceIdentity();
  wideIdentity();
  outputPosition();
}
