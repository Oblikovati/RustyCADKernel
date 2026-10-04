#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Wire.hxx>
void run(int kind,int mode,int wireori,int faceori){
 auto fixture=pcurve_fixture(kind==5?0:kind);BRep_Builder builder;TopoDS_Wire wire;
 if(kind==5){builder.MakeWire(wire);occ::handle<Geom_Curve> circle=new Geom_Circle(gp_Ax2(),1.);builder.Add(wire,BRepBuilderAPI_MakeEdge(circle,0,2*M_PI).Edge());}
 else{TopExp_Explorer ex(fixture.face,TopAbs_WIRE);wire=TopoDS::Wire(ex.Current());}
 if(mode){TopoDS_Wire composite;builder.MakeWire(composite);TopoDS_Edge empty;builder.MakeEdge(empty);if(mode==1)builder.Add(composite,empty);for(TopoDS_Iterator it(wire);it.More();it.Next())builder.Add(composite,it.Value());if(mode==2)builder.Add(composite,empty);wire=composite;}
 wire.Orientation(TopAbs_Orientation(wireori));fixture.face.Orientation(TopAbs_Orientation(faceori));
 std::cout<<kind<<' '<<mode<<' '<<wireori<<' '<<faceori<<' '<<BOPTools_AlgoTools::IsHole(wire,fixture.face)<<'\n';
}
int main(){for(int k=0;k<6;k++)for(int m=0;m<3;m++)for(int w=0;w<4;w++)for(int f=0;f<4;f++)run(k,m,w,f);}
