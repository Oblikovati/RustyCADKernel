// TCollection_ExtendedString append and in-place edit cases (included by
// tkernel_extended_string_api_probe.cxx after its helpers).
namespace extended_string_api_probe
{
using Edit = std::function<void(TCollection_ExtendedString&, std::istream&)>;

// "<AssignCat> <operator+=> <Cat> <operator+>" for one appended value.
template <class Value>
std::string append_all(const TCollection_ExtendedString& theString, const Value& theValue)
{
  TCollection_ExtendedString anAssigned(theString), anAdded(theString);
  anAssigned.AssignCat(theValue);
  anAdded += theValue;
  return describe(anAssigned) + " " + describe(anAdded) + " " + describe(theString.Cat(theValue))
         + " " + describe(theString + theValue);
}

static const std::map<std::string, std::function<std::string(const TCollection_ExtendedString&,
                                                              std::istream&)>>
  THE_APPEND = {
    {"str",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       return append_all(theString, TCollection_ExtendedString(std::u16string_view(next_units(theInput))));
     }},
    {"u16",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const Units aUnits = next_units(theInput);
       return append_all(theString, aUnits.c_str());
     }},
    {"int",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       return append_all(theString, next<int>(theInput));
     }},
    {"real",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       return append_all(theString, next_real(theInput));
     }},
    {"char",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       return append_all(theString, char(next<int>(theInput)));
     }},
    {"unit",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const char16_t anUnit = char16_t(next<int>(theInput));
       TCollection_ExtendedString anAssigned(theString);
       anAssigned.AssignCat(anUnit);
       return describe(anAssigned) + " " + describe(theString.Cat(anUnit)) + " "
              + describe(theString + anUnit);
     }},
    {"view",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const Units                aUnits = next_units(theInput);
       TCollection_ExtendedString anAssigned(theString), anAdded(theString);
       anAssigned.AssignCat(std::u16string_view(aUnits));
       anAdded += std::u16string_view(aUnits);
       return describe(anAssigned) + " " + describe(anAdded);
     }},
    {"span",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const Span                 aSpan   = next_span(theInput);
       const int                  aLength = next<int>(theInput);
       TCollection_ExtendedString anAssigned(theString), aPrepended(theString);
       anAssigned.AssignCat(aSpan.Pointer(), aLength);
       aPrepended.Prepend(aSpan.Pointer(), aLength);
       return describe(anAssigned) + " " + describe(aPrepended) + " "
              + guarded_kind([&] { return describe(theString.Cat(aSpan.Pointer(), aLength)); });
     }},
    {"prepend",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const Units                aUnits = next_units(theInput);
       TCollection_ExtendedString aByString(theString), aByPointer(theString);
       aByString.Prepend(TCollection_ExtendedString(std::u16string_view(aUnits)));
       aByPointer.Prepend(aUnits.c_str());
       return describe(aByString) + " " + describe(aByPointer);
     }},
    {"self",
     [](const TCollection_ExtendedString& theString, std::istream& theInput) {
       const int                  anOffset = next<int>(theInput);
       const int                  aLength  = next<int>(theInput);
       TCollection_ExtendedString aWhole(theString), anAppended(theString), aPrepended(theString);
       aWhole.AssignCat(aWhole);
       anAppended.AssignCat(anAppended.ToExtString() + anOffset, aLength);
       aPrepended.Prepend(aPrepended.ToExtString() + anOffset, aLength);
       return describe(aWhole) + " " + describe(anAppended) + " " + describe(aPrepended);
     }},
};

inline std::string append_case(std::istream& theInput)
{
  const std::string                aKind = next<std::string>(theInput);
  const TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  return THE_APPEND.at(aKind)(aString, theInput);
}

static const std::map<std::string, Edit> THE_EDIT = {
  {"insert_unit",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString.Insert(anIndex, char16_t(next<int>(theInput)));
   }},
  {"insert_str",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString.Insert(anIndex, TCollection_ExtendedString(std::u16string_view(next_units(theInput))));
   }},
  {"insert_u16",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int   anIndex = next<int>(theInput);
     const Units aUnits  = next_units(theInput);
     theString.Insert(anIndex, aUnits.c_str());
   }},
  {"insert_span",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int  anIndex = next<int>(theInput);
     const Span aSpan   = next_span(theInput);
     theString.Insert(anIndex, aSpan.Pointer(), next<int>(theInput));
   }},
  {"set_unit",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString.SetValue(anIndex, char16_t(next<int>(theInput)));
   }},
  {"set_str",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString.SetValue(anIndex, TCollection_ExtendedString(std::u16string_view(next_units(theInput))));
   }},
  {"set_u16",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int   anIndex = next<int>(theInput);
     const Units aUnits  = next_units(theInput);
     theString.SetValue(anIndex, aUnits.c_str());
   }},
  {"set_span",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int  anIndex = next<int>(theInput);
     const Span aSpan   = next_span(theInput);
     theString.SetValue(anIndex, aSpan.Pointer(), next<int>(theInput));
   }},
  {"remove",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int anIndex = next<int>(theInput);
     theString.Remove(anIndex, next<int>(theInput));
   }},
  {"remove_one",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     theString.Remove(next<int>(theInput));
   }},
  {"remove_all",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     theString.RemoveAll(char16_t(next<int>(theInput)));
   }},
  {"trunc",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     theString.Trunc(next<int>(theInput));
   }},
  {"split",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const TCollection_ExtendedString aTail = theString.Split(next<int>(theInput));
     theString.AssignCat(u'|');
     theString.AssignCat(aTail);
   }},
  {"sub",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int aFrom = next<int>(theInput);
     theString       = theString.SubString(aFrom, next<int>(theInput));
   }},
  {"change",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const char16_t aFrom = char16_t(next<int>(theInput));
     theString.ChangeAll(aFrom, char16_t(next<int>(theInput)));
   }},
  {"copy_span",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const Span aSpan = next_span(theInput);
     theString.Copy(aSpan.Pointer(), next<int>(theInput));
   }},
  {"copy_u16",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     theString.Copy(aUnits.c_str());
   }},
  {"copy_str",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     theString.Copy(TCollection_ExtendedString(std::u16string_view(next_units(theInput))));
   }},
  {"assign_u16",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     theString          = aUnits.c_str();
   }},
  {"assign_view",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const Units aUnits = next_units(theInput);
     theString          = std::u16string_view(aUnits);
   }},
  {"clear", [](TCollection_ExtendedString& theString, std::istream&) { theString.Clear(); }},
  {"swap",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     TCollection_ExtendedString anOther(std::u16string_view(next_units(theInput)));
     theString.Swap(anOther);
     theString.AssignCat(u'|');
     theString.AssignCat(anOther);
   }},
  {"move",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     TCollection_ExtendedString aSource(std::u16string_view(next_units(theInput)));
     theString.Move(std::move(aSource));
     theString.AssignCat(u'|');
     theString.AssignCat(aSource);
   }},
  {"left_adjust", [](TCollection_ExtendedString& theString, std::istream&) { theString.LeftAdjust(); }},
  {"right_adjust",
   [](TCollection_ExtendedString& theString, std::istream&) { theString.RightAdjust(); }},
  {"capitalize", [](TCollection_ExtendedString& theString, std::istream&) { theString.Capitalize(); }},
  {"left_justify",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int aWidth = next<int>(theInput);
     theString.LeftJustify(aWidth, char16_t(next<int>(theInput)));
   }},
  {"right_justify",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int aWidth = next<int>(theInput);
     theString.RightJustify(aWidth, char16_t(next<int>(theInput)));
   }},
  {"center",
   [](TCollection_ExtendedString& theString, std::istream& theInput) {
     const int aWidth = next<int>(theInput);
     theString.Center(aWidth, char16_t(next<int>(theInput)));
   }},
};

// "edit <op> <string> <args...>": the edited string, or the exception class.
inline std::string edit_case(std::istream& theInput)
{
  const std::string          anOperation = next<std::string>(theInput);
  TCollection_ExtendedString aString(std::u16string_view(next_units(theInput)));
  return guarded_kind([&] {
    THE_EDIT.at(anOperation)(aString, theInput);
    return describe(aString);
  });
}
} // namespace extended_string_api_probe
