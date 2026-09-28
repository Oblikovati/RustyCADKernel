// Resource package probe operations (twin of crates/tkernel/examples/oracle_probe/resource.rs).
//
// Byte strings and UTF-16 units travel hex-encoded ("-" for empty). Every operation prints
// `throw <Type>: <message>` for a Standard_Failure, followed by the observable state.
//
// res_decode <format> <hex bytes>           ConvertFormatToUnicode(format, bytes)
// res_gbk <hex bytes> / res_big5 <hex bytes> the direct GBK/Big5 decoders and their flag
// res_encode <format> <maxsize> <hex units> ConvertUnicodeToFormat into a 0xA5-filled buffer
// res_path <user> <hex dir>                  GetResourcePath with CSF_ProbeDefaults/UserDefaults
// res_load <verbose> <debug> <dir> <hex>     Resource_Manager("Probe") over dir/Probe
// res_dirs <verbose> <dir> <hex defaults dir> <hex user dir> <hex defaults> <hex user>
//                                            the four-argument constructor
// res_save <verbose> <dir> <assignments...>  SetResource i:<k>:<int> r:<k>:<bits> s:<k>:<hex>,
//                                            then Save into dir
// res_ext <format> <hex bytes> <hex units>   ExtValue of a byte value and SetResource(char16_t*)
// res_charset <dir> <hex CharSet file>       ReadFormat/GetFormat from dir/CharSet
// Byte sources carry eight trailing NUL bytes so the native overrun after a lead byte right
// before the terminator reads defined zeros, which is what the Rust model reads.
#include <Resource_Manager.hxx>
#include <Resource_Unicode.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_ExtendedString.hxx>

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

static std::string res_hex(const std::string& theBytes)
{
  static const char THE_DIGITS[] = "0123456789abcdef";
  std::string       aText;
  for (unsigned char aByte : theBytes)
  {
    aText += THE_DIGITS[aByte >> 4];
    aText += THE_DIGITS[aByte & 15];
  }
  return aText.empty() ? "-" : aText;
}

static std::string res_unhex(const std::string& theText)
{
  std::string aBytes;
  if (theText == "-")
  {
    return aBytes;
  }
  for (size_t anIndex = 0; anIndex + 1 < theText.size(); anIndex += 2)
  {
    aBytes += static_cast<char>(std::stoul(theText.substr(anIndex, 2), nullptr, 16));
  }
  return aBytes;
}

static std::string res_token(std::istream& in)
{
  std::string aToken;
  if (!(in >> aToken))
  {
    throw std::runtime_error("resource probe: missing field");
  }
  return aToken;
}

static std::string res_units(const TCollection_ExtendedString& theText)
{
  std::ostringstream anOut;
  anOut << "n=" << theText.Length() << " u=";
  char aBuffer[8];
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
  {
    std::snprintf(aBuffer, sizeof(aBuffer), "%04x", static_cast<unsigned>(theText.Value(anIndex)));
    anOut << aBuffer;
  }
  if (theText.Length() == 0)
  {
    anOut << '-';
  }
  return anOut.str();
}

static TCollection_ExtendedString res_ext_string(const std::string& theHex)
{
  std::vector<char16_t> aUnits;
  if (theHex != "-")
  {
    for (size_t anIndex = 0; anIndex + 3 < theHex.size(); anIndex += 4)
    {
      aUnits.push_back(static_cast<char16_t>(std::stoul(theHex.substr(anIndex, 4), nullptr, 16)));
    }
  }
  aUnits.push_back(0);
  return TCollection_ExtendedString(aUnits.data(), static_cast<int>(aUnits.size()) - 1);
}

static std::string res_failure(const Standard_Failure& theFailure)
{
  return std::string("throw ") + theFailure.ExceptionType() + ": " + theFailure.what() + " ";
}

static std::string res_source(std::istream& in)
{
  return res_unhex(res_token(in)) + std::string(8, '\0');
}

static void res_decode(std::istream& in, std::ostream& out)
{
  const auto                 aFormat = static_cast<Resource_FormatType>(std::stoi(res_token(in)));
  const std::string          aSource = res_source(in);
  TCollection_ExtendedString aText;
  try
  {
    Resource_Unicode::ConvertFormatToUnicode(aFormat, aSource.c_str(), aText);
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure);
  }
  out << res_units(aText);
}

static void res_direct(const std::string& theOp, std::istream& in, std::ostream& out)
{
  const std::string          aSource = res_source(in);
  TCollection_ExtendedString aText;
  try
  {
    const bool isDone = theOp == "res_gbk"
                          ? Resource_Unicode::ConvertGBKToUnicode(aSource.c_str(), aText)
                          : Resource_Unicode::ConvertBig5ToUnicode(aSource.c_str(), aText);
    out << "b=" << isDone << ' ';
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure);
  }
  out << res_units(aText);
}

static void res_encode(std::istream& in, std::ostream& out)
{
  const auto                       aFormat = static_cast<Resource_FormatType>(std::stoi(res_token(in)));
  const int                        aMaxSize = std::stoi(res_token(in));
  const TCollection_ExtendedString aText    = res_ext_string(res_token(in));
  std::vector<char>                aBuffer(static_cast<size_t>(aMaxSize) + 16, static_cast<char>(0xA5));
  Standard_PCharacter              aPointer = aBuffer.data();
  try
  {
    const bool isDone = Resource_Unicode::ConvertUnicodeToFormat(aFormat, aText, aPointer, aMaxSize);
    out << "b=" << isDone << ' ';
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure);
  }
  out << "buf=" << res_hex(std::string(aBuffer.data(), static_cast<size_t>(aMaxSize)));
}

//! Sets or clears one process environment variable ("-" clears).
static void res_setenv(const char* theName, const std::string& theValue)
{
  if (theValue == "-")
  {
    unsetenv(theName);
  }
  else
  {
    setenv(theName, theValue.c_str(), 1);
  }
}

static void res_write_file(const std::string& theDir, const char* theName, const std::string& theHex)
{
  std::filesystem::create_directories(theDir);
  std::ofstream aFile(theDir + "/" + theName, std::ios::binary);
  const std::string aBytes = res_unhex(theHex);
  aFile.write(aBytes.data(), static_cast<std::streamsize>(aBytes.size()));
}

//! Redirects std::cout into a buffer for the lifetime of the object.
struct ResourceConsoleCapture
{
  std::ostringstream myText;
  std::streambuf*    myPrevious;

  ResourceConsoleCapture()
      : myPrevious(std::cout.rdbuf(myText.rdbuf()))
  {
  }

  ~ResourceConsoleCapture() { std::cout.rdbuf(myPrevious); }
};

//! Sorted reference or user map entries with their Integer/Real readings.
static std::string res_entries(Resource_Manager& theManager, const bool theReference)
{
  std::map<std::string, std::string> aSorted;
  for (NCollection_DataMap<TCollection_AsciiString, TCollection_AsciiString>::Iterator anIter(
         theManager.GetMap(theReference));
       anIter.More();
       anIter.Next())
  {
    aSorted[anIter.Key().ToCString()] = anIter.Value().ToCString();
  }
  std::ostringstream anOut;
  for (const auto& anEntry : aSorted)
  {
    anOut << ' ' << res_hex(anEntry.first) << '=' << res_hex(anEntry.second) << ':';
    try
    {
      anOut << theManager.Integer(anEntry.first.c_str());
    }
    catch (const Standard_Failure& theFailure)
    {
      anOut << theFailure.ExceptionType();
    }
    anOut << ':';
    try
    {
      const double aReal = theManager.Real(anEntry.first.c_str());
      uint64_t     aBits = 0;
      std::memcpy(&aBits, &aReal, sizeof(aBits));
      char aBuffer[24];
      std::snprintf(aBuffer, sizeof(aBuffer), "%016llx", static_cast<unsigned long long>(aBits));
      anOut << aBuffer;
    }
    catch (const Standard_Failure& theFailure)
    {
      anOut << theFailure.ExceptionType();
    }
  }
  return anOut.str();
}

static void res_report(Resource_Manager& theManager, const std::string& theConsole, std::ostream& out)
{
  out << "init=" << theManager.IsInitialized() << " console=" << res_hex(theConsole)
      << " ref" << res_entries(theManager, true) << " user" << res_entries(theManager, false);
}

static void res_load(std::istream& in, std::ostream& out)
{
  const bool        isVerbose = res_token(in) == "1";
  const std::string aDebug    = res_token(in) == "1" ? "1" : "-";
  const std::string aDir      = res_token(in);
  std::filesystem::remove_all(aDir);
  res_write_file(aDir, "Probe", res_token(in));
  res_setenv("CSF_ProbeDefaults", aDir);
  res_setenv("CSF_ProbeUserDefaults", "-");
  res_setenv("ResourceDebug", aDebug);
  res_setenv("CSF_ResourceVerbose", "-");
  ResourceConsoleCapture aCapture;
  try
  {
    Resource_Manager aManager("Probe", isVerbose);
    res_report(aManager, aCapture.myText.str(), out);
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure) << "console=" << res_hex(aCapture.myText.str());
  }
  res_setenv("ResourceDebug", "-");
}

static void res_dirs(std::istream& in, std::ostream& out)
{
  const bool        isVerbose = res_token(in) == "1";
  const std::string aDir      = res_token(in);
  const std::string aDefaults = res_unhex(res_token(in));
  const std::string aUser     = res_unhex(res_token(in));
  const std::string aDefaultsFile = res_token(in);
  const std::string aUserFile     = res_token(in);
  std::filesystem::remove_all(aDir);
  res_write_file(aDir + "/defaults", "Probe", aDefaultsFile);
  res_write_file(aDir + "/user", "Probe", aUserFile);
  ResourceConsoleCapture aCapture;
  try
  {
    Resource_Manager aManager(TCollection_AsciiString("Probe"),
                              TCollection_AsciiString(aDefaults.c_str()),
                              TCollection_AsciiString(aUser.c_str()),
                              isVerbose);
    res_report(aManager, aCapture.myText.str(), out);
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure) << "console=" << res_hex(aCapture.myText.str());
  }
}

static void res_assign(Resource_Manager& theManager, const std::string& theAssignment)
{
  const size_t      aSecond = theAssignment.find(':', 2);
  const std::string aName   = res_unhex(theAssignment.substr(2, aSecond - 2));
  const std::string aValue  = theAssignment.substr(aSecond + 1);
  if (theAssignment[0] == 'i')
  {
    theManager.SetResource(aName.c_str(), std::stoi(aValue));
  }
  else if (theAssignment[0] == 'r')
  {
    const uint64_t aBits = std::stoull(aValue, nullptr, 16);
    double         aReal = 0.0;
    std::memcpy(&aReal, &aBits, sizeof(aReal));
    theManager.SetResource(aName.c_str(), aReal);
  }
  else
  {
    theManager.SetResource(aName.c_str(), res_unhex(aValue).c_str());
  }
}

static void res_save(std::istream& in, std::ostream& out)
{
  const bool        isVerbose = res_token(in) == "1";
  const std::string aDir      = res_token(in);
  std::filesystem::remove_all(aDir);
  res_setenv("CSF_ProbeDefaults", "-");
  res_setenv("CSF_ProbeUserDefaults", "-");
  ResourceConsoleCapture aCapture;
  Resource_Manager       aManager("Probe", isVerbose);
  std::string            anAssignment;
  while (in >> anAssignment)
  {
    res_assign(aManager, anAssignment);
  }
  res_setenv("CSF_ProbeUserDefaults", aDir + "/nested/user");
  const bool isSaved = aManager.Save();
  std::ifstream aFile(aDir + "/nested/user/Probe", std::ios::binary);
  std::ostringstream aContent;
  aContent << aFile.rdbuf();
  res_setenv("CSF_ProbeUserDefaults", "-");
  out << "saved=" << isSaved << " file=" << res_hex(aContent.str()) << ' ';
  res_report(aManager, aCapture.myText.str(), out);
}

static void res_path(std::istream& in, std::ostream& out)
{
  const bool        isUser = res_token(in) == "1";
  const std::string aDir   = res_token(in);
  res_setenv(isUser ? "CSF_ProbeUserDefaults" : "CSF_ProbeDefaults", aDir == "-" ? "-" : res_unhex(aDir));
  TCollection_AsciiString aPath("stale");
  Resource_Manager::GetResourcePath(aPath, "Probe", isUser);
  res_setenv(isUser ? "CSF_ProbeUserDefaults" : "CSF_ProbeDefaults", "-");
  out << res_hex(aPath.ToCString());
}

static void res_ext(std::istream& in, std::ostream& out)
{
  Resource_Unicode::SetFormat(static_cast<Resource_FormatType>(std::stoi(res_token(in))));
  const std::string                aBytes = res_unhex(res_token(in));
  const TCollection_ExtendedString aUnits = res_ext_string(res_token(in));
  Resource_Manager                 aManager;
  aManager.SetResource("k", aBytes.c_str());
  try
  {
    out << res_units(TCollection_ExtendedString(aManager.ExtValue("k"))) << ' ';
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure);
  }
  try
  {
    aManager.SetResource("e", aUnits.ToExtString());
    out << "e=" << res_hex(aManager.Value("e")) << ' ';
  }
  catch (const Standard_Failure& theFailure)
  {
    out << res_failure(theFailure);
  }
  out << "cached " << res_units(TCollection_ExtendedString(aManager.ExtValue("e")));
}

static void res_charset(std::istream& in, std::ostream& out)
{
  const std::string aDir = res_token(in);
  std::filesystem::remove_all(aDir);
  res_write_file(aDir, "CharSet", res_token(in));
  res_setenv("CSF_CharSetDefaults", aDir);
  res_setenv("CSF_CharSetUserDefaults", "-");
  Resource_Unicode::ReadFormat();
  res_setenv("CSF_CharSetDefaults", "-");
  out << "format=" << static_cast<int>(Resource_Unicode::GetFormat());
}

static bool resource_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "res_decode")
    res_decode(in, out);
  else if (op == "res_gbk" || op == "res_big5")
    res_direct(op, in, out);
  else if (op == "res_encode")
    res_encode(in, out);
  else if (op == "res_path")
    res_path(in, out);
  else if (op == "res_load")
    res_load(in, out);
  else if (op == "res_dirs")
    res_dirs(in, out);
  else if (op == "res_save")
    res_save(in, out);
  else if (op == "res_ext")
    res_ext(in, out);
  else if (op == "res_charset")
    res_charset(in, out);
  else
    return false;
  return true;
}
