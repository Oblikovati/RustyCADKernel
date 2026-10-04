#include <Geom2d_BezierCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <iostream>
#include <string>

class Exposed2d : public Geom2d_BezierCurve
{
public:
  using Geom2d_BezierCurve::init;
  explicit Exposed2d(const NCollection_Array1<gp_Pnt2d>& poles) : Geom2d_BezierCurve(poles) {}
};

class Exposed3d : public Geom_BezierCurve
{
public:
  using Geom_BezierCurve::init;
  explicit Exposed3d(const NCollection_Array1<gp_Pnt>& poles) : Geom_BezierCurve(poles) {}
};

NCollection_Array1<double> weights()
{
  NCollection_Array1<double> result(1, 3);
  result(1) = 1.;
  result(2) = 2.;
  result(3) = 1.;
  return result;
}

NCollection_Array1<gp_Pnt> poles3d()
{
  NCollection_Array1<gp_Pnt> result(1, 3);
  result(1) = gp_Pnt(0., 0., 0.);
  result(2) = gp_Pnt(1., 2., 0.);
  result(3) = gp_Pnt(2., 0., 0.);
  return result;
}

NCollection_Array1<gp_Pnt2d> poles2d()
{
  NCollection_Array1<gp_Pnt2d> result(1, 3);
  result(1) = gp_Pnt2d(0., 0.);
  result(2) = gp_Pnt2d(1., 2.);
  result(3) = gp_Pnt2d(2., 0.);
  return result;
}

int main(int argc, char** argv)
{
  const auto w = weights();
  if (argc != 2)
  {
    return 2;
  }
  if (std::string(argv[1]) == "2d")
  {
    const auto poles = poles2d();
    Exposed2d curve(poles);
    std::cerr << "before-2d-init\n";
    curve.init(poles, &w);
    std::cerr << "after-2d-init\n";
    return 0;
  }
  const auto poles = poles3d();
  Exposed3d curve(poles);
  std::cerr << "before-3d-init\n";
  curve.init(poles, &w);
  std::cerr << "after-3d-init\n";
  return 0;
}
