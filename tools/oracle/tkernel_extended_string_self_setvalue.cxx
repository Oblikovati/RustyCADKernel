#include <TCollection_ExtendedString.hxx>
#include <Standard_OutOfRange.hxx>

#include <iostream>
#include <string_view>

static void print_units(const char* label, const TCollection_ExtendedString& value)
{
  std::cout << label << ' ' << value.Length();
  for (int index = 1; index <= value.Length(); ++index)
  {
    std::cout << ' ' << std::hex << static_cast<unsigned int>(value.Value(index));
  }
  std::cout << std::dec << '\n';
}

static void print_overlap_results()
{
  TCollection_ExtendedString aAfterSource(std::u16string_view(u"abcd\0e", 6));
  aAfterSource.SetValue(2, aAfterSource.ToExtString());
  print_units("overlap_target_after_source", aAfterSource);

  TCollection_ExtendedString aBeforeSource(std::u16string_view(u"abcd\0e", 6));
  aBeforeSource.SetValue(1, aBeforeSource.ToExtString() + 2);
  print_units("overlap_target_before_source", aBeforeSource);

  TCollection_ExtendedString aTerminator(std::u16string_view(u"abcd\0e", 6));
  aTerminator.SetValue(7, aTerminator.ToExtString() + aTerminator.Length());
  print_units("terminator_source_noop", aTerminator);
}

static void print_invalid_position()
{
  TCollection_ExtendedString aValue(std::u16string_view(u"abcd\0e", 6));
  try
  {
    aValue.SetValue(0, aValue.ToExtString() + aValue.Length());
  }
  catch (const Standard_OutOfRange& anError)
  {
    std::cout << "invalid_empty_pointer Standard_OutOfRange " << anError.what() << '\n';
  }
}

int main()
{
  print_overlap_results();
  print_invalid_position();
}
