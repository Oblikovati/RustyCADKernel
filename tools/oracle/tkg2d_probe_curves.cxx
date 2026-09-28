// TKG2d oracle: curve evaluation, curve transformations, conics, trimmed and offset
// curves, and Geom2d_OffsetCurveUtils (tkg2d_probe.cxx).
// Reads a curve spec, reporting a construction failure and ending the line.
occ::handle<Geom2d_Curve> g2_built(G2Reader& r){
 try{return g2_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return nullptr;}
}
std::vector<double> g2_params(G2Reader& r){return r.reals();}
void g2_curve_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;
 std::vector<double> u=g2_params(r);std::cout<<c->DynamicType()->Name()<<' ';g2_summary(c,u);g2_dump(c,-1);g2_dump(c,1);
 std::cout<<'\n';
}
std::vector<double> g2_probe_params;
void g2_show_curve(const occ::handle<Geom2d_Geometry>& g){g2_summary(occ::down_cast<Geom2d_Curve>(g),g2_probe_params);}
void g2_transform_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;g2_probe_params=g2_params(r);
 gp_Trsf2d t=g2_trsf(r);
 for(double u:g2_probe_params)g2_real(c->TransformedParameter(u,t));g2_real(c->ParametricTransformation(t));
 G2_TRY(g2_show_curve(c->Reversed()));G2_TRY(c->Reverse();g2_show_curve(c));g2_moves(c,r,g2_show_curve);std::cout<<'\n';
}
void g2_conic_common(const occ::handle<Geom2d_Conic>& c){
 g2_ax(c->XAxis());g2_ax(c->YAxis());g2_pnt(c->Location());g2_ax22(c->Position());G2_TRY(g2_real(c->Eccentricity()));
}
void g2_conic_specific(const occ::handle<Geom2d_Conic>& c,double value){
 if(auto e=occ::down_cast<Geom2d_Circle>(c)){g2_real(e->Radius());G2_TRY(e->SetRadius(value));g2_real(e->Radius());
  gp_Circ2d g=e->Circ2d();g2_ax22(g.Axis());g2_real(g.Radius());e->SetCirc2d(g);}
 if(auto e=occ::down_cast<Geom2d_Ellipse>(c)){g2_real(e->MajorRadius());g2_real(e->MinorRadius());G2_TRY(g2_ax(e->Directrix1()));
  G2_TRY(g2_ax(e->Directrix2()));g2_real(e->Focal());g2_pnt(e->Focus1());g2_pnt(e->Focus2());g2_real(e->Parameter());
  G2_TRY(e->SetMinorRadius(value));G2_TRY(e->SetMajorRadius(value));g2_real(e->MajorRadius());g2_real(e->MinorRadius());
  G2_TRY(gp_Elips2d g=e->Elips2d();g2_ax22(g.Axis());e->SetElips2d(g));}
 if(auto e=occ::down_cast<Geom2d_Hyperbola>(c)){g2_real(e->MajorRadius());g2_real(e->MinorRadius());G2_TRY(g2_ax(e->Asymptote1()));
  G2_TRY(g2_ax(e->Asymptote2()));G2_TRY(g2_ax22(e->ConjugateBranch1().Axis()));G2_TRY(g2_ax22(e->ConjugateBranch2().Axis()));
  G2_TRY(g2_ax(e->Directrix1()));G2_TRY(g2_ax(e->Directrix2()));g2_real(e->Focal());g2_pnt(e->Focus1());g2_pnt(e->Focus2());
  G2_TRY(g2_ax22(e->OtherBranch().Axis()));G2_TRY(g2_real(e->Parameter()));G2_TRY(e->SetMajorRadius(value));
  G2_TRY(e->SetMinorRadius(-value));g2_real(e->MajorRadius());g2_real(e->MinorRadius());
  G2_TRY(gp_Hypr2d g=e->Hypr2d();g2_ax22(g.Axis());e->SetHypr2d(g));}
 if(auto e=occ::down_cast<Geom2d_Parabola>(c)){g2_real(e->Focal());g2_pnt(e->Focus());g2_real(e->Parameter());
  G2_TRY(g2_ax(e->Directrix()));G2_TRY(e->SetFocal(value));g2_real(e->Focal());
  G2_TRY(gp_Parab2d g=e->Parab2d();g2_ax22(g.Axis());e->SetParab2d(g));}
}
void g2_conic_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> curve=g2_built(r);if(curve.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_Conic> c=occ::down_cast<Geom2d_Conic>(curve);double value=r.real();gp_Ax2d a=r.ax();gp_Pnt2d p=r.pnt();
 g2_conic_common(c);g2_conic_specific(c,value);g2_summary(c,u);
 c->SetLocation(p);g2_conic_common(c);G2_TRY(c->SetXAxis(a));g2_conic_common(c);G2_TRY(c->SetYAxis(a));g2_conic_common(c);
 c->SetAxis(gp_Ax22d(p,a.Direction(),true));g2_conic_common(c);c->Reverse();g2_conic_common(c);g2_summary(c,u);
 g2_dump(c,-1);std::cout<<'\n';
}
void g2_trimmed_state(const occ::handle<Geom2d_TrimmedCurve>& t,const std::vector<double>& u){
 g2_real(t->FirstParameter());g2_real(t->LastParameter());G2_TRY(g2_pnt(t->StartPoint()));G2_TRY(g2_pnt(t->EndPoint()));
 std::cout<<t->BasisCurve()->DynamicType()->Name()<<' ';g2_summary(t,u);g2_dump(t,-1);
}
void g2_trimmed_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_TrimmedCurve> t=occ::down_cast<Geom2d_TrimmedCurve>(c);
 double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();gp_Trsf2d m=g2_trsf(r);
 g2_trimmed_state(t,u);G2_TRY(g2_trimmed_state(occ::down_cast<Geom2d_TrimmedCurve>(t->Copy()),u));
 G2_TRY(t->SetTrim(u1,u2,sense,adjust));g2_trimmed_state(t,u);G2_TRY(g2_trimmed_state(occ::down_cast<Geom2d_TrimmedCurve>(t->Copy()),u));
 G2_TRY(t->Reverse());g2_trimmed_state(t,u);G2_TRY(t->Transform(m));g2_trimmed_state(t,u);
 g2_real(t->TransformedParameter(0.5,m));g2_real(t->ParametricTransformation(m));std::cout<<'\n';
}
void g2_offset_state(const occ::handle<Geom2d_OffsetCurve>& o,const std::vector<double>& u){
 g2_real(o->Offset());g2_int(o->GetBasisCurveContinuity());std::cout<<o->BasisCurve()->DynamicType()->Name()<<' ';
 g2_flag(o->HasEvalRepresentation());g2_summary(o,u);g2_dump(o,-1);
}
void g2_offset_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_OffsetCurve> o=occ::down_cast<Geom2d_OffsetCurve>(c);double d=r.real();bool nocheck=r.flag();
 gp_Trsf2d m=g2_trsf(r);occ::handle<Geom2d_Curve> basis=g2_curve(r);
 g2_offset_state(o,u);g2_offset_state(occ::down_cast<Geom2d_OffsetCurve>(o->Copy()),u);o->SetOffsetValue(d);g2_offset_state(o,u);
 G2_TRY(o->Reverse());g2_offset_state(o,u);G2_TRY(o->Transform(m));g2_offset_state(o,u);
 G2_TRY(o->SetBasisCurve(basis,nocheck));g2_offset_state(o,u);G2_TRY(o->SetBasisCurve(o,nocheck));g2_offset_state(o,u);
 g2_real(o->TransformedParameter(0.5,m));g2_real(o->ParametricTransformation(m));std::cout<<'\n';
}
void g2_offset_utils_case(std::istream& in){
 G2Reader r{in};gp_Pnt2d p=r.pnt();gp_Vec2d d1=r.vec(),d2=r.vec(),d3=r.vec(),d4=r.vec();double offset=r.real();bool change=r.flag();
 gp_Pnt2d v=p;G2_TRY(g2_flag(Geom2d_OffsetCurveUtils::CalculateD0(v,d1,offset));g2_pnt(v));
 gp_Vec2d a1=d1;v=p;g2_flag(Geom2d_OffsetCurveUtils::CalculateD1(v,a1,d2,offset));g2_pnt(v);g2_vec(a1);
 gp_Vec2d a2=d2;a1=d1;v=p;g2_flag(Geom2d_OffsetCurveUtils::CalculateD2(v,a1,a2,d3,change,offset));g2_pnt(v);g2_vec(a1);g2_vec(a2);
 gp_Vec2d a3=d3;a2=d2;a1=d1;v=p;g2_flag(Geom2d_OffsetCurveUtils::CalculateD3(v,a1,a2,a3,d4,change,offset));
 g2_pnt(v);g2_vec(a1);g2_vec(a2);g2_vec(a3);std::cout<<'\n';
}
