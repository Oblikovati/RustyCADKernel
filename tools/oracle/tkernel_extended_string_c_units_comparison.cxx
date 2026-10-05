#include <TCollection_ExtendedString.hxx>

#include <iostream>

int main()
{
  const char16_t mixed[] = {u'A', u'\0', u'B', u'\0'};
  const char16_t suffix[] = {u'B', u'\0', u'A', u'\0'};
  const char16_t* nullPointer = nullptr;
  const TCollection_ExtendedString empty;
  const TCollection_ExtendedString one(u"A");
  const TCollection_ExtendedString two(u"AB");
  std::cout << "pointer " << one.IsEqual(mixed) << ' ' << one.IsDifferent(mixed) << ' '
            << one.IsLess(mixed) << ' ' << one.IsGreater(mixed) << '\n';
  std::cout << "operators " << (one == mixed) << ' ' << (one != mixed) << ' '
            << (one < mixed) << ' ' << (one > mixed) << '\n';
  std::cout << "prefix-order " << two.IsLess(mixed) << ' ' << two.IsGreater(mixed) << '\n';
  std::cout << "span " << one.IsEqual(mixed, 3) << ' ' << one.IsLess(mixed, 3) << ' '
            << two.StartsWith(mixed, 3) << ' ' << two.EndsWith(mixed, 3) << '\n';
  std::cout << "affix " << two.StartsWith(mixed) << ' ' << two.EndsWith(mixed) << ' '
            << two.EndsWith(suffix) << '\n';
  std::cout << "null-empty " << empty.IsEqual(nullPointer) << ' '
            << empty.IsDifferent(nullPointer) << ' ' << empty.IsLess(nullPointer) << ' '
            << empty.IsGreater(nullPointer) << ' ' << empty.StartsWith(nullPointer) << ' '
            << empty.EndsWith(nullPointer) << '\n';
  std::cout << "null-nonempty " << one.IsEqual(nullPointer) << ' '
            << one.IsDifferent(nullPointer) << ' ' << one.IsLess(nullPointer) << ' '
            << one.IsGreater(nullPointer) << ' ' << one.StartsWith(nullPointer) << ' '
            << one.EndsWith(nullPointer) << '\n';
}
