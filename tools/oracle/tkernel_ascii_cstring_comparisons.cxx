// Explicit-length strings retain embedded NUL; pointer arguments end at their first NUL.
#include <TCollection_AsciiString.hxx>
#include <iostream>
#include <string_view>

void check(const char* bytes, int length) {
  TCollection_AsciiString text;
  text.Copy(bytes, length);
  const char* candidates[] = {nullptr, "", "a\0tail", "b", "\xff"};
  for (const char* candidate : candidates) {
    std::cout << text.IsEqual(candidate) << text.IsDifferent(candidate)
              << text.IsLess(candidate) << text.IsGreater(candidate)
              << text.StartsWith(candidate) << '\n';
  }
}
int main() {
  check("", 0); check("a", 1); check("a\0b", 3); check("\xff", 1);
  TCollection_AsciiString text("a");
  std::cout << "literal=" << text.IsEqual("a\0tail")
            << " view=" << text.IsEqual(std::string_view("a\0tail", 6))
            << " suffix=" << text.EndsWith("a\0tail") << '\n';
}
