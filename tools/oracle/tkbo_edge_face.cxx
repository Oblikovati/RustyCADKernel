#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <IntTools_EdgeFace.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

static void run(const char* label, const TopoDS_Edge& edge, const TopoDS_Face& face,
                double first, double last, bool quick = false) {
  IntTools_EdgeFace intersector;
  intersector.SetEdge(edge);
  intersector.SetFace(face);
  intersector.SetRange(first, last);
  intersector.UseQuickCoincidenceCheck(quick);
  intersector.Perform();
  std::cout << label << ' ' << intersector.IsDone() << ' ' << intersector.ErrorStatus()
            << ' ' << intersector.CommonParts().Length() << ' '
            << intersector.MinimalDistance() << '\n';
  for (int index = 1; index <= intersector.CommonParts().Length(); ++index) {
    const IntTools_CommonPrt& part = intersector.CommonParts()(index);
    double firstRange, lastRange;
    part.Range1(firstRange, lastRange);
    gp_Pnt firstPoint, lastPoint;
    part.BoundingPoints(firstPoint, lastPoint);
    std::cout << "part " << static_cast<int>(part.Type()) << ' ' << firstRange << ' '
              << lastRange << ' ' << part.VertexParameter1() << ' '
              << firstPoint.X() << ' ' << firstPoint.Y() << ' ' << firstPoint.Z()
              << ' ' << lastPoint.X() << ' ' << lastPoint.Y() << ' ' << lastPoint.Z() << '\n';
  }
}

int main() {
  std::cout << std::setprecision(17);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), -5, 5, -5, 5).Face();
  const gp_Cylinder cylinder(gp_Ax3(), 1.0);
  const TopoDS_Face cylinderFace =
      BRepBuilderAPI_MakeFace(cylinder, -3.141592653589793, 3.141592653589793, -2, 2).Face();
  const gp_Pln tangentPlane(gp_Pnt(1, 0, 0), gp_Dir(1, 0, 0));
  const TopoDS_Face tangentFace = BRepBuilderAPI_MakeFace(tangentPlane, -2, 2, -2, 2).Face();
  const TopoDS_Edge crossing = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, -1), gp_Pnt(0, 0, 1)).Edge();
  const TopoDS_Edge contained = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  const TopoDS_Edge cylinderTouch = BRepBuilderAPI_MakeEdge(gp_Pnt(-2, 1, 0), gp_Pnt(2, 1, 0)).Edge();
  const gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0);
  const TopoDS_Edge circleTouch = BRepBuilderAPI_MakeEdge(circle).Edge();
  run("crossing", crossing, face, 0, 1);
  run("quick-contained", contained, face, 0, 1, true);
  run("line-cylinder-touch", cylinderTouch, cylinderFace, 0, 4);
  run("circle-plane-touch", circleTouch, tangentFace, 0, 6.283185307179586);
  gp_Trsf transform;transform.SetTranslation(gp_Vec(10,20,30));
  const TopLoc_Location location(transform);
  TopoDS_Edge locatedEdge=cylinderTouch;locatedEdge.Location(location);
  TopoDS_Face locatedFace=cylinderFace;locatedFace.Location(location);
  run("located-cylinder-touch",locatedEdge,locatedFace,0,4);
  return 0;
}
