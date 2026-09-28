// Shared reader, printers, transformation, curve and surface builders and summaries of
// the TKG3d oracle (tkg3d_probe.cxx). Every double is printed as raw IEEE bits, every
// raised exception as its type name.
#define G3_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& g3_error){std::cout<<g3_error.ExceptionType()<<' ';}
uint64_t g3_bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void g3_real(double v){std::cout<<g3_bits(v)<<' ';}
void g3_int(int v){std::cout<<v<<' ';}
void g3_flag(bool v){std::cout<<(v?1:0)<<' ';}
void g3_xyz(const gp_XYZ& v){g3_real(v.X());g3_real(v.Y());g3_real(v.Z());}
void g3_pnt(const gp_Pnt& p){g3_xyz(p.XYZ());}
void g3_vec(const gp_Vec& v){g3_xyz(v.XYZ());}
void g3_dir(const gp_Dir& d){g3_xyz(d.XYZ());}
void g3_ax1(const gp_Ax1& a){g3_pnt(a.Location());g3_dir(a.Direction());}
void g3_ax2(const gp_Ax2& a){g3_pnt(a.Location());g3_dir(a.Direction());g3_dir(a.XDirection());g3_dir(a.YDirection());}
void g3_ax3(const gp_Ax3& a){g3_pnt(a.Location());g3_dir(a.Direction());g3_dir(a.XDirection());g3_dir(a.YDirection());}
void g3_name(const occ::handle<Standard_Transient>& t){std::cout<<(t.IsNull()?"null":t->DynamicType()->Name())<<' ';}
void g3_dump(const occ::handle<Geom_Geometry>& g,int depth){std::ostringstream s;g->DumpJson(s,depth);std::cout<<'{'<<s.str()<<"} ";}
void g3_gtrsf2d(const gp_GTrsf2d& t){for(int i=1;i<=2;++i)for(int j=1;j<=3;++j)g3_real(t.Value(i,j));g3_int(t.Form());}
struct G3Reader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 bool flag(){return integer()!=0;}
 gp_Pnt pnt(){double x=real(),y=real();return gp_Pnt(x,y,real());}
 gp_Vec vec(){double x=real(),y=real();return gp_Vec(x,y,real());}
 gp_Dir dir(){double x=real(),y=real();return gp_Dir(x,y,real());}
 gp_Ax1 ax1(){gp_Pnt p=pnt();return gp_Ax1(p,dir());}
 gp_Ax2 ax2(){gp_Pnt p=pnt();gp_Dir n=dir();return gp_Ax2(p,n,dir());}
 gp_Ax3 ax3(){gp_Pnt p=pnt();gp_Dir n=dir();return gp_Ax3(p,n,dir());}
 std::vector<double> reals(){int n=integer();std::vector<double> v(n);for(double& x:v)x=real();return v;}
};
// One transformation: 0 mirror point P, 1 mirror axis P D, 2 mirror plane P N X,
// 3 rotation P D angle, 4 scale P factor, 5 translation V, 6 translation P Q,
// 7 rotation P D angle times scale P factor. Nine numbers follow the kind in every case.
gp_Trsf g3_trsf(G3Reader& r){
 int kind=r.integer();double p[9];for(double& v:p)v=r.real();gp_Trsf t;gp_Pnt a(p[0],p[1],p[2]);
 if(kind==0)t.SetMirror(a);
 if(kind==1)t.SetMirror(gp_Ax1(a,gp_Dir(p[3],p[4],p[5])));
 if(kind==2)t.SetMirror(gp_Ax2(a,gp_Dir(p[3],p[4],p[5]),gp_Dir(p[6],p[7],p[8])));
 if(kind==3)t.SetRotation(gp_Ax1(a,gp_Dir(p[3],p[4],p[5])),p[6]);
 if(kind==4)t.SetScale(a,p[3]);
 if(kind==5)t.SetTranslation(gp_Vec(p[0],p[1],p[2]));
 if(kind==6)t.SetTranslation(a,gp_Pnt(p[3],p[4],p[5]));
 if(kind==7){gp_Trsf s;s.SetScale(a,p[7]);t.SetRotation(gp_Ax1(a,gp_Dir(p[3],p[4],p[5])),p[6]);t=t.Multiplied(s);}
 return t;
}
// A curve specification, read recursively:
// 0 line P D | 1 circle F R | 2 ellipse F A B | 3 hyperbola F A B | 4 parabola F focal |
// 5 bezier rational n (x y z w)*n | 6 bspline rational periodic degree nk (k m)*nk np (x y z w)*np |
// 7 trimmed u1 u2 sense adjust <curve> | 8 offset d D notcheck <curve>, where F is a gp_Ax2 (P N X).
occ::handle<Geom_Curve> g3_curve(G3Reader& r);
occ::handle<Geom_Curve> g3_bezier_curve(G3Reader& r){
 bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> w(1,n);
 for(int i=1;i<=n;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom_BezierCurve(p,w);return new Geom_BezierCurve(p);
}
occ::handle<Geom_Curve> g3_bspline_curve(G3Reader& r){
 bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 int np=r.integer();NCollection_Array1<gp_Pnt> p(1,np);NCollection_Array1<double> w(1,np);
 for(int i=1;i<=np;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom_BSplineCurve(p,w,k,m,degree,periodic);return new Geom_BSplineCurve(p,k,m,degree,periodic);
}
occ::handle<Geom_Curve> g3_curve(G3Reader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind>=1&&kind<=4){gp_Ax2 f=r.ax2();double x=r.real();
  if(kind==1)return new Geom_Circle(f,x);
  if(kind==4)return new Geom_Parabola(f,x);
  double y=r.real();if(kind==2)return new Geom_Ellipse(f,x,y);return new Geom_Hyperbola(f,x,y);}
 if(kind==5)return g3_bezier_curve(r);
 if(kind==6)return g3_bspline_curve(r);
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom_Curve> c=g3_curve(r);return new Geom_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();gp_Dir v=r.dir();bool nocheck=r.flag();occ::handle<Geom_Curve> c=g3_curve(r);return new Geom_OffsetCurve(c,d,v,nocheck);}
 throw Standard_ProgramError("unknown curve kind");
}
// A surface specification, read recursively:
// 0 plane F | 1 cylinder F R | 2 cone F angle R | 3 sphere F R | 4 torus F R r |
// 5 extrusion D <curve> | 6 revolution P D <curve> |
// 7 bezier rational nu nv (x y z w)*(nu*nv) |
// 8 bspline rational uperiodic vperiodic udeg vdeg nuk (k m)* nvk (k m)* nu nv (x y z w)*(nu*nv) |
// 9 rectangular trim u1 u2 v1 v2 usense vsense <surface> | 10 one-direction trim p1 p2 utrim sense <surface> |
// 11 offset d notcheck <surface>, where F is a gp_Ax3 (P N X); poles are read row by row (U major).
occ::handle<Geom_Surface> g3_surface(G3Reader& r);
void g3_net(G3Reader& r,int nu,int nv,NCollection_Array2<gp_Pnt>& p,NCollection_Array2<double>& w){
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j){p(i,j)=r.pnt();w(i,j)=r.real();}
}
occ::handle<Geom_Surface> g3_bezier_surface(G3Reader& r){
 bool rational=r.flag();int nu=r.integer(),nv=r.integer();
 NCollection_Array2<gp_Pnt> p(1,nu,1,nv);NCollection_Array2<double> w(1,nu,1,nv);g3_net(r,nu,nv,p,w);
 if(rational)return new Geom_BezierSurface(p,w);return new Geom_BezierSurface(p);
}
void g3_knots(G3Reader& r,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 for(int i=k.Lower();i<=k.Upper();++i){k(i)=r.real();m(i)=r.integer();}
}
occ::handle<Geom_Surface> g3_bspline_surface(G3Reader& r){
 bool rational=r.flag(),up=r.flag(),vp=r.flag();int ud=r.integer(),vd=r.integer();
 int nuk=r.integer();NCollection_Array1<double> uk(1,nuk);NCollection_Array1<int> um(1,nuk);g3_knots(r,uk,um);
 int nvk=r.integer();NCollection_Array1<double> vk(1,nvk);NCollection_Array1<int> vm(1,nvk);g3_knots(r,vk,vm);
 int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> p(1,nu,1,nv);NCollection_Array2<double> w(1,nu,1,nv);g3_net(r,nu,nv,p,w);
 if(rational)return new Geom_BSplineSurface(p,w,uk,vk,um,vm,ud,vd,up,vp);
 return new Geom_BSplineSurface(p,uk,vk,um,vm,ud,vd,up,vp);
}
occ::handle<Geom_Surface> g3_surface(G3Reader& r){
 int kind=r.integer();
 if(kind>=0&&kind<=4){gp_Ax3 f=r.ax3();
  if(kind==0)return new Geom_Plane(f);
  double x=r.real();
  if(kind==1)return new Geom_CylindricalSurface(f,x);
  if(kind==3)return new Geom_SphericalSurface(f,x);
  double y=r.real();if(kind==2)return new Geom_ConicalSurface(f,x,y);return new Geom_ToroidalSurface(f,x,y);}
 if(kind==5){gp_Dir d=r.dir();occ::handle<Geom_Curve> c=g3_curve(r);return new Geom_SurfaceOfLinearExtrusion(c,d);}
 if(kind==6){gp_Ax1 a=r.ax1();occ::handle<Geom_Curve> c=g3_curve(r);return new Geom_SurfaceOfRevolution(c,a);}
 if(kind==7)return g3_bezier_surface(r);
 if(kind==8)return g3_bspline_surface(r);
 if(kind==9){double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool us=r.flag(),vs=r.flag();
  occ::handle<Geom_Surface> s=g3_surface(r);return new Geom_RectangularTrimmedSurface(s,u1,u2,v1,v2,us,vs);}
 if(kind==10){double p1=r.real(),p2=r.real();bool ut=r.flag(),sense=r.flag();
  occ::handle<Geom_Surface> s=g3_surface(r);return new Geom_RectangularTrimmedSurface(s,p1,p2,ut,sense);}
 if(kind==11){double d=r.real();bool nocheck=r.flag();occ::handle<Geom_Surface> s=g3_surface(r);return new Geom_OffsetSurface(s,d,nocheck);}
 throw Standard_ProgramError("unknown surface kind");
}
// Reads a curve (surface) spec, reporting a construction failure and ending the line.
occ::handle<Geom_Curve> g3_built_curve(G3Reader& r){
 try{return g3_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return nullptr;}
}
occ::handle<Geom_Surface> g3_built_surface(G3Reader& r){
 try{return g3_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return nullptr;}
}
// The curve-level summary: range, closure, periodicity, continuity, then per parameter
// EvalD0..D3, EvalDN(0..5), Value and ReversedParameter.
void g3_curve_summary(const occ::handle<Geom_Curve>& c,const std::vector<double>& params){
 g3_real(c->FirstParameter());g3_real(c->LastParameter());G3_TRY(g3_flag(c->IsClosed()));g3_flag(c->IsPeriodic());
 G3_TRY(g3_real(c->Period()));g3_int(c->Continuity());for(int n=-1;n<=4;++n)G3_TRY(g3_flag(c->IsCN(n)));
 for(double u:params){
  G3_TRY(g3_pnt(c->EvalD0(u)));
  G3_TRY(auto r=c->EvalD1(u);g3_pnt(r.Point);g3_vec(r.D1));
  G3_TRY(auto r=c->EvalD2(u);g3_pnt(r.Point);g3_vec(r.D1);g3_vec(r.D2));
  G3_TRY(auto r=c->EvalD3(u);g3_pnt(r.Point);g3_vec(r.D1);g3_vec(r.D2);g3_vec(r.D3));
  for(int n=0;n<=5;++n)G3_TRY(g3_vec(c->EvalDN(u,n)));
  G3_TRY(g3_pnt(c->Value(u)));g3_real(c->ReversedParameter(u));
 }
}
// The surface-level summary: bounds, closure, periodicity, continuity, then per (u, v)
// EvalD0..D3, EvalDN for 0 <= Nu, Nv <= 3 and a negative order, Value and the reversed parameters.
void g3_surface_summary(const occ::handle<Geom_Surface>& s,const std::vector<double>& uv){
 double u1,u2,v1,v2;s->Bounds(u1,u2,v1,v2);g3_real(u1);g3_real(u2);g3_real(v1);g3_real(v2);
 G3_TRY(g3_flag(s->IsUClosed()));G3_TRY(g3_flag(s->IsVClosed()));g3_flag(s->IsUPeriodic());g3_flag(s->IsVPeriodic());
 G3_TRY(g3_real(s->UPeriod()));G3_TRY(g3_real(s->VPeriod()));g3_int(s->Continuity());
 for(int n=-1;n<=4;++n){G3_TRY(g3_flag(s->IsCNu(n)));G3_TRY(g3_flag(s->IsCNv(n)));}
 for(size_t i=0;i+1<uv.size();i+=2){double u=uv[i],v=uv[i+1];
  G3_TRY(g3_pnt(s->EvalD0(u,v)));
  G3_TRY(auto r=s->EvalD1(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V));
  G3_TRY(auto r=s->EvalD2(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV));
  G3_TRY(auto r=s->EvalD3(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV);
         g3_vec(r.D3U);g3_vec(r.D3V);g3_vec(r.D3UUV);g3_vec(r.D3UVV));
  for(int nu=0;nu<=3;++nu)for(int nv=0;nv<=3;++nv)G3_TRY(g3_vec(s->EvalDN(u,v,nu,nv)));
  G3_TRY(g3_vec(s->EvalDN(u,v,-1,2)));
  G3_TRY(g3_pnt(s->Value(u,v)));g3_real(s->UReversedParameter(u));g3_real(s->VReversedParameter(v));
 }
}
// The iso curves of a surface: for each parameter the class of UIso/VIso and its summary at `params`.
void g3_isos(const occ::handle<Geom_Surface>& s,const std::vector<double>& isos,const std::vector<double>& params){
 for(double p:isos){
  G3_TRY(occ::handle<Geom_Curve> c=s->UIso(p);g3_name(c);g3_curve_summary(c,params));
  G3_TRY(occ::handle<Geom_Curve> c=s->VIso(p);g3_name(c);g3_curve_summary(c,params));
 }
}
