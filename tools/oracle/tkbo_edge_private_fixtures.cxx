#include <Geom_Hyperbola.hxx>
occ::handle<Geom_Curve> ee_geometry_fixture(int kind,double offset){
 if(kind<=6)return bo_tools_geometry(kind,0,1);
 if(kind==7)return new Geom_Ellipse(gp_Ax2(),3,1);
 if(kind==8)return new Geom_Hyperbola(gp_Ax2(),2,1);
 if(kind==9){NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt();poles(2)=gp_Pnt(1,2,0);poles(3)=gp_Pnt(3,0,0);NCollection_Array1<double> knots(1,2);knots(1)=0;knots(2)=1;NCollection_Array1<int> mults(1,2);mults.Init(3);return new Geom_BSplineCurve(poles,knots,mults,2,false);}
 auto basis=ee_geometry_fixture(kind==10?6:kind==11?1:kind==12?7:kind==13?3:9,offset);
 return new Geom_OffsetCurve(basis,offset,gp_Dir(0,0,1));
}
