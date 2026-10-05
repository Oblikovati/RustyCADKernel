#include <TCollection_ExtendedString.hxx>

#include <iomanip>
#include <iostream>
#include <string_view>

static void dump(const char* label, const TCollection_ExtendedString& value)
{
  std::cout << label << ' ' << value.Length() << ' ';
  for (int index = 1; index <= value.Length(); ++index)
    std::cout << std::hex << std::setw(4) << std::setfill('0')
              << static_cast<unsigned int>(value.Value(index));
  std::cout << std::dec << '\n';
}

int main()
{
  const char16_t input[] = {u'A', u'\0', u'B', u'\0'};
  const char16_t plain[] = {u'B', u'C', u'\0'};
  const char16_t* pointer = input;
  const char16_t* plain_pointer = plain;
  const std::u16string_view view(input, 3);
  TCollection_ExtendedString array_append(u"X");
  TCollection_ExtendedString pointer_append(u"X");
  TCollection_ExtendedString assign_array(u"X");
  TCollection_ExtendedString assign_pointer(u"X");
  TCollection_ExtendedString view_append(u"X");
  TCollection_ExtendedString array_plus(u"X");
  TCollection_ExtendedString plain_append(u"X");
  TCollection_ExtendedString null_append(u"X");
  TCollection_ExtendedString null_plus(u"X");
  const char16_t* null_pointer = nullptr;
  array_append += input;
  pointer_append += pointer;
  assign_array.AssignCat(input);
  assign_pointer.AssignCat(pointer);
  view_append += view;
  array_plus = array_plus + input;
  plain_append += plain_pointer;
  null_append += null_pointer;
  null_plus = null_plus + null_pointer;
  dump("array_append", array_append);
  dump("pointer_append", pointer_append);
  dump("assign_array", assign_array);
  dump("assign_pointer", assign_pointer);
  dump("view_append", view_append);
  dump("array_plus", array_plus);
  dump("plain_append", plain_append);
  dump("null_append", null_append);
  dump("null_plus", null_plus);
}
