#include <BRep_Builder.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeExtend.hxx>
#include <TopoDS_Face.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>

#include <iostream>

static TopoDS_Face face_on_surface(const occ::handle<Geom_Surface>& surface) {
  TopoDS_Face face;
  BRep_Builder{}.MakeFace(face, surface, 1e-7);
  return face;
}

static occ::handle<Geom_BezierSurface> poles(const bool sharp_last) {
  TColgp_Array2OfPnt points(1, 2, 1, 2);
  points(1, 1) = gp_Pnt(0, 0, 0);
  points(1, 2) = sharp_last ? gp_Pnt(0, 0, 0) : gp_Pnt(0, 5e-5, 0);
  points(2, 1) = gp_Pnt(1, 0, 0);
  points(2, 2) = sharp_last ? gp_Pnt(5e-5, 0, 0) : gp_Pnt(1, 5e-5, 0);
  return new Geom_BezierSurface(points);
}

static void probe(const char* name, const TopoDS_Face& face) {
  ShapeAnalysis_CheckSmallFace analyzer;
  analyzer.SetTolerance(1e-4);
  int row = 77, sense = 88;
  const bool found = analyzer.CheckPin(face, row, sense);
  std::cout << name << ' ' << found << ' ' << row << ' ' << sense << ' '
            << analyzer.StatusPin(ShapeExtend_DONE1) << ' '
            << analyzer.StatusPin(ShapeExtend_DONE2) << ' '
            << analyzer.StatusPin(ShapeExtend_DONE3) << '\n';
}

int main() {
  probe("elementary", face_on_surface(new Geom_Plane(gp_Ax3())));
  probe("regular", face_on_surface(poles(false)));
  probe("sharp_marker", face_on_surface(poles(true)));
}
