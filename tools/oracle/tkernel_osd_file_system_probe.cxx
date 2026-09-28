// OSD file_system probe operations (OSD_FileSystem, OSD_LocalFileSystem, OSD_CachedFileSystem,
// OSD_FileSystemSelector, OSD_StreamBuffer, OSD_OpenFile); twin of oracle_probe/osd/file_system.rs.
//
// Every request works in a fresh directory osdfs_tmp/ under the current directory, holding
// f1.bin (and f2.bin) created from content tokens ("-" empty, "NONE" absent); it is removed
// afterwards. Modes are raw libstdc++ openmode bits (app 1, ate 2, binary 4, in 8, out 16,
// trunc 32); stream states are rdstate() bits (bad 1, eof 2, fail 4).
//
// osd_fs_local_buffer <c> <mode> <offset> <withsize>  LocalFileSystem::OpenStreamBuffer,
//                                                   then sgetn/sputn/pubseekoff.
// osd_fs_istream <c> <mode> <offset> <offset2>       OpenIStream, read, IsOpenIStream, reuse.
// osd_fs_ostream <c> <mode>                          OpenOStream, write/flush/seekp/tellp.
// osd_fs_supported <url>                             IsSupportedPath of every file system.
// osd_fs_selector <fakes> <ops> <url>                selector order and delegation log.
// osd_fs_cached <c1> <c2> <ops>                      CachedFileSystem stream/buffer reuse.
// osd_fs_default                                     DefaultFileSystem and its protocols.
// osd_fs_fopen <c> <mode> <utf8>                     OSD_OpenFile then read/write.
// osd_fs_fd <c> <mode>                               OSD_OpenFileDescriptor then read/write.
// osd_fs_ctime <c>                                   OSD_FileStatCTime of f1 and a missing file.
// osd_fs_filebuf <c> <mode>                          OSD_OpenStream(std::filebuf&, ...).
// osd_fs_fstream <c> <kind> <mode> <utf8>            OSD_OpenStream(T&, ...) for i/o/io streams.
#include <OSD_CachedFileSystem.hxx>
#include <OSD_FileSystemSelector.hxx>
#include <OSD_LocalFileSystem.hxx>
#include <OSD_OpenFile.hxx>
#include <Standard_Failure.hxx>

#include <cstdio>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <sys/stat.h>
#include <unistd.h>
#include <vector>

static const char* const FSP_DIR = "osdfs_tmp";

static std::string fsp_path(int theIndex)
{
  return std::string(FSP_DIR) + "/f" + std::to_string(theIndex) + ".bin";
}

static void fsp_write_file(int theIndex, const std::string& theContent)
{
  if (theContent == "NONE")
    return;
  std::ofstream aFile(fsp_path(theIndex), std::ios::binary);
  aFile << theContent;
}

static void fsp_setup(std::istream& in, int theCount)
{
  std::filesystem::remove_all(FSP_DIR);
  std::filesystem::create_directory(FSP_DIR);
  for (int anIndex = 1; anIndex <= theCount; ++anIndex)
    fsp_write_file(anIndex, osd_token(in));
}

static std::string fsp_content(int theIndex)
{
  std::ifstream aFile(fsp_path(theIndex), std::ios::binary);
  if (!aFile.is_open())
    return "NOFILE";
  std::ostringstream aText;
  aText << aFile.rdbuf();
  return osd_escape(aText.str());
}

static void fsp_finish(std::ostream& out, int theCount)
{
  for (int anIndex = 1; anIndex <= theCount; ++anIndex)
    out << ' ' << fsp_content(anIndex);
  std::filesystem::remove_all(FSP_DIR);
}

static std::ios_base::openmode fsp_mode(std::istream& in)
{
  int aBits = 0;
  in >> aBits;
  return static_cast<std::ios_base::openmode>(aBits);
}

static void fsp_raise(std::ostream& out, const Standard_Failure& theFailure)
{
  const std::string aText = theFailure.what();
  const size_t      aCut  = aText.find("': \n");
  out << "raise " << theFailure.ExceptionType()
      << osd_escape(aCut == std::string::npos ? aText : aText.substr(aCut + 4));
}

static std::string fsp_read(std::streambuf& theBuf, int theCount)
{
  std::vector<char> aBytes(theCount);
  return osd_escape(std::string(aBytes.data(), (size_t)theBuf.sgetn(aBytes.data(), theCount)));
}

static std::string fsp_read(std::istream& theStream, int theCount)
{
  std::vector<char> aBytes(theCount);
  theStream.read(aBytes.data(), theCount);
  return std::to_string(theStream.gcount()) + osd_escape(std::string(aBytes.data(), (size_t)theStream.gcount()))
         + std::to_string(theStream.rdstate());
}

static void osd_fs_local_buffer(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::ios_base::openmode aMode = fsp_mode(in);
  long long anOffset = 0;
  int       aWithSize = 0;
  in >> anOffset >> aWithSize;
  OSD_LocalFileSystem aFs;
  int64_t             aSize = -7;
  try
  {
    std::shared_ptr<std::streambuf> aBuf =
      aFs.OpenStreamBuffer(fsp_path(1).c_str(), aMode, anOffset, aWithSize ? &aSize : nullptr);
    out << (aBuf ? "buf " : "null ") << aSize;
    if (aBuf)
    {
      out << ' ' << fsp_read(*aBuf, 64) << ' ' << aBuf->sputn("WX", 2) << ' '
          << (long long)aBuf->pubseekoff(0, std::ios_base::cur, std::ios_base::in);
    }
  }
  catch (const Standard_Failure& theFailure)
  {
    fsp_raise(out, theFailure);
  }
  fsp_finish(out, 1);
}

static void osd_fs_istream(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::ios_base::openmode aMode = fsp_mode(in);
  long long anOffset = 0, anOffset2 = 0;
  in >> anOffset >> anOffset2;
  OSD_LocalFileSystem                aFs;
  const TCollection_AsciiString      anUrl(fsp_path(1).c_str());
  const occ::handle<OSD_FileSystem>& aDefault = OSD_FileSystem::DefaultFileSystem();
  try
  {
    std::shared_ptr<std::istream> aStream = aFs.OpenIStream(anUrl, aMode, anOffset);
    out << (aStream ? "s" : "null");
    if (aStream)
    {
      out << ' ' << fsp_read(*aStream, 4) << ' ' << (long long)aStream->tellg() << ' '
          << aStream->rdstate() << ' ' << aFs.IsOpenIStream(aStream) << aDefault->IsOpenIStream(aStream);
      std::shared_ptr<std::istream> aSecond = aFs.OpenIStream(anUrl, aMode, anOffset2, aStream);
      out << ' ' << (aSecond == aStream) << ' ' << (aSecond ? fsp_read(*aSecond, 2) : "null");
    }
    // A plain std::ifstream and an OSD_IOStreamBuffer are not OSD_IStreamBuffer streams.
    std::shared_ptr<std::istream> aPlain = std::make_shared<std::ifstream>(fsp_path(1));
    std::shared_ptr<std::filebuf> aFileBuf = std::make_shared<std::filebuf>();
    aFileBuf->open(fsp_path(1), std::ios_base::in);
    std::shared_ptr<OSD_IOStreamBuffer> anIo =
      std::make_shared<OSD_IOStreamBuffer>(std::string("io-url"), aFileBuf);
    out << ' ' << aFs.IsOpenIStream(aPlain) << aFs.IsOpenIStream(anIo) << aFs.IsOpenOStream(anIo)
        << osd_escape(anIo->Url());
  }
  catch (const Standard_Failure& theFailure)
  {
    fsp_raise(out, theFailure);
  }
  fsp_finish(out, 1);
}

static void osd_fs_ostream(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::ios_base::openmode aMode = fsp_mode(in);
  OSD_LocalFileSystem           aFs;
  {
    std::shared_ptr<std::ostream> aStream = aFs.OpenOStream(fsp_path(1).c_str(), aMode);
    out << (aStream ? "s" : "null");
    if (aStream)
    {
      aStream->write("HELLO", 5);
      aStream->flush();
      out << ' ' << aStream->rdstate() << ' ' << (long long)aStream->tellp() << ' '
          << aFs.IsOpenOStream(aStream)
          << OSD_FileSystem::DefaultFileSystem()->IsOpenOStream(aStream);
      aStream->seekp(1, std::ios_base::beg);
      aStream->write("Z", 1);
      out << ' ' << aStream->rdstate() << ' ' << (long long)aStream->tellp();
    }
  }
  fsp_finish(out, 1);
}

static void osd_fs_supported(std::istream& in, std::ostream& out)
{
  const TCollection_AsciiString anUrl(osd_token(in).c_str());
  OSD_CachedFileSystem          aCached;
  OSD_FileSystemSelector        anEmpty;
  out << OSD_LocalFileSystem().IsSupportedPath(anUrl)
      << OSD_FileSystem::DefaultFileSystem()->IsSupportedPath(anUrl)
      << aCached.IsSupportedPath(anUrl) << anEmpty.IsSupportedPath(anUrl);
}

//! A logging fake protocol: supports URLs starting with its prefix ("*" = all) and
//! opens a string buffer holding its name when it is allowed to open.
class FspFake : public OSD_FileSystem
{
public:
  FspFake(const std::string& theName, const std::string& thePrefix, bool theOpens, std::string& theLog)
      : myName(theName), myPrefix(thePrefix), myOpens(theOpens), myLog(theLog)
  {
  }
  const std::string& Name() const { return myName; }
  bool IsSupportedPath(const TCollection_AsciiString& theUrl) const override
  {
    myLog += myName + "s ";
    return myPrefix == "*" || std::string(theUrl.ToCString()).rfind(myPrefix, 0) == 0;
  }
  bool IsOpenIStream(const std::shared_ptr<std::istream>& theStream) const override
  {
    myLog += myName + "i ";
    return myOpens && std::dynamic_pointer_cast<OSD_IStreamBuffer>(theStream) != nullptr;
  }
  bool IsOpenOStream(const std::shared_ptr<std::ostream>& theStream) const override
  {
    myLog += myName + "o ";
    return myOpens && std::dynamic_pointer_cast<OSD_OStreamBuffer>(theStream) != nullptr;
  }
  std::shared_ptr<std::streambuf> OpenStreamBuffer(const TCollection_AsciiString&,
                                                   const std::ios_base::openmode,
                                                   const int64_t,
                                                   int64_t* theOutBufSize) override
  {
    myLog += myName + "b ";
    if (!myOpens)
      return std::shared_ptr<std::streambuf>();
    if (theOutBufSize != nullptr)
      *theOutBufSize = (int64_t)myName.size();
    return std::make_shared<std::stringbuf>(myName, std::ios_base::in | std::ios_base::out);
  }

private:
  std::string  myName;
  std::string  myPrefix;
  bool         myOpens;
  std::string& myLog;
};

class FspSelector : public OSD_FileSystemSelector
{
public:
  std::string Order() const
  {
    std::string anOrder;
    for (NCollection_List<occ::handle<OSD_FileSystem>>::Iterator anIter(myProtocols); anIter.More(); anIter.Next())
      anOrder += occ::down_cast<FspFake>(anIter.Value())->Name();
    return anOrder.empty() ? "-" : anOrder;
  }
};

static std::vector<std::string> fsp_split(const std::string& theText, char theSep)
{
  std::vector<std::string> aParts;
  std::string              aPart;
  std::istringstream       aStream(theText);
  while (std::getline(aStream, aPart, theSep))
    aParts.push_back(aPart);
  return aParts;
}

static void fsp_flush_log(std::ostream& out, std::string& theLog)
{
  out << '|' << theLog;
  theLog.clear();
}

static void osd_fs_selector(std::istream& in, std::ostream& out)
{
  const std::string              aSpecs = osd_token(in), anOps = osd_token(in);
  const TCollection_AsciiString  anUrl(osd_token(in).c_str());
  std::string                    aLog;
  std::vector<occ::handle<OSD_FileSystem>> aFakes;
  for (const std::string& aSpec : fsp_split(aSpecs, ','))
  {
    const std::vector<std::string> aFields = fsp_split(aSpec, '/');
    aFakes.push_back(new FspFake(aFields[0], aFields[1], aFields[2] == "1", aLog));
  }
  FspSelector aSelector;
  for (const std::string& anOp : fsp_split(anOps, ','))
  {
    const occ::handle<OSD_FileSystem>& aFake = aFakes[std::stoi(anOp.substr(1))];
    if (anOp[0] == '-')
      aSelector.RemoveProtocol(aFake);
    else
      aSelector.AddProtocol(aFake, anOp[0] == '^');
  }
  out << aSelector.Order() << ' ' << aSelector.IsSupportedPath(anUrl);
  fsp_flush_log(out, aLog);
  int64_t aSize = -7;
  std::shared_ptr<std::streambuf> aBuf = aSelector.OpenStreamBuffer(anUrl, std::ios_base::in, 0, &aSize);
  out << (aBuf ? fsp_read(*aBuf, 8) : "null") << aSize;
  fsp_flush_log(out, aLog);
  std::shared_ptr<std::istream> anIn = aSelector.OpenIStream(anUrl, std::ios_base::in);
  out << (anIn ? fsp_read(*anIn, 8) : "null");
  fsp_flush_log(out, aLog);
  out << (anIn ? aSelector.IsOpenIStream(anIn) : false);
  fsp_flush_log(out, aLog);
  std::shared_ptr<std::ostream> anOut = aSelector.OpenOStream(anUrl, std::ios_base::out);
  out << (anOut ? "s" : "null");
  fsp_flush_log(out, aLog);
  out << (anOut ? aSelector.IsOpenOStream(anOut) : false);
  fsp_flush_log(out, aLog);
}

static void osd_fs_cached(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 2);
  const std::string                 anOps = osd_token(in);
  const occ::handle<OSD_FileSystem> aLocal = new OSD_LocalFileSystem();
  OSD_CachedFileSystem              aDefaultCached;
  out << (aDefaultCached.LinkedFileSystem() == OSD_FileSystem::DefaultFileSystem());
  aDefaultCached.SetLinkedFileSystem(aLocal);
  out << (aDefaultCached.LinkedFileSystem() == aLocal);
  OSD_CachedFileSystem          aCached(aLocal);
  std::shared_ptr<std::istream> aPrevious;
  for (const std::string& anOp : fsp_split(anOps, ','))
  {
    const TCollection_AsciiString anUrl(fsp_path(anOp[1] - '0').c_str());
    const long long anOffset = anOp.size() > 3 ? std::stoll(anOp.substr(3)) : 0;
    out << ' ' << anOp << '=';
    try
    {
      if (anOp[0] == 'i')
      {
        std::shared_ptr<std::istream> aStream = aCached.OpenIStream(anUrl, std::ios_base::in, anOffset, nullptr);
        out << (aStream == aPrevious) << (aStream ? fsp_read(*aStream, 3) : "null");
        aPrevious = aStream;
      }
      else if (anOp[0] == 'b' || anOp[0] == 'w')
      {
        int64_t aSize = -7;
        std::shared_ptr<std::streambuf> aBuf = aCached.OpenStreamBuffer(
          anUrl, anOp[0] == 'b' ? std::ios_base::in : std::ios_base::out, anOffset, &aSize);
        out << aSize << (aBuf ? (anOp[0] == 'b' ? fsp_read(*aBuf, 3) : std::to_string(aBuf->sputn("Q", 1))) : "null");
      }
      else
      {
        std::shared_ptr<std::ostream> aStream = aCached.OpenOStream(anUrl, std::ios_base::app);
        if (aStream)
          aStream->write("P", 1);
        out << (aStream ? aStream->rdstate() : 9);
      }
    }
    catch (const Standard_Failure& theFailure)
    {
      fsp_raise(out, theFailure);
    }
  }
  aPrevious.reset();
  fsp_finish(out, 2);
}

static void osd_fs_default(std::ostream& out)
{
  const occ::handle<OSD_FileSystem>& aDefault = OSD_FileSystem::DefaultFileSystem();
  std::string                        aLog;
  const occ::handle<OSD_FileSystem>  aFake = new FspFake("z", "z:", true, aLog);
  const TCollection_AsciiString      anUrl("z://item");
  out << (aDefault == OSD_FileSystem::DefaultFileSystem())
      << !occ::down_cast<OSD_FileSystemSelector>(aDefault).IsNull() << aDefault->IsSupportedPath(anUrl);
  OSD_FileSystem::AddDefaultProtocol(aFake, true);
  out << aDefault->IsSupportedPath(anUrl);
  std::shared_ptr<std::istream> aStream = aDefault->OpenIStream(anUrl, std::ios_base::in);
  out << ' ' << (aStream ? fsp_read(*aStream, 4) : "null");
  OSD_FileSystem::AddDefaultProtocol(aFake, false);
  OSD_FileSystem::AddDefaultProtocol(aFake, false);
  out << aDefault->IsSupportedPath("local.bin");
  OSD_FileSystem::RemoveDefaultProtocol(aFake);
  OSD_FileSystem::RemoveDefaultProtocol(aFake);
  out << aDefault->IsSupportedPath(anUrl) << ' ' << aLog;
}

#include "tkernel_osd_file_system_open_file_probe.cxx"

static bool osd_file_system_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_fs_local_buffer")
    osd_fs_local_buffer(in, out);
  else if (op == "osd_fs_istream")
    osd_fs_istream(in, out);
  else if (op == "osd_fs_ostream")
    osd_fs_ostream(in, out);
  else if (op == "osd_fs_supported")
    osd_fs_supported(in, out);
  else if (op == "osd_fs_selector")
    osd_fs_selector(in, out);
  else if (op == "osd_fs_cached")
    osd_fs_cached(in, out);
  else if (op == "osd_fs_default")
    osd_fs_default(out);
  else if (op == "osd_fs_fopen")
    osd_fs_fopen(in, out);
  else if (op == "osd_fs_fd")
    osd_fs_fd(in, out);
  else if (op == "osd_fs_ctime")
    osd_fs_ctime(in, out);
  else if (op == "osd_fs_filebuf")
    osd_fs_filebuf(in, out);
  else if (op == "osd_fs_fstream")
    osd_fs_fstream(in, out);
  else
    return false;
  return true;
}
