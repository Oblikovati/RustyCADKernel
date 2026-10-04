// ReadLine's POSIX output-reference behavior when fgets reports an I/O error.
#include <OSD_File.hxx>
#include <OSD_Path.hxx>
#include <OSD_Protection.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>

#include <unistd.h>

#include <iostream>

class ExposedOSDFile : public OSD_File
{
public:
  using OSD_File::OSD_File;
  int channel() const { return myFileChannel; }
};

static void print_read_error(ExposedOSDFile& theFile,
                             const TCollection_AsciiString& theLine,
                             int theReadCount);

static int probe_success_eof(const char* thePath)
{
  ExposedOSDFile aFile{OSD_Path(thePath)};
  aFile.Open(OSD_ReadOnly, OSD_Protection());
  TCollection_AsciiString aLine("old");
  int                      aReadCount = -7;
  aFile.ReadLine(aLine, 100, aReadCount);
  std::cout << aLine.Length() << ' ' << aReadCount << '\n';
  aFile.ReadLine(aLine, 100, aReadCount);
  std::cout << aLine.Length() << ' ' << aReadCount << ' ' << aFile.IsAtEnd() << '\n';
  aFile.Close();
  return 0;
}

static int probe_read_error(const char* thePath)
{
  ExposedOSDFile aFile{OSD_Path(thePath)};
  aFile.Open(OSD_ReadOnly, OSD_Protection());
  if (::close(aFile.channel()) != 0)
  {
    return 3;
  }
  TCollection_AsciiString aLine("keep");
  int                      aReadCount = 73;
  aFile.ReadLine(aLine, 100, aReadCount);
  print_read_error(aFile, aLine, aReadCount);
  return 0;
}

static void print_read_error(ExposedOSDFile& theFile,
                             const TCollection_AsciiString& theLine,
                             const int theReadCount)
{
  try
  {
    theFile.Perror();
  }
  catch (const Standard_Failure&)
  {
  }
  std::cout << theLine.ToCString() << ' ' << theReadCount << ' ' << theFile.Failed() << ' '
            << theFile.Error() << '\n';
  theFile.Reset();
  theFile.Close();
}

int main(int argc, char** argv)
{
  if (argc != 2)
  {
    return 2;
  }
  if (probe_success_eof(argv[1]) != 0)
  {
    return 4;
  }
  return probe_read_error(argv[1]);
}
