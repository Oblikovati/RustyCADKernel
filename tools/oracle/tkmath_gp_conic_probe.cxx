// gp conic members in 3D and 2D (included by tkmath_elementary_probe.cxx).
struct ElMotion3{gp_Pnt pivot;gp_Ax1 axis;gp_Ax2 plane;double angle,scale;gp_Vec shift;gp_Trsf transform;};
template<class Curve> void el_placed3(const Curve& c,const ElMotion3& k,const gp_Pnt& P){
 el_ax1(c.Axis());el_ax1(c.XAxis());el_ax1(c.YAxis());el_ax2(c.Mirrored(k.pivot).Position());
 EL_TRY(el_ax2(c.Mirrored(k.axis).Position()))EL_TRY(el_ax2(c.Mirrored(k.plane).Position()))
 EL_TRY(el_ax2(c.Rotated(k.axis,k.angle).Position()))el_ax2(c.Translated(k.shift).Position());el_ax2(c.Translated(k.pivot,P).Position());
 Curve s=c.Scaled(k.pivot,k.scale);el_ax2(s.Position());Curve t=c;EL_TRY(t=c.Transformed(k.transform);el_ax2(t.Position()))
 Curve a=c;EL_TRY(a.SetAxis(k.axis);el_ax2(a.Position()))
}
void el_circle3(const gp_Ax2& A,double r,const ElMotion3& k,const gp_Pnt& P){
 gp_Circ c(A,r);el_real(c.Area());el_real(c.Length());el_real(c.Distance(P));el_real(c.SquareDistance(P));el_flag(c.Contains(P,1e-7));el_placed3(c,k,P);
 el_real(c.Scaled(k.pivot,k.scale).Radius());EL_TRY(el_real(c.Transformed(k.transform).Radius()))
}
void el_ellipse3(const gp_Ax2& A,double M,double m,const ElMotion3& k,const gp_Pnt& P){
 gp_Elips e(A,M,m);el_real(e.Area());el_real(e.Eccentricity());el_real(e.Focal());el_pnt(e.Focus1());el_pnt(e.Focus2());el_real(e.Parameter());
 EL_TRY(el_ax1(e.Directrix1()))EL_TRY(el_ax1(e.Directrix2()))el_placed3(e,k,P);gp_Elips s=e.Scaled(k.pivot,k.scale);el_real(s.MajorRadius());el_real(s.MinorRadius());
 EL_TRY(gp_Elips t=e.Transformed(k.transform);el_real(t.MajorRadius());el_real(t.MinorRadius()))
}
void el_hyperbola3(const gp_Ax2& A,double M,double m,const ElMotion3& k,const gp_Pnt& P){
 gp_Hypr h(A,M,m);el_real(h.Focal());el_pnt(h.Focus1());el_pnt(h.Focus2());EL_TRY(el_real(h.Eccentricity()))EL_TRY(el_real(h.Parameter()))
 EL_TRY(el_ax1(h.Asymptote1()))EL_TRY(el_ax1(h.Asymptote2()))EL_TRY(el_ax1(h.Directrix1()))EL_TRY(el_ax1(h.Directrix2()))
 EL_TRY(gp_Hypr b=h.ConjugateBranch1();el_ax2(b.Position());el_real(b.MajorRadius()))EL_TRY(gp_Hypr b=h.ConjugateBranch2();el_ax2(b.Position()))
 EL_TRY(gp_Hypr b=h.OtherBranch();el_ax2(b.Position()))gp_Hypr l=h;EL_TRY(l.SetLocation(P);el_ax2(l.Position()))el_placed3(h,k,P);
}
void el_parabola3(const gp_Ax2& A,double F,const ElMotion3& k,const gp_Pnt& P){
 gp_Parab p(A,F);el_real(p.Parameter());el_pnt(p.Focus());el_ax1(p.Directrix());el_placed3(p,k,P);el_real(p.Scaled(k.pivot,k.scale).Focal());
 EL_TRY(gp_Parab d(k.axis,P);el_ax2(d.Position());el_real(d.Focal()))
}
void gpconic3_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);double M,m,F,r;in>>M>>m>>F>>r;
 ElMotion3 k;k.pivot=el_read_pnt(in);gp_XYZ D=el_read_xyz(in);in>>k.angle>>k.scale;k.shift=gp_Vec(el_read_xyz(in));gp_Pnt P=el_read_pnt(in);
 try{
  gp_Ax2 A(O,gp_Dir(N),gp_Dir(X));k.axis=gp_Ax1(k.pivot,gp_Dir(D));k.plane=gp_Ax2(k.pivot,gp_Dir(D));
  gp_Trsf R,S;R.SetRotation(k.axis,k.angle);S.SetScale(k.pivot,k.scale);k.transform=R.Multiplied(S);
  EL_TRY(el_circle3(A,r,k,P))EL_TRY(el_ellipse3(A,M,m,k,P))EL_TRY(el_hyperbola3(A,M,m,k,P))EL_TRY(el_parabola3(A,F,k,P))
  EL_TRY(gp_Hypr().ConjugateBranch1())
 }catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
struct ElMotion2{gp_Pnt2d pivot;gp_Ax2d axis;double angle,scale;gp_Vec2d shift;gp_Trsf2d transform;};
template<class Curve> void el_placed2(const Curve& c,const ElMotion2& k,const gp_Pnt2d& P){
 el_ax22d(c.Axis());el_flag(c.IsDirect());el_ax22d(c.Reversed().Axis());el_ax22d(c.Mirrored(k.pivot).Axis());el_ax22d(c.Mirrored(k.axis).Axis());
 el_ax22d(c.Rotated(k.pivot,k.angle).Axis());el_ax22d(c.Translated(k.shift).Axis());el_ax22d(c.Translated(k.pivot,P).Axis());
 el_ax22d(c.Scaled(k.pivot,k.scale).Axis());el_ax22d(c.Transformed(k.transform).Axis());
 double q[6];c.Coefficients(q[0],q[1],q[2],q[3],q[4],q[5]);el_reals(q,6);
}
template<class Curve> void el_axes2(const Curve& c,const ElMotion2& k){
 el_ax2d(c.XAxis());el_ax2d(c.YAxis());Curve x=c;x.SetXAxis(k.axis);el_ax22d(x.Axis());Curve y=c;y.SetYAxis(k.axis);el_ax22d(y.Axis());
}
template<class Curve> void el_radii2(const Curve& c,const ElMotion2& k){
 Curve s=c.Scaled(k.pivot,k.scale);el_real(s.MajorRadius());el_real(s.MinorRadius());Curve t=c.Transformed(k.transform);el_real(t.MajorRadius());el_real(t.MinorRadius());
 Curve u=c;u.Transform(k.transform);el_real(u.MajorRadius());el_real(u.MinorRadius());
}
void el_circle2(const gp_Ax22d& A,double r,const ElMotion2& k,const gp_Pnt2d& P){
 gp_Circ2d c(A,r);el_real(c.Area());el_real(c.Length());el_real(c.Distance(P));el_real(c.SquareDistance(P));el_flag(c.Contains(P,1e-7));
 el_placed2(c,k,P);el_axes2(c,k);el_real(c.Scaled(k.pivot,k.scale).Radius());el_real(c.Transformed(k.transform).Radius());
}
void el_ellipse2(const gp_Ax22d& A,double M,double m,const ElMotion2& k,const gp_Pnt2d& P){
 gp_Elips2d e(A,M,m);el_real(e.Area());el_real(e.Eccentricity());el_real(e.Focal());el_pnt2(e.Focus1());el_pnt2(e.Focus2());el_real(e.Parameter());
 EL_TRY(el_ax2d(e.Directrix1()))EL_TRY(el_ax2d(e.Directrix2()))el_placed2(e,k,P);el_axes2(e,k);el_radii2(e,k);
}
void el_hyperbola2(const gp_Ax22d& A,double M,double m,const ElMotion2& k,const gp_Pnt2d& P){
 gp_Hypr2d h(A,M,m);el_real(h.Focal());el_pnt2(h.Focus1());el_pnt2(h.Focus2());EL_TRY(el_real(h.Eccentricity()))EL_TRY(el_real(h.Parameter()))
 EL_TRY(el_ax2d(h.Asymptote1()))EL_TRY(el_ax2d(h.Asymptote2()))EL_TRY(el_ax2d(h.Directrix1()))EL_TRY(el_ax2d(h.Directrix2()))
 el_ax22d(h.ConjugateBranch1().Axis());el_ax22d(h.ConjugateBranch2().Axis());el_ax22d(h.OtherBranch().Axis());el_real(h.ConjugateBranch1().MajorRadius());
 el_placed2(h,k,P);el_axes2(h,k);el_radii2(h,k);
}
void el_parabola2(const gp_Ax22d& A,double F,int sense,const ElMotion2& k,const gp_Pnt2d& P){
 gp_Parab2d p(A,F);el_real(p.Parameter());el_pnt2(p.Focus());el_ax2d(p.Directrix());el_ax2d(p.MirrorAxis());el_placed2(p,k,P);
 gp_Parab2d x=p;x.SetMirrorAxis(k.axis);el_ax22d(x.Axis());el_real(p.Scaled(k.pivot,k.scale).Focal());
 EL_TRY(gp_Parab2d d(k.axis,P,sense!=0);el_ax22d(d.Axis());el_real(d.Focal()))
}
void el_frame2(const gp_Ax22d& A,const ElMotion2& k,const gp_Pnt2d& P){
 gp_Ax22d f(A.Location(),A.XDirection(),k.axis.Direction());el_ax22d(f);
 gp_Ax22d g=A;g.SetXDirection(k.axis.Direction());el_ax22d(g);g=A;g.SetYDirection(k.axis.Direction());el_ax22d(g);
 gp_Ax2d L(A.Location(),A.XDirection());el_flag(L.IsCoaxial(k.axis,1e-6,1e-6));el_real(L.Angle(k.axis));el_ax2d(L.Mirrored(k.axis));
 el_ax2d(L.Rotated(k.pivot,k.angle));el_ax2d(L.Scaled(k.pivot,k.scale));el_ax2d(L.Translated(k.pivot,P));el_ax2d(L.Transformed(k.transform));
 std::ostringstream dump;L.DumpJson(dump);std::cout<<dump.str()<<' ';std::ostringstream shallow;L.DumpJson(shallow,0);std::cout<<shallow.str()<<' ';
 std::ostringstream frame;A.DumpJson(frame);std::cout<<frame.str()<<' ';
}
void gpconic2_case(std::istream& in){
 gp_Pnt2d O=el_read_pnt2(in);gp_XY X=el_read_xy(in);int sense;double M,m,F,r;in>>sense>>M>>m>>F>>r;
 ElMotion2 k;k.pivot=el_read_pnt2(in);gp_XY D=el_read_xy(in);in>>k.angle>>k.scale;k.shift=gp_Vec2d(el_read_xy(in));gp_Pnt2d P=el_read_pnt2(in);
 try{
  gp_Ax22d A(O,gp_Dir2d(X),sense!=0);k.axis=gp_Ax2d(k.pivot,gp_Dir2d(D));
  gp_Trsf2d R,Mi;R.SetRotation(k.pivot,k.angle);Mi.SetMirror(k.axis);k.transform=R.Multiplied(Mi);
  el_frame2(A,k,P);EL_TRY(el_circle2(A,r,k,P))EL_TRY(el_ellipse2(A,M,m,k,P))EL_TRY(el_hyperbola2(A,M,m,k,P))EL_TRY(el_parabola2(A,F,sense,k,P))
 }catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
