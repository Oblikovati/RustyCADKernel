// OSD files probe operations (OSD_FileNode, OSD_File, OSD_Directory,
// OSD_DirectoryIterator, OSD_FileIterator); twin of oracle_probe/osd/files.rs.
//
// osd_files <command> ...   Runs a script in a fresh "osdf" directory under the current
//                           directory (removed before and after), printing one token per
//                           command. Names are relative to "osdf" ("-" is the empty path,
//                           a leading '/' is absolute); text travels as hex ("-" empty).
//
// Commands on the current file: file NAME | build M | buildp M s u g w | open M |
// append M | write HEX N | read N | readb N | readline N | seek OFF W | rewind | close |
// atend | size | kind | isopen | readable | writeable | executable | lock L | unlock |
// getlock | islocked | lastline D T | tempfile | path.
// Node commands (current file, or the current directory with a 'd' prefix): failed |
// error | perror | reset | exists | remove | move NAME | copy NAME | prot | setprot s u g w |
// moment.  Directories: dir NAME | dbuild | dbuildp s u g w | tempdir.
// Iteration: iter d|f MASK NAME (entries sorted, since readdir order is the file
// system's). Fixtures: fixture NAME HEX | mkfifo NAME | chmod NAME OCTAL | symlink T NAME.
#include <OSD_Directory.hxx>
#include <OSD_DirectoryIterator.hxx>
#include <OSD_File.hxx>
#include <OSD_FileIterator.hxx>
#include <OSD_Protection.hxx>
#include <Quantity_Date.hxx>
#include <Standard_Failure.hxx>

#include <algorithm>
#include <cstdio>
#include <filesystem>
#include <sys/stat.h>
#include <unistd.h>
#include <vector>

static std::string files_hex_bytes(std::istream& in)
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
  return aBytes;
}

static std::string files_name(std::istream& in)
{
  std::string aName;
  in >> aName;
  if (aName == "-")
    return std::string();
  return aName[0] == '/' ? aName : "osdf/" + aName;
}

static int files_int(std::istream& in)
{
  int aValue = 0;
  in >> aValue;
  return aValue;
}

static OSD_Protection files_protection(std::istream& in)
{
  const int s = files_int(in), u = files_int(in), g = files_int(in), w = files_int(in);
  return OSD_Protection((OSD_SingleProtection)s,
                        (OSD_SingleProtection)u,
                        (OSD_SingleProtection)g,
                        (OSD_SingleProtection)w);
}

static std::string files_prot_text(OSD_Protection theProt)
{
  OSD_SingleProtection s, u, g, w;
  theProt.Values(s, u, g, w);
  return std::to_string(s) + "," + std::to_string(u) + "," + std::to_string(g) + ","
         + std::to_string(w);
}

static std::string files_date_text(const Quantity_Date& theDate)
{
  int mm, dd, yy, hh, mn, ss, ms, us;
  theDate.Values(mm, dd, yy, hh, mn, ss, ms, us);
  return std::to_string(yy) + "-" + std::to_string(mm) + "-" + std::to_string(dd) + "T"
         + std::to_string(hh) + ":" + std::to_string(mn) + ":" + std::to_string(ss);
}

// The node commands shared by files and directories; false when not a node command.
static bool files_node_command(OSD_FileNode& theNode, const std::string& theCmd,
                               std::istream& in, std::ostream& out)
{
  if (theCmd == "failed")
    out << theNode.Failed();
  else if (theCmd == "error")
    out << theNode.Error();
  else if (theCmd == "perror")
  {
    theNode.Perror();
    out << "ok";
  }
  else if (theCmd == "reset")
  {
    theNode.Reset();
    out << "ok";
  }
  else if (theCmd == "exists")
    out << theNode.Exists();
  else if (theCmd == "remove")
  {
    theNode.Remove();
    out << theNode.Failed();
  }
  else if (theCmd == "move" || theCmd == "copy")
  {
    const OSD_Path aTarget(files_name(in).c_str());
    if (theCmd == "move")
      theNode.Move(aTarget);
    else
      theNode.Copy(aTarget);
    out << theNode.Failed();
  }
  else if (theCmd == "prot")
    out << files_prot_text(theNode.Protection());
  else if (theCmd == "setprot")
  {
    theNode.SetProtection(files_protection(in));
    out << theNode.Failed();
  }
  else if (theCmd == "moment")
  {
    // The native and Rust probes run seconds apart, so only the relation between the two
    // moments and their plausibility are compared, not the clock values.
    Quantity_Date aCreated = theNode.CreationMoment(), anAccessed = theNode.AccessMoment();
    out << (files_date_text(aCreated) == files_date_text(anAccessed) ? "same" : "differ") << "/"
        << (aCreated.Year() >= 2020 ? "recent" : "old");
  }
  else if (theCmd == "path")
  {
    OSD_Path aPath;
    theNode.Path(aPath);
    TCollection_AsciiString aName;
    aPath.SystemName(aName);
    out << osd_escape(aName.ToCString());
  }
  else
    return false;
  return true;
}

static void files_file_io(OSD_File& theFile, const std::string& theCmd, std::istream& in,
                          std::ostream& out)
{
  if (theCmd == "write")
  {
    const std::string aBytes = files_hex_bytes(in);
    theFile.Write(TCollection_AsciiString(aBytes.c_str()), files_int(in));
    out << theFile.Failed();
  }
  else if (theCmd == "read")
  {
    TCollection_AsciiString aText("old");
    theFile.Read(aText, files_int(in));
    out << osd_escape(aText.ToCString());
  }
  else if (theCmd == "readb")
  {
    const int         aCount = files_int(in);
    std::vector<char> aBuffer(aCount > 0 ? aCount : 1, '\0');
    int               aRead = -7;
    theFile.Read(aBuffer.data(), aCount, aRead);
    out << aRead << osd_escape(std::string(aBuffer.data(), aRead > 0 ? aRead : 0));
  }
  else if (theCmd == "readline")
  {
    TCollection_AsciiString aText("old");
    const int               aRead = theFile.ReadLine(aText, files_int(in));
    out << aRead << osd_escape(aText.ToCString());
  }
  else if (theCmd == "lastline")
  {
    TCollection_AsciiString aText("old");
    const int               aDelay = files_int(in);
    out << theFile.ReadLastLine(aText, aDelay, files_int(in)) << osd_escape(aText.ToCString());
  }
  else if (theCmd == "seek")
  {
    const int anOffset = files_int(in);
    theFile.Seek(anOffset, (OSD_FromWhere)files_int(in));
    out << theFile.Failed();
  }
}

static void files_file_command(OSD_File& theFile, const std::string& theCmd, std::istream& in,
                               std::ostream& out)
{
  if (theCmd == "build" || theCmd == "open" || theCmd == "append" || theCmd == "buildp")
  {
    const OSD_OpenMode   aMode = (OSD_OpenMode)files_int(in);
    const OSD_Protection aProt = theCmd == "buildp" ? files_protection(in) : OSD_Protection();
    if (theCmd == "open")
      theFile.Open(aMode, aProt);
    else if (theCmd == "append")
      theFile.Append(aMode, aProt);
    else
      theFile.Build(aMode, aProt);
    out << theFile.IsOpen() << theFile.Failed();
  }
  else if (theCmd == "rewind")
  {
    theFile.Rewind();
    out << "ok";
  }
  else if (theCmd == "close")
  {
    theFile.Close();
    out << theFile.Failed();
  }
  else if (theCmd == "atend")
    out << theFile.IsAtEnd();
  else if (theCmd == "size")
    out << theFile.Size() << theFile.Failed();
  else if (theCmd == "kind")
    out << theFile.KindOfFile();
  else if (theCmd == "isopen")
    out << theFile.IsOpen();
  else if (theCmd == "readable")
    out << theFile.IsReadable() << theFile.IsWriteable() << theFile.IsExecutable();
  else if (theCmd == "lock")
  {
    theFile.SetLock((OSD_LockType)files_int(in));
    out << theFile.GetLock() << theFile.IsLocked() << theFile.Failed();
  }
  else if (theCmd == "unlock")
  {
    theFile.UnLock();
    out << theFile.GetLock() << theFile.IsLocked() << theFile.Failed();
  }
  else if (theCmd == "tempfile")
  {
    theFile.BuildTemporary();
    OSD_Path aPath;
    theFile.Path(aPath);
    TCollection_AsciiString aName;
    aPath.SystemName(aName);
    out << theFile.IsOpen() << (aName.Search("/tmp/CSF") == 1) << aName.Length();
  }
  else
    files_file_io(theFile, theCmd, in, out);
}

static void files_iterate(std::istream& in, std::ostream& out)
{
  std::string aKind;
  in >> aKind;
  const std::string aMask  = files_hex_bytes(in);
  const OSD_Path    aPlace(files_name(in).c_str());
  std::vector<std::string> aNames;
  if (aKind == "d")
  {
    for (OSD_DirectoryIterator anIter(aPlace, aMask.c_str()); anIter.More(); anIter.Next())
    {
      OSD_Path aPath;
      anIter.Values().Path(aPath);
      aNames.push_back(std::string(aPath.Name().ToCString()) + "|" + aPath.Extension().ToCString());
    }
  }
  else
  {
    for (OSD_FileIterator anIter(aPlace, aMask.c_str()); anIter.More(); anIter.Next())
    {
      OSD_Path aPath;
      anIter.Values().Path(aPath);
      aNames.push_back(std::string(aPath.Name().ToCString()) + "|" + aPath.Extension().ToCString());
    }
  }
  std::sort(aNames.begin(), aNames.end());
  out << aNames.size();
  for (const std::string& aName : aNames)
    out << osd_escape(aName);
}

static void files_fixture(const std::string& theCmd, std::istream& in, std::ostream& out)
{
  if (theCmd == "fixture")
  {
    const std::string aName  = files_name(in);
    const std::string aBytes = files_hex_bytes(in);
    FILE*             aFile  = std::fopen(aName.c_str(), "wb");
    out << (aFile != nullptr);
    if (aFile != nullptr)
    {
      std::fwrite(aBytes.data(), 1, aBytes.size(), aFile);
      std::fclose(aFile);
    }
  }
  else if (theCmd == "mkfifo")
    out << (mkfifo(files_name(in).c_str(), 0644) == 0);
  else if (theCmd == "chmod")
  {
    const std::string aName = files_name(in);
    std::string       anOctal;
    in >> anOctal;
    out << (chmod(aName.c_str(), (mode_t)std::stoi(anOctal, nullptr, 8)) == 0);
  }
  else if (theCmd == "symlink")
  {
    std::string aTarget;
    in >> aTarget;
    out << (symlink(aTarget.c_str(), files_name(in).c_str()) == 0);
  }
}

static void files_directory_command(OSD_Directory& theDir, const std::string& theCmd,
                                    std::istream& in, std::ostream& out)
{
  if (theCmd == "dbuild" || theCmd == "dbuildp")
  {
    theDir.Build(theCmd == "dbuildp" ? files_protection(in) : OSD_Protection());
    out << theDir.Failed();
  }
  else if (theCmd == "tempdir")
  {
    theDir = OSD_Directory::BuildTemporary();
    OSD_Path aPath;
    theDir.Path(aPath);
    TCollection_AsciiString aName;
    aPath.SystemName(aName);
    out << theDir.Exists() << (aName.Search("/tmp/CSF") == 1) << aName.Length();
  }
  else
    files_node_command(theDir, theCmd.substr(1), in, out);
}

static void files_run(std::istream& in, std::ostream& out)
{
  OSD_File      aFile;
  OSD_Directory aDir;
  std::string   aCmd;
  while (in >> aCmd)
  {
    out << ' ' << aCmd << '=';
    try
    {
      if (aCmd == "file")
        aFile = OSD_File(OSD_Path(files_name(in).c_str()));
      else if (aCmd == "dir")
        aDir = OSD_Directory(OSD_Path(files_name(in).c_str()));
      else if (aCmd == "iter")
        files_iterate(in, out);
      else if (aCmd == "fixture" || aCmd == "mkfifo" || aCmd == "chmod" || aCmd == "symlink")
        files_fixture(aCmd, in, out);
      else if (aCmd[0] == 'd' || aCmd == "tempdir")
        files_directory_command(aDir, aCmd, in, out);
      else if (!files_node_command(aFile, aCmd, in, out))
        files_file_command(aFile, aCmd, in, out);
    }
    catch (const Standard_Failure& theFailure)
    {
      out << "EXC " << theFailure.ExceptionType() << osd_escape(theFailure.what());
    }
  }
  // A pending error would make Close raise, and the destructor call it again.
  aFile.Reset();
  if (aFile.IsOpen())
    aFile.Close();
}

static bool osd_files_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op != "osd_files")
    return false;
  std::error_code anError;
  std::filesystem::remove_all("osdf", anError);
  mkdir("osdf", 0755);
  files_run(in, out);
  std::filesystem::remove_all("osdf", anError);
  return true;
}
