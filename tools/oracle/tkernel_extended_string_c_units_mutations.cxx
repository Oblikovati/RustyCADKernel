#include <TCollection_ExtendedString.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>

static void print(const char* name, const TCollection_ExtendedString& value)
{
  std::cout << name << ':';
  for (int i = 1; i <= value.Length(); ++i)
  {
    std::cout << ' ' << static_cast<unsigned int>(value.Value(i));
  }
  std::cout << '\n';
}

static void printError(const char* name, const TCollection_ExtendedString& value,
                       int position, const char16_t* input, bool insert)
{
  TCollection_ExtendedString copy = value;
  try
  {
    if (insert)
    {
      copy.Insert(position, input);
    }
    else
    {
      copy.SetValue(position, input);
    }
    std::cout << name << ":none\n";
  }
  catch (const Standard_OutOfRange& error)
  {
    std::cout << name << ":Standard_OutOfRange:" << error.what() << '\n';
  }
}

int main()
{
  const char16_t nulled[] = {65, 0, 66, 0};
  TCollection_ExtendedString base(u"XY");
  print("cat", base.Cat(nulled));
  TCollection_ExtendedString inserted = base;
  inserted.Insert(2, nulled);
  print("insert", inserted);
  TCollection_ExtendedString replaced = base;
  replaced.SetValue(2, nulled);
  print("set", replaced);
  TCollection_ExtendedString prepended = base;
  prepended.Prepend(nulled);
  print("prepend", prepended);
  TCollection_ExtendedString empty = base.Cat(u"");
  print("empty", empty);
  TCollection_ExtendedString unchanged = base;
  unchanged.Insert(1, static_cast<const char16_t*>(nullptr));
  unchanged.Insert(1, u"");
  unchanged.Prepend(static_cast<const char16_t*>(nullptr));
  unchanged.Prepend(u"");
  unchanged.SetValue(1, static_cast<const char16_t*>(nullptr));
  unchanged.SetValue(1, u"");
  print("null", unchanged);
  printError("insert_bad_null", base, 0, nullptr, true);
  printError("set_bad_null", base, 0, nullptr, false);
}
