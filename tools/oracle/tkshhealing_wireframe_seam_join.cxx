#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <Precision.hxx>
#include <TopLoc_Location.hxx>
#include <cmath>
#include <BRep_Tool.hxx>
#include <ShapeFix_Wireframe.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <iostream>

static TopoDS_Edge makeSeam(BRep_Builder& b, const TopoDS_Face& f, double z0, double z1) {
  TopoDS_Edge e;
  Handle(Geom_Curve) c = new Geom_Line(gp_Lin(gp_Pnt(1,0,0), gp_Dir(0,0,1)));
  b.MakeEdge(e, c, Precision::Confusion());
  TopoDS_Vertex v0, v1;
  b.MakeVertex(v0, gp_Pnt(1,0,z0), Precision::Confusion());
  b.MakeVertex(v1, gp_Pnt(1,0,z1), Precision::Confusion());
  b.Add(e, v0.Oriented(TopAbs_FORWARD));
  b.Add(e, v1.Oriented(TopAbs_REVERSED));
  b.UpdateVertex(v0, z0, e, Precision::Confusion());
  b.UpdateVertex(v1, z1, e, Precision::Confusion());
  b.Range(e, z0, z1);
  Handle(Geom2d_Curve) p0 = new Geom2d_Line(gp_Lin2d(gp_Pnt2d(0,z0), gp_Dir2d(0,1)));
  Handle(Geom2d_Curve) p1 = new Geom2d_Line(gp_Lin2d(gp_Pnt2d(2*M_PI,z0), gp_Dir2d(0,1)));
  b.UpdateEdge(e, p0, p1, f, 0.0);
  b.Range(e, f, z0, z1);
  return e;
}
static TopoDS_Face makeFace(TopoDS_Edge& e1, TopoDS_Edge& e2, TopoDS_Edge& e3) {
  BRep_Builder b;
  TopoDS_Face f;
  Handle(Geom_Surface) s = new Geom_CylindricalSurface(gp_Cylinder(gp_Ax3(), 1.0));
  b.MakeFace(f, s, TopLoc_Location(), Precision::Confusion());
  e1 = makeSeam(b, f, 0.0, 1.0);
  e2 = makeSeam(b, f, 1.0, 1.001);
  e3 = makeSeam(b, f, 0.0, 1.001);
  TopoDS_Wire w; b.MakeWire(w);
  b.Add(w, e1); b.Add(w, e2); b.Add(w, e3.Reversed());
  b.Add(f, w);
  return f;
}
int main(){
  TopoDS_Edge e1,e2,e3;
  TopoDS_Face f=makeFace(e1,e2,e3);
  ShapeFix_Wireframe fixer(f); fixer.SetPrecision(.1); fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> small,multiple;
  NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> edgefaces,facesmall;
  small.Add(e2); NCollection_List<TopoDS_Shape> fs; fs.Append(f);
  edgefaces.Bind(e1,fs); edgefaces.Bind(e2,fs); edgefaces.Bind(e3,fs);
  NCollection_List<TopoDS_Shape> es; es.Append(e2); facesmall.Bind(f,es);
  std::cout << "seams " << BRep_Tool::IsClosed(e1,f) << ' ' << BRep_Tool::IsClosed(e2,f) << ' ' << BRep_Tool::IsClosed(e3,f) << '\n';
  const bool changed=fixer.MergeSmallEdges(small,edgefaces,facesmall,multiple,false,-1.0);
  auto result=fixer.Context()->Apply(f);
  int count=0, seam=0;
  for(TopExp_Explorer it(result,TopAbs_EDGE);it.More();it.Next()){++count; if(BRep_Tool::IsClosed(TopoDS::Edge(it.Current()),TopoDS::Face(result)))++seam;}
  std::cout << "seam_pair_join " << changed << ' ' << count << ' ' << seam << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << ' ' << fixer.StatusSmallEdges(ShapeExtend_FAIL1) << '\n';
}
