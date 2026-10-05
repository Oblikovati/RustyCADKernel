#include <TCollection_ExtendedString.hxx>

#include <functional>
#include <iostream>
#include <string_view>

void print_hash(const char* theName, const TCollection_ExtendedString& theValue);

int main()
{
  const TCollection_ExtendedString embedded(std::u16string_view(u"A\0B", 3));
  const TCollection_ExtendedString same(std::u16string_view(u"A\0B", 3));
  const TCollection_ExtendedString different(std::u16string_view(u"A\0C", 3));
  const TCollection_ExtendedString prefix(std::u16string_view(u"A\0B\0", 4));
  const TCollection_ExtendedString high(std::u16string_view(u"A\u0100", 2));
  const TCollection_ExtendedString low(std::u16string_view(u"A\u00FF", 2));
  std::cout << "embedded_equal " << (embedded == same) << '\n';
  std::cout << "embedded_not_equal " << (embedded != different) << '\n';
  std::cout << "embedded_less " << (embedded < different) << '\n';
  std::cout << "embedded_greater " << (different > embedded) << '\n';
  std::cout << "embedded_prefix_less " << (embedded < prefix) << '\n';
  std::cout << "high_unit_greater " << (high > low) << '\n';
  const TCollection_ExtendedString empty;
  const TCollection_ExtendedString single(std::u16string_view(u"\u00E9", 1));
  const TCollection_ExtendedString pair(u"ab");
  print_hash("empty", empty);
  print_hash("single_high_byte", single);
  print_hash("two_units", pair);
  print_hash("embedded_zero", embedded);
  print_hash("high_unit", high);
}

void print_hash(const char* theName, const TCollection_ExtendedString& theValue)
{
  std::cout << "hash_" << theName << ' ' << theValue.HashCode() << ' '
            << std::hash<TCollection_ExtendedString>{}(theValue) << '\n';
}
