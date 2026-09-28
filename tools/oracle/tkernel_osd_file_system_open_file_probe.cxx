// OSD_OpenFile.hxx probe operations of the file_system family (osd_fs_fopen, osd_fs_fd,
// osd_fs_ctime, osd_fs_filebuf, osd_fs_fstream); included by
// tkernel_osd_file_system_probe.cxx after its helpers. Twin of
// oracle_probe/osd/file_system/open_file_ops.rs.

static void osd_fs_fopen(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::string aMode = osd_token(in);
  int               anUtf8 = 0;
  in >> anUtf8;
  FILE* aFile = anUtf8 ? OSD_OpenFile(fsp_path(1).c_str(), aMode.c_str())
                       : OSD_OpenFile(TCollection_ExtendedString(fsp_path(1).c_str()), aMode.c_str());
  out << (aFile ? "file" : "null");
  if (aFile)
  {
    char   aBytes[16];
    size_t aCount = fread(aBytes, 1, sizeof(aBytes), aFile);
    out << ' ' << osd_escape(std::string(aBytes, aCount));
    fseek(aFile, 0, SEEK_CUR);
    fwrite("XY", 1, 2, aFile);
    fclose(aFile);
  }
  fsp_finish(out, 1);
}

static void osd_fs_fd(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::ios_base::openmode aMode = fsp_mode(in);
  const int aFd = OSD_OpenFileDescriptor(TCollection_ExtendedString(fsp_path(1).c_str()), aMode);
  out << (aFd >= 0 ? "fd" : "-1");
  if (aFd >= 0)
  {
    char          aBytes[16];
    const ssize_t aCount = ::read(aFd, aBytes, sizeof(aBytes));
    out << ' ' << (aCount < 0 ? std::string("-1") : osd_escape(std::string(aBytes, (size_t)aCount)))
        << ' ' << (long long)::write(aFd, "XY", 2);
    ::close(aFd);
  }
  struct stat aStat;
  out << ' ' << (stat(fsp_path(1).c_str(), &aStat) == 0 ? (int)(aStat.st_mode & 0777) : -1);
  fsp_finish(out, 1);
}

static void osd_fs_ctime(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  out << (OSD_FileStatCTime(fsp_path(1).c_str()) > 0) << ' '
      << (long long)OSD_FileStatCTime("osdfs_tmp/missing.bin");
  fsp_finish(out, 1);
}

static void osd_fs_filebuf(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::ios_base::openmode aMode = fsp_mode(in);
  {
    std::filebuf aBuf;
    const bool   isOpened = OSD_OpenStream(aBuf, TCollection_ExtendedString(fsp_path(1).c_str()), aMode);
    out << isOpened << aBuf.is_open();
    if (isOpened)
    {
      out << ' ' << (long long)aBuf.pubseekoff(0, std::ios_base::cur, std::ios_base::in) << ' '
          << fsp_read(aBuf, 16) << ' ' << aBuf.sputn("XY", 2) << ' '
          << (long long)aBuf.pubseekoff(0, std::ios_base::end, std::ios_base::in) << ' '
          << (long long)aBuf.pubseekoff(-1, std::ios_base::beg, std::ios_base::in) << ' '
          << OSD_OpenStream(aBuf, TCollection_ExtendedString(fsp_path(1).c_str()), aMode);
    }
  }
  fsp_finish(out, 1);
}

template <typename T>
static void fsp_fstream(T& theStream, bool theUtf8, std::ios_base::openmode theMode, std::ostream& out)
{
  if (theUtf8)
    OSD_OpenStream(theStream, fsp_path(1).c_str(), theMode);
  else
    OSD_OpenStream(theStream, TCollection_ExtendedString(fsp_path(1).c_str()), theMode);
  out << theStream.is_open() << ' ' << theStream.rdstate();
}

static void osd_fs_fstream(std::istream& in, std::ostream& out)
{
  fsp_setup(in, 1);
  const std::string             aKind = osd_token(in);
  const std::ios_base::openmode aMode = fsp_mode(in);
  int                           anUtf8 = 0;
  in >> anUtf8;
  if (aKind == "i")
  {
    std::ifstream aStream;
    fsp_fstream(aStream, anUtf8 != 0, aMode, out);
    out << ' ' << fsp_read(aStream, 16);
    aStream.close();
    out << ' ' << aStream.rdstate();
    aStream.close();
    out << ' ' << aStream.rdstate();
  }
  else if (aKind == "o")
  {
    std::ofstream aStream;
    fsp_fstream(aStream, anUtf8 != 0, aMode, out);
    aStream.write("XY", 2);
    out << ' ' << aStream.rdstate() << ' ' << (long long)aStream.tellp();
  }
  else
  {
    std::fstream aStream;
    fsp_fstream(aStream, anUtf8 != 0, aMode, out);
    out << ' ' << fsp_read(aStream, 16);
    aStream.clear();
    aStream.seekp(0, std::ios_base::end);
    aStream.write("XY", 2);
    out << ' ' << aStream.rdstate() << ' ' << (long long)aStream.tellg();
  }
  fsp_finish(out, 1);
}
