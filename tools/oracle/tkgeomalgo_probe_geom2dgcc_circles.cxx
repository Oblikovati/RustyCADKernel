// Geom2dGcc circle solver probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom2d_gcc_circles.rs. The readers, the report macros
// and the muted builder are those of tkgeomalgo_probe_geom2dgcc.cxx and
// tkgeomalgo_probe_gccana.cxx.
template<class S> void ga_g2gcc_tancen_report(const S& s,int detail,bool same){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a;s.WhichQualifier(i,a);ga_int(a));
  if(detail&2){if constexpr(std::is_same_v<S,Geom2dGcc_Circ2dTanCen>){if(same){GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);}}
   else{GA_TANGENCY(s.Tangency1);}})
}
// `detail q1 x y tol`: Geom2dGcc_Circ2dTanCen (a Geom2d_CartesianPoint centre).
void ga_g2gcc_tancen_case(std::istream& input){
 GaReader r{input};int detail=r.integer();
 GA_TRY(Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);occ::handle<Geom2d_Point> p=ga_g2gcc_point(r);double tol=r.real();
  ga_g2gcc_tancen_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Circ2dTanCen(q1,p,tol);}),detail,true));
 ga_end();
}
// `detail q1 x y tol`: Geom2dGcc_Circ2dTanCenGeo.
void ga_g2gcc_tancen_geo_case(std::istream& input){
 GaReader r{input};int detail=r.integer();
 GA_TRY(Geom2dGcc_QCurve q1=ga_g2gcc_qc(r);gp_Pnt2d p=r.pnt2d();double tol=r.real();
  ga_g2gcc_tancen_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Circ2dTanCenGeo(q1,p,tol);}),detail,false));
 ga_end();
}
template<class S> void ga_g2gcc_2tan_report(const S& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b;s.WhichQualifier(i,a,b);ga_int(a);ga_int(b));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&4)GA_SAME_TANGENCY(s.IsTheSame2,s.Tangency2);
  if constexpr(!std::is_same_v<S,Geom2dGcc_Circ2d2TanRad>&&!std::is_same_v<S,Geom2dGcc_Circ2d2TanRadGeo>){if(detail&8)GA_G2_CENTRE_I(s.CenterOn3);})
}
// `kind detail args`: Geom2dGcc_Circ2d2TanRad 0 (q1, q2, R, tol), 1 (q1, point, R, tol),
// 2 (point, point, R, tol); 3 and 4 build kind 0, then Results of GccAna_Circ2d2TanRad
// (qcirc, qcirc, R, tol) or of Geom2dGcc_Circ2d2TanRadGeo (qcirc, q, R, tol).
void ga_g2gcc_2tanrad_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d2TanRad> s;
  if(kind==1){Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);occ::handle<Geom2d_Point> p=ga_g2gcc_point(r);double rad=r.real(),tol=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRad(q1,p,rad,tol);});}
  else if(kind==2){occ::handle<Geom2d_Point> p1=ga_g2gcc_point(r),p2=ga_g2gcc_point(r);double rad=r.real(),tol=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRad(p1,p2,rad,tol);});}
  else{Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r),q2=ga_g2gcc_q(r);double rad=r.real(),tol=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRad(q1,q2,rad,tol);});
   if(kind==3){GccEnt_QualifiedCirc a=ga_read_qcirc(r),b=ga_read_qcirc(r);double r2=r.real(),t2=r.real();
    GA_TRY(GccAna_Circ2d2TanRad c(a,b,r2,t2);s->Results(c));}
   if(kind==4){GccEnt_QualifiedCirc a=ga_read_qcirc(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);double r2=r.real(),t2=r.real();
    GA_TRY(auto c=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRadGeo(a,b,r2,t2);});s->Results(*c));}}
  ga_g2gcc_2tan_report(*s,detail));
 ga_end();
}
// `kind detail args`: Geom2dGcc_Circ2d2TanRadGeo 0 (qlin, q, R, tol), 1 (qcirc, q, R, tol),
// 2 (q, q, R, tol), 3 (q, x y, R, tol).
void ga_g2gcc_2tanrad_geo_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d2TanRadGeo> s;
  if(kind==0){GccEnt_QualifiedLin a=ga_read_qlin(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);double rad=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRadGeo(a,b,rad,tol);});}
  else if(kind==1){GccEnt_QualifiedCirc a=ga_read_qcirc(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);double rad=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRadGeo(a,b,rad,tol);});}
  else if(kind==2){Geom2dGcc_QCurve a=ga_g2gcc_qc(r),b=ga_g2gcc_qc(r);double rad=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRadGeo(a,b,rad,tol);});}
  else{Geom2dGcc_QCurve a=ga_g2gcc_qc(r);gp_Pnt2d p=r.pnt2d();double rad=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanRadGeo(a,p,rad,tol);});}
  ga_g2gcc_2tan_report(*s,detail));
 ga_end();
}
void ga_g2gcc_3tan_report(const Geom2dGcc_Circ2d3Tan& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_circ2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b,c;s.WhichQualifier(i,a,b,c);ga_int(a);ga_int(b);ga_int(c));
  if(detail&2)GA_SAME_TANGENCY(s.IsTheSame1,s.Tangency1);
  if(detail&4)GA_SAME_TANGENCY(s.IsTheSame2,s.Tangency2);
  if(detail&8)GA_SAME_TANGENCY(s.IsTheSame3,s.Tangency3);)
}
// `kind detail args`: Geom2dGcc_Circ2d3Tan 0 (q1, q2, q3, tol, p1, p2, p3), 1 (q1, q2,
// point, tol, p1, p2), 2 (q1, point, point, tol, p1), 3 (point, point, point, tol); 4 builds
// kind 0, then Results of GccAna_Circ2d3Tan (qcirc x3, tol) with ranks r1 r2 r3.
void ga_g2gcc_3tan_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d3Tan> s;
  if(kind==0||kind==4){Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r),q2=ga_g2gcc_q(r),q3=ga_g2gcc_q(r);double tol=r.real(),p1=r.real(),p2=r.real(),p3=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3Tan(q1,q2,q3,tol,p1,p2,p3);});
   if(kind==4){GccEnt_QualifiedCirc a=ga_read_qcirc(r),b=ga_read_qcirc(r),c=ga_read_qcirc(r);double t2=r.real();int r1=r.integer(),r2=r.integer(),r3=r.integer();
    GA_TRY(GccAna_Circ2d3Tan g(a,b,c,t2);s->Results(g,r1,r2,r3));}}
  else if(kind==1){Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r),q2=ga_g2gcc_q(r);occ::handle<Geom2d_Point> p=ga_g2gcc_point(r);double tol=r.real(),p1=r.real(),p2=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3Tan(q1,q2,p,tol,p1,p2);});}
  else if(kind==2){Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);occ::handle<Geom2d_Point> a=ga_g2gcc_point(r),b=ga_g2gcc_point(r);double tol=r.real(),p1=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3Tan(q1,a,b,tol,p1);});}
  else{occ::handle<Geom2d_Point> a=ga_g2gcc_point(r),b=ga_g2gcc_point(r),c=ga_g2gcc_point(r);double tol=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3Tan(a,b,c,tol);});}
  ga_g2gcc_3tan_report(*s,detail));
 ga_end();
}
void ga_g2gcc_3tan_iter_report(const Geom2dGcc_Circ2d3TanIter& s){
 ga_flag(s.IsDone());ga_circ2d(s.ThisSolution());
 GA_TRY(GccEnt_Position a,b,c;s.WhichQualifier(a,b,c);ga_int(a);ga_int(b);ga_int(c));
 GA_TRY(ga_flag(s.IsTheSame1()));GA_TRY(ga_flag(s.IsTheSame2()));GA_TRY(ga_flag(s.IsTheSame3()));
 GA_G2_TANGENCY(s.Tangency1);GA_G2_TANGENCY(s.Tangency2);GA_G2_TANGENCY(s.Tangency3);
}
// `kind args ... tol`: Geom2dGcc_Circ2d3TanIter; kind 0 (q q q p1 p2 p3), 1 (qcirc q q ...),
// 2 (qcirc qcirc q ...), 3 (qcirc qlin q ...), 4 (qcirc q point p1 p2), 5 (qlin qlin q ...),
// 6 (qlin q q ...), 7 (qlin q point p1 p2), 8 (q point point p1), 9 (q q point p1 p2).
void ga_g2gcc_3tan_iter_case(std::istream& input){
 GaReader r{input};int kind=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d3TanIter> s;
  if(kind==0){Geom2dGcc_QCurve a=ga_g2gcc_qc(r),b=ga_g2gcc_qc(r),c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==1){GccEnt_QualifiedCirc a=ga_read_qcirc(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r),c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==2){GccEnt_QualifiedCirc a=ga_read_qcirc(r),b=ga_read_qcirc(r);Geom2dGcc_QCurve c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==3){GccEnt_QualifiedCirc a=ga_read_qcirc(r);GccEnt_QualifiedLin b=ga_read_qlin(r);Geom2dGcc_QCurve c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==4){GccEnt_QualifiedCirc a=ga_read_qcirc(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);gp_Pnt2d c=r.pnt2d();double p1=r.real(),p2=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,t);});}
  else if(kind==5){GccEnt_QualifiedLin a=ga_read_qlin(r),b=ga_read_qlin(r);Geom2dGcc_QCurve c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==6){GccEnt_QualifiedLin a=ga_read_qlin(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r),c=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),p3=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,p3,t);});}
  else if(kind==7){GccEnt_QualifiedLin a=ga_read_qlin(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);gp_Pnt2d c=r.pnt2d();double p1=r.real(),p2=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,t);});}
  else if(kind==8){Geom2dGcc_QCurve a=ga_g2gcc_qc(r);gp_Pnt2d b=r.pnt2d(),c=r.pnt2d();double p1=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,t);});}
  else{Geom2dGcc_QCurve a=ga_g2gcc_qc(r),b=ga_g2gcc_qc(r);gp_Pnt2d c=r.pnt2d();double p1=r.real(),p2=r.real(),t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d3TanIter(a,b,c,p1,p2,t);});}
  ga_g2gcc_3tan_iter_report(*s));
 ga_end();
}
// `kind detail args`: Geom2dGcc_Circ2d2TanOn 0 (q1, q2, oncurve, tol, p1, p2, pOn), 1 (q1,
// point, oncurve, tol, p1, pOn), 2 (point, point, oncurve, tol); 3 and 4 build kind 0, then
// Results of GccAna_Circ2d2TanOn (qcirc, qcirc, line, tol) or of Geom2dGcc_Circ2d2TanOnGeo
// (qlin, qlin, oncurve, tol).
void ga_g2gcc_2tanon_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d2TanOn> s;
  if(kind==1){Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);occ::handle<Geom2d_Point> p=ga_g2gcc_point(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double tol=r.real(),p1=r.real(),pon=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOn(q1,p,on,tol,p1,pon);});}
  else if(kind==2){occ::handle<Geom2d_Point> a=ga_g2gcc_point(r),b=ga_g2gcc_point(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double tol=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOn(a,b,on,tol);});}
  else{Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r),q2=ga_g2gcc_q(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double tol=r.real(),p1=r.real(),p2=r.real(),pon=r.real();
   s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOn(q1,q2,on,tol,p1,p2,pon);});
   if(kind==3){GccEnt_QualifiedCirc a=ga_read_qcirc(r),b=ga_read_qcirc(r);gp_Lin2d l=ga_read_lin2d(r);double t2=r.real();
    GA_TRY(GccAna_Circ2d2TanOn c(a,b,l,t2);s->Results(c));}
   if(kind==4){GccEnt_QualifiedLin a=ga_read_qlin(r),b=ga_read_qlin(r);Geom2dAdaptor_Curve on2=ga_hatch_adaptor(r);double t2=r.real();
    GA_TRY(auto c=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on2,t2);});s->Results(*c));}}
  ga_g2gcc_2tan_report(*s,detail));
 ga_end();
}
// `kind detail args oncurve tol`: Geom2dGcc_Circ2d2TanOnGeo 0 (qcirc, qcirc), 1 (qcirc,
// qlin), 2 (qcirc, point), 3 (qlin, qlin), 4 (qlin, point), 5 (point, point).
void ga_g2gcc_2tanon_geo_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d2TanOnGeo> s;
  if(kind==0){GccEnt_QualifiedCirc a=ga_read_qcirc(r),b=ga_read_qcirc(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  else if(kind==1){GccEnt_QualifiedCirc a=ga_read_qcirc(r);GccEnt_QualifiedLin b=ga_read_qlin(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  else if(kind==2){GccEnt_QualifiedCirc a=ga_read_qcirc(r);gp_Pnt2d b=r.pnt2d();Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  else if(kind==3){GccEnt_QualifiedLin a=ga_read_qlin(r),b=ga_read_qlin(r);Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  else if(kind==4){GccEnt_QualifiedLin a=ga_read_qlin(r);gp_Pnt2d b=r.pnt2d();Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  else{gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);double t=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Circ2d2TanOnGeo(a,b,on,t);});}
  ga_g2gcc_2tan_report(*s,detail));
 ga_end();
}
void ga_g2gcc_2tanon_iter_report(const Geom2dGcc_Circ2d2TanOnIter& s){
 ga_flag(s.IsDone());ga_circ2d(s.ThisSolution());
 GA_TRY(GccEnt_Position a,b;s.WhichQualifier(a,b);ga_int(a);ga_int(b));
 GA_TRY(ga_flag(s.IsTheSame1()));GA_TRY(ga_flag(s.IsTheSame2()));
 GA_G2_TANGENCY(s.Tangency1);GA_G2_TANGENCY(s.Tangency2);GA_G2_CENTRE(s.CenterOn3);
}
// The first argument of Geom2dGcc_Circ2d2TanOnIter by kind % 4: 0 qcirc, 1 qlin, 2 q, 3 q (a
// point second); the locus by kind / 4: 0 line, 1 circle, 2 curve.
template<class L> Geom2dGcc_Circ2d2TanOnIter* ga_g2gcc_on_iter(GaReader& r,int first,const L& on){
 if(first==3){Geom2dGcc_QCurve a=ga_g2gcc_qc(r);gp_Pnt2d p=r.pnt2d();double p1=r.real(),p2=r.real(),t=r.real();GaMute m;return new Geom2dGcc_Circ2d2TanOnIter(a,p,on,p1,p2,t);}
 double p1=0,p2=0,p3=0,t=0;auto tail=[&]{p1=r.real();p2=r.real();p3=r.real();t=r.real();};
 if(first==0){GccEnt_QualifiedCirc a=ga_read_qcirc(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);tail();GaMute m;return new Geom2dGcc_Circ2d2TanOnIter(a,b,on,p1,p2,p3,t);}
 if(first==1){GccEnt_QualifiedLin a=ga_read_qlin(r);Geom2dGcc_QCurve b=ga_g2gcc_qc(r);tail();GaMute m;return new Geom2dGcc_Circ2d2TanOnIter(a,b,on,p1,p2,p3,t);}
 Geom2dGcc_QCurve a=ga_g2gcc_qc(r),b=ga_g2gcc_qc(r);tail();GaMute m;return new Geom2dGcc_Circ2d2TanOnIter(a,b,on,p1,p2,p3,t);
}
// `kind locus args... tol`: the locus first (a line, a circle or a curve), then the arguments.
void ga_g2gcc_2tanon_iter_case(std::istream& input){
 GaReader r{input};int kind=r.integer();int first=kind%4,locus=kind/4;
 GA_TRY(std::unique_ptr<Geom2dGcc_Circ2d2TanOnIter> s;
  if(locus==0){gp_Lin2d on=ga_read_lin2d(r);s.reset(ga_g2gcc_on_iter(r,first,on));}
  else if(locus==1){gp_Circ2d on=ga_read_circ2d(r);s.reset(ga_g2gcc_on_iter(r,first,on));}
  else{Geom2dAdaptor_Curve on=ga_hatch_adaptor(r);s.reset(ga_g2gcc_on_iter(r,first,on));}
  ga_g2gcc_2tanon_iter_report(*s));
 ga_end();
}
