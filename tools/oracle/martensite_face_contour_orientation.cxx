// A real curve callback removes its current edge vertex; native contour processing must reread Orientation before sampling.
#include <Geom_Curve.hxx>
#include <Geom_BezierSurface.hxx>
#include <StdSelect_BRepSelectionTool.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp.hxx>
#include <iostream>
class ContourMutationCurve:public Geom_Curve
{
public:
  TopoDS_Edge* edge=nullptr;
  mutable bool mutate=false,record=false;
  mutable int calls=0;
  void Reverse() override {}
  double ReversedParameter(double u) const override{return 1.-u;}
  double FirstParameter() const override{return 0.;}
  double LastParameter() const override{return 1.;}
  bool IsClosed() const override{return false;}
  bool IsPeriodic() const override{return false;}
  GeomAbs_Shape Continuity() const override{return GeomAbs_CN;}
  bool IsCN(int) const override{return true;}
  gp_Pnt EvalD0(double u) const override
  {
    if(record)++calls;
    if(!mutate)return gp_Pnt(u,0,0);
    mutate=false;edge->Free(true);
    const TopoDS_Vertex vertex=TopExp::FirstVertex(*edge);BRep_Builder().Remove(*edge,vertex);
    return gp_Pnt(u,0,0);
  }
  ResD1 EvalD1(double u) const override{return {EvalD0(u),gp_Vec(1,0,0)};}
  ResD2 EvalD2(double u) const override{return {EvalD0(u),gp_Vec(1,0,0),gp_Vec(0,0,0)};}
  ResD3 EvalD3(double u) const override{return {EvalD0(u),gp_Vec(1,0,0),gp_Vec(0,0,0),gp_Vec(0,0,0)};}
  gp_Vec EvalDN(double,int n) const override{return n==1?gp_Vec(1,0,0):gp_Vec(0,0,0);}
  void Transform(const gp_Trsf&) override {}
  occ::handle<Geom_Geometry> Copy() const override{return new ContourMutationCurve;}
};
int main()
{
  const occ::handle<ContourMutationCurve> curve=new ContourMutationCurve;
  TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);curve->edge=&edge;
  TopoDS_Wire wire;BRep_Builder builder;builder.MakeWire(wire);builder.Add(wire,edge);
  NCollection_Array2<gp_Pnt> poles(1,2,1,2);
  poles(1,1)=gp_Pnt(0,0,0);poles(1,2)=gp_Pnt(0,1,0);poles(2,1)=gp_Pnt(1,0,0);poles(2,2)=gp_Pnt(1,1,1);
  TopoDS_Face face;builder.MakeFace(face,new Geom_BezierSurface(poles),1.e-7);builder.Add(face,wire);
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(5);
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> entities;
  entities.Append(new Select3D_SensitivePoint(owner,gp_Pnt(9,8,7)));
  curve->mutate=true;curve->record=true;bool failed=false;
  try{StdSelect_BRepSelectionTool::GetSensitiveForFace(face,owner,entities,false,9,500.,true);}
  catch(const Standard_NoSuchObject&){failed=true;}
  std::cout<<"contour-mutation "<<failed<<' '<<curve->calls<<' '<<entities.Length()<<' '<<edge.NbChildren()<<'\n';
}
