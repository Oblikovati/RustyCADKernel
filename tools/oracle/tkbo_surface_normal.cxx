#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <BRep_Tool.hxx>
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<8;kind++){
  Handle(Geom_Surface) surface;
  if(kind<5)surface=BRep_Tool::Surface(pcurve_fixture(kind).face);
  else{NCollection_Array2<gp_Pnt> poles(1,2,1,2);for(int u=1;u<=2;u++)for(int v=1;v<=2;v++)poles(u,v)=kind==5?gp_Pnt():kind==6?gp_Pnt(u*1e-9,v*1e-9,0.):gp_Pnt(u+v,0,0);surface=new Geom_BezierSurface(poles);}
  gp_Dir normal(0,1,0);int status;try{status=BOPTools_AlgoTools3D::GetNormalToSurface(surface,.3,.2,normal);}catch(const Standard_ConstructionError&){status=-1;}
  std::cout<<"normal "<<kind<<' '<<status<<' '<<normal.X()<<' '<<normal.Y()<<' '<<normal.Z()<<'\n';
 }
 double angles[]={0.,.000199,.000201,1.,M_PI,M_PI-.000199,M_PI-.000201};
 for(int first=0;first<7;first++)for(int second=0;second<7;second++){
  gp_Dir a(cos(angles[first]),sin(angles[first]),0.),b(cos(angles[second]),sin(angles[second]),0.);
  std::cout<<"sense "<<first<<' '<<second<<' '<<BOPTools_AlgoTools3D::SenseFlag(a,b)<<'\n';
 }
}
