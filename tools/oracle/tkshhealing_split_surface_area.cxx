// Repository-owned fixture for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <ShapeUpgrade_SplitSurfaceArea.hxx>
#include <gp.hxx>
#include <iomanip>
#include <iostream>

static void printValues(const occ::handle<NCollection_HSequence<double>>& values)
{
  for (int i = 1; i <= values->Length(); ++i) std::cout << ' ' << values->Value(i);
}

static void writeResult(const char* name, int parts, bool sharedU, bool sharedV,
                        const occ::handle<NCollection_HSequence<double>>& uValues,
                        const occ::handle<NCollection_HSequence<double>>& vValues)
{
  std::cout << name << ' ' << parts << ' ' << sharedU << ' ' << sharedV;
  printValues(uValues);
  std::cout << " |";
  printValues(vValues);
  std::cout << '\n';
}

class SeededArea : public ShapeUpgrade_SplitSurfaceArea
{
public:
  void SetPhysicalSizes(double uSize, double vSize) { myUsize = uSize; myVsize = vSize; }
};

static occ::handle<ShapeUpgrade_SplitSurfaceArea> newSplitter(
  double uLast, double vLast, int parts, bool squares, int uCount,
  int vCount, double area, double cylinderRadius)
{
  occ::handle<Geom_Surface> surface = cylinderRadius == 0.
    ? occ::handle<Geom_Surface>(new Geom_Plane(gp::XOY()))
    : occ::handle<Geom_Surface>(new Geom_CylindricalSurface(gp::XOY(), cylinderRadius));
  occ::handle<ShapeUpgrade_SplitSurfaceArea> splitter = new ShapeUpgrade_SplitSurfaceArea();
  splitter->Init(surface, 0., uLast, 0., vLast, area);
  splitter->NbParts() = parts;
  splitter->SetSplittingIntoSquares(squares);
  if (uCount != -1 || vCount != -1) splitter->SetNumbersUVSplits(uCount, vCount);
  return splitter;
}

static void report(const char* name, double uLast, double vLast, int parts,
                   bool squares = false, int uCount = -1, int vCount = -1,
                   double area = 0., double cylinderRadius = 0.)
{
  auto splitter = newSplitter(uLast, vLast, parts, squares, uCount, vCount, area, cylinderRadius);
  auto uValues = splitter->USplitValues();
  auto vValues = splitter->VSplitValues();
  splitter->Compute(false);
  writeResult(name, splitter->NbParts(), uValues == splitter->USplitValues(),
              vValues == splitter->VSplitValues(), uValues, vValues);
}

static void reportNegativeSquareVMeasure()
{
  SeededArea splitter;
  splitter.Init(new Geom_Plane(gp::XOY()), 0., 10., 0., 2., 4.);
  splitter.NbParts() = 4;
  splitter.SetSplittingIntoSquares(true);
  splitter.SetPhysicalSizes(3., -2.);
  auto uValues = splitter.USplitValues();
  auto vValues = splitter.VSplitValues();
  splitter.Compute(false);
  writeResult("negative-square-v-measure", splitter.NbParts(), true, true, uValues, vValues);
}

static void reportRepeatedSquare()
{
  ShapeUpgrade_SplitSurfaceArea repeated;
  repeated.Init(new Geom_Plane(gp::XOY()), 0., 10., 0., 2., 100.);
  repeated.NbParts() = 4;
  repeated.SetSplittingIntoSquares(true);
  repeated.Compute(false);
  repeated.Compute(false);
  writeResult("auto-square-second-call", repeated.NbParts(), true, true,
              repeated.USplitValues(), repeated.VSplitValues());
}

int main()
{
  std::cout << std::setprecision(17);
  report("default-noop", 4., 2., 1);
  report("square-ratio-one", 4., 4., 2);
  report("ratio-long-u", 10., 2., 4);
  report("ratio-long-v", 2., 10., 4);
  report("ratio-between", 5., 2., 4);
  report("fixed-square-u-first", 10., 2., 6, true, 2, 3);
  report("fixed-square-v-first", 2., 10., 6, true, 2, 3);
  report("fixed-square-equal-aspect", 4., 4., 6, true, 2, 3);
  report("auto-square", 10., 2., 4, true, -1, -1, 100.);
  report("cylinder-auto-square", 2. * M_PI, 4., 4, true, -1, -1, 25., 2.);
  report("invalid-fixed-count-retains-parts", 5., 2., 4, true, 2, -1, 100.);
  reportNegativeSquareVMeasure();
  reportRepeatedSquare();
}
