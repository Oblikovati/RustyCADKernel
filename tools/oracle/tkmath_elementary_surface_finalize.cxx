#include <Convert_ElementarySurfaceToBSplineSurface.hxx>
#include <Standard_Failure.hxx>
#include <Standard_OutOfMemory.hxx>
#include <Standard_RangeError.hxx>
#include <iostream>

class SurfaceStorageProbe : public Convert_ElementarySurfaceToBSplineSurface
{
public:
  SurfaceStorageProbe() : Convert_ElementarySurfaceToBSplineSurface(4, 3, 3, 2, 2, 1) {}
  SurfaceStorageProbe(int uPoles, int vPoles, int uKnots, int vKnots, int uDegree, int vDegree)
      : Convert_ElementarySurfaceToBSplineSurface(uPoles, vPoles, uKnots, vKnots, uDegree, vDegree) {}

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
  void RebaseArrays()
  {
    myPoles.UpdateLowerRow(3);
    myPoles.UpdateLowerCol(3);
    myWeights.UpdateLowerRow(3);
    myWeights.UpdateLowerCol(3);
    myUKnots.UpdateLowerBound(3);
    myVKnots.UpdateLowerBound(3);
    myUMults.UpdateLowerBound(3);
    myVMults.UpdateLowerBound(3);
  }
  const NCollection_Array2<gp_Pnt>& TestPoles() const { return myPoles; }
  const NCollection_Array2<double>& TestWeights() const { return myWeights; }
  const NCollection_Array1<double>& TestUKnots() const { return myUKnots; }
  void SetShortCounts() { SetActualCounts(1, 1, 1, 1); }
  void ShrinkPoles() { myPoles.ResizeWithTrim(1, 1, 1, 1, true); }
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

void ProbeZeroCapacity()
{
  try
  {
    SurfaceStorageProbe anEmpty(0, 0, 0, 0, 0, 0);
    std::cout << "Zero " << anEmpty.TestPoles().NbRows() << ' '
              << anEmpty.TestUKnots().Length() << '\n';
  }
  catch (const Standard_RangeError& anError)
  {
    std::cout << "ZeroRangeError " << anError.what() << '\n';
  }
  catch (const Standard_OutOfMemory& anError)
  {
    std::cout << "ZeroOutOfMemory " << anError.what() << '\n';
  }
}

void ProbeNegativeCapacity()
{
  try
  {
    SurfaceStorageProbe aNegative(-1, 1, 1, 1, 1, 1);
    std::cout << "Negative " << aNegative.TestPoles().NbRows() << '\n';
  }
  catch (const Standard_RangeError& anError)
  {
    std::cout << "NegativeRangeError " << anError.what() << '\n';
  }
  catch (const Standard_OutOfMemory& anError)
  {
    std::cout << "NegativeOutOfMemory " << anError.what() << '\n';
  }
}

void ProbeSuccessfulTrim()
{
  SurfaceStorageProbe aSurface;
  aSurface.FillArrays();
  aSurface.SetActualCounts(2, 2, 2, 1);
  aSurface.Finish();
  PrintState(aSurface);
  aSurface.Finish();
  PrintState(aSurface);
}

void ProbeFinalizeFailure()
{
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

void ProbeRebasedPole()
{
  SurfaceStorageProbe aRebased;
  aRebased.FillArrays();
  aRebased.SetActualCounts(2, 2, 2, 1);
  aRebased.RebaseArrays();
  try
  {
    (void)aRebased.Pole(1, 1);
  }
  catch (const Standard_OutOfRange& anError)
  {
    std::cout << "Pole " << anError.what() << '\n';
  }
}

void ProbeRebasedKnot()
{
  SurfaceStorageProbe aRebased;
  aRebased.SetActualCounts(2, 2, 2, 1);
  aRebased.RebaseArrays();
  try
  {
    (void)aRebased.UKnot(1);
  }
  catch (const Standard_OutOfRange& anError)
  {
    std::cout << "UKnot " << anError.what() << '\n';
  }
}

void ProbeActiveCount()
{
  SurfaceStorageProbe aRebased;
  aRebased.SetActualCounts(2, 2, 2, 1);
  aRebased.RebaseArrays();
  try
  {
    (void)aRebased.Pole(3, 1);
  }
  catch (const Standard_OutOfRange& anError)
  {
    std::cout << "Active " << anError.what() << '\n';
  }
}

void ProbeRebasedFinalize()
{
  SurfaceStorageProbe aRebased;
  aRebased.FillArrays();
  aRebased.RebaseArrays();
  aRebased.SetShortCounts();
  aRebased.Finish();
  std::cout << "RebasedFinalize " << aRebased.TestPoles().LowerRow() << ' '
            << aRebased.TestPoles().LowerCol() << ' '
            << aRebased.TestUKnots().Lower() << '\n';
}

void ProbeShortExtent()
{
  SurfaceStorageProbe aShortStorage;
  aShortStorage.SetActualCounts(2, 2, 2, 1);
  aShortStorage.ShrinkPoles();
  try
  {
    (void)aShortStorage.Pole(2, 2);
  }
  catch (const Standard_OutOfRange& anError)
  {
    std::cout << "Extent " << anError.what() << '\n';
  }
}

int main()
{
  ProbeZeroCapacity();
  ProbeNegativeCapacity();
  ProbeSuccessfulTrim();
  ProbeFinalizeFailure();
  ProbeRebasedPole();
  ProbeRebasedKnot();
  ProbeActiveCount();
  ProbeRebasedFinalize();
  ProbeShortExtent();
}
