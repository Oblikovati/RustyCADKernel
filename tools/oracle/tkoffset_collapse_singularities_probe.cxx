// Oracle for BRepOffset::CollapseSingularities on one boundary pole side.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepOffset.hxx>
#include <Geom_BSplineSurface.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

int main()
{
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0.0, 0.0, 0.0);
  poles(1, 2) = gp_Pnt(0.0, 0.001, 0.0);
  poles(2, 1) = gp_Pnt(1.0, 0.0, 0.0);
  poles(2, 2) = gp_Pnt(1.0, 1.0, 0.0);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.0;
  knots(2) = 1.0;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 2;
  mults(2) = 2;
  occ::handle<Geom_BSplineSurface> surface =
    new Geom_BSplineSurface(poles, knots, knots, mults, mults, 1, 1, false, false);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 0.0, 1.0, 0.0, 1.0, 1.0e-7).Face();
  occ::handle<Geom_Surface> result = BRepOffset::CollapseSingularities(surface, face, 0.01);
  occ::handle<Geom_BSplineSurface> collapsed = occ::down_cast<Geom_BSplineSurface>(result);
  std::cout << (surface == result) << '\n';
  for (int u = 1; u <= 2; ++u)
  {
    for (int v = 1; v <= 2; ++v)
    {
      const gp_Pnt point = collapsed->Pole(u, v);
      std::cout << u << ' ' << v << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z()
                << '\n';
    }
  }
}
