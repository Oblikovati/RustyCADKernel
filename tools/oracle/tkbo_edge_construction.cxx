#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopExp.hxx>
#include <IntTools_Curve.hxx>
#include <StdFail_NotDone.hxx>
void emit_edge(const TopoDS_Edge& edge,const TopoDS_Edge& source,const TopoDS_Vertex& first,const TopoDS_Vertex& last){
 double f,l;auto curve=BRep_Tool::Curve(edge,f,l);double sf,sl;auto original=BRep_Tool::Curve(source,sf,sl);
 std::cout<<' '<<edge.IsSame(source)<<' '<<(curve==original)<<' '<<int(edge.Orientation())<<' '<<f<<' '<<l<<' '<<BRep_Tool::Tolerance(edge)<<' '<<BRep_Tool::SameParameter(edge)<<' '<<BRep_Tool::SameRange(edge);
 int count=0;for(TopoDS_Iterator it(edge,false);it.More();it.Next())count++;std::cout<<' '<<count;
 for(TopoDS_Iterator it(edge,false);it.More();it.Next())std::cout<<' '<<it.Value().IsSame(first)<<' '<<it.Value().IsSame(last)<<' '<<int(it.Value().Orientation());
}
void copy_split(int kind,int mode,int orientation){
 auto fixture=pcurve_fixture(kind==0?0:2);auto source=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();source.Orientation(TopAbs_Orientation(orientation));TopoDS_Vertex first,last;TopExp::Vertices(source,first,last);TopoDS_Edge output;
 if(mode==0)output=BOPTools_AlgoTools::CopyEdge(source);
 else BOPTools_AlgoTools::MakeSplitEdge(source,mode>=3?TopoDS_Vertex():first,mode==2?.8:.2,mode==4?TopoDS_Vertex():last,mode==2?.2:.8,output);
 std::cout<<kind<<' '<<mode<<' '<<orientation;emit_edge(output,source,first,last);std::cout<<'\n';
}
void section(int kind,int mode,int make){
 auto fixture=pcurve_fixture(kind==0?0:2);double f=mode==1?1.:mode==2?.5:0.,l=mode==1?0.:mode==2?.5:1.;BRep_Builder b;TopoDS_Vertex first,last;b.MakeVertex(first,fixture.curve->Value(f),1e-7);b.MakeVertex(last,fixture.curve->Value(l),1e-7);IntTools_Curve curve;curve.SetCurve(fixture.curve);
 TopoDS_Edge output;int status=0;try{if(make)BOPTools_AlgoTools::MakeEdge(curve,first,f,last,l,1e-4,output);else BOPTools_AlgoTools::MakeSectEdge(curve,first,f,last,l,output);}catch(StdFail_NotDone const&){status=-1;}catch(Standard_Failure const&){status=-2;}
 std::cout<<kind<<' '<<mode<<' '<<make<<' '<<status<<' '<<BRep_Tool::Tolerance(first)<<' '<<BRep_Tool::Tolerance(last)<<' '<<output.IsNull();
 if(!output.IsNull()){double a,z;BRep_Tool::Range(output,a,z);std::cout<<' '<<a<<' '<<z<<' '<<BRep_Tool::Tolerance(output)<<' '<<int(output.Orientation());}std::cout<<'\n';
}
int main(int argc,char**){std::cout<<std::setprecision(17);if(argc>1){for(int kind=0;kind<2;kind++)for(int mode=0;mode<3;mode++)for(int make=0;make<2;make++)section(kind,mode,make);return 0;}for(int kind=0;kind<2;kind++)for(int mode=0;mode<5;mode++)for(int ori=0;ori<4;ori++)copy_split(kind,mode,ori);}
