#include <TCollection_AsciiString.hxx>

#include <iomanip>
#include <iostream>

static void dump(const char* label, const TCollection_AsciiString& value)
{
  std::cout << label << ' ' << value.Length() << ' ';
  for (int index = 1; index <= value.Length(); ++index)
    std::cout << std::hex << std::setw(2) << std::setfill('0')
              << static_cast<unsigned int>(static_cast<unsigned char>(value.Value(index)));
  std::cout << std::dec << '\n';
}

int main()
{
  const char array[] = {'x', '\0', 'y', '\0'};
  TCollection_AsciiString assigned("head");
  assigned.AssignCat(array);
  TCollection_AsciiString added("head");
  added += array;
  const TCollection_AsciiString base("head");
  dump("assign", assigned);
  dump("plus_assign", added);
  dump("cat", base.Cat(array));
  dump("plus", base + array);
  TCollection_AsciiString span("head");
  span.AssignCat(array, 3);
  dump("span", span);
}
