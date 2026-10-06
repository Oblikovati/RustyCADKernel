#include <NCollection_DataMap.hxx>
#include <TCollection_HAsciiString.hxx>
#include <TCollection_HExtendedString.hxx>
#include <iostream>

template <class T>
static void Probe(const char* label, const occ::handle<T>& first, const occ::handle<T>& second)
{
  const occ::handle<T> null;
  NCollection_DefaultHasher<occ::handle<T>> policy;
  NCollection_DataMap<occ::handle<T>, int> map;
  const bool inserted = map.Bind(first, 1);
  const bool duplicate = map.Bind(second, 2);
  map.Bind(null, 3);
  std::cout << label << ' ' << (first == second) << ' ' << policy(first, second)
            << ' ' << (policy(first) == policy(second)) << ' ' << policy(null)
            << ' ' << policy(null, null) << ' ' << policy(first, null)
            << ' ' << inserted << ' ' << duplicate << ' ' << map.Size()
            << ' ' << map.Find(first) << ' ' << map.Find(null) << '\n';
}

int main()
{
  occ::handle<TCollection_HAsciiString> ascii1 = new TCollection_HAsciiString("abc");
  occ::handle<TCollection_HAsciiString> ascii2 = new TCollection_HAsciiString("abc");
  Probe("ascii", ascii1, ascii2);
  occ::handle<TCollection_HExtendedString> ext1 = new TCollection_HExtendedString("abc");
  occ::handle<TCollection_HExtendedString> ext2 = new TCollection_HExtendedString("abc");
  Probe("extended", ext1, ext2);
  occ::handle<Standard_Transient> base1 = ascii1, base2 = ascii2;
  NCollection_DefaultHasher<occ::handle<Standard_Transient>> policy;
  NCollection_DataMap<occ::handle<Standard_Transient>, int> baseMap;
  baseMap.Bind(base1, 1);
  baseMap.Bind(base2, 2);
  std::cout << "base " << policy(base1, base2) << ' '
            << (policy(base1) == reinterpret_cast<size_t>(base1.get())) << ' '
            << baseMap.Size() << '\n';
  TCollection_AsciiString zero1, zero2;
  zero1.Copy("a\0b", 3); zero2.Copy("a\0c", 3);
  occ::handle<TCollection_HAsciiString> embedded1 = new TCollection_HAsciiString(zero1);
  occ::handle<TCollection_HAsciiString> embedded2 = new TCollection_HAsciiString(zero2);
  NCollection_DefaultHasher<occ::handle<TCollection_HAsciiString>> asciiPolicy;
  std::cout << "embedded " << embedded1->IsSameState(embedded2) << ' '
            << asciiPolicy(embedded1, embedded2) << '\n';
}
