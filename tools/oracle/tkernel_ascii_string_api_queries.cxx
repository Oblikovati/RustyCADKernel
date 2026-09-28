// TCollection_AsciiString constructors, read-only members and stream I/O (included by
// tkernel_ascii_string_api_probe.cxx inside namespace ascii_string_api_probe).

static std::string flag(bool theValue)
{
  return theValue ? "1" : "0";
}

static const std::map<std::string, Make>& makers()
{
  static const std::map<std::string, Make> THE_MAKERS = {
    {"default", [](Istream&) { return Ascii(); }},
    {"cstring", [](Istream& in) { return Ascii(next_text(in).c_str()); }},
    {"ptr_len",
     [](Istream& in) {
       const std::string t = next_text(in);
       return Ascii(t.c_str(), next_int(in));
     }},
    {"view",
     [](Istream& in) {
       const std::string t = next_text(in);
       return Ascii(std::string_view(t));
     }},
    {"char", [](Istream& in) { return Ascii(next_char(in)); }},
    {"filled",
     [](Istream& in) {
       const int aLength = next_int(in);
       return Ascii(aLength, next_char(in));
     }},
    {"int", [](Istream& in) { return Ascii(next_int(in)); }},
    {"real", [](Istream& in) { return Ascii(next_real(in)); }},
    {"copy", [](Istream& in) { return Ascii(make(next_text(in))); }},
    {"join_char",
     [](Istream& in) {
       const Ascii aHead = make(next_text(in));
       return Ascii(aHead, next_char(in));
     }},
    {"join_cstring",
     [](Istream& in) {
       const Ascii aHead = make(next_text(in));
       return Ascii(aHead, next_text(in).c_str());
     }},
    {"join",
     [](Istream& in) {
       const Ascii aHead = make(next_text(in));
       return Ascii(aHead, make(next_text(in)));
     }},
    {"wide", [](Istream& in) { return Ascii(next_wide(in).c_str()); }},
    {"extended",
     [](Istream& in) {
       const TCollection_ExtendedString u = next_extended(in);
       return Ascii(u, next_char(in));
     }},
    {"empty_string", [](Istream&) { return Ascii::EmptyString(); }},
    {"cat_char", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(next_char(in)); }},
    {"plus_char", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_char(in); }},
    {"cat_int", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(next_int(in)); }},
    {"plus_int", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_int(in); }},
    {"cat_real", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(next_real(in)); }},
    {"plus_real", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_real(in); }},
    {"cat_span",
     [](Istream& in) {
       const Ascii       s = make(next_text(in));
       const std::string t = next_text(in);
       return s.Cat(t.data(), static_cast<int>(t.size()));
     }},
    {"cat_cstring", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(next_text(in).c_str()); }},
    {"plus_cstring", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_text(in).c_str(); }},
    {"cat_view",
     [](Istream& in) {
       const Ascii       s = make(next_text(in));
       const std::string t = next_text(in);
       return s.Cat(std::string_view(t));
     }},
    {"cat_string", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(make(next_text(in))); }},
    {"plus_string", [](Istream& in) { const Ascii s = make(next_text(in)); return s + make(next_text(in)); }},
    {"cat_extended",
     [](Istream& in) {
       const Ascii                      s = make(next_text(in));
       const TCollection_ExtendedString u = next_extended(in);
       return s.Cat(u, next_char(in));
     }},
    {"plus_extended", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_extended(in); }},
    {"cat_wide", [](Istream& in) { const Ascii s = make(next_text(in)); return s.Cat(next_wide(in).c_str()); }},
    {"plus_wide", [](Istream& in) { const Ascii s = make(next_text(in)); return s + next_wide(in).c_str(); }},
  };
  return THE_MAKERS;
}

// Every comparison overload of `s` against `t`, one digit each.
static std::string compare_all(const Ascii& s, const std::string& t)
{
  const Ascii anOther = make(t);
  const int   aSize   = static_cast<int>(t.size());
  const bool  aFlags[] = {
    s.IsEqual(anOther), s.IsEqual(t.c_str()), s.IsEqual(t.data(), aSize), s.IsEqual(std::string_view(t)),
    s == anOther, s == t.c_str(), s.IsDifferent(anOther), s.IsDifferent(t.c_str()), s != anOther,
    s.IsLess(anOther), s.IsLess(t.c_str()), s.IsLess(t.data(), aSize), s < anOther,
    s.IsGreater(anOther), s.IsGreater(t.c_str()), s.IsGreater(t.data(), aSize), s > anOther,
    s.StartsWith(anOther), s.StartsWith(t.c_str()), s.StartsWith(t.data(), aSize),
    s.EndsWith(anOther), s.EndsWith(t.data(), aSize),
    Ascii::IsEqual(s, anOther), Ascii::IsEqual(s, std::string_view(t)),
    Ascii::IsSameString(s, anOther, true), Ascii::IsSameString(s, anOther, false),
    Ascii::IsSameString(s, t.c_str(), false), Ascii::IsSameString(s.ToCString(), t.c_str(), false),
    Ascii::IsSameString(s.ToCString(), s.Length(), t.data(), aSize, false)};
  std::string aDigits;
  for (const bool aFlag : aFlags)
    aDigits += flag(aFlag);
  return aDigits;
}

static const std::map<std::string, Query>& search_queries()
{
  static const std::map<std::string, Query> THE_QUERIES = {
    {"value", [](const Ascii& s, Istream& in) { return std::to_string(static_cast<unsigned char>(s.Value(next_int(in)))); }},
    {"sub_string",
     [](const Ascii& s, Istream& in) {
       const int aFrom = next_int(in);
       return show(s.SubString(aFrom, next_int(in)));
     }},
    {"search_span",
     [](const Ascii& s, Istream& in) {
       const std::string t = next_text(in);
       return std::to_string(s.Search(t.data(), static_cast<int>(t.size())));
     }},
    {"search_cstring", [](const Ascii& s, Istream& in) { return std::to_string(s.Search(next_text(in).c_str())); }},
    {"search_string", [](const Ascii& s, Istream& in) { return std::to_string(s.Search(make(next_text(in)))); }},
    {"search_from_end_span",
     [](const Ascii& s, Istream& in) {
       const std::string t = next_text(in);
       return std::to_string(s.SearchFromEnd(std::string_view(t)));
     }},
    {"search_from_end_cstring",
     [](const Ascii& s, Istream& in) { return std::to_string(s.SearchFromEnd(next_text(in).c_str())); }},
    {"location",
     [](const Ascii& s, Istream& in) {
       const Ascii t     = make(next_text(in));
       const int   aFrom = next_int(in);
       return std::to_string(s.Location(t, aFrom, next_int(in)));
     }},
    {"location_char",
     [](const Ascii& s, Istream& in) {
       const int  aCount = next_int(in);
       const char aChar  = next_char(in);
       const int  aFrom  = next_int(in);
       return std::to_string(s.Location(aCount, aChar, aFrom, next_int(in)));
     }},
    {"in_set",
     [](const Ascii& s, Istream& in) {
       const std::string t     = next_text(in);
       const int         aFrom = next_int(in);
       return std::to_string(s.FirstLocationInSet(t.data(), static_cast<int>(t.size()), aFrom, next_int(in)));
     }},
    {"not_in_set",
     [](const Ascii& s, Istream& in) {
       const Ascii t     = make(next_text(in));
       const int   aFrom = next_int(in);
       return std::to_string(s.FirstLocationNotInSet(t, aFrom, next_int(in)));
     }},
    {"token",
     [](const Ascii& s, Istream& in) {
       const std::string aSeparators = next_text(in);
       return show(s.Token(aSeparators.c_str(), next_int(in)));
     }},
    {"token_default", [](const Ascii& s, Istream&) { return show(s.Token()); }},
    {"compare", [](const Ascii& s, Istream& in) { return compare_all(s, next_text(in)); }},
  };
  return THE_QUERIES;
}

static const std::map<std::string, Query>& value_queries()
{
  static const std::map<std::string, Query> THE_QUERIES = {
    {"properties",
     [](const Ascii& s, Istream&) {
       return flag(s.IsAscii()) + flag(s.IsEmpty()) + flag(s.IsIntegerValue()) + flag(s.IsRealValue())
              + flag(s.IsRealValue(true)) + " " + std::to_string(s.UsefullLength()) + " "
              + std::to_string(s.HashCode()) + " " + std::to_string(std::hash<Ascii>{}(s));
     }},
    {"integer_value", [](const Ascii& s, Istream&) { return std::to_string(s.IntegerValue()); }},
    {"real_value", [](const Ascii& s, Istream&) { return real_bits(s.RealValue()); }},
    {"print",
     [](const Ascii& s, Istream&) {
       std::ostringstream anOut;
       s.Print(anOut);
       anOut << s;
       return tkernel_probe::hex(anOut.str());
     }},
  };
  return THE_QUERIES;
}

static const std::map<std::string, Query>& queries()
{
  static const std::map<std::string, Query> THE_QUERIES = [] {
    std::map<std::string, Query> anAll = search_queries();
    anAll.insert(value_queries().begin(), value_queries().end());
    return anAll;
  }();
  return THE_QUERIES;
}

// "ascii_stream read <hex input>": Read and operator>>, then the unread rest of the stream.
static std::string stream_case(Istream& theInput)
{
  std::string aMember;
  theInput >> aMember;
  std::istringstream aStream(next_text(theInput));
  Ascii              aString("junk");
  if (aMember == "read")
    aString.Read(aStream);
  else
    aStream >> aString;
  const bool        aFailed = aStream.fail();
  const std::string aRest((std::istreambuf_iterator<char>(aStream)), std::istreambuf_iterator<char>());
  return show(aString) + " " + flag(aFailed) + " " + tkernel_probe::hex(aRest);
}
