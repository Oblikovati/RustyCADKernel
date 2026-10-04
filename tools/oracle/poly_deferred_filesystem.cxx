#include <Poly_Triangulation.hxx>
#include <OSD_LocalFileSystem.hxx>
#include <iostream>

class FakeFileSystem : public OSD_LocalFileSystem {};
class ProbeStorage : public Poly_Triangulation
{
public:
  bool Available = true;
  bool Success = true;
  mutable int Calls = 0;
  mutable occ::handle<OSD_FileSystem> Seen;
  mutable const Poly_Triangulation* Destination = nullptr;
  int NbDeferredTriangles() const override { return Available ? 1 : 0; }
protected:
  occ::handle<Poly_Triangulation> createNewEntity() const override
  {
    occ::handle<Poly_Triangulation> result = new Poly_Triangulation();
    result->SetMeshPurpose(Poly_MeshPurpose_USER);
    return result;
  }
  bool loadDeferredData(const occ::handle<OSD_FileSystem>& fs,
                        const occ::handle<Poly_Triangulation>& destination) const override
  {
    ++Calls;
    Seen = fs;
    Destination = destination.get();
    return Success;
  }
};

int main()
{
  occ::handle<OSD_FileSystem> fs = new FakeFileSystem();
  occ::handle<ProbeStorage> storage = new ProbeStorage();
  storage->SetMeshPurpose(Poly_MeshPurpose_Calculation);
  bool loaded = storage->LoadDeferredData(fs);
  std::cout << "supplied " << loaded << ' ' << storage->Calls << ' '
            << (storage->Seen == fs) << ' ' << (storage->Destination == storage.get())
            << ' ' << storage->MeshPurpose() << '\n';
  auto detached = storage->DetachedLoadDeferredData();
  std::cout << "detached " << !detached.IsNull() << ' ' << storage->Calls << ' '
            << storage->Seen.IsNull() << ' ' << (storage->Destination == detached.get())
            << ' ' << detached->MeshPurpose() << ' ' << storage->MeshPurpose() << '\n';
  storage->Success = false;
  storage->SetMeshPurpose(Poly_MeshPurpose_Calculation);
  loaded = storage->LoadDeferredData();
  detached = storage->DetachedLoadDeferredData(fs);
  std::cout << "failed " << loaded << ' ' << detached.IsNull() << ' ' << storage->Calls
            << ' ' << (storage->Seen == fs) << ' ' << storage->MeshPurpose() << '\n';
  storage->Available = false;
  loaded = storage->LoadDeferredData(fs);
  detached = storage->DetachedLoadDeferredData(fs);
  std::cout << "empty " << loaded << ' ' << detached.IsNull() << ' ' << storage->Calls << '\n';
}
