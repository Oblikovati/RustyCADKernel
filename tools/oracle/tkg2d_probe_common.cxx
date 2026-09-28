// Shared reader, printers and curve builders of the TKG2d oracle (tkg2d_probe.cxx).
// Every double is printed as raw IEEE bits, every raised exception as its type name.
#define G2_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& g2_error){std::cout<<g2_error.ExceptionType()<<' ';}
uint64_t bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void g2_real(double v){std::cout<<bits(v)<<' ';}
void g2_int(int v){std::cout<<v<<' ';}
void g2_flag(bool v){std::cout<<(v?1:0)<<' ';}
void g2_xy(const gp_XY& v){g2_real(v.X());g2_real(v.Y());}
void g2_pnt(const gp_Pnt2d& p){g2_xy(p.XY());}
void g2_vec(const gp_Vec2d& v){g2_xy(v.XY());}
void g2_dir(const gp_Dir2d& d){g2_xy(d.XY());}
void g2_ax(const gp_Ax2d& a){g2_pnt(a.Location());g2_dir(a.Direction());}
void g2_ax22(const gp_Ax22d& a){g2_pnt(a.Location());g2_dir(a.XDirection());g2_dir(a.YDirection());}
void g2_dump(const occ::handle<Geom2d_Geometry>& g,int depth){std::ostringstream s;g->DumpJson(s,depth);std::cout<<'{'<<s.str()<<"} ";}
struct G2Reader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 bool flag(){return integer()!=0;}
 gp_Pnt2d pnt(){double x=real();return gp_Pnt2d(x,real());}
 gp_Vec2d vec(){double x=real();return gp_Vec2d(x,real());}
 gp_Dir2d dir(){double x=real();return gp_Dir2d(x,real());}
 gp_Ax2d ax(){gp_Pnt2d p=pnt();return gp_Ax2d(p,dir());}
 std::vector<double> reals(){int n=integer();std::vector<double> v(n);for(double& x:v)x=real();return v;}
};
// One transformation: 0 mirror point, 1 mirror axis, 2 rotation, 3 scale, 4 translation,
// 5 translation between points, 6 rotation times scale.
gp_Trsf2d g2_trsf(G2Reader& r){
 int kind=r.integer();double p[5];for(double& v:p)v=r.real();gp_Trsf2d t;gp_Pnt2d a(p[0],p[1]);
 if(kind==0)t.SetMirror(a);
 if(kind==1)t.SetMirror(gp_Ax2d(a,gp_Dir2d(p[2],p[3])));
 if(kind==2)t.SetRotation(a,p[2]);
 if(kind==3)t.SetScale(a,p[2]);
 if(kind==4)t.SetTranslation(gp_Vec2d(p[0],p[1]));
 if(kind==5)t.SetTranslation(a,gp_Pnt2d(p[2],p[3]));
 if(kind==6){gp_Trsf2d s;s.SetScale(gp_Pnt2d(p[2],p[3]),p[4]);t.SetRotation(a,p[2]);t=t.Multiplied(s);}
 return t;
}
// A curve specification, read recursively:
// 0 line P D | 1 circle P XD sense R | 2 ellipse P XD sense A B | 3 hyperbola P XD sense A B |
// 4 parabola P XD sense F | 5 bezier rational n (x y w)*n | 6 bspline rational periodic degree
// nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d notcheck <curve> |
// 9 circle on a gp_Ax22d P XD YD R | 10 parabola from directrix D F.
occ::handle<Geom2d_Curve> g2_curve(G2Reader& r);
occ::handle<Geom2d_Curve> g2_bezier(G2Reader& r){
 bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);
 for(int i=1;i<=n;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom2d_BezierCurve(p,w);return new Geom2d_BezierCurve(p);
}
occ::handle<Geom2d_Curve> g2_bspline(G2Reader& r){
 bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
 for(int i=1;i<=np;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom2d_BSplineCurve(p,w,k,m,degree,periodic);return new Geom2d_BSplineCurve(p,k,m,degree,periodic);
}
occ::handle<Geom2d_Curve> g2_curve(G2Reader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt();return new Geom2d_Line(p,r.dir());}
 if(kind>=1&&kind<=4){gp_Ax2d a=r.ax();bool sense=r.flag();double x=r.real();
  if(kind==1)return new Geom2d_Circle(a,x,sense);
  if(kind==4)return new Geom2d_Parabola(a,x,sense);
  double y=r.real();if(kind==2)return new Geom2d_Ellipse(a,x,y,sense);return new Geom2d_Hyperbola(a,x,y,sense);}
 if(kind==5)return g2_bezier(r);
 if(kind==6)return g2_bspline(r);
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom2d_Curve> c=g2_curve(r);return new Geom2d_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();bool nocheck=r.flag();occ::handle<Geom2d_Curve> c=g2_curve(r);return new Geom2d_OffsetCurve(c,d,nocheck);}
 if(kind==9){gp_Pnt2d p=r.pnt();gp_Dir2d x=r.dir();gp_Dir2d y=r.dir();return new Geom2d_Circle(gp_Ax22d(p,x,y),r.real());}
 if(kind==10){gp_Ax2d d=r.ax();return new Geom2d_Parabola(d,r.pnt());}
 throw Standard_ProgramError("unknown curve kind");
}
// The curve-level summary: range, closure, periodicity, continuity, then per parameter
// EvalD0..D3, EvalDN(0..5), Value and ReversedParameter.
void g2_summary(const occ::handle<Geom2d_Curve>& c,const std::vector<double>& params){
 g2_real(c->FirstParameter());g2_real(c->LastParameter());G2_TRY(g2_flag(c->IsClosed()));g2_flag(c->IsPeriodic());
 G2_TRY(g2_real(c->Period()));g2_int(c->Continuity());for(int n=-1;n<=4;++n)G2_TRY(g2_flag(c->IsCN(n)));
 for(double u:params){
  G2_TRY(g2_pnt(c->EvalD0(u)));
  G2_TRY(auto r=c->EvalD1(u);g2_pnt(r.Point);g2_vec(r.D1));
  G2_TRY(auto r=c->EvalD2(u);g2_pnt(r.Point);g2_vec(r.D1);g2_vec(r.D2));
  G2_TRY(auto r=c->EvalD3(u);g2_pnt(r.Point);g2_vec(r.D1);g2_vec(r.D2);g2_vec(r.D3));
  for(int n=0;n<=5;++n)G2_TRY(g2_vec(c->EvalDN(u,n)));
  G2_TRY(g2_pnt(c->Value(u)));g2_real(c->ReversedParameter(u));
 }
}
