#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <ElCLib.hxx>
#include <GeomAbs_SurfaceType.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <algorithm>
#include <gp_Ax3.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Torus.hxx>
#include <iomanip>
#include <iostream>

int main()
{
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(
    gp_Circ(gp_Ax2(gp_Pnt(), gp_Dir(0., 0., 1.)), 1.), 0., 6.2831853071795864769);
  occ::handle<Geom_ToroidalSurface> torus = new Geom_ToroidalSurface(gp_Ax3(), 5., 2.);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(
    torus, 0., 6.2831853071795864769, 0., 6.2831853071795864769, 1.e-7);
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, true);
  const double periodT = curve.Period();
  const double periodU = surface.UPeriod();
  const double periodV = surface.VPeriod();
  const double parameter = ElCLib::InPeriod(0.1, 1., 1. + periodT);
  double u = ElCLib::InPeriod(0.2, 2., 2. + periodU);
  double v = ElCLib::InPeriod(0.3, 4., 4. + periodV);
  u = std::min(std::max(u, 0.), 6.2831853071795864769);
  v = std::min(std::max(v, 0.), 6.2831853071795864769);
  std::cout << std::setprecision(17) << curve.IsPeriodic() << ' '
            << surface.IsUPeriodic() << ' ' << surface.IsVPeriodic() << ' '
            << parameter << ' ' << u << ' ' << v << '\n';
  return 0;
}
