#include <BOPTools_Set.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <iostream>
class ProbeSet:public BOPTools_Set {public:using BOPTools_Set::Clear;bool valid_sum()const{size_t sum=0;for(auto const& s:myShapes){size_t id=TopTools_ShapeMapHasher{}(s)%432123+1;size_t threshold=SIZE_MAX/myNbShapes;sum+=id>threshold?id%threshold:id;}return sum==GetSum();}};
TopoDS_Shape fixture(int mode,TopoDS_Vertex const&a,TopoDS_Vertex const&b,TopoDS_Edge const&e){BRep_Builder builder;TopoDS_Compound c;builder.MakeCompound(c);if(mode==0){builder.Add(c,a);builder.Add(c,b);}if(mode==1){builder.Add(c,a);builder.Add(c,a);}if(mode==2){builder.Add(c,b);builder.Add(c,a);}if(mode==3)builder.Add(c,a.Oriented(TopAbs_INTERNAL));if(mode==4)builder.Add(c,e);if(mode==5){builder.Add(c,e);builder.Add(c,e.Oriented(TopAbs_INTERNAL));}return c;}
void run(int first,int second,int type,int orientation){BRep_Builder builder;TopoDS_Vertex a,b;builder.MakeVertex(a,gp_Pnt(),1e-7);builder.MakeVertex(b,gp_Pnt(1,0,0),1e-7);TopoDS_Edge e;builder.MakeEdge(e);builder.Degenerated(e,true);builder.Add(e,a);
 auto x=fixture(first,a,b,e),y=fixture(second,a,b,e);x.Orientation(TopAbs_Orientation(orientation));ProbeSet one,two;one.Add(x,TopAbs_ShapeEnum(type));two.Add(y,TopAbs_ShapeEnum(type));ProbeSet copied=one;ProbeSet assigned;assigned.Assign(two);
 std::cout<<first<<' '<<second<<' '<<type<<' '<<orientation<<' '<<one.NbShapes()<<' '<<two.NbShapes()<<' '<<one.IsEqual(two)<<' '<<two.IsEqual(one)<<' '<<one.valid_sum()<<' '<<two.valid_sum()<<' '<<one.Shape().IsEqual(x)<<' '<<copied.IsEqual(one)<<' '<<assigned.IsEqual(two);
 one.Clear();std::cout<<' '<<one.NbShapes()<<' '<<one.GetSum()<<' '<<one.Shape().IsEqual(x)<<' '<<copied.NbShapes()<<'\n';}
int main(){for(int a=0;a<7;a++)for(int b=0;b<7;b++)for(int type:{6,7})for(int o=0;o<4;o++)run(a,b,type,o);}
