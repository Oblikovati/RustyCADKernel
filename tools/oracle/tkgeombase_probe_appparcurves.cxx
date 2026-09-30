// AppParCurves requests on the package functions and the containers
// (tools/tkgeombase_corpus_appparcurves.py); the generics are in
// tkgeombase_probe_appparcurves_fit.cxx.

#include <AppParCurves.hxx>
#include <AppParCurves_MultiBSpCurve.hxx>
#include <AppParCurves_MultiCurve.hxx>
#include <AppParCurves_MultiPoint.hxx>
#include <math_IntegerVector.hxx>
void gb_apc_ivector(const math_IntegerVector& v){gb_int(v.Lower());gb_int(v.Upper());for(int i=v.Lower();i<=v.Upper();++i)gb_int(v(i));}
// The Dump text of an object, one token stream with '|' for newlines.
template<class T> void gb_apc_dump(const T& object){
 std::ostringstream text;object.Dump(text);
 for(char c:text.str())std::cout<<(c=='\n'?'|':c);std::cout<<' ';
}
// `apc_bernstein nbpoles lower count u... dda_length`.
void gb_apc_bernstein_case(std::istream& in){
 GbReader r{in};const int nbpoles=r.integer();math_Vector u=gb_fem_read_vector(r);const int ddalen=r.integer();
 try{
  math_Matrix a(u.Lower(),u.Upper(),1,nbpoles,-7.),a2(u.Lower(),u.Upper(),1,nbpoles,-7.),da(u.Lower(),u.Upper(),1,nbpoles,-7.);
  AppParCurves::BernsteinMatrix(nbpoles,u,a);gb_fem_matrix(a);
  AppParCurves::Bernstein(nbpoles,u,a2,da);gb_fem_matrix(a2);gb_fem_matrix(da);
  for(int i=u.Lower();i<=u.Upper();++i){math_Vector dda(1,ddalen,-7.);AppParCurves::SecondDerivativeBernstein(u(i),dda);gb_fem_vector(dda);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `apc_spline nbpoles degree (parameters vector) (flat knots vector)`.
void gb_apc_spline_case(std::istream& in){
 GbReader r{in};const int nbpoles=r.integer(),deg=r.integer();math_Vector u=gb_fem_read_vector(r),flat=gb_fem_read_vector(r);
 try{
  math_Matrix a(u.Lower(),u.Upper(),1,nbpoles,-7.),da(u.Lower(),u.Upper(),1,nbpoles,-7.);math_IntegerVector index(u.Lower(),u.Upper(),-7);
  AppParCurves::SplineFunction(nbpoles,deg,u,flat,a,da,index);gb_fem_matrix(a);gb_fem_matrix(da);gb_apc_ivector(index);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// A MultiPoint: `kind nb3d nb2d points...`; kind 0 (NbPoints, NbPoints2d), 1 tabP,
// 2 tabP2d, 3 (tabP, tabP2d).
AppParCurves_MultiPoint gb_apc_read_point(GbReader& r){
 const int kind=r.integer(),nb3d=r.integer(),nb2d=r.integer();
 NCollection_Array1<gp_Pnt> p(1,std::max(nb3d,1));NCollection_Array1<gp_Pnt2d> q(1,std::max(nb2d,1));
 for(int i=1;i<=nb3d;++i)p(i)=r.pnt();
 for(int i=1;i<=nb2d;++i)q(i)=r.pnt2d();
 if(kind==0){AppParCurves_MultiPoint m(nb3d,nb2d);for(int i=1;i<=nb3d;++i)m.SetPoint(i,p(i));for(int i=1;i<=nb2d;++i)m.SetPoint2d(nb3d+i,q(i));return m;}
 NCollection_Array1<gp_Pnt> p3(1,nb3d);NCollection_Array1<gp_Pnt2d> p2(1,nb2d);
 for(int i=1;i<=nb3d;++i)p3(i)=p(i);
 for(int i=1;i<=nb2d;++i)p2(i)=q(i);
 if(kind==1)return AppParCurves_MultiPoint(p3);
 if(kind==2)return AppParCurves_MultiPoint(p2);
 return AppParCurves_MultiPoint(p3,p2);
}
void gb_apc_point(const AppParCurves_MultiPoint& m){
 gb_int(m.NbPoints());gb_int(m.NbPoints2d());
 for(int i=1;i<=m.NbPoints();++i)gb_pnt(m.Point(i));
 for(int i=m.NbPoints()+1;i<=m.NbPoints()+m.NbPoints2d();++i)gb_pnt2d(m.Point2d(i));
}
// MultiPoint commands: 1 SetPoint, 2 Point, 3 SetPoint2d, 4 Point2d, 5 Dimension,
// 6 Transform, 7 Transform2d, 8 Dump, 9 a copy set then the original read.
void gb_apc_point_command(GbReader& r,AppParCurves_MultiPoint& m){
 const int code=r.integer();
 if(code==1){const int i=r.integer();m.SetPoint(i,r.pnt());}
 else if(code==2)gb_pnt(m.Point(r.integer()));
 else if(code==3){const int i=r.integer();m.SetPoint2d(i,r.pnt2d());}
 else if(code==4)gb_pnt2d(m.Point2d(r.integer()));
 else if(code==5)gb_int(m.Dimension(r.integer()));
 else if(code==6){const int i=r.integer();double c[6];for(double& v:c)v=r.real();m.Transform(i,c[0],c[1],c[2],c[3],c[4],c[5]);}
 else if(code==7){const int i=r.integer();double c[4];for(double& v:c)v=r.real();m.Transform2d(i,c[0],c[1],c[2],c[3]);}
 else if(code==8)gb_apc_dump(m);
 else{AppParCurves_MultiPoint copy=m;const int i=r.integer();copy.SetPoint(i,r.pnt());gb_apc_point(m);}
}
void gb_apc_multipoint_case(std::istream& in){
 GbReader r{in};
 try{
  AppParCurves_MultiPoint m=gb_apc_read_point(r);gb_apc_point(m);const int count=r.integer();
  for(int k=0;k<count;++k){try{gb_apc_point_command(r,m);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
NCollection_Array1<AppParCurves_MultiPoint> gb_apc_read_points(GbReader& r){
 const int n=r.integer();NCollection_Array1<AppParCurves_MultiPoint> poles(1,n);
 for(int i=1;i<=n;++i)poles(i)=gb_apc_read_point(r);return poles;
}
// Every pole of every curve, through Value(j).Point / Point2d.
void gb_apc_curve(const AppParCurves_MultiCurve& c){
 gb_int(c.NbPoles());gb_int(c.NbCurves());
 for(int j=1;j<=c.NbPoles();++j)gb_apc_point(c.Value(j));
}
// Curve commands: 1 Value(k, u), 2 D1, 3 D2, 4 Dimension, 5 Curve, 6 Pole/Pole2d,
// 7 Transform, 8 Transform2d, 9 Dump, 10 Degree, 11 SetValue, 12 SetNbPoles.
void gb_apc_curve_command(GbReader& r,AppParCurves_MultiCurve& c){
 const int code=r.integer();
 if(code<=3){const int k=r.integer();const double u=r.real();
  if(c.Dimension(k)==3){gp_Pnt p;gp_Vec v1,v2;if(code==1)c.Value(k,u,p);else if(code==2)c.D1(k,u,p,v1);else c.D2(k,u,p,v1,v2);gb_pnt(p);gb_vec(v1);gb_vec(v2);}
  else{gp_Pnt2d p;gp_Vec2d v1,v2;if(code==1)c.Value(k,u,p);else if(code==2)c.D1(k,u,p,v1);else c.D2(k,u,p,v1,v2);gb_pnt2d(p);gb_vec2d(v1);gb_vec2d(v2);}}
 else if(code==4)gb_int(c.Dimension(r.integer()));
 else if(code==5){const int k=r.integer();
  if(r.integer()==3){NCollection_Array1<gp_Pnt> t(1,c.NbPoles());c.Curve(k,t);for(const gp_Pnt& p:t)gb_pnt(p);}
  else{NCollection_Array1<gp_Pnt2d> t(1,c.NbPoles());c.Curve(k,t);for(const gp_Pnt2d& p:t)gb_pnt2d(p);}}
 else if(code==6){const int k=r.integer(),n=r.integer();if(r.integer()==3)gb_pnt(c.Pole(k,n));else gb_pnt2d(c.Pole2d(k,n));}
 else if(code==7){const int k=r.integer();double v[6];for(double& x:v)x=r.real();c.Transform(k,v[0],v[1],v[2],v[3],v[4],v[5]);}
 else if(code==8){const int k=r.integer();double v[4];for(double& x:v)x=r.real();c.Transform2d(k,v[0],v[1],v[2],v[3]);}
 else if(code==9)gb_apc_dump(c);
 else if(code==10)gb_int(c.Degree());
 else if(code==11){const int i=r.integer();c.SetValue(i,gb_apc_read_point(r));}
 else c.SetNbPoles(r.integer());
}
void gb_apc_curve_commands(GbReader& r,AppParCurves_MultiCurve& c){
 const int count=r.integer();
 for(int k=0;k<count;++k){try{gb_apc_curve_command(r,c);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
}
// `apc_multicurve kind ...`: kind 0 (NbPol), 1 tabMU, 2 MultiBSpCurve(tabMU, knots,
// mults), 3 MultiBSpCurve(MultiCurve, knots, mults) then SetKnots/SetMultiplicities.
void gb_apc_multicurve_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();
 try{
  if(kind==0){AppParCurves_MultiCurve c(r.integer());gb_int(c.NbPoles());gb_int(c.NbCurves());gb_apc_curve_commands(r,c);}
  else{NCollection_Array1<AppParCurves_MultiPoint> poles=gb_apc_read_points(r);
   if(kind==1){AppParCurves_MultiCurve c(poles);gb_apc_curve(c);gb_apc_curve_commands(r,c);}
   else{math_Vector knots=gb_fem_read_vector(r);const int nm=r.integer();NCollection_Array1<int> mults(1,nm);for(int& m:mults)m=r.integer();
    NCollection_Array1<double> k(knots.Lower(),knots.Upper());for(int i=knots.Lower();i<=knots.Upper();++i)k(i)=knots(i);
    AppParCurves_MultiBSpCurve c=kind==2?AppParCurves_MultiBSpCurve(poles,k,mults):AppParCurves_MultiBSpCurve(AppParCurves_MultiCurve(poles),k,mults);
    gb_apc_curve(c);gb_int(c.Degree());gb_fem_array1(c.Knots());for(int m:c.Multiplicities())gb_int(m);
    if(kind==3){c.SetKnots(k);c.SetMultiplicities(mults);gb_int(c.Degree());}
    gb_apc_curve_commands(r,c);}}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
