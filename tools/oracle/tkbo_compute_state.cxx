#include <BOPTools_AlgoTools.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <IntTools_Context.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <Geom_Line.hxx>
#include <iostream>
#include <iomanip>
TopoDS_Solid solid_fixture(int kind){if(kind==0)return BRepPrimAPI_MakeBox(1.,2.,3.).Solid();if(kind==1)return BRepPrimAPI_MakeCylinder(2.,3.).Solid();return BRepPrimAPI_MakeSphere(2.).Solid();}
void point_case(int kind,int orientation,int point_index){
 auto solid=solid_fixture(kind);solid.Orientation(TopAbs_Orientation(orientation));occ::handle<IntTools_Context> context=new IntTools_Context;
 gp_Pnt points[]={gp_Pnt(.5,.5,1.),gp_Pnt(0,0,0),gp_Pnt(10,10,10),gp_Pnt(1.+1e-8,1.,1.)};auto point=points[point_index];TopoDS_Vertex vertex;BRep_Builder().MakeVertex(vertex,point,1e-7);
 std::cout<<kind<<' '<<orientation<<' '<<point_index<<' '<<int(BOPTools_AlgoTools::ComputeState(point,solid,1e-7,context))<<' '<<int(BOPTools_AlgoTools::ComputeState(vertex,solid,1e-7,context))<<' '<<int(BOPTools_AlgoTools::ComputeStateByOnePoint(vertex,solid,1e-7,context))<<'\n';
}
void topology_case(int mode,int orientation){
 auto solid=solid_fixture(0);solid.Orientation(TopAbs_Orientation(orientation));occ::handle<IntTools_Context> context=new IntTools_Context;BRep_Builder builder;TopoDS_Shape shape;int direct=3;
 if(mode<6){TopoDS_Edge edge;if(mode<4){occ::handle<Geom_Curve> curve=new Geom_Line(gp_Pnt(.5,.5,1.),gp_Dir(1,0,0));double firsts[]={-.1,-2e100,-.1,-2e100};double lasts[]={.1,.1,2e100,2e100};builder.MakeEdge(edge,curve,1e-7);builder.Range(edge,firsts[mode],lasts[mode]);}else{builder.MakeEdge(edge);if(mode==4){TopoDS_Vertex vertex;builder.MakeVertex(vertex,gp_Pnt(.5,.5,1.),1e-7);builder.Add(edge,vertex);}}shape=edge;direct=int(BOPTools_AlgoTools::ComputeState(edge,solid,1e-7,context));}
 else if(mode<8){TopExp_Explorer ex(solid,TopAbs_FACE);auto face=TopoDS::Face(ex.Current());NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> bounds;if(mode==7)TopExp::MapShapes(solid,TopAbs_EDGE,bounds);shape=face;direct=int(BOPTools_AlgoTools::ComputeState(face,solid,1e-7,bounds,context));}
 else{TopoDS_Compound compound;builder.MakeCompound(compound);if(mode==8){TopoDS_Vertex vertex;builder.MakeVertex(vertex,gp_Pnt(.5,.5,1.),1e-7);builder.Add(compound,vertex);}shape=compound;}
 std::cout<<mode<<' '<<orientation<<' '<<direct<<' '<<int(BOPTools_AlgoTools::ComputeStateByOnePoint(shape,solid,1e-7,context))<<'\n';
}
int main(int argc,char**){std::cout<<std::setprecision(17);if(argc>1){for(int m=0;m<10;m++)for(int o=0;o<4;o++)topology_case(m,o);return 0;}for(int k=0;k<3;k++)for(int o=0;o<4;o++)for(int p=0;p<4;p++)point_case(k,o,p);}
