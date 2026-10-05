// Pinned operator contracts: char NUL differs between AsciiString + and +=.
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <iostream>
#include <string_view>

template <typename Operand>
void ascii_case(const char* name, const Operand& operand,
                std::string_view prefix = "h")
{
  TCollection_AsciiString base;
  base += prefix;
  const auto joined = base + operand;
  auto assigned = base;
  assigned += operand;
  std::cout << name << ":";
  for (int i = 1; i <= joined.Length(); ++i)
    std::cout << static_cast<unsigned int>(static_cast<unsigned char>(joined.Value(i))) << ",";
  std::cout << ":";
  for (int i = 1; i <= assigned.Length(); ++i)
    std::cout << static_cast<unsigned int>(static_cast<unsigned char>(assigned.Value(i))) << ",";
  std::cout << ":" << base.Length() << "\n";
}

template <typename Operand>
void extended_case(const char* name, const Operand& operand)
{
  const TCollection_ExtendedString base("h");
  const auto joined = base + operand;
  std::cout << name << ":";
  for (int i = 1; i <= joined.Length(); ++i)
    std::cout << static_cast<unsigned int>(joined.Value(i)) << ",";
  std::cout << ":" << base.Length() << "\n";
}

void ascii_cases()
{
  ascii_case("ascii_zero", static_cast<char>(0));
  ascii_case("ascii_char", 'x');
  ascii_case("ascii_int", -42);
  ascii_case("ascii_real", 1.25);
  TCollection_AsciiString embedded;
  embedded += std::string_view("x\0y", 3);
  ascii_case("ascii_string", embedded);
  ascii_case("ascii_extended", TCollection_ExtendedString(u"\u03bb"));
  ascii_case("ascii_null", static_cast<const char*>(nullptr));
  ascii_case("ascii_empty", static_cast<const char*>(""));
  ascii_case("ascii_pointer", static_cast<const char*>("x\0y"));
  ascii_case("ascii_array", "x\0y");
  ascii_case("ascii_span", std::string_view("x\0y", 3));
  ascii_case("ascii_embedded_base", static_cast<const char*>("x\0y"),
             std::string_view("h\0i", 3));
}

void extended_cases()
{
  extended_case("extended_zero", static_cast<char>(0));
  extended_case("extended_char", 'x');
  extended_case("extended_int", -42);
  extended_case("extended_real", 1.25);
  extended_case("extended_unit_zero", static_cast<char16_t>(0));
  extended_case("extended_unit", static_cast<char16_t>(0x03bb));
  extended_case("extended_string", TCollection_ExtendedString(std::u16string_view(u"x\0y", 3)));
}

int main()
{
  ascii_cases();
  extended_cases();
}
