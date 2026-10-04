#include <Poly_Triangulation.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <iomanip>
#include <iostream>

class ExpandedTriangulation : public Poly_Triangulation
{
public:
  explicit ExpandedTriangulation(const gp_Pnt& thePoint)
      : Poly_Triangulation(3, 1, false, false)
  {
    SetNode(1, thePoint);
    SetNode(2, thePoint);
    SetNode(3, thePoint);
    SetTriangle(1, Poly_Triangle(1, 2, 3));
  }
  mutable int Calls = 0;
  Bnd_Box ExposedBoundingBox(const gp_Trsf& theTrsf) const { return computeBoundingBox(theTrsf); }
  Bnd_Box ExposedBaseBoundingBox(const gp_Trsf& theTrsf) const
  {
    return Poly_Triangulation::computeBoundingBox(theTrsf);
  }

protected:
  Bnd_Box computeBoundingBox(const gp_Trsf& theTrsf) const override
  {
    ++Calls;
    Bnd_Box aBox = Poly_Triangulation::computeBoundingBox(theTrsf);
    aBox.Enlarge(2.0);
    return aBox;
  }
};

class DeferredSource : public Poly_Triangulation
{
public:
  int NbDeferredTriangles() const override { return 1; }

protected:
  occ::handle<Poly_Triangulation> createNewEntity() const override
  {
    return new ExpandedTriangulation(gp_Pnt(4.0, 5.0, 6.0));
  }
  bool loadDeferredData(const occ::handle<OSD_FileSystem>&,
                        const occ::handle<Poly_Triangulation>&) const override
  {
    return true;
  }
};

void print_box(const char* theName, const Bnd_Box& theBox)
{
  double aXMin, aYMin, aZMin, aXMax, aYMax, aZMax;
  theBox.Get(aXMin, aYMin, aZMin, aXMax, aYMax, aZMax);
  std::cout << theName << ' ' << aXMin << ' ' << aYMin << ' ' << aZMin << ' ' << aXMax
            << ' ' << aYMax << ' ' << aZMax << '\n';
}

gp_Trsf probe_rotation()
{
  gp_Trsf aRotation;
  aRotation.SetRotation(gp_Ax1(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 0.5);
  return aRotation;
}

void probe_base_and_virtual_bounds(const occ::handle<ExpandedTriangulation>& theMesh)
{
  print_box("base", theMesh->ExposedBaseBoundingBox(gp_Trsf()));
  std::cout << "base_calls " << theMesh->Calls << '\n';
  print_box("virtual", theMesh->ExposedBoundingBox(gp_Trsf()));
  std::cout << "virtual_calls " << theMesh->Calls << '\n';
  Bnd_Box aResult;
  theMesh->MinMax(aResult, gp_Trsf(), true);
  std::cout << "accurate " << theMesh->Calls << '\n';
}

void probe_cache_selection(const occ::handle<ExpandedTriangulation>& theMesh,
                           const gp_Trsf& theRotation)
{
  Bnd_Box aResult;
  Bnd_Box aCached;
  aCached.Add(gp_Pnt(-5.0, -5.0, -5.0));
  aCached.Add(gp_Pnt(5.0, 5.0, 5.0));
  theMesh->SetCachedMinMax(aCached);
  aResult.SetVoid();
  theMesh->MinMax(aResult, gp_Trsf(), false);
  std::cout << "cached " << theMesh->Calls << '\n';
  aResult.SetVoid();
  theMesh->MinMax(aResult, theRotation, true);
  std::cout << "rotated " << theMesh->Calls << '\n';
}

void probe_copy_slices_override(const occ::handle<ExpandedTriangulation>& theMesh,
                                const gp_Trsf& theRotation)
{
  Bnd_Box aResult;
  occ::handle<Poly_Triangulation> aCopy = theMesh->Copy();
  aCopy->MinMax(aResult, theRotation, true);
  print_box("copy", aResult);
  std::cout << "copy_calls " << theMesh->Calls << '\n';
}

void probe_detached_factory()
{
  occ::handle<DeferredSource> aSource = new DeferredSource();
  occ::handle<Poly_Triangulation> aDetached = aSource->DetachedLoadDeferredData();
  auto* aCustom = dynamic_cast<ExpandedTriangulation*>(aDetached.get());
  print_box("detached", aCustom->ExposedBoundingBox(gp_Trsf()));
  std::cout << "factory " << (aCustom != nullptr) << ' ' << aCustom->Calls << ' '
            << aDetached->MeshPurpose() << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  occ::handle<ExpandedTriangulation> aMesh = new ExpandedTriangulation(gp_Pnt(1.0, 2.0, 3.0));
  gp_Trsf aRotation = probe_rotation();
  probe_base_and_virtual_bounds(aMesh);
  probe_cache_selection(aMesh, aRotation);
  probe_copy_slices_override(aMesh, aRotation);
  probe_detached_factory();
}
