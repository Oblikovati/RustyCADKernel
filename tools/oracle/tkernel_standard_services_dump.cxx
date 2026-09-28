// Standard_Dump: writing, formatting, splitting and reading JSON dump text.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard_Dump.hxx>
#include <Standard_Transient.hxx>
#include <sstream>
#include <vector>

namespace standard_services_probe
{
// Runs a writer against a Standard_SStream (s), a std::ostringstream (o) or a plain
// std::ostream over a stringbuf, whose content Standard_Dump cannot read (x); the stream
// first holds `thePrefix` with its put position moved to `thePosition`.
template <class Writer>
static std::string with_dump_stream(const std::string& theMode,
                                    const std::string& thePrefix,
                                    std::streamoff     thePosition,
                                    Writer             theWriter)
{
  Standard_SStream   aSStream;
  std::ostringstream anOStream;
  std::stringbuf     aBuffer;
  std::ostream       anOpaque(&aBuffer);
  std::ostream&      aStream =
    theMode == "s" ? static_cast<std::ostream&>(aSStream)
                   : (theMode == "o" ? static_cast<std::ostream&>(anOStream) : anOpaque);
  aStream << thePrefix;
  aStream.seekp(thePosition);
  theWriter(aStream);
  const std::string aText =
    theMode == "s" ? aSStream.str() : (theMode == "o" ? anOStream.str() : aBuffer.str());
  return tkernel_probe::hex(aText) + " pos=" + std::to_string(static_cast<std::streamoff>(aStream.tellp()));
}

// "services_dump_separator <s|o|x> <hex prefix> <put position>".
static std::string dump_separator(std::istream& theInput)
{
  std::string    aMode;
  std::streamoff aPosition = 0;
  theInput >> aMode;
  const std::string aPrefix = tkernel_probe::next_bytes(theInput);
  theInput >> aPosition;
  return with_dump_stream(aMode, aPrefix, aPosition, [](std::ostream& theStream) {
    Standard_Dump::AddValuesSeparator(theStream);
  });
}

// "services_dump_key_to_class <s|o|x> <hex prefix> <hex key> <hex field>".
static std::string dump_key_to_class(std::istream& theInput)
{
  std::string aMode;
  theInput >> aMode;
  const std::string aPrefix = tkernel_probe::next_bytes(theInput);
  const std::string aKey    = tkernel_probe::next_bytes(theInput);
  const std::string aField  = tkernel_probe::next_bytes(theInput);
  return with_dump_stream(aMode, aPrefix, static_cast<std::streamoff>(aPrefix.size()), [&](std::ostream& theStream) {
    Standard_Dump::DumpKeyToClass(theStream,
                                  TCollection_AsciiString(aKey.data(), static_cast<int>(aKey.size())),
                                  TCollection_AsciiString(aField.data(), static_cast<int>(aField.size())));
  });
}

// "services_dump_character_values <hex>...": up to four C strings.
static std::string dump_character_values(std::istream& theInput)
{
  std::vector<std::string> aValues;
  std::string              aHex;
  while (theInput >> aHex)
    aValues.push_back(tkernel_probe::decode(aHex));
  std::ostringstream aStream;
  const auto         at = [&](std::size_t i) { return aValues[i].c_str(); };
  switch (aValues.size())
  {
    case 0: Standard_Dump::DumpCharacterValues(aStream, 0); break;
    case 1: Standard_Dump::DumpCharacterValues(aStream, 1, at(0)); break;
    case 2: Standard_Dump::DumpCharacterValues(aStream, 2, at(0), at(1)); break;
    case 3: Standard_Dump::DumpCharacterValues(aStream, 3, at(0), at(1), at(2)); break;
    default: Standard_Dump::DumpCharacterValues(aStream, 4, at(0), at(1), at(2), at(3)); break;
  }
  return tkernel_probe::hex(aStream.str());
}

// "services_dump_real_values <bits>...": up to four doubles.
static std::string dump_real_values(std::istream& theInput)
{
  std::vector<double> aValues;
  std::uint64_t       aBits = 0;
  while (theInput >> aBits)
  {
    double aValue = 0.0;
    std::memcpy(&aValue, &aBits, sizeof(aValue));
    aValues.push_back(aValue);
  }
  std::ostringstream aStream;
  switch (aValues.size())
  {
    case 0: Standard_Dump::DumpRealValues(aStream, 0); break;
    case 1: Standard_Dump::DumpRealValues(aStream, 1, aValues[0]); break;
    case 2: Standard_Dump::DumpRealValues(aStream, 2, aValues[0], aValues[1]); break;
    case 3: Standard_Dump::DumpRealValues(aStream, 3, aValues[0], aValues[1], aValues[2]); break;
    default: Standard_Dump::DumpRealValues(aStream, 4, aValues[0], aValues[1], aValues[2], aValues[3]); break;
  }
  return tkernel_probe::hex(aStream.str());
}

static TCollection_AsciiString ascii(const std::string& theBytes)
{
  return TCollection_AsciiString(theBytes.data(), static_cast<int>(theBytes.size()));
}

// "services_dump_field_to_name <hex>".
static std::string dump_field_to_name(std::istream& theInput)
{
  const TCollection_AsciiString aField = ascii(tkernel_probe::next_bytes(theInput));
  return tkernel_probe::guarded([&] {
    const TCollection_AsciiString aName = Standard_Dump::DumpFieldToName(aField);
    return tkernel_probe::hex(aName.ToCString(), static_cast<std::size_t>(aName.Length()));
  });
}

// "services_dump_format <indent> <hex>": Text and FormatJson of a stream holding the bytes.
static std::string dump_format(std::istream& theInput)
{
  int aIndent = 0;
  theInput >> aIndent;
  Standard_SStream aStream;
  aStream << tkernel_probe::next_bytes(theInput);
  const TCollection_AsciiString aText   = Standard_Dump::Text(aStream);
  const TCollection_AsciiString aFormat = Standard_Dump::FormatJson(aStream, aIndent);
  return tkernel_probe::hex(aText.ToCString(), static_cast<std::size_t>(aText.Length())) + " "
         + tkernel_probe::hex(aFormat.ToCString(), static_cast<std::size_t>(aFormat.Length()));
}

// "services_dump_split <hex>": SplitJson's result and entries, then HierarchicalValueIndices
// and HasChildKey over the split values.
static std::string dump_split(std::istream& theInput)
{
  const TCollection_AsciiString aText = ascii(tkernel_probe::next_bytes(theInput));
  return tkernel_probe::guarded([&] {
    NCollection_IndexedDataMap<TCollection_AsciiString, Standard_DumpValue>      aValues;
    NCollection_IndexedDataMap<TCollection_AsciiString, TCollection_AsciiString> aTexts;
    std::string aResult = Standard_Dump::SplitJson(aText, aValues) ? "1" : "0";
    for (int anIndex = 1; anIndex <= aValues.Extent(); ++anIndex)
    {
      const TCollection_AsciiString& aKey   = aValues.FindKey(anIndex);
      const Standard_DumpValue&      aValue = aValues.FindFromIndex(anIndex);
      aTexts.Add(aKey, aValue.myValue);
      aResult += " " + tkernel_probe::hex(aKey.ToCString(), static_cast<std::size_t>(aKey.Length())) + "="
                 + tkernel_probe::hex(aValue.myValue.ToCString(), static_cast<std::size_t>(aValue.myValue.Length()))
                 + "@" + std::to_string(aValue.myStartPosition)
                 + (Standard_Dump::HasChildKey(aValue.myValue) ? "c" : "");
    }
    aResult += " hier=";
    for (const int anIndex : Standard_Dump::HierarchicalValueIndices(aTexts))
      aResult += std::to_string(anIndex) + ",";
    return aResult;
  });
}

// "services_dump_keys": every key's text and length, the pointer prefix and DumpValue.
static std::string dump_keys(std::istream&)
{
  std::string aResult;
  for (int aKey = Standard_JsonKey_None; aKey <= Standard_JsonKey_SeparatorValueToValue; ++aKey)
  {
    const auto aJsonKey = static_cast<Standard_JsonKey>(aKey);
    aResult += tkernel_probe::hex(std::string(Standard_Dump::JsonKeyToString(aJsonKey))) + ":"
               + std::to_string(Standard_Dump::JsonKeyLength(aJsonKey)) + " ";
  }
  const Standard_DumpValue anEmpty, aValue(TCollection_AsciiString("v"), 7);
  return aResult + Standard_Dump::GetPointerPrefix().ToCString() + " " + anEmpty.myValue.ToCString() + "@"
         + std::to_string(anEmpty.myStartPosition) + " " + aValue.myValue.ToCString() + "@"
         + std::to_string(aValue.myStartPosition);
}

// "services_dump_pointer <address> <short>": GetPointerInfo of a raw address, plus the
// null-handle overload.
static std::string dump_pointer(std::istream& theInput)
{
  std::uintptr_t anAddress = 0;
  int            isShort   = 0;
  theInput >> anAddress >> isShort;
  const TCollection_AsciiString anInfo =
    Standard_Dump::GetPointerInfo(reinterpret_cast<const void*>(anAddress), isShort != 0);
  const TCollection_AsciiString aNull =
    Standard_Dump::GetPointerInfo(occ::handle<Standard_Transient>(), isShort != 0);
  return tkernel_probe::hex(anInfo.ToCString(), static_cast<std::size_t>(anInfo.Length())) + " null="
         + std::to_string(aNull.Length());
}

// "services_dump_process <stream|field> <hex text> <hex name> <position>".
static std::string dump_process(std::istream& theInput)
{
  std::string aKind;
  theInput >> aKind;
  const TCollection_AsciiString aText = ascii(tkernel_probe::next_bytes(theInput));
  const TCollection_AsciiString aName = ascii(tkernel_probe::next_bytes(theInput));
  int                           aPosition = 0;
  theInput >> aPosition;
  return tkernel_probe::guarded([&] {
    const bool isFound = aKind == "stream" ? Standard_Dump::ProcessStreamName(aText, aName, aPosition)
                                           : Standard_Dump::ProcessFieldName(aText, aName, aPosition);
    return std::to_string(isFound) + " " + std::to_string(aPosition);
  });
}

// "services_dump_init_value <hex text> <position>".
static std::string dump_init_value(std::istream& theInput)
{
  const TCollection_AsciiString aText = ascii(tkernel_probe::next_bytes(theInput));
  int                           aPosition = 0;
  theInput >> aPosition;
  return tkernel_probe::guarded([&] {
    TCollection_AsciiString aValue;
    Standard_Dump::InitValue(aText, aPosition, aValue);
    return tkernel_probe::hex(aValue.ToCString(), static_cast<std::size_t>(aValue.Length())) + " "
           + std::to_string(aPosition);
  });
}

// "services_dump_init_reals <hex text> <position> <count 0..4>": the result, the position
// and every output (initially 0.5) as bits, including values stored before a failure.
static std::string dump_init_reals(std::istream& theInput)
{
  const TCollection_AsciiString aText = ascii(tkernel_probe::next_bytes(theInput));
  int                           aPosition = 0, aCount = 0;
  theInput >> aPosition >> aCount;
  double     aValues[4] = {0.5, 0.5, 0.5, 0.5};
  std::string aResult = tkernel_probe::guarded([&] {
    bool isDone = false;
    switch (aCount)
    {
      case 0: isDone = Standard_Dump::InitRealValues(aText, aPosition, 0); break;
      case 1: isDone = Standard_Dump::InitRealValues(aText, aPosition, 1, &aValues[0]); break;
      case 2: isDone = Standard_Dump::InitRealValues(aText, aPosition, 2, &aValues[0], &aValues[1]); break;
      case 3:
        isDone = Standard_Dump::InitRealValues(aText, aPosition, 3, &aValues[0], &aValues[1], &aValues[2]);
        break;
      default:
        isDone = Standard_Dump::InitRealValues(aText, aPosition, 4, &aValues[0], &aValues[1], &aValues[2], &aValues[3]);
        break;
    }
    return std::to_string(isDone) + " " + std::to_string(aPosition);
  });
  for (int i = 0; i < aCount; ++i)
    aResult += " " + std::to_string(tkernel_probe::bits(aValues[i]));
  return aResult;
}
} // namespace standard_services_probe
