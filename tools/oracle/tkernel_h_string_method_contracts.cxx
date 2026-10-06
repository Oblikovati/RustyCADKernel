#include <TCollection_HAsciiString.hxx>
#include <TCollection_HExtendedString.hxx>
#include <Standard_NullObject.hxx>
#include <iostream>
#include <functional>
#include <string_view>
using ASCII = occ::handle<TCollection_HAsciiString>;
static void ascii_pointer() {
  ASCII text = new TCollection_HAsciiString("abc"); ASCII alias = text;
  bool wrapped = text->ToCString() == text->String().ToCString();
  bool shared = text->ToCString() == alias->ToCString(); alias->Clear();
  TCollection_HAsciiString empty;
  std::cout << "ascii_pointer " << wrapped << ' ' << shared << ' ' << (text->ToCString() == empty.ToCString()) << '\n';
}
static void extended_pointer() {
  occ::handle<TCollection_HExtendedString> text = new TCollection_HExtendedString("abc"); auto alias = text;
  bool wrapped = text->ToExtString() == text->String().ToExtString();
  bool shared = text->ToExtString() == alias->ToExtString(); alias->Clear();
  TCollection_HExtendedString empty;
  std::cout << "extended_pointer " << wrapped << ' ' << shared << ' ' << (text->ToExtString() == empty.ToExtString()) << '\n';
}
static void moving() {
  TCollection_AsciiString ascii("abc"); const char* first = ascii.ToCString();
  TCollection_HAsciiString wrapped_ascii(std::move(ascii));
  TCollection_ExtendedString extended("abc"); const char16_t* second = extended.ToExtString();
  TCollection_HExtendedString wrapped_extended(std::move(extended));
  std::cout << "moving " << (wrapped_ascii.ToCString() == first) << ' ' << ascii.Length() << ' ' << (wrapped_extended.ToExtString() == second) << ' ' << extended.Length() << '\n';
}
static void nullable() {
  int failures = 0;
  try { TCollection_HAsciiString text(static_cast<const char*>(nullptr)); } catch (const Standard_NullObject&) { ++failures; }
  try { TCollection_HExtendedString text(static_cast<const char*>(nullptr)); } catch (const Standard_NullObject&) { ++failures; }
  try { TCollection_HExtendedString text(static_cast<const char16_t*>(nullptr)); } catch (const Standard_NullObject&) { ++failures; }
  std::cout << "nullable " << failures << '\n';
}
static void hashing() {
  ASCII null; ASCII first = new TCollection_HAsciiString("abc"); ASCII second = new TCollection_HAsciiString("abc");
  std::hash<ASCII> hash; std::equal_to<ASCII> equal;
  std::cout << "hashing " << (first == second) << ' ' << equal(first, second) << ' ' << equal(null, null) << ' ' << equal(first, null) << ' ' << (hash(null) == 0) << ' ' << (hash(first) == hash(second)) << '\n';
  const char a[] = {'a', 0, 'b'}; const char b[] = {'a', 0, 'c'};
  TCollection_AsciiString a_string, b_string; a_string.Copy(a, 3); b_string.Copy(b, 3);
  first = new TCollection_HAsciiString(a_string);
  second = new TCollection_HAsciiString(b_string);
  std::cout << "embedded " << first->IsSameState(second) << ' ' << equal(first, second) << '\n';
}
int main() { ascii_pointer(); extended_pointer(); moving(); nullable(); hashing(); }
