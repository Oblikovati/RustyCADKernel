#include <TCollection_AsciiString.hxx>

#include <iostream>

int main()
{
  TCollection_AsciiString text("ab--a");
  text.Insert(2, '\0');
  const char* const cStringNeedle = "b\0a";
  const char fromEndNeedle[] = "a\0b";
  const char inSet[] = {'x', '\0', 'z', '\0'};
  const char notInSet[] = {'a', '\0', 'z', '\0'};
  TCollection_AsciiString shortText("xay");
  shortText.Insert(2, '\0');

  std::cout << text.Search(cStringNeedle) << ' ' << text.Search(cStringNeedle, 3) << ' '
            << text.SearchFromEnd(fromEndNeedle) << ' ' << text.SearchFromEnd(fromEndNeedle, 3) << '\n';
  std::cout << shortText.FirstLocationNotInSet("x", 1, 3) << ' '
            << shortText.FirstLocationNotInSet("x\0", 2, 1, 3) << '\n';
  std::cout << text.FirstLocationInSet(inSet, 1, 6) << ' ' << text.FirstLocationNotInSet(notInSet, 1, 6)
            << '\n';
  std::cout << text.Search(static_cast<const char*>(nullptr)) << ' '
            << text.SearchFromEnd(static_cast<const char*>(nullptr)) << '\n';
  std::cout << text.FirstLocationInSet(nullptr, 0, 0, -1) << ' '
            << text.FirstLocationNotInSet(nullptr, 0, 0, -1) << '\n';
}
