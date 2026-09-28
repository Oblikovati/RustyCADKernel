// TCollection_AsciiString members that change the string (included by
// tkernel_ascii_string_api_probe.cxx inside namespace ascii_string_api_probe).
// Each entry reads its fields after the base string and returns extra output.

static const std::map<std::string, Edit>& concatenation_edits()
{
  static const std::map<std::string, Edit> THE_EDITS = {
    {"assign_cat_char", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(next_char(in)); return ""; }},
    {"plus_assign_char", [](Ascii& s, Istream& in) -> std::string { s += next_char(in); return ""; }},
    {"assign_cat_int", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(next_int(in)); return ""; }},
    {"plus_assign_int", [](Ascii& s, Istream& in) -> std::string { s += next_int(in); return ""; }},
    {"assign_cat_real", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(next_real(in)); return ""; }},
    {"plus_assign_real", [](Ascii& s, Istream& in) -> std::string { s += next_real(in); return ""; }},
    {"assign_cat_span",
     [](Ascii& s, Istream& in) -> std::string {
       const std::string t = next_text(in);
       s.AssignCat(t.data(), static_cast<int>(t.size()));
       return "";
     }},
    {"assign_cat_cstring", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(next_text(in).c_str()); return ""; }},
    {"plus_assign_cstring", [](Ascii& s, Istream& in) -> std::string { s += next_text(in).c_str(); return ""; }},
    {"assign_cat_view",
     [](Ascii& s, Istream& in) -> std::string {
       const std::string t = next_text(in);
       s.AssignCat(std::string_view(t));
       return "";
     }},
    {"assign_cat_string", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(make(next_text(in))); return ""; }},
    {"plus_assign_string", [](Ascii& s, Istream& in) -> std::string { s += make(next_text(in)); return ""; }},
    {"assign_cat_extended",
     [](Ascii& s, Istream& in) -> std::string {
       const TCollection_ExtendedString u = next_extended(in);
       s.AssignCat(u, next_char(in));
       return "";
     }},
    {"plus_assign_extended", [](Ascii& s, Istream& in) -> std::string { s += next_extended(in); return ""; }},
    {"assign_cat_wide", [](Ascii& s, Istream& in) -> std::string { s.AssignCat(next_wide(in).c_str()); return ""; }},
    {"plus_assign_wide", [](Ascii& s, Istream& in) -> std::string { s += next_wide(in).c_str(); return ""; }},
    {"assign_cat_self",
     [](Ascii& s, Istream& in) -> std::string {
       const int anOffset = next_int(in);
       s.AssignCat(s.ToCString() + anOffset, next_int(in));
       return "";
     }},
    {"prepend", [](Ascii& s, Istream& in) -> std::string { s.Prepend(make(next_text(in))); return ""; }},
  };
  return THE_EDITS;
}

static const std::map<std::string, Edit>& insertion_edits()
{
  static const std::map<std::string, Edit> THE_EDITS = {
    {"insert_char",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.Insert(aWhere, next_char(in));
       return "";
     }},
    {"insert_span",
     [](Ascii& s, Istream& in) -> std::string {
       const int         aWhere = next_int(in);
       const std::string t      = next_text(in);
       s.Insert(aWhere, t.data(), static_cast<int>(t.size()));
       return "";
     }},
    {"insert_cstring",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.Insert(aWhere, next_text(in).c_str());
       return "";
     }},
    {"insert_string",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.Insert(aWhere, make(next_text(in)));
       return "";
     }},
    {"insert_self",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere  = next_int(in);
       const int anOffset = next_int(in);
       s.Insert(aWhere, s.ToCString() + anOffset, next_int(in));
       return "";
     }},
    {"insert_after",
     [](Ascii& s, Istream& in) -> std::string {
       const int anIndex = next_int(in);
       s.InsertAfter(anIndex, make(next_text(in)));
       return "";
     }},
    {"insert_before",
     [](Ascii& s, Istream& in) -> std::string {
       const int anIndex = next_int(in);
       s.InsertBefore(anIndex, make(next_text(in)));
       return "";
     }},
    {"set_value_char",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.SetValue(aWhere, next_char(in));
       return "";
     }},
    {"set_value_span",
     [](Ascii& s, Istream& in) -> std::string {
       const int         aWhere = next_int(in);
       const std::string t      = next_text(in);
       s.SetValue(aWhere, t.data(), static_cast<int>(t.size()));
       return "";
     }},
    {"set_value_string",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.SetValue(aWhere, make(next_text(in)));
       return "";
     }},
  };
  return THE_EDITS;
}

static const std::map<std::string, Edit>& removal_edits()
{
  static const std::map<std::string, Edit> THE_EDITS = {
    {"remove",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWhere = next_int(in);
       s.Remove(aWhere, next_int(in));
       return "";
     }},
    {"remove_all", [](Ascii& s, Istream& in) -> std::string { s.RemoveAll(next_char(in)); return ""; }},
    {"remove_all_case",
     [](Ascii& s, Istream& in) -> std::string {
       const char aChar = next_char(in);
       s.RemoveAll(aChar, next_flag(in));
       return "";
     }},
    {"trunc", [](Ascii& s, Istream& in) -> std::string { s.Trunc(next_int(in)); return ""; }},
    {"split", [](Ascii& s, Istream& in) -> std::string { return " " + show(s.Split(next_int(in))); }},
    {"copy_span",
     [](Ascii& s, Istream& in) -> std::string {
       const std::string t = next_text(in);
       s.Copy(t.data(), static_cast<int>(t.size()));
       return "";
     }},
    {"copy_cstring", [](Ascii& s, Istream& in) -> std::string { s = next_text(in).c_str(); return ""; }},
    {"copy_view",
     [](Ascii& s, Istream& in) -> std::string {
       const std::string t = next_text(in);
       s = std::string_view(t);
       return "";
     }},
    {"copy_string", [](Ascii& s, Istream& in) -> std::string { s = make(next_text(in)); return ""; }},
    {"copy_self",
     [](Ascii& s, Istream& in) -> std::string {
       const int anOffset = next_int(in);
       s.Copy(s.ToCString() + anOffset, next_int(in));
       return "";
     }},
    {"move",
     [](Ascii& s, Istream& in) -> std::string {
       Ascii anOther = make(next_text(in));
       s.Move(std::move(anOther));
       return " " + show(anOther);
     }},
    {"swap",
     [](Ascii& s, Istream& in) -> std::string {
       Ascii anOther = make(next_text(in));
       s.Swap(anOther);
       return " " + show(anOther);
     }},
    {"clear", [](Ascii& s, Istream&) -> std::string { s.Clear(); return ""; }},
  };
  return THE_EDITS;
}

static const std::map<std::string, Edit>& layout_edits()
{
  static const std::map<std::string, Edit> THE_EDITS = {
    {"center",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       s.Center(aWidth, next_char(in));
       return "";
     }},
    {"left_justify",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       s.LeftJustify(aWidth, next_char(in));
       return "";
     }},
    {"right_justify",
     [](Ascii& s, Istream& in) -> std::string {
       const int aWidth = next_int(in);
       s.RightJustify(aWidth, next_char(in));
       return "";
     }},
    {"left_adjust", [](Ascii& s, Istream&) -> std::string { s.LeftAdjust(); return ""; }},
    {"right_adjust", [](Ascii& s, Istream&) -> std::string { s.RightAdjust(); return ""; }},
    {"change_all",
     [](Ascii& s, Istream& in) -> std::string {
       const char aFrom = next_char(in);
       const char aTo   = next_char(in);
       s.ChangeAll(aFrom, aTo, next_flag(in));
       return "";
     }},
    {"upper", [](Ascii& s, Istream&) -> std::string { s.UpperCase(); return ""; }},
    {"lower", [](Ascii& s, Istream&) -> std::string { s.LowerCase(); return ""; }},
    {"capitalize", [](Ascii& s, Istream&) -> std::string { s.Capitalize(); return ""; }},
  };
  return THE_EDITS;
}

static const std::map<std::string, Edit>& edits()
{
  static const std::map<std::string, Edit> THE_EDITS = [] {
    std::map<std::string, Edit> anAll;
    for (const auto* aTable : {&concatenation_edits(), &insertion_edits(), &removal_edits(), &layout_edits()})
      anAll.insert(aTable->begin(), aTable->end());
    return anAll;
  }();
  return THE_EDITS;
}
