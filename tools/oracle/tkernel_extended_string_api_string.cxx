// TCollection_ExtendedString construction, conversion and query cases (included by
// tkernel_extended_string_api_probe.cxx after its helpers).
namespace extended_string_api_probe
{
// A char16_t buffer as the template (const char16_t(&)[N]) constructor sees it.
inline TCollection_ExtendedString from_literal(const Units& theUnits)
{
  char16_t aBuffer[40] = {};
  for (std::size_t i = 0; i < theUnits.size() && i < 39; ++i)
    aBuffer[i] = theUnits[i];
  return TCollection_ExtendedString(aBuffer);
}

inline std::string wide_case(std::istream& theInput)
{
  const std::string    aBytes = tkernel_probe::next_bytes(theInput);
  std::vector<wchar_t> aWide(aBytes.size() / 4 + 1, 0);
  std::memcpy(aWide.data(), aBytes.data(), aBytes.size() / 4 * 4);
  return describe(TCollection_ExtendedString(aWide.data()));
}

static const std::map<std::string, SubCase> THE_NEW = {
  {"empty",
   [](std::istream&) {
     return describe(TCollection_ExtendedString()) + " "
            + describe(TCollection_ExtendedString::EmptyString());
   }},
  {"cstr",
   [](std::istream& theInput) {
     const std::string aBytes = tkernel_probe::next_bytes(theInput);
     const bool        aMulti = next<int>(theInput) != 0;
     return describe(TCollection_ExtendedString(aBytes.c_str(), aMulti));
   }},
  {"ascii",
   [](std::istream& theInput) {
     const TCollection_AsciiString anAscii = ascii_of(tkernel_probe::next_bytes(theInput));
     const bool                    aMulti  = next<int>(theInput) != 0;
     return describe(TCollection_ExtendedString(anAscii, aMulti));
   }},
  {"u16",
   [](std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     return describe(TCollection_ExtendedString(aUnits.c_str())) + " "
            + describe(from_literal(aUnits));
   }},
  {"span",
   [](std::istream& theInput) {
     const Span aSpan   = next_span(theInput);
     const int  aLength = next<int>(theInput);
     return guarded_kind(
       [&] { return describe(TCollection_ExtendedString(aSpan.Pointer(), aLength)); });
   }},
  {"view",
   [](std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     return describe(TCollection_ExtendedString(std::u16string_view(aUnits)));
   }},
  {"wide", wide_case},
  {"char",
   [](std::istream& theInput) {
     return describe(TCollection_ExtendedString(char(next<int>(theInput))));
   }},
  {"unit",
   [](std::istream& theInput) {
     return describe(TCollection_ExtendedString(char16_t(next<int>(theInput))));
   }},
  {"fill",
   [](std::istream& theInput) {
     const int      aLength = next<int>(theInput);
     const char16_t aFiller = char16_t(next<int>(theInput));
     return guarded_kind([&] { return describe(TCollection_ExtendedString(aLength, aFiller)); });
   }},
  {"int",
   [](std::istream& theInput) { return describe(TCollection_ExtendedString(next<int>(theInput))); }},
  {"real",
   [](std::istream& theInput) { return describe(TCollection_ExtendedString(next_real(theInput))); }},
};

inline std::string new_case(std::istream& theInput)
{
  return dispatch(THE_NEW, theInput);
}

// Every UTF-8 view of an explicit code-unit sequence (zero units and surrogates included).
inline std::string convert_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  std::vector<char>                aBuffer(std::size_t(aString.LengthOfCString()) + 1, '\x7f');
  char*                            aWrite   = aBuffer.data();
  const int                        aWritten = aString.ToUTF8CString(aWrite);
  std::ostringstream               aPrinted, aStreamed;
  aString.Print(aPrinted);
  aStreamed << aString;
  const TCollection_AsciiString anAscii(aString);
  return std::to_string(aString.LengthOfCString()) + " " + std::to_string(aWritten) + " "
         + tkernel_probe::hex(aBuffer.data(), aBuffer.size()) + " " + flag(aString.IsAscii())
         + " " + std::to_string(aString.HashCode()) + " "
         + std::to_string(std::hash<TCollection_ExtendedString>{}(aString)) + " "
         + tkernel_probe::hex(aPrinted.str()) + " " + tkernel_probe::hex(aStreamed.str()) + " "
         + tkernel_probe::hex(anAscii.ToCString(), std::size_t(anAscii.Length())) + " "
         + units_hex(aString.ToExtString(), aString.Length() + 1) + " "
         + units_hex(std::u16string_view(aString).data(), int(std::u16string_view(aString).size()));
}

inline std::string numeric_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  return flag(aString.IsIntegerValue()) + " "
         + guarded_kind([&] { return std::to_string(aString.IntegerValue()); }) + " "
         + flag(aString.IsRealValue()) + " " + flag(aString.IsRealValue(true)) + " "
         + guarded_kind([&] { return std::to_string(tkernel_probe::bits(aString.RealValue())); });
}

// Token with the default and an explicit index; separators end at their first zero.
inline std::string token_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  const Units                      aSeparators = next_units(theInput);
  const int                        aWhich      = next<int>(theInput);
  return describe(aString.Token(aSeparators.c_str(), aWhich)) + " "
         + describe(aString.Token(aSeparators.c_str()));
}

inline std::string locate_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  const TCollection_ExtendedString aSet(std::u16string_view(next_units(theInput)));
  const int                        aFrom = next<int>(theInput);
  const int                        aTo   = next<int>(theInput);
  return guarded_kind([&] { return std::to_string(aString.FirstLocationInSet(aSet, aFrom, aTo)); })
         + " " + guarded_kind([&] {
             return std::to_string(aString.FirstLocationNotInSet(aSet, aFrom, aTo));
           });
}

inline std::string value_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  const int                        anIndex = next<int>(theInput);
  return guarded_kind([&] { return std::to_string(int(aString.Value(anIndex))); });
}

// The (pointer, length) comparison and search cores; the corpus never names more units
// than the span holds, so no core reads past the buffer.
inline std::string span_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  const Span                       aSpan    = next_span(theInput);
  const int                        aLength  = next<int>(theInput);
  const char16_t*                  aPointer = aSpan.Pointer();
  return flag(aString.IsEqual(aPointer, aLength)) + flag(aString.IsDifferent(aPointer, aLength))
         + flag(aString.IsLess(aPointer, aLength)) + flag(aString.IsGreater(aPointer, aLength))
         + flag(aString.StartsWith(aPointer, aLength)) + flag(aString.EndsWith(aPointer, aLength))
         + " " + std::to_string(aString.Search(aPointer, aLength)) + " "
         + std::to_string(aString.SearchFromEnd(aPointer, aLength));
}

// The whole-string, char16_t* and operator forms of every comparison and search.
inline std::string whole_case(std::istream& theInput)
{
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  const Units                      anUnits = next_units(theInput);
  const TCollection_ExtendedString anOther(std::u16string_view{anUnits});
  const char16_t*                  aPtr = anUnits.c_str();
  return flag(aString.IsEqual(anOther)) + flag(aString == anOther) + flag(aString.IsEqual(aPtr))
         + flag(aString == aPtr) + flag(TCollection_ExtendedString::IsEqual(aString, anOther))
         + flag(aString.IsDifferent(anOther)) + flag(aString != anOther)
         + flag(aString.IsDifferent(aPtr)) + flag(aString != aPtr) + flag(aString.IsLess(anOther))
         + flag(aString < anOther) + flag(aString.IsLess(aPtr)) + flag(aString < aPtr)
         + flag(aString.IsGreater(anOther)) + flag(aString > anOther)
         + flag(aString.IsGreater(aPtr)) + flag(aString > aPtr) + flag(aString.StartsWith(anOther))
         + flag(aString.StartsWith(aPtr)) + flag(aString.EndsWith(anOther))
         + flag(aString.EndsWith(aPtr)) + flag(aString.IsSameString(anOther, true))
         + flag(aString.IsSameString(anOther, false)) + " " + std::to_string(aString.Search(anOther))
         + " " + std::to_string(aString.Search(aPtr)) + " "
         + std::to_string(aString.SearchFromEnd(anOther)) + " "
         + std::to_string(aString.SearchFromEnd(aPtr));
}
} // namespace extended_string_api_probe
