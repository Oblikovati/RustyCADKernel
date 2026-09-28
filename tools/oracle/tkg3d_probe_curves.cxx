// TKG3d oracle: curve evaluation, curve transformations, lines and conics (tkg3d_probe.cxx).
void g3_curve_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;
 std::vector<double> u=r.reals();g3_name(c);g3_curve_summary(c,u);g3_dump(c,-1);g3_dump(c,1);g3_dump(c,0);
 std::cout<<'\n';
}
std::vector<double> g3_probe_params;
void g3_show_curve(const occ::handle<Geom_Geometry>& g){g3_curve_summary(occ::down_cast<Geom_Curve>(g),g3_probe_params);}
void g3_transform_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;g3_probe_params=r.reals();
 gp_Trsf t=g3_trsf(r);
 for(double u:g3_probe_params)g3_real(c->TransformedParameter(u,t));g3_real(c->ParametricTransformation(t));
 G3_TRY(g3_show_curve(c->Reversed()));G3_TRY(c->Reverse();g3_show_curve(c));g3_moves(c,r,g3_show_curve);std::cout<<'\n';
}
void g3_line_case(std::istream& in){
 G3Reader r{in};gp_Ax1 a=r.ax1();gp_Dir d=r.dir();gp_Pnt p=r.pnt();gp_Ax1 b=r.ax1();std::vector<double> u=r.reals();
 occ::handle<Geom_Line> l=new Geom_Line(a),m=new Geom_Line(gp_Lin(b)),n=new Geom_Line(p,d);
 g3_ax1(l->Position());g3_ax1(l->Lin().Position());g3_ax1(m->Position());g3_ax1(n->Position());
 l->SetDirection(d);g3_ax1(l->Position());l->SetLocation(p);g3_ax1(l->Position());l->SetPosition(b);g3_ax1(l->Position());
 l->SetLin(gp_Lin(a));g3_curve_summary(l,u);g3_dump(l,-1);std::cout<<'\n';
}
void g3_conic_common(const occ::handle<Geom_Conic>& c){
 g3_ax1(c->XAxis());g3_ax1(c->YAxis());g3_ax1(c->Axis());g3_pnt(c->Location());g3_ax2(c->Position());G3_TRY(g3_real(c->Eccentricity()));
}
void g3_conic_specific(const occ::handle<Geom_Conic>& c,double value){
 if(auto e=occ::down_cast<Geom_Circle>(c)){g3_real(e->Radius());G3_TRY(e->SetRadius(value));g3_real(e->Radius());
  G3_TRY(gp_Circ g=e->Circ();g3_ax2(g.Position());g3_real(g.Radius());e->SetCirc(g));}
 if(auto e=occ::down_cast<Geom_Ellipse>(c)){g3_real(e->MajorRadius());g3_real(e->MinorRadius());G3_TRY(g3_ax1(e->Directrix1()));
  G3_TRY(g3_ax1(e->Directrix2()));g3_real(e->Focal());g3_pnt(e->Focus1());g3_pnt(e->Focus2());g3_real(e->Parameter());
  G3_TRY(e->SetMinorRadius(value));G3_TRY(e->SetMajorRadius(value));g3_real(e->MajorRadius());g3_real(e->MinorRadius());
  G3_TRY(gp_Elips g=e->Elips();g3_ax2(g.Position());e->SetElips(g));}
 if(auto e=occ::down_cast<Geom_Hyperbola>(c)){g3_real(e->MajorRadius());g3_real(e->MinorRadius());G3_TRY(g3_ax1(e->Asymptote1()));
  G3_TRY(g3_ax1(e->Asymptote2()));G3_TRY(g3_ax2(e->ConjugateBranch1().Position()));G3_TRY(g3_ax2(e->ConjugateBranch2().Position()));
  G3_TRY(g3_ax1(e->Directrix1()));G3_TRY(g3_ax1(e->Directrix2()));g3_real(e->Focal());g3_pnt(e->Focus1());g3_pnt(e->Focus2());
  G3_TRY(g3_ax2(e->OtherBranch().Position()));G3_TRY(g3_real(e->Parameter()));G3_TRY(e->SetMajorRadius(value));
  G3_TRY(e->SetMinorRadius(-value));g3_real(e->MajorRadius());g3_real(e->MinorRadius());
  G3_TRY(gp_Hypr g=e->Hypr();g3_ax2(g.Position());e->SetHypr(g));}
 if(auto e=occ::down_cast<Geom_Parabola>(c)){g3_real(e->Focal());g3_pnt(e->Focus());g3_real(e->Parameter());
  G3_TRY(g3_ax1(e->Directrix()));G3_TRY(e->SetFocal(value));g3_real(e->Focal());
  G3_TRY(gp_Parab g=e->Parab();g3_ax2(g.Position());e->SetParab(g));}
}
// Conic getters, the class setters with `value`, then SetLocation, SetAxis, SetPosition and Reverse.
void g3_conic_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> curve=g3_built_curve(r);if(curve.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_Conic> c=occ::down_cast<Geom_Conic>(curve);double value=r.real();gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt();gp_Ax2 f=r.ax2();
 g3_conic_common(c);g3_conic_specific(c,value);g3_curve_summary(c,u);
 c->SetLocation(p);g3_conic_common(c);G3_TRY(c->SetAxis(a));g3_conic_common(c);
 c->SetPosition(f);g3_conic_common(c);c->Reverse();g3_conic_common(c);g3_curve_summary(c,u);
 g3_dump(c,-1);std::cout<<'\n';
}
// The constructors of the conics from gp objects and the other native forms.
void g3_conic_build_case(std::istream& in){
 G3Reader r{in};int kind=r.integer();gp_Ax2 f=r.ax2();double a=r.real(),b=r.real();gp_Ax1 d=r.ax1();gp_Pnt p=r.pnt();
 occ::handle<Geom_Conic> c;
 G3_TRY(if(kind==1)c=new Geom_Circle(gp_Circ(f,a)));
 G3_TRY(if(kind==2)c=new Geom_Ellipse(gp_Elips(f,a,b)));
 G3_TRY(if(kind==3)c=new Geom_Hyperbola(gp_Hypr(f,a,b)));
 G3_TRY(if(kind==4)c=new Geom_Parabola(gp_Parab(f,a)));
 G3_TRY(if(kind==5)c=new Geom_Parabola(d,p));
 if(!c.IsNull()){g3_conic_common(c);g3_dump(c,-1);}
 std::cout<<'\n';
}
void g3_trimmed_state(const occ::handle<Geom_TrimmedCurve>& t,const std::vector<double>& u){
 g3_real(t->FirstParameter());g3_real(t->LastParameter());G3_TRY(g3_pnt(t->StartPoint()));G3_TRY(g3_pnt(t->EndPoint()));
 g3_name(t->BasisCurve());g3_curve_summary(t,u);g3_dump(t,-1);
}
// Trimmed curves: state, Copy (which re-trims), SetTrim, Reverse, Transform and the
// parameter transformations.
void g3_trimmed_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_TrimmedCurve> t=occ::down_cast<Geom_TrimmedCurve>(c);
 double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();gp_Trsf m=g3_trsf(r);
 g3_trimmed_state(t,u);G3_TRY(g3_trimmed_state(occ::down_cast<Geom_TrimmedCurve>(t->Copy()),u));
 G3_TRY(t->SetTrim(u1,u2,sense,adjust));g3_trimmed_state(t,u);G3_TRY(g3_trimmed_state(occ::down_cast<Geom_TrimmedCurve>(t->Copy()),u));
 G3_TRY(t->Reverse());g3_trimmed_state(t,u);G3_TRY(t->Transform(m));g3_trimmed_state(t,u);
 g3_real(t->TransformedParameter(0.5,m));g3_real(t->ParametricTransformation(m));std::cout<<'\n';
}
