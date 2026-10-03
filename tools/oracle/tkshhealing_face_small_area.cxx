// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: state effects of FixSmallAreaWire.
#include <ShapeFix_Face.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Pln.hxx>
#include <iostream>
class FaceProbe : public ShapeFix_Face { public: FaceProbe(const TopoDS_Face& face) : ShapeFix_Face(face) {} const TopoDS_Shape& RootShape() const { return myShape; } };
static TopoDS_Wire square(double side,double offset) {
  gp_Pnt p[]={{offset,0,0},{offset+side,0,0},{offset+side,side,0},{offset,side,0}};
  BRepBuilderAPI_MakeWire builder;
  for(int i=0;i<4;++i) builder.Add(BRepBuilderAPI_MakeEdge(p[i],p[(i+1)%4]).Edge());
  return builder.Wire();
}
static int count(const TopoDS_Shape& shape) {
  int result=0; for(TopoDS_Iterator it(shape);it.More();it.Next()) if(it.Value().ShapeType()==TopAbs_WIRE) ++result;
  return result;
}
static void inspect(const char* name,int mode) {
  TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  if(mode!=1 && mode!=2) BRep_Builder().Add(face,square(2.,0.));
  if(mode!=0) BRep_Builder().Add(face,square(1e-5,0.5));
  if(mode==4) face.Reverse();
  FaceProbe fixer(face); fixer.SetPrecision(1e-3);
  if(mode!=5) fixer.SetContext(new ShapeBuild_ReShape);
  if(mode==6) {
    TopoDS_Face replacement=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
    BRep_Builder().Add(replacement,square(1e-5,0.5));
    fixer.Context()->Replace(face,replacement);
  }
  TopoDS_Shape beforeResult=fixer.Result();
  bool changed=fixer.FixSmallAreaWire(mode!=2);
  std::cout<<name<<' '<<changed<<' '<<count(fixer.Face())<<' '<<(fixer.Face().IsSame(face))
    <<' '<<fixer.Face().Orientation()<<' '<<fixer.RootShape().IsSame(face)<<' '<<fixer.Result().IsSame(beforeResult)
    <<' '<<(!fixer.Context().IsNull() && fixer.Context()->Apply(face).IsNull())<<'\n';
}
int main() {inspect("large",0);inspect("all_remove",1);inspect("all_keep",2);inspect("mixed",3);inspect("reversed",4);inspect("no_context",5);inspect("context_replaced",6);}
