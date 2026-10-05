#include <TCollection_ExtendedString.hxx>

#include <iostream>
#include <string_view>

int main()
{
  const char16_t data[] = {u'B', u'A', u'B', u'A', u'\0'};
  const char16_t mixed[] = {u'A', u'\0', u'B', u'\0'};
  const char16_t pair[] = {u'B', u'A', u'\0'};
  const char16_t empty[] = {u'\0'};
  const TCollection_ExtendedString value(std::u16string_view(data, 4));
  const char16_t* nullPointer = nullptr;
  std::cout << "mixed " << value.Search(mixed) << ' '
            << value.SearchFromEnd(mixed) << '\n';
  std::cout << "span " << value.Search(mixed, 3) << '\n';
  std::cout << "pair " << value.Search(pair) << ' '
            << value.SearchFromEnd(pair) << '\n';
  std::cout << "empty " << value.Search(empty) << '\n';
  std::cout << "null " << value.Search(nullPointer) << ' '
            << value.SearchFromEnd(nullPointer) << '\n';
}
