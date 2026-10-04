#include <FSD_CmpFile.hxx>
#include <FSD_File.hxx>
#include <Storage_BaseDriver.hxx>
#include <Storage_OpenMode.hxx>

#include <fstream>
#include <iostream>

class DerivedFsd final : public FSD_File
{
public:
  int wh = 0, bwp = 0, bwo = 0, ewo = 0, ewp = 0;
  int rh = 0, brp = 0, bro = 0, ero = 0, erp = 0;
  void WritePersistentObjectHeader(const int r, const int t) override { ++wh; FSD_File::WritePersistentObjectHeader(r, t); }
  void BeginWritePersistentObjectData() override { ++bwp; FSD_File::BeginWritePersistentObjectData(); }
  void BeginWriteObjectData() override { ++bwo; FSD_File::BeginWriteObjectData(); }
  void EndWriteObjectData() override { ++ewo; FSD_File::EndWriteObjectData(); }
  void EndWritePersistentObjectData() override { ++ewp; FSD_File::EndWritePersistentObjectData(); }
  void ReadPersistentObjectHeader(int& r, int& t) override { ++rh; FSD_File::ReadPersistentObjectHeader(r, t); }
  void BeginReadPersistentObjectData() override { ++brp; FSD_File::BeginReadPersistentObjectData(); }
  void BeginReadObjectData() override { ++bro; FSD_File::BeginReadObjectData(); }
  void EndReadObjectData() override { ++ero; FSD_File::EndReadObjectData(); }
  void EndReadPersistentObjectData() override { ++erp; FSD_File::EndReadPersistentObjectData(); }
};

class DerivedCmp final : public FSD_CmpFile
{
public:
  int wh = 0, bwp = 0, bwo = 0, ewo = 0, ewp = 0;
  int rh = 0, brp = 0, bro = 0, ero = 0, erp = 0;
  void WritePersistentObjectHeader(const int r, const int t) override { ++wh; FSD_CmpFile::WritePersistentObjectHeader(r, t); }
  void BeginWritePersistentObjectData() override { ++bwp; FSD_CmpFile::BeginWritePersistentObjectData(); }
  void BeginWriteObjectData() override { ++bwo; FSD_CmpFile::BeginWriteObjectData(); }
  void EndWriteObjectData() override { ++ewo; FSD_CmpFile::EndWriteObjectData(); }
  void EndWritePersistentObjectData() override { ++ewp; FSD_CmpFile::EndWritePersistentObjectData(); }
  void ReadPersistentObjectHeader(int& r, int& t) override { ++rh; FSD_CmpFile::ReadPersistentObjectHeader(r, t); }
  void BeginReadPersistentObjectData() override { ++brp; FSD_CmpFile::BeginReadPersistentObjectData(); }
  void BeginReadObjectData() override { ++bro; FSD_CmpFile::BeginReadObjectData(); }
  void EndReadObjectData() override { ++ero; FSD_CmpFile::EndReadObjectData(); }
  void EndReadPersistentObjectData() override { ++erp; FSD_CmpFile::EndReadPersistentObjectData(); }
};

template <class File>
void writeRecord(File& f, const char* path)
{
  f.Open(path, Storage_VSWrite);
  Storage_BaseDriver& d = f;
  d.BeginWriteDataSection(); d.WritePersistentObjectHeader(42, 3);
  d.BeginWritePersistentObjectData(); d.BeginWriteObjectData();
  d.EndWriteObjectData(); d.EndWritePersistentObjectData(); d.EndWriteDataSection();
  f.Close();
}

template <class File>
void readRecord(File& f, const char* path)
{
  f.Open(path, Storage_VSRead);
  Storage_BaseDriver& d = f;
  d.BeginReadDataSection(); int r = 0, t = 0; d.ReadPersistentObjectHeader(r, t);
  d.BeginReadPersistentObjectData(); d.BeginReadObjectData(); d.EndReadObjectData();
  d.EndReadPersistentObjectData(); d.EndReadDataSection();
  std::cout << r << ' ' << t << ' ' << f.rh << ' ' << f.brp << ' ' << f.bro << ' '
            << f.ero << ' ' << f.erp << '\n';
  f.Close();
}

int main()
{
  const char* fp = "/tmp/fsd-object-hooks.txt";
  const char* cp = "/tmp/cmp-object-hooks.txt";
  DerivedFsd fw; writeRecord(fw, fp);
  std::ifstream fi(fp, std::ios::binary); std::cout << fi.rdbuf(); fi.close();
  std::cout << fw.wh << ' ' << fw.bwp << ' ' << fw.bwo << ' ' << fw.ewo << ' ' << fw.ewp << '\n';
  DerivedFsd fr; readRecord(fr, fp);
  DerivedCmp cw; writeRecord(cw, cp);
  std::ifstream ci(cp, std::ios::binary); std::cout << ci.rdbuf(); ci.close();
  std::cout << cw.wh << ' ' << cw.bwp << ' ' << cw.bwo << ' ' << cw.ewo << ' ' << cw.ewp << '\n';
  DerivedCmp cr; readRecord(cr, cp);
  std::remove(fp); std::remove(cp);
}
