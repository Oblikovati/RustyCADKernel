// NCollection utilities probe operations (twin of oracle_probe/ncollection/utilities.rs).
//
// ncoll_utilities_utf <8|16|32> <hex units...>
//   Walks the source with NCollection_UtfIterator and converts it into UtfString<char>,
//   <char16_t>, <char32_t> and <wchar_t>, printing every observable of each result.
// ncoll_utilities_locale <hex bytes...>
//   FromLocale in the startup "C" locale, with and without a length limit.
// Sources are followed by eight zero units so that the unprotected overrun of a truncated
// sequence reads defined zeros, which is what the Rust slice model reads past its end.
#include <NCollection_String.hxx>

#include <cstdint>
#include <iomanip>
#include <istream>
#include <ostream>
#include <sstream>
#include <string>
#include <vector>

template <typename T>
static std::string utf_units(const T* theUnits, size_t theCount)
{
  std::ostringstream aText;
  aText << std::hex;
  for (size_t anIndex = 0; anIndex < theCount; ++anIndex)
  {
    aText << (anIndex == 0 ? "" : ",") << static_cast<uint32_t>(static_cast<std::make_unsigned_t<T>>(theUnits[anIndex]));
  }
  return theCount == 0 ? "-" : aText.str();
}

template <typename T>
static std::vector<T> utf_read_source(std::istream& in)
{
  std::vector<T> aUnits;
  std::string    aToken;
  while (in >> aToken)
  {
    aUnits.push_back(static_cast<T>(std::stoul(aToken, nullptr, 16)));
  }
  aUnits.resize(aUnits.size() + 8, T(0));
  return aUnits;
}

template <typename T>
static void utf_walk(std::ostream& out, const T* theSource)
{
  NCollection_UtfIterator<T> anIter(theSource);
  out << " walk";
  for (int aCount = 0; aCount < 64 && *anIter != 0; ++aCount, ++anIter)
  {
    char     aBuf8[8];
    char16_t aBuf16[4];
    char*    anEnd8  = anIter.GetUtf8(aBuf8);
    char16_t* anEnd16 = anIter.GetUtf16(aBuf16);
    out << ' ' << std::hex << *anIter << std::dec << ':' << anIter.Index() << ':'
        << (anIter.BufferHere() - theSource) << ':' << (anIter.BufferNext() - theSource) << ':'
        << anIter.IsValid() << ':' << anIter.AdvanceBytesUtf8() << ':'
        << anIter.AdvanceBytesUtf16() << ':' << anIter.AdvanceCodeUnitsUtf16() << ':'
        << anIter.AdvanceBytesUtf32() << ':' << anIter.template AdvanceBytesUtf<char16_t>() << ':'
        << utf_units(aBuf8, anEnd8 - aBuf8) << ':' << utf_units(aBuf16, anEnd16 - aBuf16);
  }
}

template <typename T>
static void utf_string_state(std::ostream& out, const NCollection_UtfString<T>& theText)
{
  out << ' ' << theText.Size() << '/' << theText.Length() << '/' << theText.IsEmpty() << '/'
      << utf_units(theText.ToCString(), theText.Size() / sizeof(T) + 1);
}

template <typename T>
static void utf_string_queries(std::ostream& out, const NCollection_UtfString<T>& theText)
{
  out << " chars";
  for (int anIndex = -1; anIndex <= theText.Length() + 1; ++anIndex)
  {
    const T* aBuffer = theText.GetCharBuffer(anIndex);
    out << ' ' << std::hex << theText.GetChar(anIndex) << std::dec << '@'
        << (aBuffer == nullptr ? -1 : static_cast<long>(aBuffer - theText.ToCString()));
  }
  out << " sub";
  utf_string_state(out, theText.SubString(1, 3));
  utf_string_state(out, theText.SubString(0, 1));
  utf_string_state(out, theText.SubString(2, 2));
  out << " to";
  utf_string_state(out, theText.ToUtf8());
  utf_string_state(out, theText.ToUtf16());
  utf_string_state(out, theText.ToUtf32());
  utf_string_state(out, theText.ToUtfWide());
}

template <typename T>
static void utf_string_joins(std::ostream& out, const NCollection_UtfString<T>& theText)
{
  NCollection_UtfString<T> anAppended(theText);
  anAppended += theText;
  out << " join";
  utf_string_state(out, theText + theText);
  utf_string_state(out, anAppended);
  out << ' ' << (anAppended == theText) << (theText.SubString(0, 1000) != theText);
}

template <typename T>
static void utf_string_locale(std::ostream& out, const NCollection_UtfString<T>& theText)
{
  out << " locale";
  for (int aSize : {0, 1, 2, 5, 16})
  {
    char aBuffer[24];
    std::fill(aBuffer, aBuffer + 24, char(0x55));
    const bool isDone = theText.ToLocale(aBuffer, aSize);
    out << ' ' << isDone << '/' << utf_units(aBuffer, static_cast<size_t>(aSize));
  }
}

template <typename T, typename F>
static void utf_convert(std::ostream& out, const F* theSource)
{
  for (int aLength : {-1, 2})
  {
    NCollection_UtfString<T> aText(theSource, aLength);
    out << " |";
    utf_string_state(out, aText);
    if (aLength < 0)
    {
      utf_string_queries(out, aText);
      utf_string_joins(out, aText);
      utf_string_locale(out, aText);
    }
  }
}

template <typename F>
static void utf_probe(std::istream& in, std::ostream& out)
{
  const std::vector<F> aSource = utf_read_source<F>(in);
  utf_walk(out, aSource.data());
  utf_convert<char>(out, aSource.data());
  utf_convert<char16_t>(out, aSource.data());
  utf_convert<char32_t>(out, aSource.data());
  utf_convert<wchar_t>(out, aSource.data());
}

static void utf_locale_probe(std::istream& in, std::ostream& out)
{
  const std::vector<char> aSource = utf_read_source<char>(in);
  for (int aLength : {-1, 1})
  {
    NCollection_String aText("seed");
    aText.FromLocale(aSource.data(), aLength);
    utf_string_state(out, aText);
  }
}

static bool ncollection_utilities_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_utilities_locale")
  {
    utf_locale_probe(in, out);
    return true;
  }
  if (op != "ncoll_utilities_utf")
  {
    return false;
  }
  int aKind = 0;
  in >> aKind;
  if (aKind == 8)
    utf_probe<char>(in, out);
  else if (aKind == 16)
    utf_probe<char16_t>(in, out);
  else
    utf_probe<char32_t>(in, out);
  return true;
}
