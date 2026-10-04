#include <Convert_ElementarySurfaceToBSplineSurface.hxx>
#include <Standard_Failure.hxx>
#include <Standard_OutOfMemory.hxx>
#include <iostream>

class SurfaceStorageProbe : public Convert_ElementarySurfaceToBSplineSurface
{
public:
  SurfaceStorageProbe() : Convert_ElementarySurfaceToBSplineSurface(4, 3, 3, 2, 2, 1) {}

  void SetActualCounts(int uPoles, int vPoles, int uKnots, int vKnots)
  {
    myNbUPoles = uPoles;
    myNbVPoles = vPoles;
    myNbUKnots = uKnots;
    myNbVKnots = vKnots;
  }

  void FillArrays()
  {
    for (int u = 1; u <= myPoles.NbRows(); ++u)
      for (int v = 1; v <= myPoles.NbColumns(); ++v)
      {
        myPoles(u, v) = gp_Pnt(10. * u + v, u, v);
        myWeights(u, v) = 100. * u + v;
      }
    for (int u = 1; u <= myUKnots.Length(); ++u)
    {
      myUKnots(u) = 10. * u;
      myUMults(u) = 100 + u;
    }
    for (int v = 1; v <= myVKnots.Length(); ++v)
    {
      myVKnots(v) = 20. * v;
      myVMults(v) = 200 + v;
    }
  }

  void Finish() { Finalize(); }
  void SetBadUKnotCount() { myNbUKnots = -1; }
  const NCollection_Array2<gp_Pnt>& TestPoles() const { return myPoles; }
  const NCollection_Array2<double>& TestWeights() const { return myWeights; }
  const NCollection_Array1<double>& TestUKnots() const { return myUKnots; }
};

void PrintState(const SurfaceStorageProbe& probe)
{
  const auto& poles = probe.TestPoles();
  const auto& weights = probe.TestWeights();
  const auto& knots = probe.TestUKnots();
  std::cout << poles.NbRows() << ' ' << poles.NbColumns() << ' '
            << weights.NbRows() << ' ' << weights.NbColumns() << ' '
            << knots.Length() << ' ' << poles(2, 2).X() << ' '
            << weights(2, 2) << ' ' << knots(2) << '\n';
}

int main()
{
  SurfaceStorageProbe aSurface;
  aSurface.FillArrays();
  aSurface.SetActualCounts(2, 2, 2, 1);
  aSurface.Finish();
  PrintState(aSurface);
  aSurface.Finish();
  PrintState(aSurface);

  SurfaceStorageProbe aFailure;
  aFailure.FillArrays();
  aFailure.SetActualCounts(2, 2, -1, 1);
  try
  {
    aFailure.Finish();
  }
  catch (const Standard_OutOfMemory& anError)
  {
    std::cout << "Standard_OutOfMemory " << anError.what() << '\n';
  }
  PrintState(aFailure);
}
