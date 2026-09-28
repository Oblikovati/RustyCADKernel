// TCollection_HExtendedString cases (included by tkernel_extended_string_api_probe.cxx
// after its helpers). Handles are never null here: OCCT dereferences them unchecked.
namespace extended_string_api_probe
{
using HString = occ::handle<TCollection_HExtendedString>;

inline HString next_handle(std::istream& theInput)
{
  return new TCollection_HExtendedString(
    TCollection_ExtendedString(std::u16string_view(next_units(theInput))));
}

inline std::string describe(const HString& theString)
{
  return describe(theString->String());
}

static const std::map<std::string, SubCase> THE_HNEW = {
  {"empty", [](std::istream&) { return describe(HString(new TCollection_HExtendedString())); }},
  {"cstr",
   [](std::istream& theInput) {
     const std::string aBytes = tkernel_probe::next_bytes(theInput);
     return describe(HString(new TCollection_HExtendedString(aBytes.c_str())));
   }},
  {"u16",
   [](std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     return describe(HString(new TCollection_HExtendedString(aUnits.c_str())));
   }},
  {"unit",
   [](std::istream& theInput) {
     return describe(HString(new TCollection_HExtendedString(char16_t(next<int>(theInput)))));
   }},
  {"fill",
   [](std::istream& theInput) {
     const int      aLength = next<int>(theInput);
     const char16_t aFiller = char16_t(next<int>(theInput));
     return guarded_kind(
       [&] { return describe(HString(new TCollection_HExtendedString(aLength, aFiller))); });
   }},
  {"str",
   [](std::istream& theInput) {
     const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
     TCollection_ExtendedString       aMoved(aString);
     return describe(HString(new TCollection_HExtendedString(aString))) + " "
            + describe(HString(new TCollection_HExtendedString(std::move(aMoved))));
   }},
  {"hascii",
   [](std::istream& theInput) {
     const occ::handle<TCollection_HAsciiString> anAscii =
       new TCollection_HAsciiString(ascii_of(tkernel_probe::next_bytes(theInput)));
     return describe(HString(new TCollection_HExtendedString(anAscii)));
   }},
  {"hcopy",
   [](std::istream& theInput) {
     const HString aSource = next_handle(theInput);
     const HString aCopy   = new TCollection_HExtendedString(aSource);
     aSource->AssignCat(aSource);
     return describe(aCopy) + " " + describe(aSource);
   }},
};

inline std::string hnew_case(std::istream& theInput)
{
  return dispatch(THE_HNEW, theInput);
}

using HEdit = std::function<std::string(const HString&, std::istream&)>;

static const std::map<std::string, HEdit> THE_HEDIT = {
  {"assign_cat",
   [](const HString& theString, std::istream& theInput) {
     theString->AssignCat(next_handle(theInput));
     return describe(theString);
   }},
  {"assign_self",
   [](const HString& theString, std::istream&) {
     theString->AssignCat(theString);
     return describe(theString);
   }},
  {"cat",
   [](const HString& theString, std::istream& theInput) {
     return describe(theString->Cat(next_handle(theInput))) + " " + describe(theString);
   }},
  {"change",
   [](const HString& theString, std::istream& theInput) {
     const char16_t aFrom = char16_t(next<int>(theInput));
     theString->ChangeAll(aFrom, char16_t(next<int>(theInput)));
     return describe(theString);
   }},
  {"clear",
   [](const HString& theString, std::istream&) {
     theString->Clear();
     return describe(theString) + " " + flag(theString->IsEmpty());
   }},
  {"insert_unit",
   [](const HString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString->Insert(anIndex, char16_t(next<int>(theInput)));
     return describe(theString);
   }},
  {"insert",
   [](const HString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString->Insert(anIndex, next_handle(theInput));
     return describe(theString);
   }},
  {"query",
   [](const HString& theString, std::istream& theInput) {
     const HString anOther = next_handle(theInput);
     return flag(theString->IsEmpty()) + flag(theString->IsLess(anOther))
            + flag(theString->IsGreater(anOther)) + flag(theString->IsAscii())
            + flag(theString->IsSameState(anOther)) + " " + std::to_string(theString->Length())
            + " " + std::to_string(theString->Search(anOther)) + " "
            + std::to_string(theString->SearchFromEnd(anOther)) + " "
            + units_hex(theString->ToExtString(), theString->Length() + 1);
   }},
  {"remove",
   [](const HString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString->Remove(anIndex, next<int>(theInput));
     return describe(theString);
   }},
  {"remove_one",
   [](const HString& theString, std::istream& theInput) {
     theString->Remove(next<int>(theInput));
     return describe(theString);
   }},
  {"remove_all",
   [](const HString& theString, std::istream& theInput) {
     theString->RemoveAll(char16_t(next<int>(theInput)));
     return describe(theString);
   }},
  {"set_unit",
   [](const HString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString->SetValue(anIndex, char16_t(next<int>(theInput)));
     return describe(theString);
   }},
  {"set",
   [](const HString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString->SetValue(anIndex, next_handle(theInput));
     return describe(theString);
   }},
  {"split",
   [](const HString& theString, std::istream& theInput) {
     // Split first: the operands of + are unsequenced.
     const HString aTail = theString->Split(next<int>(theInput));
     return describe(aTail) + " " + describe(theString);
   }},
  {"token",
   [](const HString& theString, std::istream& theInput) {
     const Units aSeparators = next_units(theInput);
     const int   aWhich      = next<int>(theInput);
     return describe(theString->Token(aSeparators.c_str(), aWhich)) + " "
            + describe(theString->Token(aSeparators.c_str()));
   }},
  {"trunc",
   [](const HString& theString, std::istream& theInput) {
     theString->Trunc(next<int>(theInput));
     return describe(theString);
   }},
  {"value",
   [](const HString& theString, std::istream& theInput) {
     return std::to_string(int(theString->Value(next<int>(theInput))));
   }},
  {"print",
   [](const HString& theString, std::istream&) {
     std::ostringstream aStream;
     theString->Print(aStream);
     return tkernel_probe::hex(aStream.str());
   }},
};

inline std::string hedit_case(std::istream& theInput)
{
  const std::string anOperation = next<std::string>(theInput);
  const HString     aString     = next_handle(theInput);
  return guarded_kind([&] { return THE_HEDIT.at(anOperation)(aString, theInput); });
}

// std::hash and std::equal_to of handles: null, shared and equal-content handles.
inline std::string hkey_case(std::istream& theInput)
{
  const HString                       aFirst  = next_handle(theInput);
  const HString                       aSecond = next_handle(theInput);
  const HString                       aNull;
  std::hash<HString>                  aHash;
  std::equal_to<HString>              anEqual;
  return std::to_string(aHash(aFirst)) + " " + std::to_string(aHash(aNull)) + " "
         + flag(anEqual(aFirst, aSecond)) + flag(anEqual(aFirst, aFirst))
         + flag(anEqual(aNull, aNull)) + flag(anEqual(aFirst, aNull))
         + flag(anEqual(aNull, aSecond));
}
} // namespace extended_string_api_probe
