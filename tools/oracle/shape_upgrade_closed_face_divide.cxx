#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <ShapeExtend.hxx>
#include <ShapeUpgrade_ClosedFaceDivide.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_SphericalSurface.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static void report(const char* name, ShapeUpgrade_ClosedFaceDivide& divider, bool changed) {
  const TopoDS_Shape result = divider.Result();
  int faces = 0;
  int edges = 0;
  int seams = 0;
  for (TopExp_Explorer faceIt(result, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++faces;
    for (TopExp_Explorer edgeIt(faceIt.Current(), TopAbs_EDGE); edgeIt.More(); edgeIt.Next()) {
      ++edges;
      if (BRep_Tool::IsClosed(TopoDS::Edge(edgeIt.Current()), TopoDS::Face(faceIt.Current()))) {
        ++seams;
      }
    }
  }
  std::cout << name << " changed=" << changed << " faces=" << faces << " edges=" << edges
            << " seams=" << seams << " done2=" << divider.Status(ShapeExtend_DONE2)
            << " fail2=" << divider.Status(ShapeExtend_FAIL2)
            << " fail3=" << divider.Status(ShapeExtend_FAIL3) << '\n';
}

int main() {
  ShapeUpgrade_ClosedFaceDivide invalidDivider;
  const bool invalidChanged = invalidDivider.SplitSurface(0.0);
  report("empty_result_fail3", invalidDivider, invalidChanged);

  BRepPrimAPI_MakeCylinder cylinder(2.0, 5.0);
  TopoDS_Face cylinderFace = cylinder.Face();
  ShapeUpgrade_ClosedFaceDivide defaultDivider(cylinderFace);
  const bool defaultChanged = defaultDivider.Perform();
  report("cylinder_seam_default", defaultDivider, defaultChanged);

  ShapeUpgrade_ClosedFaceDivide multiDivider(cylinderFace);
  multiDivider.SetNbSplitPoints(2);
  const bool multiChanged = multiDivider.Perform();
  report("cylinder_seam_two_points", multiDivider, multiChanged);

  BRepPrimAPI_MakeSphere sphere(2.0);
  TopoDS_Face sphereFace = sphere.Face();
  ShapeUpgrade_ClosedFaceDivide sphereDivider(sphereFace);
  const bool sphereChanged = sphereDivider.Perform();
  report("sphere_geometric_closure", sphereDivider, sphereChanged);

  occ::handle<Geom_SphericalSurface> nearClosedSurface =
      new Geom_SphericalSurface(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 2.0);
  TopoDS_Face nearClosedFace =
      BRepBuilderAPI_MakeFace(nearClosedSurface, 0.0, 6.283185307179586 - 0.01, -1.0, 1.0, 1e-7);
  ShapeUpgrade_ClosedFaceDivide defaultPrecisionDivider(nearClosedFace);
  const bool defaultPrecisionChanged = defaultPrecisionDivider.Perform();
  report("near_closed_default_precision", defaultPrecisionDivider, defaultPrecisionChanged);
  ShapeUpgrade_ClosedFaceDivide precisionDivider(nearClosedFace);
  precisionDivider.SetPrecision(0.1);
  const bool precisionChanged = precisionDivider.Perform();
  report("near_closed_precision", precisionDivider, precisionChanged);
}
