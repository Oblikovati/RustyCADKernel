#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Vertex.hxx>
TopoDS_Vertex vertex(gp_Pnt point,double tolerance){TopoDS_Vertex result;BRep_Builder().MakeVertex(result,point,tolerance);return result;}
void emit(const TopoDS_Vertex& result,const TopoDS_Vertex& initial){auto point=BRep_Tool::Pnt(result);std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<' '<<BRep_Tool::Tolerance(result)<<' '<<result.IsSame(initial)<<' '<<int(result.Orientation());}
void construct(int count){
 auto initial=vertex(gp_Pnt(9,8,7),.25);initial.Orientation(TopAbs_REVERSED);auto output=initial;NCollection_List<TopoDS_Shape> vertices;
 for(int i=0;i<count;i++){auto item=vertex(gp_Pnt(i,i%2,-i),.1*(i+1));if(i==0)item.Orientation(TopAbs_REVERSED);vertices.Append(item);}
 BOPTools_AlgoTools::MakeVertex(vertices,output);std::cout<<count;emit(output,initial);std::cout<<'\n';
}
void pair(int kind){
 auto first=vertex(gp_Pnt(-1,2,3),.1),second=vertex(gp_Pnt(4,-2,1),.3),initial=vertex(gp_Pnt(9,8,7),.25);initial.Orientation(TopAbs_REVERSED);auto output=initial;
 auto fixture=pcurve_fixture(kind%5);auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();auto otherFixture=pcurve_fixture((kind+1)%5);auto other=BRepBuilderAPI_MakeEdge(otherFixture.curve,otherFixture.first,otherFixture.last).Edge();
 if(kind<5)BOPTools_AlgoTools::MakeNewVertex(edge,fixture.first,other,otherFixture.last,output);
 else if(kind<10)BOPTools_AlgoTools::MakeNewVertex(edge,fixture.first,fixture.face,output);
 else if(kind==10)BOPTools_AlgoTools::MakeNewVertex(first,second,output);
 else BOPTools_AlgoTools::MakeNewVertex(gp_Pnt(-1,2,3),.1,output);
 std::cout<<kind;emit(output,initial);std::cout<<'\n';
}
int main(int argc,char** argv){std::cout<<std::setprecision(17);
 if(argc>1){if(std::string(argv[1])=="list"){for(int count=0;count<6;count++)construct(count);}else{for(int kind=0;kind<12;kind++)pair(kind);}return 0;}
 for(int tol=0;tol<3;tol++)for(int fuzzy=0;fuzzy<3;fuzzy++)for(int pos=0;pos<4;pos++){
 double t1=tol==0?1e-7:tol==1?.01:1.,t2=2*t1,fuzz=fuzzy==0?-1.:fuzzy==1?0.:.1;double sum=t1+t2+std::max(fuzz,Precision::Confusion());double distance=pos==0?0.:pos==1?sum-1e-12:pos==2?sum:sum+1e-12;
 auto first=vertex(gp_Pnt(),t1),second=vertex(gp_Pnt(distance,0,0),t2);std::cout<<tol<<' '<<fuzzy<<' '<<pos<<' '<<BOPTools_AlgoTools::ComputeVV(first,second,fuzz)<<' '<<BOPTools_AlgoTools::ComputeVV(first,gp_Pnt(distance,0,0),t2)<<' '<<BOPTools_AlgoTools::DTolerance()<<'\n';
 }
}
