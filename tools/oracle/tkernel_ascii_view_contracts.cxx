#include <TCollection_AsciiString.hxx>
#include <iostream>
#include <string_view>

static void Print(const char* label, const TCollection_AsciiString& text)
{
  std::cout << label << ' ' << text.Length();
  for (int index = 1; index <= text.Length(); ++index)
    std::cout << ' ' << static_cast<unsigned int>(static_cast<unsigned char>(text.Value(index)));
  std::cout << '\n';
}

int main()
{
  const char bytes[] = {'a', '\0', 'b'};
  const std::string_view view(bytes, 3);
  Print("constructor", TCollection_AsciiString(view));
  Print("bounded_constructor", TCollection_AsciiString(bytes, 3));
  TCollection_AsciiString copied;
  copied.Copy(view);
  Print("copy_view", copied);
  copied.Copy(bytes, 3);
  Print("copy_span", copied);
  TCollection_AsciiString assigned("before");
  assigned = view;
  Print("assign_view", assigned);
  TCollection_AsciiString appended("X");
  appended.AssignCat(view);
  Print("append_view", appended);
  Print("cat_view", TCollection_AsciiString("X").Cat(view));
  Print("add_view", TCollection_AsciiString("X") + view);
  TCollection_AsciiString plusAssigned("X");
  plusAssigned += view;
  Print("plus_assign_view", plusAssigned);
  Print("zero_prefix_constructor", TCollection_AsciiString(std::string_view(bytes + 1, 2)));
  return 0;
}
