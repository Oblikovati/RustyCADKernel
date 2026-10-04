#include "tkbo_seam_fixture.hxx"
#include <BRepBuilderAPI_MakeEdge.hxx>
void observe(int kind,int mode,int orientation){
 auto fixture=pcurve_fixture(kind);auto face=fixture.face;
 auto old=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
 if(mode!=3)BOPTools_AlgoTools2D::BuildPCurveForEdgeOnFace(old,face);
 Handle(Geom_Curve) curve=occ::down_cast<Geom_Curve>(fixture.curve->Copy());
 double first=fixture.first,last=fixture.last;
 if(mode==1){curve->Reverse();first=curve->ReversedParameter(fixture.last);last=curve->ReversedParameter(fixture.first);}
 if(mode==2)curve->Translate(gp_Vec(1,2,3));
 if(mode==4)curve->Translate(gp_Vec(0,0,1e-4));
 auto edge=BRepBuilderAPI_MakeEdge(curve,first,last).Edge();
 old.Orientation(TopAbs_Orientation(orientation));edge.Orientation(TopAbs_Orientation(orientation));face.Orientation(TopAbs_Orientation(orientation));
 Handle(IntTools_Context) context=new IntTools_Context();
 std::cout<<kind<<' '<<mode<<' '<<orientation;
 try{
 int status=BOPTools_AlgoTools2D::AttachExistingPCurve(old,edge,face,context);
 double f,l;auto stored=BRep_Tool::CurveOnSurface(edge,face,f,l);
 std::cout<<' '<<status<<' '<<stored.IsNull()<<' '<<BRep_Tool::Tolerance(edge)<<' '<<BRep_Tool::SameParameter(edge)<<' '<<BRep_Tool::SameRange(edge);
 if(!stored.IsNull()){auto point=stored->Value(.5*(f+l));std::cout<<' '<<f<<' '<<l<<' '<<point.X()<<' '<<point.Y();}
 double vertexTol=0;for(TopExp_Explorer ex(edge,TopAbs_VERTEX);ex.More();ex.Next())vertexTol=std::max(vertexTol,BRep_Tool::Tolerance(TopoDS::Vertex(ex.Current())));std::cout<<' '<<vertexTol;
 }catch(Standard_Failure const& error){std::cout<<" failure "<<error.what();}
 std::cout<<'\n';
}
void observe_seam(int kind,int orientation){
 auto face=pcurve_fixture(kind).face;TopoDS_Edge old;
 for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next()) {auto edge=TopoDS::Edge(ex.Current());if(BRep_Tool::IsClosed(edge,face)){old=edge;break;}}
 double first,last;auto curve=BRep_Tool::Curve(old,first,last);TopoDS_Edge edge;
 BRep_Builder builder;builder.MakeEdge(edge,curve,1e-7);builder.Range(edge,first,last);
 old.Orientation(TopAbs_Orientation(orientation));edge.Orientation(TopAbs_Orientation(orientation));face.Orientation(TopAbs_Orientation(orientation));
 Handle(IntTools_Context) context=new IntTools_Context();
 int status=BOPTools_AlgoTools2D::AttachExistingPCurve(old,edge,face,context);
 std::cout<<kind<<' '<<orientation<<' '<<status<<' '<<BRep_Tool::IsClosed(edge,face);
 face.Orientation(TopAbs_FORWARD);
 for(int side=0;side<2;side++) {edge.Orientation(TopAbs_Orientation(side));auto pc=BRep_Tool::CurveOnSurface(edge,face,first,last);auto point=pc->Value(.5*(first+last));std::cout<<' '<<first<<' '<<last<<' '<<point.X()<<' '<<point.Y();}
 std::cout<<'\n';
}
int main(int argc,char**){if(argc>1){std::cout<<std::setprecision(17);for(int kind:{1,2})for(int ori=0;ori<4;ori++)observe_seam(kind,ori);return 0;}std::cout<<std::setprecision(17);for(int kind=0;kind<5;kind++)for(int mode=0;mode<5;mode++)for(int ori=0;ori<4;ori++)observe(kind,mode,ori);}
