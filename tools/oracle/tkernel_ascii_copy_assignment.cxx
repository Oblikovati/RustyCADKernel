#include <TCollection_AsciiString.hxx>
#include <iostream>
#include <string_view>

TCollection_AsciiString explicit_span(std::string_view bytes)
{
  TCollection_AsciiString result;
  result.AssignCat(bytes);
  return result;
}

void assignment_case(const char* name, std::string_view before, std::string_view input)
{
  auto destination = explicit_span(before);
  const auto source = explicit_span(input);
  const char* prior = destination.ToCString();
  auto& returned = (destination = source);
  const bool must_reuse = !input.empty() && input.size() <= before.size();
  std::cout << name << ":" << (&returned == &destination) << ":"
            << (!must_reuse || prior == destination.ToCString()) << ":";
  for (int i = 1; i <= destination.Length(); ++i)
    std::cout << static_cast<unsigned int>(static_cast<unsigned char>(destination.Value(i))) << ",";
  std::cout << ":" << source.Length() << "\n";
}

void self_assignment_case()
{
  auto destination = explicit_span(std::string_view("h\0i", 3));
  const char* before = destination.ToCString();
  auto& returned = (destination = destination);
  std::cout << "self:" << (&returned == &destination) << ":"
            << (before == destination.ToCString()) << ":" << destination.Length() << "\n";
}

int main()
{
  assignment_case("same_length", "old", "new");
  assignment_case("shorter", "longer", "x");
  assignment_case("longer", "x", "longer");
  assignment_case("embedded", "longer", std::string_view("x\0y", 3));
  assignment_case("empty_source", "longer", "");
  assignment_case("empty_destination", "", "new");
  self_assignment_case();
}
