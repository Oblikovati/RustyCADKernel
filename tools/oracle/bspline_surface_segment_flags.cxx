#include <GeomEval_RepSurfaceDesc.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>

class SegmentProbe final : public Geom_BSplineSurface
{
public:
  using Geom_BSplineSurface::Geom_BSplineSurface;
  void Run(const bool theU, const bool theV)
  {
    segment(0.25, 0.75, 0.25, 0.75, 1.0e-12, 1.0e-12, theU, theV);
  }
};

NCollection_Array1<double> doubles()
{
  NCollection_Array1<double> aResult(1, 5);
  const double aValues[] = {0.0, 0.25, 0.5, 0.75, 1.0};
  for (int i = 1; i <= 5; ++i) aResult(i) = aValues[i - 1];
  return aResult;
}

NCollection_Array1<int> multiplicities()
{
  NCollection_Array1<int> aResult(1, 5);
  const int aValues[] = {3, 1, 1, 1, 3};
  for (int i = 1; i <= 5; ++i) aResult(i) = aValues[i - 1];
  return aResult;
}

SegmentProbe make_surface()
{
  NCollection_Array2<gp_Pnt> aPoles(1, 6, 1, 6);
  for (int i = 1; i <= 6; ++i)
    for (int j = 1; j <= 6; ++j) aPoles(i, j) = gp_Pnt(i, j, i * j * 0.1);
  return SegmentProbe(aPoles, doubles(), doubles(), multiplicities(), multiplicities(), 2, 2, false, false);
}

int cache_valid(const Geom_BSplineSurface& theSurface)
{
  std::ostringstream aStream;
  theSurface.DumpJson(aStream, -1);
  return aStream.str().find("\"MaxDerivInvOk\": 1") != std::string::npos;
}

void emit(const char* theName, const NCollection_Array1<double>& theArray)
{
  std::cout << theName << ' ' << theArray.Length();
  for (int i = theArray.Lower(); i <= theArray.Upper(); ++i) std::cout << ' ' << theArray(i);
  std::cout << '\n';
}

void emit(const char* theName, const NCollection_Array1<int>& theArray)
{
  std::cout << theName << ' ' << theArray.Length();
  for (int i = theArray.Lower(); i <= theArray.Upper(); ++i) std::cout << ' ' << theArray(i);
  std::cout << '\n';
}

void run(const bool theU, const bool theV)
{
  SegmentProbe aSurface = make_surface();
  occ::handle<GeomEval_RepSurfaceDesc::Full> aDesc = new GeomEval_RepSurfaceDesc::Full();
  aDesc->Representation = new Geom_Plane(gp_Pln(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0))));
  aSurface.SetEvalRepresentation(aDesc);
  double aUTolerance = 0.0, aVTolerance = 0.0;
  aSurface.Resolution(1.0, aUTolerance, aVTolerance);
  aSurface.Run(theU, theV);
  double u1, u2, v1, v2;
  aSurface.Bounds(u1, u2, v1, v2);
  std::cout << "flags " << theU << ' ' << theV << " bounds " << u1 << ' ' << u2 << ' ' << v1 << ' ' << v2
            << " poles " << aSurface.NbUPoles() << ' ' << aSurface.NbVPoles() << " rep "
            << aSurface.HasEvalRepresentation() << " cache " << cache_valid(aSurface) << '\n';
  emit("uknots", aSurface.UKnots());
  emit("umults", aSurface.UMultiplicities());
  emit("vknots", aSurface.VKnots());
  emit("vmults", aSurface.VMultiplicities());
}

int main()
{
  std::cout << std::setprecision(17);
  run(false, false);
  run(false, true);
  run(true, false);
  run(true, true);
}
