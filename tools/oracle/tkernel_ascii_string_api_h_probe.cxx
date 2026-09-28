// TCollection_HAsciiString and TCollection::NextPrimeForMap cases (included by
// tkernel_ascii_string_api_probe.cxx inside namespace ascii_string_api_probe).
// A handle field is the hex of its string, or "null" for a null handle.

using HAscii  = occ::handle<TCollection_HAsciiString>;
using HEdit   = std::function<std::string(const HAscii&, Istream&)>;
using HQuery  = HEdit;
using HMake   = std::function<HAscii(Istream&)>;

static HAscii next_handle(Istream& theInput)
{
  std::string aHex;
  theInput >> aHex;
  if (aHex == "null")
    return HAscii();
  return new TCollection_HAsciiString(make(tkernel_probe::decode(aHex)));
}

static std::string show_handle(const HAscii& theString)
{
  return show(theString->String());
}

static const std::map<std::string, HMake>& h_makers()
{
  static const std::map<std::string, HMake> THE_MAKERS = {
    {"default", [](Istream&) { return HAscii(new TCollection_HAsciiString()); }},
    {"cstring", [](Istream& in) { return HAscii(new TCollection_HAsciiString(next_text(in).c_str())); }},
    {"char", [](Istream& in) { return HAscii(new TCollection_HAsciiString(next_char(in))); }},
    {"filled",
     [](Istream& in) {
       const int aLength = next_int(in);
       return HAscii(new TCollection_HAsciiString(aLength, next_char(in)));
     }},
    {"int", [](Istream& in) { return HAscii(new TCollection_HAsciiString(next_int(in))); }},
    {"real", [](Istream& in) { return HAscii(new TCollection_HAsciiString(next_real(in))); }},
    {"string", [](Istream& in) { return HAscii(new TCollection_HAsciiString(make(next_text(in)))); }},
    {"moved_string",
     [](Istream& in) {
       Ascii aSource = make(next_text(in));
       return HAscii(new TCollection_HAsciiString(std::move(aSource)));
     }},
    {"handle", [](Istream& in) { return HAscii(new TCollection_HAsciiString(next_handle(in))); }},
    {"extended",
     [](Istream& in) {
       const occ::handle<TCollection_HExtendedString> anExtended =
         new TCollection_HExtendedString(next_extended(in));
       return HAscii(new TCollection_HAsciiString(anExtended, next_char(in)));
     }},
  };
  return THE_MAKERS;
}

static const std::map<std::string, HEdit>& h_edits()
{
  static const std::map<std::string, HEdit> THE_EDITS = {
    {"assign_cat", [](const HAscii& h, Istream& in) -> std::string { h->AssignCat(next_text(in).c_str()); return ""; }},
    {"assign_cat_handle", [](const HAscii& h, Istream& in) -> std::string { h->AssignCat(next_handle(in)); return ""; }},
    {"assign_cat_self", [](const HAscii& h, Istream&) -> std::string { h->AssignCat(h); return ""; }},
    {"cat", [](const HAscii& h, Istream& in) { return " " + show_handle(h->Cat(next_text(in).c_str())); }},
    {"cat_handle", [](const HAscii& h, Istream& in) { return " " + show_handle(h->Cat(next_handle(in))); }},
    {"capitalize", [](const HAscii& h, Istream&) -> std::string { h->Capitalize(); return ""; }},
    {"upper", [](const HAscii& h, Istream&) -> std::string { h->UpperCase(); return ""; }},
    {"lower", [](const HAscii& h, Istream&) -> std::string { h->LowerCase(); return ""; }},
    {"center",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       h->Center(aWidth, next_char(in));
       return "";
     }},
    {"left_justify",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       h->LeftJustify(aWidth, next_char(in));
       return "";
     }},
    {"right_justify",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       h->RightJustify(aWidth, next_char(in));
       return "";
     }},
    {"change_all",
     [](const HAscii& h, Istream& in) -> std::string {
       const char aFrom = next_char(in);
       const char aTo   = next_char(in);
       h->ChangeAll(aFrom, aTo, next_flag(in));
       return "";
     }},
    {"clear", [](const HAscii& h, Istream&) -> std::string { h->Clear(); return ""; }},
    {"left_adjust", [](const HAscii& h, Istream&) -> std::string { h->LeftAdjust(); return ""; }},
    {"right_adjust", [](const HAscii& h, Istream&) -> std::string { h->RightAdjust(); return ""; }},
    {"insert_char",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->Insert(aWhere, next_char(in));
       return "";
     }},
    {"insert",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->Insert(aWhere, next_text(in).c_str());
       return "";
     }},
    {"insert_handle",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->Insert(aWhere, next_handle(in));
       return "";
     }},
    {"insert_self", [](const HAscii& h, Istream& in) -> std::string { h->Insert(next_int(in), h); return ""; }},
    {"insert_after",
     [](const HAscii& h, Istream& in) -> std::string {
       const int anIndex = next_int(in);
       h->InsertAfter(anIndex, next_handle(in));
       return "";
     }},
    {"insert_before",
     [](const HAscii& h, Istream& in) -> std::string {
       const int anIndex = next_int(in);
       h->InsertBefore(anIndex, next_handle(in));
       return "";
     }},
    {"prepend", [](const HAscii& h, Istream& in) -> std::string { h->Prepend(next_handle(in)); return ""; }},
    {"prepend_self", [](const HAscii& h, Istream&) -> std::string { h->Prepend(h); return ""; }},
  };
  return THE_EDITS;
}

static const std::map<std::string, HEdit>& h_removal_edits()
{
  static const std::map<std::string, HEdit> THE_EDITS = {
    {"remove_all", [](const HAscii& h, Istream& in) -> std::string { h->RemoveAll(next_char(in)); return ""; }},
    {"remove_all_case",
     [](const HAscii& h, Istream& in) -> std::string {
       const char aChar = next_char(in);
       h->RemoveAll(aChar, next_flag(in));
       return "";
     }},
    {"remove",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->Remove(aWhere, next_int(in));
       return "";
     }},
    {"set_value_char",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->SetValue(aWhere, next_char(in));
       return "";
     }},
    {"set_value",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->SetValue(aWhere, next_text(in).c_str());
       return "";
     }},
    {"set_value_handle",
     [](const HAscii& h, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       h->SetValue(aWhere, next_handle(in));
       return "";
     }},
    {"split", [](const HAscii& h, Istream& in) { return " " + show_handle(h->Split(next_int(in))); }},
    {"sub_string",
     [](const HAscii& h, Istream& in) {
       const int aFrom = next_int(in);
       return " " + show_handle(h->SubString(aFrom, next_int(in)));
     }},
    {"token",
     [](const HAscii& h, Istream& in) {
       const std::string aSeparators = next_text(in);
       return " " + show_handle(h->Token(aSeparators.c_str(), next_int(in)));
     }},
    {"token_default", [](const HAscii& h, Istream&) { return " " + show_handle(h->Token()); }},
    {"trunc", [](const HAscii& h, Istream& in) -> std::string { h->Trunc(next_int(in)); return ""; }},
  };
  return THE_EDITS;
}

static const std::map<std::string, HQuery>& h_queries()
{
  static const std::map<std::string, HQuery> THE_QUERIES = {
    {"is_less", [](const HAscii& h, Istream& in) { return flag(h->IsLess(next_handle(in))); }},
    {"is_greater", [](const HAscii& h, Istream& in) { return flag(h->IsGreater(next_handle(in))); }},
    {"is_different", [](const HAscii& h, Istream& in) { return flag(h->IsDifferent(next_handle(in))); }},
    {"is_same_string", [](const HAscii& h, Istream& in) { return flag(h->IsSameString(next_handle(in))); }},
    {"is_same_string_case",
     [](const HAscii& h, Istream& in) {
       const HAscii anOther = next_handle(in);
       return flag(h->IsSameString(anOther, next_flag(in)));
     }},
    {"is_same_state", [](const HAscii& h, Istream& in) { return flag(h->IsSameState(next_handle(in))); }},
    {"in_set",
     [](const HAscii& h, Istream& in) {
       const HAscii aSet  = next_handle(in);
       const int    aFrom = next_int(in);
       return std::to_string(h->FirstLocationInSet(aSet, aFrom, next_int(in)));
     }},
    {"not_in_set",
     [](const HAscii& h, Istream& in) {
       const HAscii aSet  = next_handle(in);
       const int    aFrom = next_int(in);
       return std::to_string(h->FirstLocationNotInSet(aSet, aFrom, next_int(in)));
     }},
    {"location",
     [](const HAscii& h, Istream& in) {
       const HAscii aWhat = next_handle(in);
       const int    aFrom = next_int(in);
       return std::to_string(h->Location(aWhat, aFrom, next_int(in)));
     }},
    {"location_char",
     [](const HAscii& h, Istream& in) {
       const int  aCount = next_int(in);
       const char aChar  = next_char(in);
       const int  aFrom  = next_int(in);
       return std::to_string(h->Location(aCount, aChar, aFrom, next_int(in)));
     }},
    {"search", [](const HAscii& h, Istream& in) { return std::to_string(h->Search(next_text(in).c_str())); }},
    {"search_handle", [](const HAscii& h, Istream& in) { return std::to_string(h->Search(next_handle(in))); }},
    {"search_from_end",
     [](const HAscii& h, Istream& in) { return std::to_string(h->SearchFromEnd(next_text(in).c_str())); }},
    {"search_from_end_handle",
     [](const HAscii& h, Istream& in) { return std::to_string(h->SearchFromEnd(next_handle(in))); }},
    {"value", [](const HAscii& h, Istream& in) { return std::to_string(static_cast<unsigned char>(h->Value(next_int(in)))); }},
    {"integer_value", [](const HAscii& h, Istream&) { return std::to_string(h->IntegerValue()); }},
    {"real_value", [](const HAscii& h, Istream&) { return real_bits(h->RealValue()); }},
    {"properties",
     [](const HAscii& h, Istream&) {
       std::ostringstream anOut;
       h->Print(anOut);
       return flag(h->IsAscii()) + flag(h->IsEmpty()) + flag(h->IsIntegerValue()) + flag(h->IsRealValue()) + " "
              + std::to_string(h->Length()) + " " + std::to_string(h->UsefullLength()) + " "
              + tkernel_probe::hex(h->ToCString(), h->Length()) + " " + tkernel_probe::hex(anOut.str()) + " "
              + h->DynamicType()->Name() + " " + flag(h->IsKind(STANDARD_TYPE(Standard_Transient)));
     }},
  };
  return THE_QUERIES;
}

static std::string h_make_case(Istream& theInput)
{
  const HMake& aMake = member(h_makers(), theInput);
  return kind_only(tkernel_probe::guarded([&] { return show_handle(aMake(theInput)); }));
}

static std::string h_edit_case(Istream& theInput)
{
  static const std::map<std::string, HEdit> THE_ALL = [] {
    std::map<std::string, HEdit> anAll = h_edits();
    anAll.insert(h_removal_edits().begin(), h_removal_edits().end());
    return anAll;
  }();
  const HEdit& anEdit = member(THE_ALL, theInput);
  const HAscii aString = next_handle(theInput);
  return kind_only(tkernel_probe::guarded([&] {
    const std::string anExtra = anEdit(aString, theInput);
    return show_handle(aString) + anExtra;
  }));
}

static std::string h_query_case(Istream& theInput)
{
  const HQuery& aQuery  = member(h_queries(), theInput);
  const HAscii  aString = next_handle(theInput);
  return kind_only(tkernel_probe::guarded([&] { return aQuery(aString, theInput); }));
}

// TCollection is deprecated since OCCT 7.9; the probe still pins its behavior.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
static std::string next_prime_case(Istream& theInput)
{
  return std::to_string(TCollection::NextPrimeForMap(next_int(theInput)));
}
#pragma GCC diagnostic pop
