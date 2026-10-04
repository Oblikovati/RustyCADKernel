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

class RefreshSurface final : public Geom_BSplineSurface
{
public:
  using Geom_BSplineSurface::Geom_BSplineSurface;
  void RefreshU() { updateUKnots(); }
  void RefreshV() { updateVKnots(); }
};

template <size_t N>
NCollection_Array1<double> doubles(const double (&theValues)[N])
{
  NCollection_Array1<double> aResult(1, static_cast<int>(N));
  for (int anIndex = 1; anIndex <= static_cast<int>(N); ++anIndex)
  {
    aResult(anIndex) = theValues[anIndex - 1];
  }
  return aResult;
}

template <size_t N>
NCollection_Array1<int> integers(const int (&theValues)[N])
{
  NCollection_Array1<int> aResult(1, static_cast<int>(N));
  for (int anIndex = 1; anIndex <= static_cast<int>(N); ++anIndex)
  {
    aResult(anIndex) = theValues[anIndex - 1];
  }
  return aResult;
}

int json_integer(const std::string& theJson, const char* theKey)
{
  const std::string aToken = std::string("\"") + theKey + "\": ";
  const size_t aPosition = theJson.find(aToken);
  return aPosition == std::string::npos ? -1 : std::stoi(theJson.substr(aPosition + aToken.size()));
}

std::string json_of(const Geom_BSplineSurface& theSurface)
{
  std::ostringstream aStream;
  theSurface.DumpJson(aStream, -1);
  return aStream.str();
}

void emit_sequence(const char* theName, const NCollection_Array1<double>& theKnots)
{
  std::cout << theName << ' ' << theKnots.Length();
  for (int anIndex = theKnots.Lower(); anIndex <= theKnots.Upper(); ++anIndex)
  {
    std::cout << ' ' << theKnots(anIndex);
  }
  std::cout << '\n';
}

void emit_snapshot(const char* theStage, const Geom_BSplineSurface& theSurface)
{
  const std::string aJson = json_of(theSurface);
  std::cout << theStage << ' ' << static_cast<int>(theSurface.UKnotDistribution()) << ' '
            << theSurface.IsUPeriodic() << ' ' << json_integer(aJson, "USmooth") << ' '
            << static_cast<int>(theSurface.VKnotDistribution()) << ' ' << theSurface.IsVPeriodic()
            << ' ' << json_integer(aJson, "VSmooth") << '\n';
  emit_sequence((std::string(theStage) + "_u").c_str(), theSurface.UKnotSequence());
  emit_sequence((std::string(theStage) + "_v").c_str(), theSurface.VKnotSequence());
  std::cout << theStage << "_state " << json_integer(aJson, "MaxDerivInvOk") << ' '
            << theSurface.HasEvalRepresentation() << '\n';
}

RefreshSurface make_surface()
{
  NCollection_Array2<gp_Pnt> aPoles(1, 6, 1, 5);
  for (int aRow = 1; aRow <= 6; ++aRow)
  {
    for (int aColumn = 1; aColumn <= 5; ++aColumn)
    {
      aPoles(aRow, aColumn) = gp_Pnt(aRow, aColumn, aRow * aColumn * 0.1);
    }
  }
  const double aUValues[] = {0.0, 0.25, 0.75, 1.0};
  const int aUMultiplicities[] = {3, 1, 2, 3};
  const double aVValues[] = {0.0, 0.2, 0.4, 0.6, 0.8, 1.0};
  const int aVMultiplicities[] = {1, 1, 1, 1, 1, 1};
  return RefreshSurface(aPoles, doubles(aUValues), doubles(aVValues), integers(aUMultiplicities),
                        integers(aVMultiplicities), 2, 3, false, true);
}

void run_probe()
{
  RefreshSurface aSurface = make_surface();
  occ::handle<GeomEval_RepSurfaceDesc::Full> aDesc = new GeomEval_RepSurfaceDesc::Full();
  aDesc->Representation = new Geom_Plane(gp_Pln(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0))));
  aSurface.SetEvalRepresentation(aDesc);
  double aUTolerance = 0.0;
  double aVTolerance = 0.0;
  aSurface.Resolution(1.0, aUTolerance, aVTolerance);
  emit_snapshot("before", aSurface);
  aSurface.RefreshU();
  emit_snapshot("after_u", aSurface);
  aSurface.Resolution(1.0, aUTolerance, aVTolerance);
  aSurface.RefreshV();
  emit_snapshot("after_v", aSurface);
}

int main()
{
  std::cout << std::setprecision(17);
  run_probe();
}
