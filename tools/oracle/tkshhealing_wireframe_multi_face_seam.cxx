#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Compound.hxx>
#include <Precision.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Trsf.hxx>
#include <gp_Ax1.hxx>
#include <cmath>
#include <BRep_Tool.hxx>
#include <ShapeFix_Wireframe.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
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
static TopoDS_Edge makeSeam(BRep_Builder& b, const TopoDS_Face& f1,const TopoDS_Face& f2,double z0,double z1,double u2){
  TopoDS_Edge e; Handle(Geom_Curve)c=new Geom_Line(gp_Lin(gp_Pnt(1,0,0),gp_Dir(0,0,1))); b.MakeEdge(e,c,Precision::Confusion());
  TopoDS_Vertex v0,v1; b.MakeVertex(v0,gp_Pnt(1,0,z0),Precision::Confusion());b.MakeVertex(v1,gp_Pnt(1,0,z1),Precision::Confusion());
  b.Add(e,v0.Oriented(TopAbs_FORWARD));b.Add(e,v1.Oriented(TopAbs_REVERSED));b.UpdateVertex(v0,z0,e,Precision::Confusion());b.UpdateVertex(v1,z1,e,Precision::Confusion());b.Range(e,z0,z1);
  Handle(Geom2d_Curve)p0=new Geom2d_Line(gp_Lin2d(gp_Pnt2d(0,z0),gp_Dir2d(0,1)));Handle(Geom2d_Curve)p1=new Geom2d_Line(gp_Lin2d(gp_Pnt2d(2*M_PI,z0),gp_Dir2d(0,1)));
  b.UpdateEdge(e,p0,p1,f1,0.0);b.Range(e,f1,z0,z1);
  Handle(Geom2d_Curve)q0=new Geom2d_Line(gp_Lin2d(gp_Pnt2d(u2,z0),gp_Dir2d(0,1)));Handle(Geom2d_Curve)q1=new Geom2d_Line(gp_Lin2d(gp_Pnt2d(u2+2*M_PI,z0),gp_Dir2d(0,1)));
  b.UpdateEdge(e,q0,q1,f2,0.0);b.Range(e,f2,z0,z1); return e;
}
int main(){BRep_Builder b; TopoDS_Face f1,f2;Handle(Geom_Surface)s=new Geom_CylindricalSurface(gp_Cylinder(gp_Ax3(),1.0));b.MakeFace(f1,s,TopLoc_Location(),Precision::Confusion());
 gp_Trsf tr;tr.SetRotation(gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,0,1)),0.7);TopLoc_Location loc(tr);b.MakeFace(f2,s,loc,Precision::Confusion());
 TopoDS_Edge e1=makeSeam(b,f1,f2,0,1,-0.7),e2=makeSeam(b,f1,f2,1,1.001,-0.7),e3=makeSeam(b,f1,f2,0,1.001,-0.7);
 TopoDS_Wire w1,w2;b.MakeWire(w1);b.Add(w1,e1);b.Add(w1,e2);b.Add(w1,e3.Reversed());b.MakeWire(w2);b.Add(w2,e1);b.Add(w2,e2);b.Add(w2,e3.Reversed());b.Add(f1,w1);b.Add(f2,w2);
 std::cout<<"input "<<BRep_Tool::IsClosed(e1,f1)<<' '<<BRep_Tool::IsClosed(e1,f2)<<' '<<BRep_Tool::IsClosed(e2,f1)<<' '<<BRep_Tool::IsClosed(e2,f2)<<'\n';
 TopoDS_Compound comp;b.MakeCompound(comp);b.Add(comp,f1);b.Add(comp,f2);ShapeFix_Wireframe fix(comp);fix.SetPrecision(.1);fix.SetContext(new ShapeBuild_ReShape);
 NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> small,multiple;NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> ef,fs;
 small.Add(e2); NCollection_List<TopoDS_Shape> faces;faces.Append(f1);faces.Append(f2);ef.Bind(e1,faces);ef.Bind(e2,faces);ef.Bind(e3,faces);NCollection_List<TopoDS_Shape> es;es.Append(e2);fs.Bind(f1,es);
 bool changed=fix.MergeSmallEdges(small,ef,fs,multiple,false,-1);TopoDS_Shape res=fix.Context()->Apply(comp);
 int edges=0,s1=0,s2=0;for(TopExp_Explorer it(res,TopAbs_EDGE);it.More();it.Next()){edges++;auto e=TopoDS::Edge(it.Current());if(BRep_Tool::IsClosed(e,f1))s1++;if(BRep_Tool::IsClosed(e,f2))s2++;}
 std::cout<<"joined "<<changed<<' '<<edges<<' '<<s1<<' '<<s2<<' '<<fix.StatusSmallEdges(ShapeExtend_DONE)<<' '<<fix.StatusSmallEdges(ShapeExtend_FAIL1)<<'\n';
}
