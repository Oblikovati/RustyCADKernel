#include <BRep_Builder.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeExtend.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Face face_on_surface(const occ::handle<Geom_Surface>& surface) {
  TopoDS_Face face;
  BRep_Builder{}.MakeFace(face, surface, 1e-7);
  return face;
}

static occ::handle<Geom_BezierSurface> bezier(const bool twisted) {
  TColgp_Array2OfPnt poles(1, 3, 1, 2);
  for (int u = 1; u <= 3; ++u) {
    const double x = twisted ? (u == 2 ? 10. : 0.) : static_cast<double>(u - 1);
    poles(u, 1) = gp_Pnt(x, 0., 0.);
    poles(u, 2) = gp_Pnt(x, 1., 0.);
  }
  return new Geom_BezierSurface(poles);
}

static void probe(const char* name, const TopoDS_Face& face) {
  ShapeAnalysis_CheckSmallFace analyzer;
  analyzer.SetTolerance(1e-4);
  double u = 77., v = 88.;
  const bool found = analyzer.CheckTwisted(face, u, v);
  std::cout << name << ' ' << found << ' ' << u << ' ' << v << ' '
            << analyzer.StatusTwisted(ShapeExtend_DONE) << '\n';
}

int main() {
  probe("elementary", face_on_surface(new Geom_Plane(gp_Ax3())));
  probe("flat", face_on_surface(bezier(false)));
  probe("twisted", face_on_surface(bezier(true)));
}
