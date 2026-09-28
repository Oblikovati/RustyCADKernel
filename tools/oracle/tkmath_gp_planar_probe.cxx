// Planar gp oracle at pinned OCCT 3d097a0: gp_XY, gp_Mat2d, gp_Trsf2d, gp_GTrsf2d and the
// gp_Dir2d/gp_Vec2d/gp_Pnt2d/gp_Ax2d/gp_Lin2d members that use them. Every value is printed as
// raw IEEE bits and every raised exception as its type name.
#include <gp_GTrsf2d.hxx>
#include <gp_Lin2d.hxx>
#define GP2_TRY(statement) try{statement;}catch(const Standard_Failure& gp2_error){std::cout<<gp2_error.ExceptionType()<<' ';}
void gp2_real(double value){std::cout<<bits(value)<<' ';}
void gp2_flag(bool value){std::cout<<value<<' ';}
void gp2_mat(const gp_Mat2d& m){for(int r=1;r<=2;++r)for(int c=1;c<=2;++c)gp2_real(m(r,c));}
void gp2_dir(const gp_Dir2d& d){xy(d.XY());}
void gp2_ax(const gp_Ax2d& a){xy(a.Location().XY());gp2_dir(a.Direction());}
struct Gp2Reader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 gp_XY pair(){double x=real();return gp_XY(x,real());}
 gp_Pnt2d pnt(){return gp_Pnt2d(pair());}
 gp_Mat2d mat(){double a=real(),b=real(),c=real(),d=real();gp_Mat2d m;m.SetRows(gp_XY(a,b),gp_XY(c,d));return m;}
};
void gp2_xy_case(std::istream& in){
 Gp2Reader r{in};gp_XY a=r.pair(),b=r.pair();double s=r.real(),t=r.real();gp_Mat2d m=r.mat();int index=r.integer();
 gp2_real(a.Modulus());gp2_real(a.SquareModulus());gp2_flag(a.IsEqual(b,s));gp2_real(a.Crossed(b));gp2_real(a^b);
 gp2_real(a.CrossMagnitude(b));gp2_real(a.CrossSquareMagnitude(b));gp2_real(a.Dot(b));gp2_real(a*b);
 xy(a.Added(b));xy(a.Subtracted(b));xy(a.Multiplied(s));xy(s*a);xy(a.Multiplied(b));xy(a.Multiplied(m));xy(m*a);
 xy(a.Divided(s));xy(a.Reversed());xy(-a);gp_XY c=a;c.Multiply(m);xy(c);c=a;c*=b;xy(c);c=a;c/=s;xy(c);
 GP2_TRY(xy(a.Normalized()));c=a;GP2_TRY(c.Normalize());xy(c);
 c.SetLinearForm(s,a,t,b);xy(c);c.SetLinearForm(s,a,t,b,a);xy(c);c.SetLinearForm(s,a,b);xy(c);c.SetLinearForm(a,b);xy(c);
 c=a;GP2_TRY(c.SetCoord(index,s));xy(c);GP2_TRY(gp2_real(a.Coord(index)));std::cout<<'\n';
}
void gp2_mat_case(std::istream& in){
 Gp2Reader r{in};gp_Mat2d m=r.mat(),o=r.mat();double s=r.real();int n=r.integer(),index=r.integer();
 gp2_real(m.Determinant());gp2_flag(m.IsSingular());xy(m.Diagonal());xy(m.Column(index));xy(m.Row(index));
 gp2_mat(m.Transposed());gp2_mat(m.Inverted());gp2_mat(m.Powered(n));gp2_mat(m.Multiplied(o));gp2_mat(m*o);
 gp_Mat2d c=m;c.PreMultiply(o);gp2_mat(c);gp2_mat(m.Added(o));gp2_mat(m-o);gp2_mat(m.Multiplied(s));gp2_mat(s*m);gp2_mat(m/s);
 c=m;c.SetCol(index,gp_XY(s,-s));gp2_mat(c);c=m;c.SetRow(index,gp_XY(s,-s));gp2_mat(c);
 c=m;c.SetRotation(s);gp2_mat(c);c=m;c.SetScale(s);gp2_mat(c);c=m;c.SetDiagonal(s,-s);gp2_mat(c);
 gp2_mat(gp_Mat2d(m.Row(1),m.Row(2)));c.SetCols(m.Row(1),m.Row(2));gp2_mat(c);c=m;c.Transpose();gp2_mat(c);
 c=m;c.Power(n);gp2_mat(c);c=m;c*=s;gp2_mat(c);c=m;c+=o;gp2_mat(c);c=m;c-=o;gp2_mat(c);c=m;c.Multiply(o);gp2_mat(c);c=m;c/=s;gp2_mat(c);
 GP2_TRY(gp2_real(m.Value(index,1)));c=m;GP2_TRY(c.SetValue(1,index,s));gp2_mat(c);std::cout<<'\n';
}
// One planar transformation: kind, six parameters, then SetScaleFactor/SetTranslationPart modifiers.
gp_Trsf2d gp2_trsf(Gp2Reader& r){
 int kind=r.integer();double p[6];for(double& v:p)v=r.real();int sk=r.integer();double sf=r.real();int tk=r.integer();gp_XY tv=r.pair();
 gp_Trsf2d t;gp_Pnt2d a(p[0],p[1]),b(p[2],p[3]);
 if(kind==1)t.SetTranslation(gp_Vec2d(a.XY()));
 if(kind==2)t.SetTranslation(a,b);
 if(kind==3)t.SetRotation(a,p[2]);
 if(kind==4)t.SetMirror(a);
 if(kind==5)t.SetMirror(gp_Ax2d(a,gp_Dir2d(p[2],p[3])));
 if(kind==6)t.SetScale(a,p[2]);
 if(kind==7)t.SetTransformation(gp_Ax2d(a,gp_Dir2d(p[2],p[3])));
 if(kind==8)t.SetTransformation(gp_Ax2d(a,gp_Dir2d(p[2],p[3])),gp_Ax2d(gp_Pnt2d(p[4],p[5]),gp_Dir2d(p[3]-p[5],p[2]+p[4])));
 if(kind==9)t.SetValues(p[0],p[1],p[2],p[3],p[4],p[5]);
 if(kind==10){gp_Trsf s;s.SetRotation(gp_Ax1(gp_Pnt(p[0],p[1],0),gp_Dir(p[2],p[3],p[4])),p[5]);t=gp_Trsf2d(s);}
 if(kind==11){gp_Trsf s;s.SetScale(gp_Pnt(p[0],p[1],p[2]),p[3]);t=gp_Trsf2d(s);}
 if(sk)t.SetScaleFactor(sf);
 if(tk)t.SetTranslationPart(gp_Vec2d(tv));
 return t;
}
void gp2_trsf_state(const gp_Trsf2d& t){
 std::cout<<int(t.Form())<<' ';gp2_real(t.ScaleFactor());gp2_flag(t.IsNegative());
 for(int row=1;row<=2;++row)for(int col=1;col<=3;++col)gp2_real(t.Value(row,col));
 gp2_mat(t.VectorialPart());gp2_mat(t.HVectorialPart());gp2_real(t.RotationPart());xy(t.TranslationPart());
}
void gp2_trsf_objects(const gp_Trsf2d& t,const gp_Pnt2d& p,const gp_XY& d){
 xy(p.Transformed(t).XY());gp_XY c=p.XY();t.Transforms(c);xy(c);double x=p.X(),y=p.Y();t.Transforms(x,y);gp2_real(x);gp2_real(y);
 xy(gp_Vec2d(d).Transformed(t).XY());
 GP2_TRY(gp_Dir2d dir(d);gp2_dir(dir.Transformed(t));gp2_ax(gp_Ax2d(p,dir).Transformed(t));gp2_ax(gp_Lin2d(p,dir).Transformed(t).Position()));
}
void gp2_trsf_case(std::istream& in){
 Gp2Reader r{in};gp_Trsf2d a,b;
 GP2_TRY(a=gp2_trsf(r));GP2_TRY(b=gp2_trsf(r));gp_Pnt2d p=r.pnt();gp_XY d=r.pair();int n=r.integer();
 gp2_trsf_state(a);gp2_trsf_state(b);gp2_trsf_state(a.Multiplied(b));gp2_trsf_state(a*b);
 gp_Trsf2d c=a;c.PreMultiply(b);gp2_trsf_state(c);c=a;c*=b;gp2_trsf_state(c);
 GP2_TRY(gp2_trsf_state(a.Inverted()));GP2_TRY(gp2_trsf_state(a.Powered(n)));
 c=a;GP2_TRY(c.Invert());gp2_trsf_state(c);c=a;GP2_TRY(c.Power(n));gp2_trsf_state(c);
 gp2_trsf_objects(a,p,d);std::cout<<'\n';
}
// One general transformation: kind, a planar transformation, a matrix, a translation, an
// affinity axis and ratio, and one SetValue cell.
gp_GTrsf2d gp2_gtrsf(Gp2Reader& r){
 int kind=r.integer();gp_Trsf2d t;GP2_TRY(t=gp2_trsf(r));gp_Mat2d m=r.mat();gp_XY v=r.pair();gp_Pnt2d ap=r.pnt();gp_XY ad=r.pair();double ratio=r.real();
 int row=r.integer(),col=r.integer();double value=r.real();
 gp_GTrsf2d g(t);
 if(kind==1)g=gp_GTrsf2d(m,v);
 if(kind==2)GP2_TRY(g.SetAffinity(gp_Ax2d(ap,gp_Dir2d(ad)),ratio));
 if(kind==3)GP2_TRY(g.SetValue(row,col,value));
 if(kind==4)g.SetTranslationPart(v);
 if(kind==5){g=gp_GTrsf2d(m,v);g.SetTrsf2d(t);}
 if(kind==6)g.SetVectorialPart(m);
 return g;
}
void gp2_gtrsf_state(const gp_GTrsf2d& g){
 std::cout<<int(g.Form())<<' ';gp2_flag(g.IsNegative());gp2_flag(g.IsSingular());
 for(int row=1;row<=2;++row)for(int col=1;col<=3;++col)gp2_real(g(row,col));
 gp2_mat(g.VectorialPart());xy(g.TranslationPart());GP2_TRY(gp2_trsf_state(g.Trsf2d()));
}
void gp2_gtrsf_case(std::istream& in){
 Gp2Reader r{in};gp_GTrsf2d a=gp2_gtrsf(r),b=gp2_gtrsf(r);gp_XY p=r.pair();int n=r.integer();
 gp2_gtrsf_state(a);gp2_gtrsf_state(b);GP2_TRY(gp2_gtrsf_state(a.Multiplied(b)));GP2_TRY(gp2_gtrsf_state(a*b));
 gp_GTrsf2d c=a;GP2_TRY(c.PreMultiply(b));gp2_gtrsf_state(c);c=a;GP2_TRY(c*=b);gp2_gtrsf_state(c);
 GP2_TRY(gp2_gtrsf_state(a.Inverted()));GP2_TRY(gp2_gtrsf_state(a.Powered(n)));
 xy(a.Transformed(p));gp_XY q=p;a.Transforms(q);xy(q);double x=p.X(),y=p.Y();a.Transforms(x,y);gp2_real(x);gp2_real(y);
 std::cout<<'\n';
}
void gp2_dir_block(const gp_Dir2d& d,const gp_Dir2d& e,const gp_Pnt2d& p,double angle,double tol){
 gp2_real(d.Angle(e));gp2_flag(d.IsEqual(e,tol));gp2_flag(d.IsNormal(e,tol));gp2_flag(d.IsOpposite(e,tol));gp2_flag(d.IsParallel(e,tol));
 gp2_real(d.Crossed(e));gp2_real(d^e);gp2_real(d.Dot(e));gp2_real(d*e);gp2_dir(d.Mirrored(e));gp2_dir(d.Mirrored(gp_Ax2d(p,e)));
 gp2_dir(d.Rotated(angle));gp2_dir(d.Reversed());gp2_dir(-d);gp_Dir2d c=d;c.Mirror(e);gp2_dir(c);c=d;c.Rotate(angle);gp2_dir(c);
}
void gp2_vec_block(const gp_Vec2d& v,const gp_Vec2d& w,const gp_Pnt2d& p,const gp_Dir2d& e,const double* k){
 double angle=k[0],s=k[1],lin=k[2],ang=k[3];
 gp2_real(v.Magnitude());gp2_real(v.SquareMagnitude());GP2_TRY(gp2_real(v.Angle(w)));GP2_TRY(gp2_flag(v.IsEqual(w,lin,ang)));
 GP2_TRY(gp2_flag(v.IsNormal(w,ang)));GP2_TRY(gp2_flag(v.IsOpposite(w,ang)));GP2_TRY(gp2_flag(v.IsParallel(w,ang)));
 gp2_real(v.Crossed(w));gp2_real(v.CrossMagnitude(w));gp2_real(v.CrossSquareMagnitude(w));gp2_real(v.Dot(w));xy(v.GetNormal().XY());
 GP2_TRY(xy(v.Normalized().XY()));gp_Vec2d c=v;GP2_TRY(c.Normalize());xy(c.XY());
 xy(v.Mirrored(w).XY());xy(v.Mirrored(gp_Ax2d(p,e)).XY());xy(v.Rotated(angle).XY());xy(v.Scaled(s).XY());
 xy((v+w).XY());xy((v-w).XY());xy((v*s).XY());xy((s*v).XY());xy((v/s).XY());xy((-v).XY());
 c.SetLinearForm(s,v,angle,w,v);xy(c.XY());c.SetLinearForm(s,v,angle,w);xy(c.XY());c.SetLinearForm(s,v,w);xy(c.XY());c.SetLinearForm(v,w);xy(c.XY());
}
void gp2_pnt_block(const gp_Pnt2d& p,const gp_Pnt2d& q,const gp_Dir2d& e,double angle,double s,double lin){
 gp2_real(p.Distance(q));gp2_real(p.SquareDistance(q));gp2_flag(p.IsEqual(q,lin));xy(p.Mirrored(q).XY());xy(p.Mirrored(gp_Ax2d(q,e)).XY());
 xy(p.Rotated(q,angle).XY());xy(p.Scaled(q,s).XY());xy(p.Translated(gp_Vec2d(q.XY())).XY());xy(p.Translated(q,p).XY());
 gp_Pnt2d c=p;c.Translate(q,p);xy(c.XY());
}
void gp2_axis_block(const gp_Ax2d& a,const gp_Ax2d& b,double angle,double s,double lin,double ang){
 gp2_flag(a.IsCoaxial(b,ang,lin));gp2_flag(a.IsNormal(b,ang));gp2_flag(a.IsOpposite(b,ang));gp2_flag(a.IsParallel(b,ang));gp2_real(a.Angle(b));
 gp2_ax(a.Mirrored(b.Location()));gp2_ax(a.Mirrored(b));gp2_ax(a.Rotated(b.Location(),angle));gp2_ax(a.Scaled(b.Location(),s));
 gp2_ax(a.Translated(a.Location(),b.Location()));
}
void gp2_line_block(const gp_Lin2d& l,const gp_Lin2d& m,double angle,double s,double lin){
 double la,lb,lc;l.Coefficients(la,lb,lc);gp2_real(la);gp2_real(lb);gp2_real(lc);const gp_Pnt2d& q=m.Location();
 gp2_real(l.Distance(q));gp2_real(l.SquareDistance(q));gp2_real(l.Distance(m));gp2_real(l.SquareDistance(m));gp2_flag(l.Contains(q,lin));
 gp2_real(l.Angle(m));gp2_ax(l.Normal(q).Position());gp2_ax(l.Mirrored(q).Position());gp2_ax(l.Mirrored(m.Position()).Position());
 gp2_ax(l.Rotated(q,angle).Position());gp2_ax(l.Scaled(q,s).Position());gp2_ax(l.Translated(l.Location(),q).Position());
 gp_Lin2d c=l;c.Translate(l.Location(),q);gp2_ax(c.Position());gp2_ax(l.Reversed().Position());
}
void gp2_geom_case(std::istream& in){
 Gp2Reader r{in};gp_Pnt2d p=r.pnt(),q=r.pnt();gp_XY dv=r.pair(),ev=r.pair();double k[4];for(double& v:k)v=r.real();
 double la=r.real(),lb=r.real(),lc=r.real();int index=r.integer(),standard=r.integer();
 const gp_Dir2d::D names[4]={gp_Dir2d::D::X,gp_Dir2d::D::Y,gp_Dir2d::D::NX,gp_Dir2d::D::NY};gp_Dir2d::D name=names[standard];
 gp2_dir(gp_Dir2d(name));gp2_ax(gp_Ax2d(p,name));gp2_ax(gp_Ax2d(name));
 gp2_vec_block(gp_Vec2d(p,q),gp_Vec2d(dv),p,gp_Dir2d(name),k);gp2_pnt_block(p,q,gp_Dir2d(name),k[0],k[1],k[2]);
 gp_Pnt2d pc=p;GP2_TRY(pc.SetCoord(index,k[1]));xy(pc.XY());GP2_TRY(gp_Lin2d l(la,lb,lc);gp2_ax(l.Position()));
 GP2_TRY(gp_Dir2d d(dv);gp2_dir(d);gp_Dir2d c=d;GP2_TRY(c.SetCoord(index,ev.X()));gp2_dir(c);c=d;GP2_TRY(c.SetCoord(ev.X(),ev.Y()));gp2_dir(c);
  c=d;GP2_TRY(c.SetX(ev.X()));gp2_dir(c);c=d;GP2_TRY(c.SetY(ev.Y()));gp2_dir(c);GP2_TRY(gp2_real(d.Coord(index)));
  gp_Dir2d e(ev);gp2_dir_block(d,e,p,k[0],k[3]);gp2_axis_block(gp_Ax2d(p,d),gp_Ax2d(q,e),k[0],k[1],k[2],k[3]);
  gp2_line_block(gp_Lin2d(p,d),gp_Lin2d(q,e),k[0],k[1],k[2]));
 std::cout<<'\n';
}
