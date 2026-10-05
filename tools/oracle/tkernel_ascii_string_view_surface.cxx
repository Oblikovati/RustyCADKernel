#include <TCollection_AsciiString.hxx>

#include <iomanip>
#include <iostream>
#include <string_view>

static void dump(std::string_view label, std::string_view value)
{
  std::cout << label << ' ' << value.size() << ' ';
  for (unsigned char byte : value)
    std::cout << std::hex << std::setw(2) << std::setfill('0')
              << static_cast<unsigned int>(byte);
  std::cout << std::dec << '\n';
}

int main()
{
  const char literal[] = {'a', 'b', '\0', 'c', 'd', '\0'};
  const TCollection_AsciiString fromLiteral(literal);
  const std::string_view input("a\0b", 3);
  const TCollection_AsciiString fromView(input);
  TCollection_AsciiString embedded("Az");
  embedded.Insert(2, '\0');
  const std::string_view lower("A\0a", 3);
  dump("literal", std::string_view(fromLiteral));
  dump("view_constructor", std::string_view(fromView));
  dump("converted", static_cast<std::string_view>(embedded));
  std::cout << "less " << (embedded < lower) << '\n';
  std::cout << "greater " << (embedded > lower) << '\n';
}
