// gp quadric and axis-placement members (included by tkmath_elementary_probe.cxx).
template<class Surface> void el_placed_surface(const Surface& s,const ElMotion3& k,const gp_Pnt& P){
 el_ax3(s.Position());EL_TRY(el_flag(s.Direct()))el_ax1(s.XAxis());el_ax1(s.YAxis());el_ax3(s.Mirrored(k.pivot).Position());el_ax3(s.Mirrored(k.axis).Position());
 el_ax3(s.Mirrored(k.plane).Position());EL_TRY(el_ax3(s.Rotated(k.axis,k.angle).Position()))el_ax3(s.Translated(k.shift).Position());el_ax3(s.Translated(k.pivot,P).Position());
 el_ax3(s.Scaled(k.pivot,k.scale).Position());el_ax3(s.Transformed(k.transform).Position());
 Surface u=s;u.UReverse();el_ax3(u.Position());Surface v=s;v.VReverse();el_ax3(v.Position());Surface l=s;l.SetLocation(P);el_ax3(l.Position());
 double q[10];s.Coefficients(q[0],q[1],q[2],q[3],q[4],q[5],q[6],q[7],q[8],q[9]);el_reals(q,10);
}
template<class Surface> void el_axis_surface(const Surface& s,const ElMotion3& k){
 el_ax1(s.Axis());Surface a=s;EL_TRY(a.SetAxis(k.axis);el_ax3(a.Position()))
}
void el_sphere(const gp_Ax3& A,double r,const ElMotion3& k,const gp_Pnt& P){
 gp_Sphere s(A,r);el_real(s.Area());el_real(s.Volume());el_placed_surface(s,k,P);el_real(s.Scaled(k.pivot,k.scale).Radius());el_real(s.Transformed(k.transform).Radius());
}
void el_cylinder(const gp_Ax3& A,double r,const ElMotion3& k,const gp_Pnt& P){
 gp_Cylinder s(A,r);el_placed_surface(s,k,P);el_axis_surface(s,k);el_real(s.Scaled(k.pivot,k.scale).Radius());el_real(s.Transformed(k.transform).Radius());
}
void el_cone(const gp_Ax3& A,double a,double r,const ElMotion3& k,const gp_Pnt& P){
 gp_Cone s(A,a,r);el_pnt(s.Apex());el_placed_surface(s,k,P);el_axis_surface(s,k);gp_Cone v=s;v.VReverse();el_real(v.SemiAngle());
 el_real(s.Scaled(k.pivot,k.scale).RefRadius());el_real(s.Transformed(k.transform).RefRadius());gp_Cone w=s;EL_TRY(w.SetSemiAngle(2.0*a);el_real(w.SemiAngle()))
}
void el_torus(const gp_Ax3& A,double M,double m,const ElMotion3& k,const gp_Pnt& P){
 gp_Torus s(A,M,m);el_real(s.Area());el_real(s.Volume());el_axis_surface(s,k);
 el_ax3(s.Mirrored(k.axis).Position());el_ax3(s.Translated(k.pivot,P).Position());EL_TRY(el_flag(s.Direct()))
 NCollection_Array1<double> q(3,37);s.Coefficients(q);for(int i=3;i<=37;++i)el_real(q(i));NCollection_Array1<double> shortArray(1,34);EL_TRY(s.Coefficients(shortArray))
 gp_Torus a=s.Scaled(k.pivot,k.scale);el_real(a.MajorRadius());el_real(a.MinorRadius());gp_Torus b=s;b.Scale(k.pivot,k.scale);el_real(b.MajorRadius());el_real(b.MinorRadius());
 gp_Torus c=s.Transformed(k.transform);el_real(c.MajorRadius());el_real(c.MinorRadius());gp_Torus d=s;d.Transform(k.transform);el_real(d.MajorRadius());el_real(d.MinorRadius());
 gp_Torus e=s;EL_TRY(e.SetMajorRadius(m);el_real(e.MajorRadius()))EL_TRY(e.SetMinorRadius(M);el_real(e.MinorRadius()))
}
void gpquadric_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);int indirect;double r,a,M,m;in>>indirect>>r>>a>>M>>m;
 ElMotion3 k;k.pivot=el_read_pnt(in);gp_XYZ D=el_read_xyz(in);in>>k.angle>>k.scale;k.shift=gp_Vec(el_read_xyz(in));gp_Pnt P=el_read_pnt(in);
 try{
  gp_Ax3 A(O,gp_Dir(N),gp_Dir(X));if(indirect)A.YReverse();k.axis=gp_Ax1(k.pivot,gp_Dir(D));k.plane=gp_Ax2(k.pivot,gp_Dir(D));
  gp_Trsf R,S;R.SetRotation(k.axis,k.angle);S.SetScale(k.pivot,k.scale);k.transform=R.Multiplied(S);
  EL_TRY(el_sphere(A,r,k,P))EL_TRY(el_cylinder(A,r,k,P))EL_TRY(el_cone(A,a,r,k,P))EL_TRY(el_torus(A,M,m,k,P))
 }catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
void gpaxis_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);gp_Pnt Q=el_read_pnt(in);gp_XYZ D=el_read_xyz(in);double angle,scale,lin,ang;in>>angle>>scale>>lin>>ang;
 try{
  gp_Ax2 A(O,gp_Dir(N),gp_Dir(X));gp_Ax1 L(Q,gp_Dir(D));gp_Ax2 B(Q,gp_Dir(D));gp_Ax3 C(A);gp_Trsf T;T.SetRotation(L,angle);
  gp_Ax1 M(O,gp_Dir(N));el_flag(M.IsCoaxial(L,ang,lin));el_flag(M.IsNormal(L,ang));el_flag(M.IsOpposite(L,ang));el_flag(M.IsParallel(L,ang));el_real(M.Angle(L));
  el_ax1(M.Reversed());el_ax1(M.Mirrored(Q));el_ax1(M.Mirrored(L));el_ax1(M.Mirrored(B));el_ax1(M.Rotated(L,angle));el_ax1(M.Scaled(Q,scale));el_ax1(M.Transformed(T));el_ax1(M.Translated(O,Q));
  std::ostringstream d1;M.DumpJson(d1);std::cout<<d1.str()<<' ';
  gp_Ax2 s=A;EL_TRY(s.SetAxis(L);el_ax2(s))s=A;EL_TRY(s.SetDirection(gp_Dir(D));el_ax2(s))s=A;EL_TRY(s.SetDirection(A.XDirection());el_ax2(s))
  s=A;EL_TRY(s.SetDirection(A.XDirection().Reversed());el_ax2(s))s=A;EL_TRY(s.SetXDirection(gp_Dir(D));el_ax2(s))s=A;EL_TRY(s.SetYDirection(gp_Dir(D));el_ax2(s))
  el_real(A.Angle(B));el_flag(A.IsCoplanar(B,lin,ang));el_flag(A.IsCoplanar(L,lin,ang));EL_TRY(el_ax2(A.Mirrored(L)))EL_TRY(el_ax2(A.Mirrored(B)))
  el_ax2(A.Translated(O,Q));std::ostringstream d2;A.DumpJson(d2);std::cout<<d2.str()<<' ';
  EL_TRY(el_ax2(C.Ax2()))gp_Ax3 c=C;c.YReverse();EL_TRY(el_ax2(c.Ax2()))el_real(C.Angle(gp_Ax3(B)));el_flag(C.IsCoplanar(gp_Ax3(B),lin,ang));el_flag(C.IsCoplanar(L,lin,ang));
  el_ax3(C.Mirrored(Q));el_ax3(C.Mirrored(L));el_ax3(C.Translated(O,Q));
 }catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
