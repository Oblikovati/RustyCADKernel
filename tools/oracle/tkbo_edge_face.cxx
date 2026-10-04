#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <IntTools_EdgeFace.hxx>
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
  const TopoDS_Edge crossing = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, -1), gp_Pnt(0, 0, 1)).Edge();
  const TopoDS_Edge contained = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  run("crossing", crossing, face, 0, 1);
  run("quick-contained", contained, face, 0, 1, true);
}
