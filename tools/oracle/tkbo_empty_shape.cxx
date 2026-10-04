#include <gp_Pln.hxx>
#include <BOPTools_AlgoTools3D.hxx>
#include <BRep_Builder.hxx>
#include <BRep_TEdge.hxx>
#include <BRep_Polygon3D.hxx>
#include <BRep_PolygonOnSurface.hxx>
#include <BRep_CurveOn2Surfaces.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_Plane.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_Triangulation.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <Standard_DomainError.hxx>
#include <iostream>
TopoDS_Shape fixture(int kind){
 BRep_Builder builder;TopoDS_Edge edge;builder.MakeEdge(edge);TopoDS_Vertex vertex;builder.MakeVertex(vertex);
 if(kind==0)return TopoDS_Shape();
 if(kind==2)return vertex;
 if(kind==3)return edge;
 if(kind==4)return BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(1,0,0)).Edge();
 if(kind==5||kind==6||kind==13){TopoDS_Face face;builder.MakeFace(face);if(kind==6)builder.UpdateFace(face,new Geom_Plane(gp_Pln()),TopLoc_Location(),1e-7);if(kind==13)builder.UpdateFace(face,new Poly_Triangulation(3,1,false),true);return face;}
 if(kind>=9){auto rep=occ::down_cast<BRep_TEdge>(edge.TShape());
  if(kind==9||kind==10){Handle(Poly_Polygon3D) polygon;if(kind==9)polygon=new Poly_Polygon3D(2,false);rep->ChangeCurves().Append(new BRep_Polygon3D(polygon,TopLoc_Location()));}
  if(kind==11)rep->ChangeCurves().Append(new BRep_PolygonOnSurface(Handle(Poly_Polygon2D)(),Handle(Geom_Surface)(),TopLoc_Location()));
  if(kind==12)rep->ChangeCurves().Append(new BRep_CurveOn2Surfaces(Handle(Geom_Surface)(),Handle(Geom_Surface)(),TopLoc_Location(),TopLoc_Location(),GeomAbs_G1));
  return edge;
 }
 TopoDS_Compound compound;builder.MakeCompound(compound);if(kind==7)builder.Add(compound,edge);if(kind==8)builder.Add(compound,vertex);return compound;
}
int main(){for(int kind=0;kind<14;kind++){int result;try{result=BOPTools_AlgoTools3D::IsEmptyShape(fixture(kind));}catch(const Standard_DomainError&){result=-1;}std::cout<<kind<<' '<<result<<'\n';}}
