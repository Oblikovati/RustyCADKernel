// Native oracle for a fuse of a box with a trimmed torus ring crossing its seams (the input of
// BRepMesh_IncrementalMeshTest.BooleanTrimmedTorus_CrossingSeamProducesDenseMesh).
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include "tkbo_shape_dump.hxx"

int main()
{
  const TopoDS_Shape aBox = BRepPrimAPI_MakeBox(gp_Pnt(-187.0, -20.0, -67.5), gp_Pnt(187.0, 0.0, 67.5)).Shape();
  const gp_Circ      aCircle(gp_Ax2(gp_Pnt(200.0, -20.0, 0.0), gp_Dir(0.0, -1.0, 0.0)), 17.5);
  const TopoDS_Edge  anEdge  = BRepBuilderAPI_MakeEdge(aCircle).Edge();
  const TopoDS_Wire  aWire   = BRepBuilderAPI_MakeWire(anEdge).Wire();
  const TopoDS_Face  aProfile = BRepBuilderAPI_MakeFace(aWire, true).Face();
  const TopoDS_Shape aRing =
    BRepPrimAPI_MakeRevol(aProfile, gp_Ax1(gp_Pnt(0.0, -20.0, 0.0), gp_Dir(0.0, 0.0, -1.0)), 2.0 * M_PI).Shape();
  BRepAlgoAPI_Fuse aFuse(aBox, aRing);
  std::cout << "done " << aFuse.IsDone() << " warnings " << aFuse.HasWarnings() << '\n';
  ShapeIndex all;
  dumpResult(aFuse.Shape(), all);
  return 0;
}
