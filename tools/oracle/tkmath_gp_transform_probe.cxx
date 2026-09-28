// gp_Trsf, gp_GTrsf, gp_Quaternion and the quaternion interpolators at pinned OCCT 3d097a0.
// Every value is printed as raw IEEE bits and every raised exception as its type name.
#include <gp_GTrsf.hxx>
#include <gp_Quaternion.hxx>
#include <gp_QuaternionNLerp.hxx>
#include <gp_QuaternionSLerp.hxx>
#define GP3_TRY(statement) try{statement;}catch(const Standard_Failure& gp3_error){std::cout<<gp3_error.ExceptionType()<<' ';}
void gp3_real(double value){std::cout<<bits(value)<<' ';}
void gp3_mat(const gp_Mat& m){for(int r=1;r<=3;++r)for(int c=1;c<=3;++c)gp3_real(m(r,c));}
void gp3_quat(const gp_Quaternion& q){gp3_real(q.X());gp3_real(q.Y());gp3_real(q.Z());gp3_real(q.W());}
template<class T> void gp3_mat4(const NCollection_Mat4<T>& m){for(size_t r=0;r<4;++r)for(size_t c=0;c<4;++c)gp3_real(double(m.GetValue(r,c)));}
struct Gp3Reader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 gp_XYZ xyz(){double x=real(),y=real();return gp_XYZ(x,y,real());}
 gp_Quaternion quat(){double x=real(),y=real(),z=real();return gp_Quaternion(x,y,z,real());}
 gp_Mat mat(){gp_Mat m;for(int r=1;r<=3;++r)for(int c=1;c<=3;++c)m.SetValue(r,c,real());return m;}
};
// One transformation: kind, twelve parameters, then SetScaleFactor, SetTranslationPart and
// SetRotationPart modifiers.
gp_Trsf gp3_trsf(Gp3Reader& r){
 int kind=r.integer();double p[12];for(double& v:p)v=r.real();int sk=r.integer();double sf=r.real();
 int tk=r.integer();gp_XYZ tv=r.xyz();int rk=r.integer();gp_Quaternion rq=r.quat();
 gp_Trsf t;gp_Pnt a(p[0],p[1],p[2]),b(p[3],p[4],p[5]);
 if(kind==1)t.SetTranslation(gp_Vec(a.XYZ()));
 if(kind==2)t.SetTranslation(a,b);
 if(kind==3)t.SetRotation(gp_Ax1(a,gp_Dir(p[3],p[4],p[5])),p[6]);
 if(kind==4)t.SetMirror(a);
 if(kind==5)t.SetMirror(gp_Ax1(a,gp_Dir(p[3],p[4],p[5])));
 if(kind==6)t.SetMirror(gp_Ax2(a,gp_Dir(p[3],p[4],p[5])));
 if(kind==7)t.SetScale(a,p[3]);
 if(kind==8)t.SetTransformation(gp_Ax3(a,gp_Dir(p[3],p[4],p[5])));
 if(kind==9)t.SetTransformation(gp_Ax3(a,gp_Dir(p[3],p[4],p[5])),gp_Ax3(gp_Pnt(p[6],p[7],p[8]),gp_Dir(p[5],p[3],p[4])));
 if(kind==10)t.SetDisplacement(gp_Ax3(a,gp_Dir(p[3],p[4],p[5])),gp_Ax3(gp_Pnt(p[6],p[7],p[8]),gp_Dir(p[5],p[3],p[4])));
 if(kind==11)t.SetValues(p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7],p[8],p[9],p[10],p[11]);
 if(kind==12)t.SetRotation(gp_Quaternion(p[0],p[1],p[2],p[3]));
 if(kind==13)t.SetTransformation(gp_Quaternion(p[0],p[1],p[2],p[3]),gp_Vec(p[4],p[5],p[6]));
 if(kind==14){gp_Trsf2d s;if(p[4]>0)s.SetMirror(gp_Ax2d(gp_Pnt2d(p[0],p[1]),gp_Dir2d(p[2],p[3])));else s.SetRotation(gp_Pnt2d(p[0],p[1]),p[2]);t=gp_Trsf(s);}
 if(sk)t.SetScaleFactor(sf);
 if(tk)t.SetTranslationPart(gp_Vec(tv));
 if(rk)t.SetRotationPart(rq);
 return t;
}
void gp3_trsf_state(const gp_Trsf& t){
 std::cout<<int(t.Form())<<' ';gp3_real(t.ScaleFactor());std::cout<<t.IsNegative()<<' ';
 for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)gp3_real(t.Value(row,col));
 gp3_mat(t.VectorialPart());gp3_mat(t.HVectorialPart());xyz(t.TranslationPart());gp3_quat(t.GetRotation());
 gp_XYZ axis;double angle=91;std::cout<<t.GetRotation(axis,angle)<<' ';xyz(axis);gp3_real(angle);
 NCollection_Mat4<double> md;t.GetMat4(md);gp3_mat4(md);NCollection_Mat4<float> mf;t.GetMat4(mf);gp3_mat4(mf);
 std::ostringstream dump;t.DumpJson(dump);std::cout<<dump.str()<<" | ";
}
void gp3_trsf_case(std::istream& in){
 Gp3Reader r{in};gp_Trsf a,b;GP3_TRY(a=gp3_trsf(r));GP3_TRY(b=gp3_trsf(r));gp_Pnt p(r.xyz());gp_XYZ d=r.xyz();int n=r.integer();
 gp3_trsf_state(a);gp3_trsf_state(b);gp3_trsf_state(a.Multiplied(b));gp3_trsf_state(a*b);
 gp_Trsf c=a;c.PreMultiply(b);gp3_trsf_state(c);c=a;c*=b;gp3_trsf_state(c);
 GP3_TRY(gp3_trsf_state(a.Inverted()));GP3_TRY(gp3_trsf_state(a.Powered(n)));
 c=a;GP3_TRY(c.Invert());gp3_trsf_state(c);c=a;GP3_TRY(c.Power(n));gp3_trsf_state(c);
 xyz(p.Transformed(a).XYZ());gp_XYZ q=p.XYZ();a.Transforms(q);xyz(q);double x=p.X(),y=p.Y(),z=p.Z();a.Transforms(x,y,z);gp3_real(x);gp3_real(y);gp3_real(z);
 GP3_TRY(gp_Dir dir(d);xyz(dir.Transformed(a).XYZ());xyz(gp_Ax1(p,dir).Transformed(a).Direction().XYZ()));
 std::ostringstream dump;a.DumpJson(dump);Standard_SStream stream(dump.str());gp_Trsf parsed;int pos=1;
 std::cout<<parsed.InitFromJson(stream,pos)<<' '<<pos<<' ';gp3_trsf_state(parsed);
 std::cout<<'\n';
}
// One general transformation: kind, a transformation, a matrix, a translation, an affinity
// axis/frame (point, direction) and ratio, and one SetValue cell.
gp_GTrsf gp3_gtrsf(Gp3Reader& r){
 int kind=r.integer();gp_Trsf t;GP3_TRY(t=gp3_trsf(r));gp_Mat m=r.mat();gp_XYZ v=r.xyz();gp_Pnt ap(r.xyz());gp_XYZ ad=r.xyz();double ratio=r.real();
 int row=r.integer(),col=r.integer();double value=r.real();
 gp_GTrsf g(t);
 if(kind==1)g=gp_GTrsf(m,v);
 if(kind==2)GP3_TRY(g.SetAffinity(gp_Ax1(ap,gp_Dir(ad)),ratio));
 if(kind==3)GP3_TRY(g.SetAffinity(gp_Ax2(ap,gp_Dir(ad)),ratio));
 if(kind==4)GP3_TRY(g.SetValue(row,col,value));
 if(kind==5)g.SetTranslationPart(v);
 if(kind==6)g.SetVectorialPart(m);
 if(kind==7){NCollection_Mat4<double> m4;for(int i=0;i<3;++i){for(int j=0;j<3;++j)m4.SetValue(i,j,m(i+1,j+1));m4.SetValue(i,3,v.Coord(i+1));}m4.SetValue(3,0,ratio);g.SetMat4(m4);}
 if(kind==8){g=gp_GTrsf(m,v);GP3_TRY(g.SetForm());}
 if(kind==9)GP3_TRY(g.SetForm());
 return g;
}
void gp3_gtrsf_state(const gp_GTrsf& g){
 std::cout<<int(g.Form())<<' '<<g.IsNegative()<<' '<<g.IsSingular()<<' ';
 for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)gp3_real(g(row,col));
 gp3_mat(g.VectorialPart());xyz(g.TranslationPart());GP3_TRY(gp3_trsf_state(g.Trsf()));
 NCollection_Mat4<double> md;g.GetMat4(md);gp3_mat4(md);std::ostringstream dump;g.DumpJson(dump);std::cout<<dump.str()<<" | ";
}
void gp3_gtrsf_case(std::istream& in){
 Gp3Reader r{in};gp_GTrsf a=gp3_gtrsf(r),b=gp3_gtrsf(r);gp_XYZ p=r.xyz();int n=r.integer();
 gp3_gtrsf_state(a);gp3_gtrsf_state(b);gp3_gtrsf_state(a.Multiplied(b));gp3_gtrsf_state(a*b);
 gp_GTrsf c=a;c.PreMultiply(b);gp3_gtrsf_state(c);c=a;c*=b;gp3_gtrsf_state(c);
 GP3_TRY(gp3_gtrsf_state(a.Inverted()));GP3_TRY(gp3_gtrsf_state(a.Powered(n)));
 gp_XYZ q=p;a.Transforms(q);xyz(q);double x=p.X(),y=p.Y(),z=p.Z();a.Transforms(x,y,z);gp3_real(x);gp3_real(y);gp3_real(z);
 std::cout<<'\n';
}
void gp3_quat_case(std::istream& in){
 Gp3Reader r{in};gp_Quaternion a=r.quat(),b=r.quat();gp_Vec u(r.xyz()),v(r.xyz()),h(r.xyz());double angle=r.real(),t=r.real();
 std::cout<<a.IsEqual(b)<<' '<<a.IsEqual(a)<<' ';gp3_quat(gp_Quaternion(u,v));gp3_quat(gp_Quaternion(u,v,h));
 gp_Quaternion c;GP3_TRY(c.SetVectorAndAngle(u,angle);gp3_quat(c));gp_Vec axis;double got=91;a.GetVectorAndAngle(axis,got);xyz(axis.XYZ());gp3_real(got);
 gp3_mat(a.GetMatrix());c.SetMatrix(a.GetMatrix());gp3_quat(c);gp3_quat(gp_Quaternion(b.GetMatrix()));
 gp3_quat(a.Reversed());gp3_quat(a.Inverted());gp3_real(a.SquareNorm());gp3_real(a.Norm());gp3_quat(a.Scaled(t));gp3_quat(a*t);
 c=a;c.StabilizeLength();gp3_quat(c);gp3_quat(a.Normalized());c=a;c.Normalize();gp3_quat(c);gp3_quat(a.Negated());gp3_quat(-a);
 gp3_quat(a.Added(b));gp3_quat(a+b);gp3_quat(a.Subtracted(b));gp3_quat(a-b);gp3_quat(a.Multiplied(b));gp3_quat(a*b);
 c=a;c.Add(b);gp3_quat(c);c=a;c-=b;gp3_quat(c);c=a;c.Multiply(b);gp3_quat(c);c=a;c*=b;gp3_quat(c);c=a;c*=t;gp3_quat(c);c=a;c.Scale(t);gp3_quat(c);
 c=a;c.Reverse();gp3_quat(c);c=a;c.Invert();gp3_quat(c);c=a;c.SetIdent();gp3_quat(c);c.Set(b);gp3_quat(c);
 gp3_real(a.Dot(b));gp3_real(a.GetRotationAngle());xyz(a.Multiply(u).XYZ());xyz((a*u).XYZ());
 gp3_quat(gp_QuaternionNLerp::Interpolate(a,b,t));gp3_quat(gp_QuaternionSLerp::Interpolate(a,b,t));
 gp_QuaternionNLerp nl;nl.InitFromUnit(a,b);nl.Interpolate(t,c);gp3_quat(c);gp_QuaternionSLerp sl;sl.InitFromUnit(a,b);sl.Interpolate(t,c);gp3_quat(c);
 std::cout<<'\n';
}
