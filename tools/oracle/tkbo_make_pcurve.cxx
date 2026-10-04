#include "tkbo_pcurve_fixture.hxx"
int main(){
 std::cout<<std::setprecision(17);double tolerances[]={0.,1e-7,1e-5,1e-3,2.};
 for(int kind=0;kind<5;kind++){
  auto fixture=pcurve_fixture(kind);auto face=fixture.face;auto curve=fixture.curve;double first=fixture.first,last=fixture.last;
  for(int tolerance=0;tolerance<5;tolerance++)for(int cached=0;cached<2;cached++)for(int full=0;full<2;full++){
   Handle(IntTools_Context) context;if(cached)context=new IntTools_Context();Handle(Geom2d_Curve) result;double reached=tolerances[tolerance];
   try{if(full){Handle(Geom_Curve) trimmed=new Geom_TrimmedCurve(curve,first,last,true,false);BOPTools_AlgoTools2D::MakePCurveOnFace(face,trimmed,result,reached,context);}else BOPTools_AlgoTools2D::MakePCurveOnFace(face,curve,first,last,result,reached,context);std::cout<<kind<<' '<<tolerance<<' '<<cached<<' '<<full<<" ok "<<reached<<' '<<result->FirstParameter()<<' '<<result->LastParameter();for(double t:{first,.5*(first+last),last}){auto point=result->Value(t);std::cout<<' '<<point.X()<<' '<<point.Y();}std::cout<<'\n';}
   catch(const Standard_Failure& failure){std::cout<<kind<<' '<<tolerance<<' '<<cached<<' '<<full<<' '<<failure.what()<<'\n';}
  }
 }
}
