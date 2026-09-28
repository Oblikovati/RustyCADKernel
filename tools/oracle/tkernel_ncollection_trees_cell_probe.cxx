// NCollection trees probe, part two: CellFilter scripts and KDTree Array1 builds/moves
// (twin of oracle_probe/ncollection/trees_cell.rs); included by the trees family file.
#include <NCollection_CellFilter.hxx>
#include <NCollection_KDTree.hxx>

namespace trees_probe
{
struct CellPoint
{
  double Xyz[3] = {0.0, 0.0, 0.0};
};

// Records every visited target and purges the one equal to `Purge`.
struct Inspector
{
  typedef int       Target;
  typedef CellPoint Point;

  static constexpr int Dimension = 2;

  static double Coord(int i, const Point& thePnt) { return thePnt.Xyz[i]; }

  static bool IsEqual(const Target& theT1, const Target& theT2) { return theT1 == theT2; }

  NCollection_CellFilter_Action Inspect(const Target& theObject)
  {
    Visited.push_back(theObject);
    return theObject == Purge ? CellFilter_Purge : CellFilter_Keep;
  }

  std::vector<int> Visited;
  int              Purge = -1;
};

static CellPoint readPoint(std::istream& theIn)
{
  CellPoint aPoint;
  theIn >> aPoint.Xyz[0] >> aPoint.Xyz[1] >> aPoint.Xyz[2];
  return aPoint;
}

static void cellInspect(std::istream& theIn, std::ostream& theOut, NCollection_CellFilter<Inspector>& theFilter, bool theRange)
{
  Inspector anInspector;
  theIn >> anInspector.Purge;
  const CellPoint aMin = readPoint(theIn);
  if (theRange)
    theFilter.Inspect(aMin, readPoint(theIn), anInspector);
  else
    theFilter.Inspect(aMin, anInspector);
  theOut << " [";
  for (size_t i = 0; i < anInspector.Visited.size(); ++i)
    theOut << (i ? "," : "") << anInspector.Visited[i];
  theOut << ']';
}

static void cellResetArray(std::istream& theIn, std::ostream& theOut, NCollection_CellFilter<Inspector>& theFilter)
{
  int aLower = 0, aCount = 0;
  theIn >> aLower >> aCount;
  NCollection_Array1<double> aSizes(aLower, aLower + aCount - 1);
  for (int i = 0; i < aCount; ++i)
    theIn >> aSizes(aLower + i);
  theFilter.Reset(aSizes);
  theOut << " S";
}

static void cellReset(std::istream& theIn, std::ostream& theOut, NCollection_CellFilter<Inspector>& theFilter)
{
  double aSize = 0.0;
  theIn >> aSize;
  try
  {
    theFilter.Reset(aSize);
    theOut << " s";
  }
  catch (const Standard_Failure& theError)
  {
    theOut << " throw " << theError.ExceptionType();
  }
}

static void cellStep(const std::string& theStep, std::istream& theIn, std::ostream& theOut, NCollection_CellFilter<Inspector>& theFilter)
{
  int aTarget = 0;
  if (theStep == "i" || theStep == "I")
    return cellInspect(theIn, theOut, theFilter, theStep == "I");
  if (theStep == "s")
    return cellReset(theIn, theOut, theFilter);
  if (theStep == "S")
    return cellResetArray(theIn, theOut, theFilter);
  theIn >> aTarget;
  const CellPoint aMin = readPoint(theIn);
  if (theStep == "a")
    theFilter.Add(aTarget, aMin);
  else if (theStep == "A")
    theFilter.Add(aTarget, aMin, readPoint(theIn));
  else if (theStep == "r")
    theFilter.Remove(aTarget, aMin);
  else
    theFilter.Remove(aTarget, aMin, readPoint(theIn));
}

// ncoll_trees_cell <dim|-1> <size> <steps> {step}
static void cell(std::istream& theIn, std::ostream& theOut)
{
  int    aDim = -1, aSteps = 0;
  double aSize = 0.0;
  theIn >> aDim >> aSize >> aSteps;
  NCollection_CellFilter<Inspector> aFilter = aDim < 0 ? NCollection_CellFilter<Inspector>(aSize)
                                                       : NCollection_CellFilter<Inspector>(aDim, aSize);
  for (int i = 0; i < aSteps; ++i)
  {
    std::string aStep;
    theIn >> aStep;
    cellStep(aStep, theIn, theOut, aFilter);
  }
}

struct KdPoint
{
  double Xy[2] = {0.0, 0.0};

  double Coord(int i) const { return Xy[i - 1]; }
};

typedef NCollection_KDTree<KdPoint, 2, true> RadiusTree;

static void printCloud(std::ostream& theOut, const RadiusTree& theTree)
{
  double aGap = 0.0;
  const KdPoint aQuery;
  theOut << ' ' << theTree.Size() << ' ' << theTree.IsEmpty();
  if (theTree.IsEmpty())
    return;
  const size_t aBest = theTree.NearestWeighted(aQuery, aGap);
  theOut << ' ' << aBest << ' ' << bits(aGap) << ' ' << bits(theTree.Radius(aBest)) << ' '
         << bits(theTree.Point(1).Xy[0]);
}

// ncoll_trees_kd_array <lower> <n> {x y} <radiiLower> <m> {r}
static void kdArray(std::istream& theIn, std::ostream& theOut)
{
  int aLower = 1, aCount = 0;
  theIn >> aLower >> aCount;
  NCollection_Array1<KdPoint> aPoints(aLower, aLower + aCount - 1);
  for (int i = 0; i < aCount; ++i)
    theIn >> aPoints(aLower + i).Xy[0] >> aPoints(aLower + i).Xy[1];
  int aRadiiLower = 1, aRadiiCount = 0;
  theIn >> aRadiiLower >> aRadiiCount;
  NCollection_Array1<double> aRadii(aRadiiLower, aRadiiLower + aRadiiCount - 1);
  for (int i = 0; i < aRadiiCount; ++i)
    theIn >> aRadii(aRadiiLower + i);
  NCollection_KDTree<KdPoint, 2> aPlain;
  aPlain.Build(aPoints);
  theOut << aPlain.Size() << ' ' << aPlain.NearestPoint(KdPoint());
  RadiusTree aTree;
  try
  {
    aTree.Build(aPoints, aRadii);
    printCloud(theOut, aTree);
  }
  catch (const Standard_Failure& theError)
  {
    theOut << " throw " << theError.ExceptionType();
    return;
  }
  RadiusTree aMoved(std::move(aTree));
  printCloud(theOut, aTree);
  printCloud(theOut, aMoved);
  aTree = std::move(aMoved);
  printCloud(theOut, aMoved);
  printCloud(theOut, aTree);
}
} // namespace trees_probe

static bool ncollection_trees_probe_rest(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_trees_cell")
    trees_probe::cell(in, out);
  else if (op == "ncoll_trees_kd_array")
    trees_probe::kdArray(in, out);
  else
    return false;
  return true;
}
