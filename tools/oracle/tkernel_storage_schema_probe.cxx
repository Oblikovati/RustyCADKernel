// Storage data, schema and binary-stream scripts (twin of oracle_probe/storage/schema.rs).
//
// stor_schema <f|c|b> <n|r|w|u> <config>...   Storage_Schema::Write of a data set, then the
//   data status, header and file bytes. Config: app=<ext> ver=<hex> type=<ext> info=<hex>
//   com=<ext> root=<hex> (null object) aroot (unnamed null object) oroot=<hex> (object)
//   sname=<hex> sver=<hex>
// stor_data <op>...        Storage_Data roots, types and header (see stor_data_op)
// stor_bucket <size> <op>...  Storage_BucketOfPersistent: a (new object) n (null) v<i> len clr it
// stor_cb <op>...          Storage_Schema callback registry: add= addn= rm= ls clr use dont is rst setn dflt
// stor_complete <hex>      FSD_BinaryFile::ReadCompleteInfo from a stream over the bytes
// stor_bstream <hex> <op>...  FSD_BinaryFile's stream overloads on a stringstream
#include <FSD_BinaryFile.hxx>
#include <Standard_Persistent.hxx>
#include <Storage_BucketOfPersistent.hxx>
#include <Storage_Data.hxx>
#include <Storage_DefaultCallBack.hxx>
#include <Storage_Schema.hxx>

#include <map>

namespace
{
void stor_print_data(std::ostream& out, const occ::handle<Storage_Data>& theData)
{
  out << " hdr=";
  stor_print_header(out, theData->HeaderData());
  out << " rdt=";
  stor_print_roots(out, theData->RootData());
  out << " tdt=";
  stor_print_types(out, theData->TypeData());
}

void stor_configure(const std::string& theToken, const occ::handle<Storage_Data>& theData, Storage_Schema& theSchema)
{
  const size_t      anEqual = theToken.find('=');
  const std::string aKey    = theToken.substr(0, anEqual);
  const std::string aValue  = anEqual == std::string::npos ? "" : theToken.substr(anEqual + 1);
  if (aKey == "app")
    theData->SetApplicationName(stor_ext_unhex(aValue));
  else if (aKey == "ver")
    theData->SetApplicationVersion(stor_unhex(aValue).c_str());
  else if (aKey == "type")
    theData->SetDataType(stor_ext_unhex(aValue));
  else if (aKey == "info")
    theData->AddToUserInfo(stor_unhex(aValue).c_str());
  else if (aKey == "com")
    theData->AddToComments(stor_ext_unhex(aValue));
  else if (aKey == "root")
    theData->AddRoot(stor_unhex(aValue).c_str(), occ::handle<Standard_Persistent>());
  else if (aKey == "aroot")
    theData->AddRoot(occ::handle<Standard_Persistent>());
  else if (aKey == "oroot")
    theData->AddRoot(stor_unhex(aValue).c_str(), new Standard_Persistent());
  else if (aKey == "sname")
    theSchema.SetName(stor_unhex(aValue).c_str());
  else if (aKey == "sver")
    theSchema.SetVersion(stor_unhex(aValue).c_str());
}

void stor_schema(std::istream& in, std::ostream& out)
{
  std::string aKind, aMode, aToken;
  in >> aKind >> aMode;
  const std::string aPath = stor_scratch("schema");
  std::remove(aPath.c_str());
  occ::handle<Storage_Data>   aData   = new Storage_Data();
  occ::handle<Storage_Schema> aSchema = new Storage_Schema();
  while (in >> aToken)
    stor_configure(aToken, aData, *aSchema);
  {
    occ::handle<Storage_BaseDriver> aDriver = stor_driver(aKind[0]);
    out << "open=" << static_cast<int>(aDriver->Open(aPath.c_str(), stor_mode(aMode[0])));
    try
    {
      aSchema->Write(aDriver, aData);
      out << " write=ok";
    }
    catch (Standard_Failure const& anException)
    {
      out << " write!" << anException.ExceptionType();
    }
    if (aDriver->OpenMode() != Storage_VSNone)
      aDriver->Close();
  }
  out << " st=" << static_cast<int>(aData->ErrorStatus()) << ','
      << stor_ascii_hex(aData->ErrorStatusExtension());
  stor_print_data(out, aData);
  out << " F:" << stor_read_file(aPath);
  std::remove(aPath.c_str());
}

void stor_data_op(const std::string& theToken, const occ::handle<Storage_Data>& theData, std::ostream& out)
{
  const size_t                   anEqual = theToken.find('=');
  const std::string              anOp    = theToken.substr(0, anEqual);
  const std::vector<std::string> anArgs =
    anEqual == std::string::npos ? std::vector<std::string>() : stor_split(theToken.substr(anEqual + 1), ',');
  const TCollection_AsciiString aName = anArgs.empty() ? "" : stor_unhex(anArgs[0]).c_str();
  out << ' ' << anOp;
  std::ostringstream aResult;
  try
  {
    if (anOp == "ar")
      theData->AddRoot(aName, occ::handle<Standard_Persistent>());
    else if (anOp == "au")
      theData->AddRoot(occ::handle<Standard_Persistent>());
    else if (anOp == "ao")
      theData->AddRoot(aName, new Standard_Persistent());
    else if (anOp == "rm")
      theData->RemoveRoot(aName);
    else if (anOp == "fd")
    {
      occ::handle<Storage_Root> aRoot = theData->Find(aName);
      if (aRoot.IsNull())
        aResult << "null";
      else
        aResult << stor_ascii_hex(aRoot->Name()) << ':' << aRoot->Reference() << ':'
                << stor_ascii_hex(aRoot->Type()) << ':' << !aRoot->Object().IsNull();
    }
    else if (anOp == "ir")
      aResult << theData->IsRoot(aName);
    else if (anOp == "ur")
      theData->RootData()->UpdateRoot(aName, new Standard_Persistent());
    else if (anOp == "at")
      theData->TypeData()->AddType(aName, std::stoi(anArgs[1]));
    else if (anOp == "tn")
      aResult << stor_ascii_hex(theData->TypeData()->Type(std::stoi(anArgs[0])));
    else if (anOp == "tt")
      aResult << theData->TypeData()->Type(aName);
    else if (anOp == "it")
      aResult << theData->IsType(aName);
    else if (anOp == "ai")
      theData->AddToUserInfo(aName);
    else if (anOp == "ac")
      theData->AddToComments(stor_ext_unhex(anArgs[0]));
    else if (anOp == "cl")
      theData->Clear();
    else if (anOp == "ce")
      theData->ClearErrorStatus();
    else if (anOp == "st")
      aResult << static_cast<int>(theData->ErrorStatus()) << ','
              << stor_ascii_hex(theData->ErrorStatusExtension());
    else if (anOp == "dump")
      stor_print_data(aResult, theData);
    out << '=' << aResult.str();
  }
  catch (Standard_Failure const& anException)
  {
    out << '!' << anException.ExceptionType();
  }
}

void stor_bucket(std::istream& in, std::ostream& out)
{
  int aSize = 0;
  in >> aSize;
  Storage_BucketOfPersistent                   aBucket(aSize, 2);
  std::vector<occ::handle<Standard_Persistent>> anObjects;
  std::map<const Standard_Persistent*, int>     anIds;
  std::string                                  anOp;
  auto anId = [&anIds](const Standard_Persistent* theObject) {
    return theObject == nullptr ? 0 : anIds[theObject];
  };
  while (in >> anOp)
  {
    out << ' ' << anOp;
    if (anOp == "a" || anOp == "n")
    {
      occ::handle<Standard_Persistent> anObject;
      if (anOp == "a")
      {
        anObject = new Standard_Persistent();
        anObjects.push_back(anObject);
        anIds[anObject.get()] = static_cast<int>(anObjects.size());
      }
      aBucket.Append(anObject);
    }
    else if (anOp[0] == 'v')
      out << '=' << anId(aBucket.Value(std::stoi(anOp.substr(1))));
    else if (anOp == "len")
      out << '=' << aBucket.Length();
    else if (anOp == "clr")
      aBucket.Clear();
    else if (anOp == "it")
    {
      out << '=';
      for (Storage_BucketIterator anIter(&aBucket); anIter.More(); anIter.Next())
        out << anId(anIter.Value()) << ';';
    }
  }
}

void stor_callbacks(std::istream& in, std::ostream& out)
{
  Storage_Schema aSchema;
  std::string    aToken;
  while (in >> aToken)
  {
    const size_t      anEqual = aToken.find('=');
    const std::string anOp    = aToken.substr(0, anEqual);
    const TCollection_AsciiString aName =
      anEqual == std::string::npos ? "" : stor_unhex(aToken.substr(anEqual + 1)).c_str();
    out << ' ' << anOp;
    if (anOp == "add")
      aSchema.AddReadUnknownTypeCallBack(aName, new Storage_DefaultCallBack());
    else if (anOp == "addn")
      aSchema.AddReadUnknownTypeCallBack(aName, occ::handle<Storage_CallBack>());
    else if (anOp == "rm")
      aSchema.RemoveReadUnknownTypeCallBack(aName);
    else if (anOp == "ls")
    {
      occ::handle<NCollection_HSequence<TCollection_AsciiString>> aList = aSchema.InstalledCallBackList();
      out << '=';
      for (int anIndex = 1; anIndex <= aList->Length(); ++anIndex)
        out << stor_ascii_hex(aList->Value(anIndex)) << ';';
    }
    else if (anOp == "clr")
      aSchema.ClearCallBackList();
    else if (anOp == "use")
      aSchema.UseDefaultCallBack();
    else if (anOp == "dont")
      aSchema.DontUseDefaultCallBack();
    else if (anOp == "is")
      out << '=' << aSchema.IsUsingDefaultCallBack();
    else if (anOp == "rst")
      aSchema.ResetDefaultCallBack();
    else if (anOp == "setn")
      aSchema.SetDefaultCallBack(occ::handle<Storage_CallBack>());
    else if (anOp == "dflt")
      out << '=' << !aSchema.DefaultCallBack().IsNull();
  }
}

void stor_complete(std::istream& in, std::ostream& out)
{
  std::string aHex;
  in >> aHex;
  std::stringstream         aStream(stor_unhex(aHex), std::ios::in | std::ios::binary);
  occ::handle<Storage_Data> aData;
  FSD_BinaryFile            aDriver;
  // Readers take the magic number first, so the section offsets match stream positions.
  out << stor_ascii_hex(Storage_BaseDriver::ReadMagicNumber(aStream)) << ' ';
  try
  {
    aDriver.ReadCompleteInfo(aStream, aData);
    out << "ok";
  }
  catch (Standard_Failure const& anException)
  {
    out << '!' << anException.ExceptionType();
  }
  if (!aData.IsNull())
    stor_print_data(out, aData);
}

void stor_bstream_op(const std::string& theToken, std::stringstream& theStream, StorDriverValues& v, std::ostream& out)
{
  const size_t                   anEqual = theToken.find('=');
  const std::string              anOp    = theToken.substr(0, anEqual);
  const std::vector<std::string> anArgs =
    anEqual == std::string::npos ? std::vector<std::string>() : stor_split(theToken.substr(anEqual + 1), ',');
  const bool isCount = !anArgs.empty() && anArgs.back() == "1";
  out << ' ' << anOp;
  std::ostringstream aResult;
  std::string        aFailure;
  FSD_FileHeader     aHeader{};
  try
  {
    if (anOp == "pi")
      aResult << FSD_BinaryFile::PutInteger(theStream, std::stoi(anArgs[0]), isCount);
    else if (anOp == "wh")
    {
      int* aFields[] = {&aHeader.testindian, &aHeader.binfo, &aHeader.einfo, &aHeader.bcomment,
                        &aHeader.ecomment, &aHeader.btype, &aHeader.etype, &aHeader.broot,
                        &aHeader.eroot, &aHeader.bref, &aHeader.eref, &aHeader.bdata, &aHeader.edata};
      for (int anIndex = 0; anIndex < 13; ++anIndex)
        *aFields[anIndex] = std::stoi(anArgs[anIndex]);
      aResult << FSD_BinaryFile::WriteHeader(theStream, aHeader, isCount);
    }
    else if (anOp == "wi")
    {
      NCollection_Sequence<TCollection_AsciiString> anInfo;
      if (anArgs[8] != "-")
        for (const std::string& aLine : stor_split(anArgs[8], ';'))
          anInfo.Append(TCollection_AsciiString(stor_unhex(aLine).c_str()));
      aResult << FSD_BinaryFile::WriteInfo(theStream, std::stoi(anArgs[0]), stor_unhex(anArgs[1]).c_str(),
                                           stor_unhex(anArgs[2]).c_str(), stor_unhex(anArgs[3]).c_str(),
                                           stor_unhex(anArgs[4]).c_str(), stor_ext_unhex(anArgs[5]),
                                           stor_unhex(anArgs[6]).c_str(), stor_ext_unhex(anArgs[7]),
                                           anInfo, isCount);
    }
    else if (anOp == "wc")
    {
      NCollection_Sequence<TCollection_ExtendedString> aComments;
      if (anArgs[0] != "-")
        for (const std::string& aLine : stor_split(anArgs[0], ';'))
          aComments.Append(stor_ext_unhex(aLine));
      aResult << FSD_BinaryFile::WriteComment(theStream, aComments, isCount);
    }
    else if (anOp == "gi")
    {
      try { FSD_BinaryFile::GetInteger(theStream, v.anInt1); }
      catch (Standard_Failure const& anException) { aFailure = std::string("!") + anException.ExceptionType() + "="; }
      aResult << v.anInt1;
    }
    else if (anOp == "rs")
    {
      FSD_BinaryFile::ReadString(theStream, v.aText1);
      aResult << stor_ascii_hex(v.aText1);
    }
    else if (anOp == "re")
    {
      FSD_BinaryFile::ReadExtendedString(theStream, v.anAppName);
      aResult << stor_ext_hex(v.anAppName);
    }
    else if (anOp == "rh")
    {
      FSD_BinaryFile::ReadHeader(theStream, aHeader);
      aResult << aHeader.testindian << ';' << aHeader.binfo << ';' << aHeader.edata;
    }
    else if (anOp == "rhd")
    {
      FSD_BinaryFile::ReadHeaderData(theStream, v.aHeader);
      stor_print_header(aResult, v.aHeader);
    }
    else if (anOp == "rc")
    {
      FSD_BinaryFile::ReadComment(theStream, v.aComments);
      aResult << stor_ext_list(v.aComments);
    }
    else if (anOp == "ts")
      aResult << FSD_BinaryFile::TypeSectionSize(theStream);
    else if (anOp == "rt")
    {
      FSD_BinaryFile::ReadTypeInformations(theStream, v.anInt1, v.aText1);
      aResult << v.anInt1 << ',' << stor_ascii_hex(v.aText1);
    }
    else if (anOp == "rr")
    {
      FSD_BinaryFile::ReadRoot(theStream, v.aText1, v.anInt1, v.aText2);
      aResult << stor_ascii_hex(v.aText1) << ',' << v.anInt1 << ',' << stor_ascii_hex(v.aText2);
    }
    else if (anOp == "rf")
    {
      FSD_BinaryFile::ReadReferenceType(theStream, v.anInt1, v.anInt2);
      aResult << v.anInt1 << ',' << v.anInt2;
    }
    else if (anOp == "tg")
      aResult << static_cast<long long>(theStream.tellg());
    else if (anOp == "tp")
      aResult << static_cast<long long>(theStream.tellp());
    out << (aFailure.empty() ? "=" : aFailure) << aResult.str();
  }
  catch (Standard_Failure const& anException)
  {
    out << '!' << anException.ExceptionType();
  }
}

void stor_bstream(std::istream& in, std::ostream& out)
{
  std::string aHex, aToken;
  in >> aHex;
  std::stringstream aStream(stor_unhex(aHex), std::ios::in | std::ios::out | std::ios::binary);
  StorDriverValues  v;
  while (in >> aToken)
    stor_bstream_op(aToken, aStream, v, out);
  out << " F:" << stor_hex(aStream.str());
}
} // namespace

static bool storage_schema_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "stor_schema")
    stor_schema(in, out);
  else if (op == "stor_data")
  {
    occ::handle<Storage_Data> aData = new Storage_Data();
    aData->ClearErrorStatus();
    std::string aToken;
    while (in >> aToken)
      stor_data_op(aToken, aData, out);
  }
  else if (op == "stor_bucket")
    stor_bucket(in, out);
  else if (op == "stor_cb")
    stor_callbacks(in, out);
  else if (op == "stor_complete")
    stor_complete(in, out);
  else if (op == "stor_bstream")
    stor_bstream(in, out);
  else
    return false;
  return true;
}
