#include <TCollection_ExtendedString.hxx>

#include <iostream>

void print_units(const char* label, const TCollection_ExtendedString& value)
{
  std::cout << label << ' ' << value.Length();
  for (int index = 0; index < value.Length(); ++index)
  {
    std::cout << ' ' << static_cast<unsigned int>(value.ToExtString()[index]);
  }
  std::cout << '\n';
}

int main()
{
  const char16_t mixed[] = {u'A', u'\0', u'B', u'\0'};
  const char16_t empty[] = {u'\0'};
  const char16_t* nullPointer = nullptr;
  TCollection_ExtendedString value(u"old");
  value.Copy(mixed);
  print_units("pointer", value);
  value.Copy(mixed, 3);
  print_units("span", value);
  TCollection_ExtendedString& assigned = (value = mixed);
  std::cout << "assignment-reference " << (&assigned == &value) << '\n';
  print_units("assignment", value);
  value.Copy(nullPointer);
  print_units("null", value);
  value.Copy(empty);
  print_units("empty", value);
}
