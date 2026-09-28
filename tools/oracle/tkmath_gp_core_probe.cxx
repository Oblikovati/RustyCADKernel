// gp_Vec, gp_Dir, gp_Pnt, gp_Lin, gp_Pln, gp_Ax2/gp_Ax3 members and the NCollection_Lerp /
// gp_TrsfNLerp interpolators at pinned OCCT 3d097a0. Every value is printed as raw IEEE bits,
// every raised exception as its type name. Transformations reuse gp3_trsf and gp3_trsf_state
// from tkmath_gp_transform_probe.cxx, which is included first.
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
#include <gp_TrsfNLerp.hxx>
#include <NCollection_Lerp.hxx>
#include <functional>
#define GPC_TRY(statement) try{statement;}catch(const Standard_Failure& gpc_error){std::cout<<gpc_error.ExceptionType()<<' ';}
void gpc_flag(bool v){std::cout<<v<<' ';}
void gpc_pnt(const gp_Pnt& p){xyz(p.XYZ());}
void gpc_dir(const gp_Dir& d){xyz(d.XYZ());}
void gpc_vec(const gp_Vec& v){xyz(v.XYZ());}
void gpc_ax1(const gp_Ax1& a){gpc_pnt(a.Location());gpc_dir(a.Direction());}
void gpc_ax2(const gp_Ax2& a){gpc_pnt(a.Location());gpc_dir(a.Direction());gpc_dir(a.XDirection());gpc_dir(a.YDirection());}
void gpc_ax3(const gp_Ax3& a){gpc_pnt(a.Location());gpc_dir(a.Direction());gpc_dir(a.XDirection());gpc_dir(a.YDirection());}
void gpc_lin(const gp_Lin& l){gpc_ax1(l.Position());}
void gpc_pln(const gp_Pln& p){gpc_ax3(p.Position());}
template<class T> void gpc_dump(const T& value){std::ostringstream s;value.DumpJson(s);std::cout<<s.str()<<' ';}
// Dump, then parse the dump into a default value: flag, cursor and the parsed value.
template<class T,class P> void gpc_json(const T& value,P print){
 std::ostringstream s;value.DumpJson(s);std::cout<<s.str()<<' ';Standard_SStream stream(s.str());T parsed;int pos=1;
 GPC_TRY(gpc_flag(parsed.InitFromJson(stream,pos));std::cout<<pos<<' ';print(parsed));
}

void gpc_vec_access(const gp_Vec& a,const gp_Vec& b,const gp_Vec& c,double s,double t,int idx){
 gp_Vec v=a;GPC_TRY(v.SetCoord(idx,s));gpc_vec(v);GPC_TRY(gp3_real(a.Coord(idx)));
 v.SetCoord(s,t,s);gpc_vec(v);v.SetX(t);v.SetY(s);v.SetZ(t);gpc_vec(v);v.SetXYZ(b.XYZ());gpc_vec(v);
 double x=91,y=91,z=91;a.Coord(x,y,z);gp3_real(x);gp3_real(y);gp3_real(z);
 gpc_vec(gp_Vec(gp_Pnt(a.XYZ()),gp_Pnt(b.XYZ())));GPC_TRY(gpc_vec(gp_Vec(gp_Dir(c.XYZ()))));
}
void gpc_vec_metrics(const gp_Vec& a,const gp_Vec& b,const gp_Vec& c,double s,double t){
 gp3_real(a.Magnitude());gp3_real(a.SquareMagnitude());
 GPC_TRY(gpc_flag(a.IsEqual(b,s,t)));GPC_TRY(gpc_flag(a.IsNormal(b,t)));
 GPC_TRY(gpc_flag(a.IsOpposite(b,t)));GPC_TRY(gpc_flag(a.IsParallel(b,t)));
 GPC_TRY(gp3_real(a.Angle(b)));GPC_TRY(gp3_real(a.AngleWithRef(b,c)));
 GPC_TRY(gpc_vec(a.Normalized()));gp_Vec v=a;GPC_TRY(v.Normalize());gpc_vec(v);
}
void gpc_vec_algebra(const gp_Vec& a,const gp_Vec& b,const gp_Vec& c,const gp_Vec& d,double s,double t){
 gpc_vec(a+b);gpc_vec(a-b);gpc_vec(a*s);gpc_vec(a/s);gpc_vec(a^b);gp3_real(a*b);gpc_vec(-a);gpc_vec(s*a);
 gp_Vec v=a;v+=b;v-=c;v*=s;v/=t;v^=d;gpc_vec(v);
 v=a;v.Add(b);v.Subtract(c);v.Multiply(s);v.Divide(t);v.Cross(d);gpc_vec(v);
 gpc_vec(a.Added(b));gpc_vec(a.Subtracted(b));gpc_vec(a.Multiplied(s));gpc_vec(a.Divided(s));gpc_vec(a.Crossed(b));
 gp3_real(a.CrossMagnitude(b));gp3_real(a.CrossSquareMagnitude(b));gpc_vec(a.CrossCrossed(b,c));
 v=a;v.CrossCross(b,c);gpc_vec(v);gp3_real(a.Dot(b));gp3_real(a.DotCross(b,c));gpc_vec(a.Reversed());v=a;v.Reverse();gpc_vec(v);
 v.SetLinearForm(s,a,t,b,s,c,d);gpc_vec(v);v.SetLinearForm(s,a,t,b,s,c);gpc_vec(v);v.SetLinearForm(s,a,t,b,c);gpc_vec(v);
 v.SetLinearForm(s,a,t,b);gpc_vec(v);v.SetLinearForm(s,a,b);gpc_vec(v);v.SetLinearForm(a,b);gpc_vec(v);
 gpc_vec(a.Scaled(s));v=a;v.Scale(t);gpc_vec(v);
}
void gpc_vec_motions(const gp_Vec& a,const gp_Vec& b,const gp_Pnt& o,const gp_XYZ& n,double angle,const gp_Trsf& T){
 gp_Ax1 axis(o,gp_Dir(n));gp_Ax2 plane(o,gp_Dir(n));
 gpc_vec(a.Mirrored(b));gpc_vec(a.Mirrored(axis));gpc_vec(a.Mirrored(plane));
 gp_Vec v=a;v.Mirror(b);v.Mirror(axis);v.Mirror(plane);gpc_vec(v);
 gpc_vec(a.Rotated(axis,angle));v=a;v.Rotate(axis,angle);gpc_vec(v);
 gpc_vec(a.Transformed(T));v=a;v.Transform(T);gpc_vec(v);gpc_dump(a);
}
void gpc_vec_case(std::istream& in){
 Gp3Reader r{in};gp_Vec a(r.xyz()),b(r.xyz()),c(r.xyz()),d(r.xyz());double s=r.real(),t=r.real();int idx=r.integer();
 gp_Pnt o(r.xyz());gp_XYZ n=r.xyz();double angle=r.real();gp_Trsf T;GPC_TRY(T=gp3_trsf(r));
 gpc_vec_access(a,b,c,s,t,idx);gpc_vec_metrics(a,b,c,s,t);gpc_vec_algebra(a,b,c,d,s,t);
 GPC_TRY(gpc_vec_motions(a,b,o,n,angle,T));std::cout<<'\n';
}

void gpc_dir_setters(const gp_Dir& d,const gp_XYZ& w,int idx,double s){
 gp_Dir e=d;GPC_TRY(e.SetCoord(idx,s));gpc_dir(e);e=d;GPC_TRY(e.SetCoord(w.X(),w.Y(),w.Z()));gpc_dir(e);
 e=d;GPC_TRY(e.SetX(s));gpc_dir(e);e=d;GPC_TRY(e.SetY(s));gpc_dir(e);e=d;GPC_TRY(e.SetZ(s));gpc_dir(e);
 e=d;GPC_TRY(e.SetXYZ(w));gpc_dir(e);GPC_TRY(gp3_real(d.Coord(idx)));
 double x=91,y=91,z=91;d.Coord(x,y,z);gp3_real(x);gp3_real(y);gp3_real(z);
}
void gpc_dir_products(const gp_Dir& d,const gp_Dir& e,const gp_Dir& f,double s){
 gpc_dir(-d);gp3_real(d*e);gp3_real(d.Dot(e));gp3_real(d.DotCross(e,f));gpc_flag(d.IsEqual(e,s));
 GPC_TRY(gpc_dir(d^e));gp_Dir g=d;GPC_TRY(g^=e);gpc_dir(g);g=d;GPC_TRY(g.Cross(e));gpc_dir(g);
 GPC_TRY(gpc_dir(d.CrossCrossed(e,f)));g=d;GPC_TRY(g.CrossCross(e,f));gpc_dir(g);
 gp_Dir copy(d);gp_Dir moved(std::move(copy));g=moved;gpc_dir(g);g=gp_Dir(e);gpc_dir(g);
}
void gpc_dir_case(std::istream& in){
 Gp3Reader r{in};gp_XYZ v=r.xyz(),w=r.xyz(),u=r.xyz();int idx=r.integer();double s=r.real();
 gp_Dir d,e,f;GPC_TRY(d=gp_Dir(v));GPC_TRY(e=gp_Dir(gp_Vec(w)));GPC_TRY(f=gp_Dir(u.X(),u.Y(),u.Z()));
 gpc_dir(d);gpc_dir(e);gpc_dir(f);gpc_dir_setters(d,w,idx,s);gpc_dir_products(d,e,f,s);
 gpc_json(d,gpc_dir);std::cout<<'\n';
}

void gpc_pnt_access(const gp_Pnt& p,const gp_XYZ& v,double s,int idx){
 gp_Pnt b=p;GPC_TRY(b.SetCoord(idx,s));gpc_pnt(b);GPC_TRY(gp3_real(p.Coord(idx)));
 b.SetCoord(s,v.X(),v.Y());b.SetX(v.Z());b.SetY(s);b.SetZ(v.X());gpc_pnt(b);b.SetXYZ(v);gpc_pnt(b);
 double x=91,y=91,z=91;p.Coord(x,y,z);gp3_real(x);gp3_real(y);gp3_real(z);xyz(p.Coord());
 b=p;b.ChangeCoord().Add(v);b.ChangeCoord().Multiply(s);gpc_pnt(b);gpc_pnt(gp_Pnt());
}
void gpc_pnt_case(std::istream& in){
 Gp3Reader r{in};gp_Pnt p(r.xyz()),q(r.xyz());gp_XYZ v=r.xyz();double alpha=r.real(),beta=r.real(),s=r.real();int idx=r.integer();
 gp_Trsf T;GPC_TRY(T=gp3_trsf(r));
 gp_Pnt b=p;b.BaryCenter(alpha,q,beta);gpc_pnt(b);gpc_pnt_access(p,v,s,idx);
 gp3_real(p.Distance(q));gp3_real(p.SquareDistance(q));gpc_flag(p.IsEqual(q,s));
 gpc_pnt(p.Translated(gp_Vec(v)));gpc_pnt(p.Translated(q,gp_Pnt(v)));b=p;b.Translate(q,gp_Pnt(v));gpc_pnt(b);b.Translate(gp_Vec(v));gpc_pnt(b);
 gpc_pnt(p.Scaled(q,s));b=p;b.Scale(q,s);gpc_pnt(b);gpc_pnt(p.Mirrored(q));b=p;b.Mirror(q);gpc_pnt(b);
 gpc_pnt(p.Transformed(T));b=p;b.Transform(T);gpc_pnt(b);
 std::cout<<std::hash<gp_Pnt>{}(p)<<' '<<std::hash<gp_Pnt>{}(q)<<' ';
 gpc_flag(std::equal_to<gp_Pnt>{}(p,q));gpc_flag(std::equal_to<gp_Pnt>{}(q,p));
 gpc_json(p,gpc_pnt);std::cout<<'\n';
}

void gpc_lin_motions(const gp_Lin& l,const gp_Pnt& p,const gp_Pnt& q,const gp_Ax1& axis,const gp_Ax2& plane,double angle,double s,const gp_Trsf& T){
 gpc_lin(l.Mirrored(p));gpc_lin(l.Mirrored(axis));gpc_lin(l.Mirrored(plane));
 gp_Lin e=l;e.Mirror(p);e.Mirror(axis);e.Mirror(plane);gpc_lin(e);
 gpc_lin(l.Rotated(axis,angle));e=l;e.Rotate(axis,angle);gpc_lin(e);gpc_lin(l.Scaled(p,s));e=l;e.Scale(p,s);gpc_lin(e);
 gpc_lin(l.Transformed(T));e=l;e.Transform(T);gpc_lin(e);
 gpc_lin(l.Translated(gp_Vec(p.XYZ())));e=l;e.Translate(gp_Vec(q.XYZ()));gpc_lin(e);
 gpc_lin(l.Translated(p,q));e=l;e.Translate(p,q);gpc_lin(e);
}
void gpc_lin_members(const gp_Pnt& o,const gp_Pnt& p,const gp_Pnt& q,const gp_XYZ& n,const gp_XYZ& m,double angle,double s,const gp_Trsf& T){
 gp_Lin l(o,gp_Dir(n)),l2(gp_Ax1(q,gp_Dir(m)));gpc_lin(gp_Lin());gpc_lin(l);gpc_lin(l2);
 gpc_pnt(l.Location());gpc_dir(l.Direction());
 gp3_real(l.Distance(p));gp3_real(l.SquareDistance(p));GPC_TRY(gp3_real(l.Distance(l2)));GPC_TRY(gp3_real(l.SquareDistance(l2)));
 gpc_flag(l.Contains(p,s));gp3_real(l.Angle(l2));GPC_TRY(gpc_lin(l.Normal(p)));
 gpc_lin(l.Reversed());gp_Lin e=l;e.Reverse();e.SetLocation(q);e.SetDirection(gp_Dir(m));gpc_lin(e);e.SetPosition(l.Position());gpc_lin(e);
 gpc_lin_motions(l,p,q,gp_Ax1(q,gp_Dir(m)),gp_Ax2(q,gp_Dir(m)),angle,s,T);
}
void gpc_lin_case(std::istream& in){
 Gp3Reader r{in};gp_Pnt o(r.xyz()),p(r.xyz()),q(r.xyz());gp_XYZ n=r.xyz(),m=r.xyz();double angle=r.real(),s=r.real();
 gp_Trsf T;GPC_TRY(T=gp3_trsf(r));GPC_TRY(gpc_lin_members(o,p,q,n,m,angle,s,T));std::cout<<'\n';
}

void gpc_pln_metrics(const gp_Pln& pl,const gp_Pnt& p,const gp_Lin& line,const gp_Pln& other,double lin,double ang){
 gp3_real(pl.Distance(p));gp3_real(pl.SignedDistance(p));gp3_real(pl.SquareDistance(p));
 gp3_real(pl.Distance(line));gp3_real(pl.SignedDistance(line));gp3_real(pl.SquareDistance(line));
 gp3_real(pl.Distance(other));gp3_real(pl.SignedDistance(other));gp3_real(pl.SquareDistance(other));
 gpc_flag(pl.Contains(p,lin));gpc_flag(pl.Contains(line,lin,ang));
 gpc_ax1(pl.Axis());gpc_ax1(pl.XAxis());gpc_ax1(pl.YAxis());gpc_flag(pl.Direct());gpc_pnt(pl.Location());
 double a=91,b=91,c=91,d=91;pl.Coefficients(a,b,c,d);gp3_real(a);gp3_real(b);gp3_real(c);gp3_real(d);
}
void gpc_pln_edits(const gp_Pln& pl,const gp_Pnt& p,const gp_Ax1& axis,const gp_Pln& other){
 gp_Pln e=pl;e.UReverse();gpc_flag(e.Direct());double a=91,b=91,c=91,d=91;e.Coefficients(a,b,c,d);
 gp3_real(a);gp3_real(b);gp3_real(c);gp3_real(d);e.VReverse();gpc_pln(e);
 e=pl;GPC_TRY(e.SetAxis(axis));gpc_pln(e);e.SetLocation(p);gpc_pln(e);e.SetPosition(other.Position());gpc_pln(e);
}
void gpc_pln_motions(const gp_Pln& pl,const gp_Pnt& p,const gp_Pnt& q,const gp_Ax1& axis,const gp_Ax2& plane,double angle,double s,const gp_Trsf& T){
 gpc_pln(pl.Mirrored(p));gpc_pln(pl.Mirrored(axis));gpc_pln(pl.Mirrored(plane));
 gp_Pln e=pl;e.Mirror(p);e.Mirror(axis);e.Mirror(plane);gpc_pln(e);
 gpc_pln(pl.Rotated(axis,angle));e=pl;e.Rotate(axis,angle);gpc_pln(e);gpc_pln(pl.Scaled(p,s));e=pl;e.Scale(p,s);gpc_pln(e);
 gpc_pln(pl.Transformed(T));e=pl;e.Transform(T);gpc_pln(e);
 gpc_pln(pl.Translated(gp_Vec(q.XYZ())));e=pl;e.Translate(gp_Vec(p.XYZ()));gpc_pln(e);
 gpc_pln(pl.Translated(p,q));e=pl;e.Translate(p,q);gpc_pln(e);
 std::ostringstream full,none;pl.DumpJson(full);pl.DumpJson(none,0);std::cout<<full.str()<<" | "<<none.str()<<" | ";
}
void gpc_pln_case(std::istream& in){
 Gp3Reader r{in};gp_Pnt o(r.xyz()),p(r.xyz()),q(r.xyz());gp_XYZ n=r.xyz(),m=r.xyz();
 double A=r.real(),B=r.real(),C=r.real(),D=r.real(),angle=r.real(),s=r.real(),lin=r.real(),ang=r.real();
 gp_Trsf T;GPC_TRY(T=gp3_trsf(r));GPC_TRY(gpc_pln(gp_Pln(A,B,C,D)));
 GPC_TRY(
  gp_Pln pl(o,gp_Dir(n));gp_Ax1 axis(q,gp_Dir(m));gp_Pln other(q,gp_Dir(m));
  gpc_pln(gp_Pln());gpc_pln(pl);gpc_pln(gp_Pln(pl.Position()));
  gpc_pln_metrics(pl,p,gp_Lin(axis),other,lin,ang);gpc_pln_edits(pl,p,axis,other);
  gpc_pln_motions(pl,p,q,axis,gp_Ax2(q,gp_Dir(m)),angle,s,T));
 std::cout<<'\n';
}

void gpc_ax2_members(const gp_Pnt& o,const gp_Pnt& p,const gp_XYZ& n,const gp_XYZ& x,const gp_XYZ& m,double angle,double s,const gp_Trsf& T){
 GPC_TRY(gpc_ax2(gp_Ax2(o,gp_Dir(n),gp_Dir(x))));
 gp_Ax2 a(o,gp_Dir(n));gpc_ax2(a);gpc_pnt(a.Location());gpc_dir(a.Direction());gpc_dir(a.XDirection());gpc_dir(a.YDirection());
 gp_Ax1 axis(p,gp_Dir(m));gpc_ax2(a.Mirrored(p));gp_Ax2 e=a;e.Mirror(p);gpc_ax2(e);
 GPC_TRY(gpc_ax2(a.Rotated(axis,angle)));e=a;GPC_TRY(e.Rotate(axis,angle));gpc_ax2(e);
 gpc_ax2(a.Scaled(p,s));e=a;e.Scale(p,s);gpc_ax2(e);
 GPC_TRY(gpc_ax2(a.Transformed(T)));e=a;GPC_TRY(e.Transform(T));gpc_ax2(e);
 gpc_ax2(a.Translated(gp_Vec(m)));e=a;e.Translate(gp_Vec(m));gpc_ax2(e);gpc_json(a,gpc_ax2);
}
void gpc_ax3_edits(const gp_Ax3& a,const gp_Pnt& p,const gp_XYZ& m){
 gp_Ax3 e=a;e.XReverse();gpc_ax3(e);GPC_TRY(gpc_flag(e.Direct()));e.YReverse();e.ZReverse();gpc_ax3(e);GPC_TRY(gpc_flag(e.Direct()));
 e=a;GPC_TRY(e.SetAxis(gp_Ax1(p,gp_Dir(m))));gpc_ax3(e);e=a;GPC_TRY(e.SetDirection(gp_Dir(m)));gpc_ax3(e);
 e=a;GPC_TRY(e.SetXDirection(gp_Dir(m)));gpc_ax3(e);e=a;GPC_TRY(e.SetYDirection(gp_Dir(m)));gpc_ax3(e);e=a;e.SetLocation(p);gpc_ax3(e);
}
void gpc_ax3_members(const gp_Pnt& o,const gp_Pnt& p,const gp_XYZ& n,const gp_XYZ& x,const gp_XYZ& m,double angle,double s,const gp_Trsf& T){
 GPC_TRY(gpc_ax3(gp_Ax3(o,gp_Dir(n),gp_Dir(x))));
 gp_Ax3 a(o,gp_Dir(n));gpc_ax3(a);gpc_ax3(gp_Ax3(gp_Ax2(o,gp_Dir(n))));gpc_pnt(a.Location());gpc_dir(a.Direction());
 gpc_dir(a.XDirection());gpc_dir(a.YDirection());gpc_flag(a.Direct());gpc_ax3_edits(a,p,m);
 gp_Ax1 axis(p,gp_Dir(m));gp_Ax2 plane(p,gp_Dir(m));
 gpc_ax3(a.Mirrored(plane));gp_Ax3 e=a;e.Mirror(plane);gpc_ax3(e);gpc_ax3(a.Mirrored(p));gpc_ax3(a.Mirrored(axis));
 gpc_ax3(a.Rotated(axis,angle));e=a;e.Rotate(axis,angle);gpc_ax3(e);gpc_ax3(a.Scaled(p,s));e=a;e.Scale(p,s);gpc_ax3(e);
 gpc_ax3(a.Transformed(T));e=a;e.Transform(T);gpc_ax3(e);
 gpc_ax3(a.Translated(gp_Vec(m)));e=a;e.Translate(gp_Vec(m));gpc_ax3(e);gpc_ax3(a.Translated(o,p));e=a;e.Translate(o,p);gpc_ax3(e);
 GPC_TRY(gpc_ax2(a.Ax2()));gpc_json(a,gpc_ax3);
}
void gpc_frame_case(std::istream& in){
 Gp3Reader r{in};gp_Pnt o(r.xyz()),p(r.xyz());gp_XYZ n=r.xyz(),x=r.xyz(),m=r.xyz();double angle=r.real(),s=r.real();
 gp_Trsf T;GPC_TRY(T=gp3_trsf(r));gpc_ax2(gp_Ax2());gpc_ax3(gp_Ax3());
 GPC_TRY(gpc_ax2_members(o,p,n,x,m,angle,s,T));GPC_TRY(gpc_ax3_members(o,p,n,x,m,angle,s,T));std::cout<<'\n';
}

void gpc_lerp_case(std::istream& in){
 Gp3Reader r{in};double s0=r.real(),s1=r.real(),t=r.real();gp_XYZ x0=r.xyz(),x1=r.xyz();
 gp_Trsf a,b;GPC_TRY(a=gp3_trsf(r));GPC_TRY(b=gp3_trsf(r));
 gp3_real(NCollection_Lerp<double>::Interpolate(s0,s1,t));NCollection_Lerp<double> ls;double rs=91;ls.Interpolate(t,rs);gp3_real(rs);
 ls.Init(s0,s1);ls.Interpolate(t,rs);gp3_real(rs);NCollection_Lerp<double> lc(s1,s0);lc.Interpolate(t,rs);gp3_real(rs);
 xyz(NCollection_Lerp<gp_XYZ>::Interpolate(x0,x1,t));NCollection_Lerp<gp_XYZ> lx(x0,x1);gp_XYZ rx;lx.Interpolate(t,rx);xyz(rx);
 gp_Trsf rt;NCollection_Lerp<gp_Trsf> lt(a,b);GPC_TRY(lt.Interpolate(t,rt);gp3_trsf_state(rt));
 NCollection_Lerp<gp_Trsf> l0;GPC_TRY(l0.Interpolate(t,rt);gp3_trsf_state(rt));l0.Init(b,a);GPC_TRY(l0.Interpolate(t,rt);gp3_trsf_state(rt));
 std::cout<<'\n';
}
