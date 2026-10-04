#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2dEval_RepCurveDesc.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Line.hxx>
#include <GeomEval_RepCurveDesc.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>

class Refresh2d final : public Geom2d_BSplineCurve
{
public:
  using Geom2d_BSplineCurve::Geom2d_BSplineCurve;
  void Refresh() { updateKnots(); }
};

class Refresh3d final : public Geom_BSplineCurve
{
public:
  using Geom_BSplineCurve::Geom_BSplineCurve;
  void Refresh() { updateKnots(); }
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

template <class Curve>
bool cached_resolution(const Curve& theCurve)
{
  std::ostringstream aStream;
  theCurve.DumpJson(aStream, -1);
  return aStream.str().find("\"MaxDerivInvOk\": 1") != std::string::npos;
}

template <class Curve>
void emit_metadata(const char* theName, const Curve& theCurve)
{
  std::cout << theName << ' ' << static_cast<int>(theCurve.KnotDistribution()) << ' '
            << static_cast<int>(theCurve.Continuity()) << ' ' << theCurve.IsPeriodic() << ' '
            << theCurve.KnotSequence().Length();
  for (int anIndex = theCurve.KnotSequence().Lower();
       anIndex <= theCurve.KnotSequence().Upper(); ++anIndex)
  {
    std::cout << ' ' << theCurve.KnotSequence()(anIndex);
  }
  std::cout << '\n';
}

template <class Curve>
void emit_state(const char* theName, const char* theStage, const Curve& theCurve)
{
  std::cout << theName << "_state_" << theStage << ' ' << cached_resolution(theCurve) << ' '
            << theCurve.HasEvalRepresentation() << '\n';
}

template <class Curve>
void run_probe(const char* theName, Curve& theCurve)
{
  double aParametricTolerance = 0.0;
  theCurve.Resolution(1.0, aParametricTolerance);
  emit_metadata((std::string(theName) + "_before").c_str(), theCurve);
  emit_state(theName, "before", theCurve);
  theCurve.Refresh();
  emit_metadata((std::string(theName) + "_after").c_str(), theCurve);
  emit_state(theName, "after", theCurve);
}

void probe_2d()
{
  NCollection_Array1<gp_Pnt2d> aPoles(1, 6);
  for (int anIndex = 1; anIndex <= 6; ++anIndex)
  {
    aPoles(anIndex) = gp_Pnt2d(anIndex, anIndex % 2);
  }
  const double                          aKnotValues[] = {0.0, 0.25, 0.75, 1.0};
  const int                             aMultiplicityValues[] = {3, 1, 2, 3};
  const NCollection_Array1<double>      aKnots = doubles(aKnotValues);
  const NCollection_Array1<int>         aMults = integers(aMultiplicityValues);
  Refresh2d aCurve(aPoles, aKnots, aMults, 2, false);
  occ::handle<Geom2dEval_RepCurveDesc::Full> aDesc = new Geom2dEval_RepCurveDesc::Full();
  aDesc->Representation = new Geom2d_Line(gp_Lin2d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)));
  aCurve.SetEvalRepresentation(aDesc);
  run_probe("2d", aCurve);
}

void probe_3d_periodic()
{
  NCollection_Array1<gp_Pnt> aPoles(1, 5);
  const gp_Pnt aPoints[] = {gp_Pnt(1, 0, 0), gp_Pnt(0.3, 1, 0), gp_Pnt(-0.8, 0.6, 0),
                            gp_Pnt(-0.8, -0.6, 0), gp_Pnt(0.3, -1, 0)};
  for (int anIndex = 1; anIndex <= 5; ++anIndex)
  {
    aPoles(anIndex) = aPoints[anIndex - 1];
  }
  const double                     aKnotValues[] = {0.0, 0.2, 0.4, 0.6, 0.8, 1.0};
  const int                        aMultiplicityValues[] = {1, 1, 1, 1, 1, 1};
  const NCollection_Array1<double> aKnots = doubles(aKnotValues);
  const NCollection_Array1<int>    aMults = integers(aMultiplicityValues);
  Refresh3d aCurve(aPoles, aKnots, aMults, 3, true);
  occ::handle<GeomEval_RepCurveDesc::Full> aDesc = new GeomEval_RepCurveDesc::Full();
  aDesc->Representation = new Geom_Line(gp_Lin(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(1.0, 0.0, 0.0)));
  aCurve.SetEvalRepresentation(aDesc);
  run_probe("3d", aCurve);
}

int main()
{
  std::cout << std::setprecision(17);
  probe_2d();
  probe_3d_periodic();
}
