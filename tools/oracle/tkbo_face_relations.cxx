#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
void domain_row(int kind,int mode,int fuzzy,int tol){
 auto first=pcurve_fixture(kind).face;auto second=mode==0?first:pcurve_fixture(kind).face;
 if(mode==2||mode==3){gp_Trsf tr;tr.SetTranslation(mode==2?gp_Vec(0,0,1e-5):gp_Vec(100,0,0));second.Move(TopLoc_Location(tr));}
 BRep_Builder b;for(TopExp_Explorer ex(first,TopAbs_EDGE);ex.More();ex.Next())b.UpdateEdge(TopoDS::Edge(ex.Current()),tol==0?1e-7:1e-3);
 if(mode==4){TopoDS_Face empty;b.MakeFace(empty,BRep_Tool::Surface(first),1e-7);first=empty;}
 Handle(IntTools_Context) context=new IntTools_Context();double fuzz=fuzzy==0?0:fuzzy==1?1e-6:1e-4;
 std::cout<<kind<<' '<<mode<<' '<<fuzzy<<' '<<tol<<' '<<BOPTools_AlgoTools::AreFacesSameDomain(first,second,context,fuzz)<<'\n';
}
void lookup_row(int kind,int foreign,int eori,int fori){
 auto face=pcurve_fixture(kind).face;auto source=foreign?pcurve_fixture(kind).face:face;
 auto edge=TopoDS::Edge(TopExp_Explorer(source,TopAbs_EDGE).Current());edge.Orientation(TopAbs_Orientation(eori));face.Orientation(TopAbs_Orientation(fori));
 TopoDS_Edge initial;BRep_Builder().MakeEdge(initial);auto off=initial,on=initial;
 bool offFound=BOPTools_AlgoTools::GetEdgeOff(edge,face,off);bool onFound=BOPTools_AlgoTools::GetEdgeOnFace(edge,face,on);
 std::cout<<kind<<' '<<foreign<<' '<<eori<<' '<<fori<<' '<<offFound<<' '<<int(off.Orientation())<<' '<<off.IsSame(initial)<<' '<<onFound<<' '<<int(on.Orientation())<<' '<<on.IsSame(initial)<<'\n';
}
int main(int argc,char**){if(argc>1){for(int kind=0;kind<5;kind++)for(int foreign=0;foreign<2;foreign++)for(int eori=0;eori<4;eori++)for(int fori=0;fori<4;fori++)lookup_row(kind,foreign,eori,fori);return 0;}
 for(int kind=0;kind<5;kind++)for(int mode=0;mode<5;mode++)for(int fuzz=0;fuzz<3;fuzz++)for(int tol=0;tol<2;tol++)domain_row(kind,mode,fuzz,tol);
}
