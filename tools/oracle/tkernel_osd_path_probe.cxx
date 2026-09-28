// OSD path probe operations (OSD_Path); twin of oracle_probe/osd/path.rs.
//
// Strings travel as lowercase hex ("-" is empty) and print as [text] with bytes outside
// 0x20..0x7e and the characters '\\', '[' and ']' as \xHH.
//
// osd_path_parse <systype> <path>        OSD_Path(path, systype): Values, TrekLength,
//                                        SystemName in seven dialects, IsValid.
// osd_path_values <7 components>         OSD_Path(node, ..., ext): the same report.
// osd_path_trek <trek> <op> [args]       SetValues with <trek>, then UpTrek | DownTrek |
//                                        RemoveATrek(int|name) | TrekValue | InsertATrek.
// osd_path_rel <dir> <file>              RelativePath.
// osd_path_abs <dir> <rel>               AbsolutePath.
// osd_path_split <path>                  FolderAndFileFromPath, FileNameAndExtension.
// osd_path_kind <path>                   The nine static syntax predicates.
// osd_path_setters <7 components>        Each Set* on a default path, then the report.
#include <OSD_Path.hxx>
#include <Standard_Failure.hxx>

#include <cstdio>

static TCollection_AsciiString path_hex(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  std::string aBytes;
  if (aToken != "-")
  {
    for (size_t anIndex = 0; anIndex + 1 < aToken.size(); anIndex += 2)
    {
      aBytes += static_cast<char>(std::stoi(aToken.substr(anIndex, 2), nullptr, 16));
    }
  }
  return TCollection_AsciiString(aBytes.c_str());
}

static std::string path_text(const TCollection_AsciiString& theText)
{
  std::string anOut = "[";
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
  {
    const unsigned char aByte = static_cast<unsigned char>(theText.Value(anIndex));
    if (aByte < 0x20 || aByte > 0x7e || aByte == '\\' || aByte == '[' || aByte == ']')
    {
      char aBuffer[8];
      std::snprintf(aBuffer, sizeof(aBuffer), "\\x%02x", aByte);
      anOut += aBuffer;
    }
    else
    {
      anOut += static_cast<char>(aByte);
    }
  }
  return anOut + "]";
}

static void path_failure(std::ostream& out, const Standard_Failure& theFailure)
{
  out << "EXC " << theFailure.ExceptionType() << path_text(theFailure.what());
}

static void path_report(std::ostream& out, const OSD_Path& thePath)
{
  TCollection_AsciiString aNode, aUser, aPassword, aDisk, aTrek, aName, anExt;
  thePath.Values(aNode, aUser, aPassword, aDisk, aTrek, aName, anExt);
  out << path_text(aNode) << path_text(aUser) << path_text(aPassword) << path_text(aDisk)
      << path_text(aTrek) << path_text(aName) << path_text(anExt) << ' ' << thePath.TrekLength();
  const OSD_SysType aTypes[] = {OSD_Default, OSD_UnixBSD, OSD_VMS,    OSD_OS2,
                                OSD_MacOs,   OSD_WindowsNT, OSD_Unknown};
  for (const OSD_SysType aType : aTypes)
  {
    out << ' ';
    try
    {
      TCollection_AsciiString aFull;
      thePath.SystemName(aFull, aType);
      out << path_text(aFull);
    }
    catch (const Standard_Failure& theFailure)
    {
      path_failure(out, theFailure);
    }
  }
}

static void path_parse(std::istream& in, std::ostream& out)
{
  int aType = 0;
  in >> aType;
  const TCollection_AsciiString aText = path_hex(in);
  out << OSD_Path::IsValid(aText, static_cast<OSD_SysType>(aType))
      << OSD_Path::IsValid(aText, OSD_Default) << ' ';
  try
  {
    path_report(out, OSD_Path(aText, static_cast<OSD_SysType>(aType)));
  }
  catch (const Standard_Failure& theFailure)
  {
    path_failure(out, theFailure);
  }
}

static void path_values(std::istream& in, std::ostream& out)
{
  const TCollection_AsciiString aNode = path_hex(in), aUser = path_hex(in),
                                aPassword = path_hex(in), aDisk = path_hex(in),
                                aTrek = path_hex(in), aName = path_hex(in), anExt = path_hex(in);
  path_report(out, OSD_Path(aNode, aUser, aPassword, aDisk, aTrek, aName, anExt));
}

static void path_setters(std::istream& in, std::ostream& out)
{
  OSD_Path aPath;
  aPath.SetNode(path_hex(in));
  aPath.SetUserName(path_hex(in));
  aPath.SetPassword(path_hex(in));
  aPath.SetDisk(path_hex(in));
  aPath.SetTrek(path_hex(in));
  aPath.SetName(path_hex(in));
  aPath.SetExtension(path_hex(in));
  out << path_text(aPath.Node()) << path_text(aPath.UserName()) << path_text(aPath.Password())
      << path_text(aPath.Disk()) << path_text(aPath.Trek()) << path_text(aPath.Name())
      << path_text(aPath.Extension()) << ' ';
  TCollection_AsciiString anExpanded("x");
  aPath.ExpandedName(anExpanded);
  out << path_text(anExpanded) << ' ';
  path_report(out, aPath);
}

static void path_trek_op(OSD_Path& thePath, const std::string& theOp, std::istream& in,
                         std::ostream& out)
{
  int aWhere = 0;
  if (theOp == "up")
    thePath.UpTrek();
  else if (theOp == "down")
    thePath.DownTrek(path_hex(in));
  else if (theOp == "rm")
  {
    in >> aWhere;
    thePath.RemoveATrek(aWhere);
  }
  else if (theOp == "rmname")
    thePath.RemoveATrek(path_hex(in));
  else if (theOp == "value")
  {
    in >> aWhere;
    out << path_text(thePath.TrekValue(aWhere));
  }
  else
  {
    const TCollection_AsciiString aName = path_hex(in);
    in >> aWhere;
    thePath.InsertATrek(aName, aWhere);
  }
}

static void path_trek(std::istream& in, std::ostream& out)
{
  OSD_Path aPath;
  aPath.SetValues("", "", "", "", path_hex(in), "n", ".e");
  std::string anOp;
  in >> anOp;
  try
  {
    path_trek_op(aPath, anOp, in, out);
    out << "ok";
  }
  catch (const Standard_Failure& theFailure)
  {
    path_failure(out, theFailure);
  }
  out << ' ' << path_text(aPath.Trek()) << ' ' << aPath.TrekLength();
}

static void path_rel_abs(const std::string& op, std::istream& in, std::ostream& out)
{
  const TCollection_AsciiString aDir = path_hex(in), aFile = path_hex(in);
  try
  {
    out << path_text(op == "osd_path_rel" ? OSD_Path::RelativePath(aDir, aFile)
                                          : OSD_Path::AbsolutePath(aDir, aFile));
  }
  catch (const Standard_Failure& theFailure)
  {
    path_failure(out, theFailure);
  }
}

static void path_split(std::istream& in, std::ostream& out)
{
  const TCollection_AsciiString aText = path_hex(in);
  TCollection_AsciiString       aFolder("old"), aFile("old"), aName("old"), anExt("old");
  OSD_Path::FolderAndFileFromPath(aText, aFolder, aFile);
  OSD_Path::FileNameAndExtension(aText, aName, anExt);
  out << path_text(aFolder) << path_text(aFile) << path_text(aName) << path_text(anExt);
}

static void path_kind(std::istream& in, std::ostream& out)
{
  const TCollection_AsciiString aText = path_hex(in);
  const char*                   aPath = aText.ToCString();
  out << OSD_Path::IsDosPath(aPath) << OSD_Path::IsNtExtendedPath(aPath)
      << OSD_Path::IsUncPath(aPath) << OSD_Path::IsUncExtendedPath(aPath)
      << OSD_Path::IsUnixPath(aPath) << OSD_Path::IsContentProtocolPath(aPath)
      << OSD_Path::IsRemoteProtocolPath(aPath) << OSD_Path::IsRelativePath(aPath)
      << OSD_Path::IsAbsolutePath(aPath);
}

static bool osd_path_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_path_parse")
    path_parse(in, out);
  else if (op == "osd_path_values")
    path_values(in, out);
  else if (op == "osd_path_setters")
    path_setters(in, out);
  else if (op == "osd_path_trek")
    path_trek(in, out);
  else if (op == "osd_path_rel" || op == "osd_path_abs")
    path_rel_abs(op, in, out);
  else if (op == "osd_path_split")
    path_split(in, out);
  else if (op == "osd_path_kind")
    path_kind(in, out);
  else
    return false;
  return true;
}
