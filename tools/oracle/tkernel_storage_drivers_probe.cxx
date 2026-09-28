// Storage driver scripts (twin of oracle_probe/storage/drivers.rs).
//
// stor_drv <f|c|b> <hex file, @ or -> <op>...
//   Runs driver operations on FSD_File (f), FSD_CmpFile (c) or FSD_BinaryFile (b) over a
//   file holding the given bytes (empty for "@", absent for "-"), then closes an open driver and prints
//   the final file bytes after " F:". Each operation prints " <op>=<result>" or
//   " <op>!<exception class>"; read operations print their out-parameters in both cases.
//   o<n|r|w|u> open, cl close, ie IsEnd, tl Tell, sk SkipObject;
//   b/e + w/r + i/c/t/r/f/d: section begins and ends (bwi, eri, brt, ...);
//   wi=<nb>,<sv>,<date>,<schema>,<schema version>,<app ext>,<app version>,<type ext>,<info;...>
//   ri, wc=<ext;...>, rc, sts|ssr|ssf=<n>, ts|rs|fs, wt=<n>,<name>, rt, wr=<name>,<n>,<type>,
//   rr, wf=<r>,<t>, rf, woh=<r>,<t>, roh, bwp|bwo|ewo|ewp|brp|bro|ero|erp object framing,
//   pr|pc|pe|pi|pb=<value>, pd|pf=<hex bits>, gr|gc|ge|gi|gb|gd|gf,
//   hr|rdr|tdr: Storage_HeaderData/RootData/TypeData::Read
#include <FSD_BinaryFile.hxx>
#include <FSD_CmpFile.hxx>
#include <FSD_File.hxx>
#include <Storage_HeaderData.hxx>
#include <Storage_Root.hxx>
#include <Storage_RootData.hxx>
#include <Storage_TypeData.hxx>

namespace
{
struct StorDriverValues
{
  int                                              aCount = 0, anInt1 = 0, anInt2 = 0;
  TCollection_AsciiString                          aVersion, aDate, aSchema, aSchemaVersion;
  TCollection_AsciiString                          anAppVersion, aText1, aText2;
  TCollection_ExtendedString                       anAppName, aDataType;
  NCollection_Sequence<TCollection_AsciiString>    anInfo;
  NCollection_Sequence<TCollection_ExtendedString> aComments;
  char                                             aChar  = 0;
  char16_t                                         aUnit  = 0;
  bool                                             aBool  = false;
  double                                           aReal  = 0.0;
  float                                            aShort = 0.0f;
  occ::handle<Storage_HeaderData>                  aHeader = new Storage_HeaderData();
  occ::handle<Storage_RootData>                    aRoots  = new Storage_RootData();
  occ::handle<Storage_TypeData>                    aTypes  = new Storage_TypeData();
};

std::string stor_ascii_list(const NCollection_Sequence<TCollection_AsciiString>& theItems)
{
  std::string aText = "[";
  for (int anIndex = 1; anIndex <= theItems.Length(); ++anIndex)
    aText += (anIndex > 1 ? ";" : "") + stor_ascii_hex(theItems.Value(anIndex));
  return aText + "]";
}

std::string stor_ext_list(const NCollection_Sequence<TCollection_ExtendedString>& theItems)
{
  std::string aText = "[";
  for (int anIndex = 1; anIndex <= theItems.Length(); ++anIndex)
    aText += (anIndex > 1 ? ";" : "") + stor_ext_hex(theItems.Value(anIndex));
  return aText + "]";
}

Storage_OpenMode stor_mode(char theMode)
{
  switch (theMode)
  {
    case 'r':
      return Storage_VSRead;
    case 'w':
      return Storage_VSWrite;
    case 'u':
      return Storage_VSReadWrite;
    default:
      return Storage_VSNone;
  }
}

void stor_print_info(std::ostream& out, const StorDriverValues& v)
{
  out << v.aCount << ',' << stor_ascii_hex(v.aVersion) << ',' << stor_ascii_hex(v.aDate) << ','
      << stor_ascii_hex(v.aSchema) << ',' << stor_ascii_hex(v.aSchemaVersion) << ','
      << stor_ext_hex(v.anAppName) << ',' << stor_ascii_hex(v.anAppVersion) << ','
      << stor_ext_hex(v.aDataType) << ',' << stor_ascii_list(v.anInfo);
}

void stor_print_header(std::ostream& out, const occ::handle<Storage_HeaderData>& theHeader)
{
  out << static_cast<int>(theHeader->ErrorStatus()) << ','
      << stor_ascii_hex(theHeader->ErrorStatusExtension()) << ',' << theHeader->NumberOfObjects()
      << ',' << stor_ascii_hex(theHeader->StorageVersion()) << ','
      << stor_ascii_hex(theHeader->CreationDate()) << ',' << stor_ascii_hex(theHeader->SchemaName())
      << ',' << stor_ascii_hex(theHeader->SchemaVersion()) << ','
      << stor_ext_hex(theHeader->ApplicationName()) << ','
      << stor_ascii_hex(theHeader->ApplicationVersion()) << ','
      << stor_ext_hex(theHeader->DataType()) << ',' << stor_ascii_list(theHeader->UserInfo()) << ','
      << stor_ext_list(theHeader->Comments());
}

void stor_print_roots(std::ostream& out, const occ::handle<Storage_RootData>& theRoots)
{
  out << static_cast<int>(theRoots->ErrorStatus()) << ','
      << stor_ascii_hex(theRoots->ErrorStatusExtension()) << ',' << theRoots->NumberOfRoots() << ",[";
  occ::handle<NCollection_HSequence<occ::handle<Storage_Root>>> aList = theRoots->Roots();
  for (int anIndex = 1; anIndex <= aList->Length(); ++anIndex)
  {
    const occ::handle<Storage_Root>& aRoot = aList->Value(anIndex);
    out << (anIndex > 1 ? ";" : "") << stor_ascii_hex(aRoot->Name()) << ':' << aRoot->Reference()
        << ':' << stor_ascii_hex(aRoot->Type()) << ':' << !aRoot->Object().IsNull();
  }
  out << ']';
}

void stor_print_types(std::ostream& out, const occ::handle<Storage_TypeData>& theTypes)
{
  out << static_cast<int>(theTypes->ErrorStatus()) << ','
      << stor_ascii_hex(theTypes->ErrorStatusExtension()) << ',' << theTypes->NumberOfTypes() << ",[";
  occ::handle<NCollection_HSequence<TCollection_AsciiString>> aNames = theTypes->Types();
  for (int anIndex = 1; anIndex <= aNames->Length(); ++anIndex)
    out << (anIndex > 1 ? ";" : "") << stor_ascii_hex(aNames->Value(anIndex)) << ':'
        << theTypes->Type(aNames->Value(anIndex));
  out << ']';
}

// Section begins/ends: returns true when the operation name is one of them.
bool stor_section_op(const std::string& theOp, Storage_BaseDriver& theDriver, std::ostream& out)
{
  static const std::pair<const char*, Storage_Error (Storage_BaseDriver::*)()> THE_SECTIONS[] = {
    {"bwi", &Storage_BaseDriver::BeginWriteInfoSection},
    {"ewi", &Storage_BaseDriver::EndWriteInfoSection},
    {"bri", &Storage_BaseDriver::BeginReadInfoSection},
    {"eri", &Storage_BaseDriver::EndReadInfoSection},
    {"bwc", &Storage_BaseDriver::BeginWriteCommentSection},
    {"ewc", &Storage_BaseDriver::EndWriteCommentSection},
    {"brc", &Storage_BaseDriver::BeginReadCommentSection},
    {"erc", &Storage_BaseDriver::EndReadCommentSection},
    {"bwt", &Storage_BaseDriver::BeginWriteTypeSection},
    {"ewt", &Storage_BaseDriver::EndWriteTypeSection},
    {"brt", &Storage_BaseDriver::BeginReadTypeSection},
    {"ert", &Storage_BaseDriver::EndReadTypeSection},
    {"bwr", &Storage_BaseDriver::BeginWriteRootSection},
    {"ewr", &Storage_BaseDriver::EndWriteRootSection},
    {"brr", &Storage_BaseDriver::BeginReadRootSection},
    {"err", &Storage_BaseDriver::EndReadRootSection},
    {"bwf", &Storage_BaseDriver::BeginWriteRefSection},
    {"ewf", &Storage_BaseDriver::EndWriteRefSection},
    {"brf", &Storage_BaseDriver::BeginReadRefSection},
    {"erf", &Storage_BaseDriver::EndReadRefSection},
    {"bwd", &Storage_BaseDriver::BeginWriteDataSection},
    {"ewd", &Storage_BaseDriver::EndWriteDataSection},
    {"brd", &Storage_BaseDriver::BeginReadDataSection},
    {"erd", &Storage_BaseDriver::EndReadDataSection},
    {"cl", &Storage_BaseDriver::Close}};
  for (const auto& aSection : THE_SECTIONS)
  {
    if (theOp == aSection.first)
    {
      out << static_cast<int>((theDriver.*aSection.second)());
      return true;
    }
  }
  return false;
}

// Object framing without results.
bool stor_frame_op(const std::string& theOp, Storage_BaseDriver& theDriver)
{
  static const std::pair<const char*, void (Storage_BaseDriver::*)()> THE_FRAMES[] = {
    {"bwp", &Storage_BaseDriver::BeginWritePersistentObjectData},
    {"bwo", &Storage_BaseDriver::BeginWriteObjectData},
    {"ewo", &Storage_BaseDriver::EndWriteObjectData},
    {"ewp", &Storage_BaseDriver::EndWritePersistentObjectData},
    {"brp", &Storage_BaseDriver::BeginReadPersistentObjectData},
    {"bro", &Storage_BaseDriver::BeginReadObjectData},
    {"ero", &Storage_BaseDriver::EndReadObjectData},
    {"erp", &Storage_BaseDriver::EndReadPersistentObjectData},
    {"sk", &Storage_BaseDriver::SkipObject}};
  for (const auto& aFrame : THE_FRAMES)
  {
    if (theOp == aFrame.first)
    {
      (theDriver.*aFrame.second)();
      return true;
    }
  }
  return false;
}

bool stor_put_op(const std::string&              theOp,
                 const std::vector<std::string>& theArgs,
                 Storage_BaseDriver&             theDriver)
{
  if (theOp == "pr")
    theDriver.PutReference(std::stoi(theArgs[0]));
  else if (theOp == "pc")
    theDriver.PutCharacter(static_cast<char>(std::stoi(theArgs[0])));
  else if (theOp == "pe")
    theDriver.PutExtCharacter(static_cast<char16_t>(std::stoi(theArgs[0])));
  else if (theOp == "pi")
    theDriver.PutInteger(std::stoi(theArgs[0]));
  else if (theOp == "pb")
    theDriver.PutBoolean(theArgs[0] == "1");
  else if (theOp == "pd")
    theDriver.PutReal(stor_double(theArgs[0]));
  else if (theOp == "pf")
    theDriver.PutShortReal(stor_float(theArgs[0]));
  else
    return false;
  return true;
}

// Reads print their target after the call, so a failure still shows the value it left.
bool stor_get_op(const std::string& theOp, Storage_BaseDriver& theDriver, StorDriverValues& v, std::string& theShown)
{
  bool isKnown = true;
  try
  {
    if (theOp == "gr")
      theDriver.GetReference(v.anInt1);
    else if (theOp == "gc")
      theDriver.GetCharacter(v.aChar);
    else if (theOp == "ge")
      theDriver.GetExtCharacter(v.aUnit);
    else if (theOp == "gi")
      theDriver.GetInteger(v.anInt1);
    else if (theOp == "gb")
      theDriver.GetBoolean(v.aBool);
    else if (theOp == "gd")
      theDriver.GetReal(v.aReal);
    else if (theOp == "gf")
      theDriver.GetShortReal(v.aShort);
    else
      isKnown = false;
  }
  catch (Standard_Failure const& aFailure)
  {
    theShown = std::string("!") + aFailure.ExceptionType() + "=";
  }
  if (!isKnown)
    return false;
  std::ostringstream aValue;
  if (theOp == "gr" || theOp == "gi")
    aValue << v.anInt1;
  else if (theOp == "gc")
    aValue << static_cast<int>(static_cast<unsigned char>(v.aChar));
  else if (theOp == "ge")
    aValue << static_cast<int>(v.aUnit);
  else if (theOp == "gb")
    aValue << v.aBool;
  else if (theOp == "gd")
    aValue << stor_bits(v.aReal);
  else
    aValue << stor_bits(v.aShort);
  theShown += aValue.str();
  return true;
}

void stor_table_op(const std::string&              theOp,
                   const std::vector<std::string>& theArgs,
                   const occ::handle<Storage_BaseDriver>& theDriver,
                   StorDriverValues&               v,
                   std::ostream&                   out)
{
  if (theOp == "wi")
  {
    NCollection_Sequence<TCollection_AsciiString> anInfo;
    if (theArgs[8] != "-")
      for (const std::string& aLine : stor_split(theArgs[8], ';'))
        anInfo.Append(TCollection_AsciiString(stor_unhex(aLine).c_str()));
    theDriver->WriteInfo(std::stoi(theArgs[0]), stor_unhex(theArgs[1]).c_str(),
                         stor_unhex(theArgs[2]).c_str(), stor_unhex(theArgs[3]).c_str(),
                         stor_unhex(theArgs[4]).c_str(), stor_ext_unhex(theArgs[5]),
                         stor_unhex(theArgs[6]).c_str(), stor_ext_unhex(theArgs[7]), anInfo);
  }
  else if (theOp == "wc")
  {
    NCollection_Sequence<TCollection_ExtendedString> aComments;
    if (theArgs[0] != "-")
      for (const std::string& aLine : stor_split(theArgs[0], ';'))
        aComments.Append(stor_ext_unhex(aLine));
    theDriver->WriteComment(aComments);
  }
  else if (theOp == "sts")
    theDriver->SetTypeSectionSize(std::stoi(theArgs[0]));
  else if (theOp == "ssr")
    theDriver->SetRootSectionSize(std::stoi(theArgs[0]));
  else if (theOp == "ssf")
    theDriver->SetRefSectionSize(std::stoi(theArgs[0]));
  else if (theOp == "ts")
    out << theDriver->TypeSectionSize();
  else if (theOp == "rs")
    out << theDriver->RootSectionSize();
  else if (theOp == "fs")
    out << theDriver->RefSectionSize();
  else if (theOp == "wt")
    theDriver->WriteTypeInformations(std::stoi(theArgs[0]), stor_unhex(theArgs[1]).c_str());
  else if (theOp == "wr")
    theDriver->WriteRoot(stor_unhex(theArgs[0]).c_str(), std::stoi(theArgs[1]),
                         stor_unhex(theArgs[2]).c_str());
  else if (theOp == "wf")
    theDriver->WriteReferenceType(std::stoi(theArgs[0]), std::stoi(theArgs[1]));
  else if (theOp == "woh")
    theDriver->WritePersistentObjectHeader(std::stoi(theArgs[0]), std::stoi(theArgs[1]));
  else if (theOp == "hr")
    out << v.aHeader->Read(theDriver);
  else if (theOp == "rdr")
    out << v.aRoots->Read(theDriver);
  else if (theOp == "tdr")
    out << v.aTypes->Read(theDriver);
  else if (!stor_put_op(theOp, theArgs, *theDriver) && !stor_frame_op(theOp, *theDriver)
           && !stor_section_op(theOp, *theDriver, out))
    throw std::runtime_error("unknown storage driver operation " + theOp);
}

// Reads with out-parameters print them whether or not the call throws.
bool stor_read_op(const std::string& theOp, Storage_BaseDriver& theDriver, StorDriverValues& v, std::ostream& out)
{
  std::string aFailure;
  try
  {
    if (theOp == "ri")
      theDriver.ReadInfo(v.aCount, v.aVersion, v.aDate, v.aSchema, v.aSchemaVersion, v.anAppName,
                         v.anAppVersion, v.aDataType, v.anInfo);
    else if (theOp == "rc")
      theDriver.ReadComment(v.aComments);
    else if (theOp == "rt")
      theDriver.ReadTypeInformations(v.anInt1, v.aText1);
    else if (theOp == "rr")
      theDriver.ReadRoot(v.aText1, v.anInt1, v.aText2);
    else if (theOp == "rf")
      theDriver.ReadReferenceType(v.anInt1, v.anInt2);
    else if (theOp == "roh")
      theDriver.ReadPersistentObjectHeader(v.anInt1, v.anInt2);
    else
      return false;
  }
  catch (Standard_Failure const& anException)
  {
    aFailure = std::string("!") + anException.ExceptionType() + "=";
  }
  out << aFailure;
  if (theOp == "ri")
    stor_print_info(out, v);
  else if (theOp == "rc")
    out << stor_ext_list(v.aComments);
  else if (theOp == "rt")
    out << v.anInt1 << ',' << stor_ascii_hex(v.aText1);
  else if (theOp == "rr")
    out << stor_ascii_hex(v.aText1) << ',' << v.anInt1 << ',' << stor_ascii_hex(v.aText2);
  else
    out << v.anInt1 << ',' << v.anInt2;
  return true;
}

occ::handle<Storage_BaseDriver> stor_driver(char theKind)
{
  if (theKind == 'b')
    return new FSD_BinaryFile();
  if (theKind == 'c')
    return new FSD_CmpFile();
  return new FSD_File();
}

void stor_driver_op(const std::string&                     theToken,
                    const std::string&                     thePath,
                    const occ::handle<Storage_BaseDriver>& theDriver,
                    StorDriverValues&                      v,
                    std::ostream&                          out)
{
  const size_t      anEqual = theToken.find('=');
  const std::string anOp    = theToken.substr(0, anEqual);
  const std::vector<std::string> anArgs =
    anEqual == std::string::npos ? std::vector<std::string>() : stor_split(theToken.substr(anEqual + 1), ',');
  out << ' ' << anOp;
  std::string aShown;
  if (stor_get_op(anOp, *theDriver, v, aShown))
  {
    out << (aShown[0] == '!' ? "" : "=") << aShown;
    return;
  }
  std::ostringstream aResult;
  if (stor_read_op(anOp, *theDriver, v, aResult))
  {
    const std::string aText = aResult.str();
    out << (aText[0] == '!' ? "" : "=") << aText;
    return;
  }
  try
  {
    if (anOp[0] == 'o' && anOp.size() == 2)
      aResult << static_cast<int>(theDriver->Open(thePath.c_str(), stor_mode(anOp[1])));
    else if (anOp == "ie")
      aResult << theDriver->IsEnd();
    else if (anOp == "tl")
      aResult << static_cast<long long>(theDriver->Tell());
    else
      stor_table_op(anOp, anArgs, theDriver, v, aResult);
    out << '=' << aResult.str();
  }
  catch (Standard_Failure const& anException)
  {
    out << '!' << anException.ExceptionType();
  }
}

void stor_driver_script(std::istream& in, std::ostream& out)
{
  std::string aKind, aHex, aToken;
  in >> aKind >> aHex;
  const std::string aPath = stor_scratch("driver");
  std::remove(aPath.c_str());
  if (aHex != "-")
    stor_write_file(aPath, stor_unhex(aHex));
  StorDriverValues v;
  {
    occ::handle<Storage_BaseDriver> aDriver = stor_driver(aKind[0]);
    while (in >> aToken)
      stor_driver_op(aToken, aPath, aDriver, v, out);
    if (aDriver->OpenMode() != Storage_VSNone)
      aDriver->Close();
  }
  out << " hdr=";
  stor_print_header(out, v.aHeader);
  out << " rdt=";
  stor_print_roots(out, v.aRoots);
  out << " tdt=";
  stor_print_types(out, v.aTypes);
  out << " F:" << stor_read_file(aPath);
  std::remove(aPath.c_str());
}
} // namespace

static bool storage_drivers_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "stor_drv")
    stor_driver_script(in, out);
  else if (op == "stor_good")
  {
    std::string aKind, aHex;
    in >> aKind >> aHex;
    const std::string aPath = stor_scratch("good");
    std::remove(aPath.c_str());
    if (aHex != "-")
      stor_write_file(aPath, stor_unhex(aHex));
    Storage_Error aStatus = Storage_VSOk;
    if (aKind == "b")
      aStatus = FSD_BinaryFile::IsGoodFileType(aPath.c_str());
    else if (aKind == "c")
      aStatus = FSD_CmpFile::IsGoodFileType(aPath.c_str());
    else
      aStatus = FSD_File::IsGoodFileType(aPath.c_str());
    out << static_cast<int>(aStatus);
    std::remove(aPath.c_str());
  }
  else
    return false;
  return true;
}
