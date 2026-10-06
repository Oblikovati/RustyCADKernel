#include <NCollection_UtfString.hxx>
#include <iostream>
using Text = NCollection_UtfString<char>;
static void pointers() {
  Text text("A\xC3\xA9\xF0\x9F\x98\x80");
  std::cout << "pointers " << (text.GetCharBuffer(1) - text.ToCString()) << ' ' << (text.GetCharBuffer(2) - text.ToCString()) << ' ' << (text.GetCharBuffer(-1) == nullptr) << ' ' << (text.GetCharBuffer(3) == nullptr) << '\n';
}
static void moving() {
  NCollection_UtfString<char16_t> source("abc"); auto pointer = source.ToCString();
  NCollection_UtfString<char16_t> moved(std::move(source));
  std::cout << "move_constructor " << (pointer == moved.ToCString()) << ' ' << (source.ToCString() == nullptr) << ' ' << source.Size() << ' ' << source.Length() << '\n';
  source.Clear(); std::cout << "clear_moved " << (source.ToCString() != nullptr) << ' ' << int(*source.ToCString()) << '\n';
}
static void move_assignment() {
  Text first("first"), second("two"); auto left = first.ToCString(); auto right = second.ToCString();
  bool returned = &(first = std::move(second)) == &first;
  std::cout << "move_assignment " << returned << ' ' << (first.ToCString() == right) << ' ' << (second.ToCString() == left) << ' ' << first.ToCString() << ' ' << second.ToCString() << '\n';
}
static void copy_assignment() {
  const char16_t invalid[] = {0xD800, 0}; Text source(invalid), destination("old");
  bool returned = &destination.Assign(source) == &destination;
  auto pointer = destination.ToCString(); destination.Assign(destination);
  std::cout << "copy " << returned << ' ' << destination.Size() << ' ' << destination.Length() << ' ' << (destination.ToCString() == pointer) << '\n';
}
static void self_unicode() {
  Text text("A\xC3\xA9\xF0\x9F\x98\x80z");
  text.FromUnicode(text.GetCharBuffer(1), 2);
  std::cout << "self_unicode " << text.Size() << ' ' << text.Length() << ' ' << unsigned(text.GetChar(0)) << ' ' << unsigned(text.GetChar(1)) << '\n';
  text.FromUnicode(static_cast<const char*>(nullptr));
  std::cout << "nullable " << text.Size() << ' ' << text.Length() << ' ' << (text.ToCString() != nullptr) << '\n';
}
static void append() {
  Text text("ab"), suffix("c"); bool returned = &(text += suffix) == &text; text += text;
  std::cout << "append " << returned << ' ' << text.ToCString() << ' ' << text.Length() << '\n';
  const char16_t invalid[] = {0xD800, 0}; Text empty(invalid); auto pointer = empty.ToCString(); empty += empty;
  std::cout << "invalid_append " << empty.Size() << ' ' << empty.Length() << ' ' << (pointer == empty.ToCString()) << '\n';
}
int main() { pointers(); moving(); move_assignment(); copy_assignment(); self_unicode(); append(); }
