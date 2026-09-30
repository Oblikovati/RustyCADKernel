// IntTools_FClass2d real faces, periodic copies, exact boundary fallback and repeated Init.
#include <IntTools_FClass2d.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <TopoDS.hxx>
TopoDS_Wire bo_fclass_circle(double r) {return BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),r))).Wire();}
TopoDS_Face bo_fclass_face(int kind,double r) {
 switch(kind) {
  case 1:return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(),r),0.25,5.5,-1.,1.);
  case 2:return BRepBuilderAPI_MakeFace(gp_Cone(gp_Ax3(),0.3,r),0.25,5.5,-1.,1.);
  case 3:return BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(),r),0.25,5.5,-1.,1.);
  case 4:return BRepBuilderAPI_MakeFace(gp_Torus(gp_Ax3(),r*2.,r),0.25,5.5,-1.,1.);
  case 5:case 6:{BRepBuilderAPI_MakeFace maker(gp_Pln(),bo_fclass_circle(r),true);
   if(kind==6)maker.Add(TopoDS::Wire(bo_fclass_circle(r*0.5).Reversed()));return maker.Face();}
  case 7:return BRepBuilderAPI_MakeFace(gp_Pln());
  case 9:return BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(),r));
  case 10:{TopoDS_Edge e=BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(),r,r*1e-5));return BRepBuilderAPI_MakeFace(gp_Pln(),BRepBuilderAPI_MakeWire(e).Wire(),true);}
  case 11:case 12:case 13:{TopoDS_Face f=BRepBuilderAPI_MakeFace(gp_Pln());BRep_Builder b;TopoDS_Wire w;b.MakeWire(w);
   if(kind==11)w=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(r,0.,0.))).Wire();
   if(kind==13){TopoDS_Edge e;b.MakeEdge(e);b.Add(w,e);}b.Add(f,w);return f;}

  default:{TopoDS_Face f=BRepBuilderAPI_MakeFace(gp_Pln(),0.,r,0.,r);if(kind==8)f.Reverse();return f;}
 }
}
void bo_fclass_state(IntTools_FClass2d& c,const gp_Pnt2d& p,const std::vector<double>& v) {
 bo_int(c.IsHole());bo_int(c.PerformInfinitePoint());bo_int(c.Perform(p,v[6]!=0.));bo_int(c.TestOnRestriction(p,v[5],v[6]!=0.));
}
void bo_fclass_case(const std::vector<double>& v) {
 try {
  TopoDS_Face f=bo_fclass_face(int(v[0]),v[1]);gp_Pnt2d p(v[3],v[4]);IntTools_FClass2d c(f,v[2]);bo_fclass_state(c,p,v);
  c.Init(f,v[2]);bo_fclass_state(c,p,v);c.Init(bo_fclass_face(0,v[1]*0.5),v[2]);bo_fclass_state(c,p,v);
  IntTools_FClass2d fresh;bo_int(fresh.Perform(p,true));fresh.Init(f,v[2]);bo_fclass_state(fresh,p,v);
 } catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
