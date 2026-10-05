#include <TCollection_ExtendedString.hxx>
#include <iostream>

static void print(const char* name, const TCollection_ExtendedString& value)
{
  std::cout << name << ':';
  for (int index = 1; index <= value.Length(); ++index)
  {
    std::cout << ' ' << static_cast<unsigned int>(value.Value(index));
  }
  std::cout << '\n';
}

static void prependFrom(TCollection_ExtendedString& value, int start)
{
  value.Prepend(value.ToExtString() + start);
}

int main()
{
  TCollection_ExtendedString small(u"ABC");
  prependFrom(small, 1);
  print("interior", small);

  TCollection_ExtendedString embedded(u"AB\0CD", 5);
  prependFrom(embedded, 3);
  print("embedded_suffix", embedded);

  TCollection_ExtendedString full(u"ABCDEFGH");
  prependFrom(full, 0);
  print("whole_buffer", full);
  prependFrom(full, full.Length());
  print("terminator", full);
}
