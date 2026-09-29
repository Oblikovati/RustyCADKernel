// Geom2dHatch_Hatcher and Geom2dHatch_Hatching probe operations (tkgeomalgo_probe.cxx); the
// Rust twin is crates/tkgeomalgo/examples/ga_probe/geom2d_hatcher.rs. The readers and
// printers come from tkgeomalgo_probe_geom2dhatch.cxx and tkgeomalgo_probe_hatchgen.cxx.
// Every hatcher member runs under GaMute (see tkgeomalgo_probe_geom2dhatch.cxx).
void ga_hatcher_hatching_out(const Geom2dHatch_Hatcher& h,int i){
 GA_TRY(ga_adaptor_out(h.HatchingCurve(i)));
 GA_TRY(ga_flag(h.TrimDone(i));ga_flag(h.TrimFailed(i));ga_flag(h.IsDone(i));ga_int(h.Status(i));int np=h.NbPoints(i);ga_int(np);
  for(int k=0;k<=np+1;k++)GA_TRY(ga_hgen_ph(h.Point(i,k))));
 GA_TRY(int nd=h.NbDomains(i);ga_int(nd);for(int k=0;k<=nd+1;k++)GA_TRY(ga_hgen_domain_out(h.Domain(i,k))));
}
// The settings, every element curve and hatching from index 0 to count + 1, the dump.
void ga_hatcher_report(const Geom2dHatch_Hatcher& h,int ne,int nh){
 ga_real(h.Confusion2d());ga_real(h.Confusion3d());ga_flag(h.KeepPoints());ga_flag(h.KeepSegments());
 for(int i=0;i<=ne+1;i++)GA_TRY(ga_adaptor_out(h.ElementCurve(i)));
 for(int i=0;i<=nh+1;i++)ga_hatcher_hatching_out(h,i);
 ga_captured([&]{h.Dump();});
}
// Exposes the protected Element and Hatching members.
struct GaHatcher:Geom2dHatch_Hatcher{
 using Geom2dHatch_Hatcher::Geom2dHatch_Hatcher;using Geom2dHatch_Hatcher::Element;using Geom2dHatch_Hatcher::Hatching;
};
// One hatcher operation `op value`: 0 Trim(), 1 ComputeDomains(), 2 Trim(IndH), 3
// ComputeDomains(IndH), 4 RemElement, 5 RemHatching, 6 Confusion2d, 7 Confusion3d, 8
// KeepPoints, 9 KeepSegments, 10 Intersector(value, value), 11 Clear, 12 ClrElements, 13
// ClrHatchings, 14 AddHatching of the X line at y = value, 15 Trim of that line (the index
// printed for 14 and 15), 16 the protected Element(IndE) reversed, 17 the protected
// Hatching(IndH) marked not trimmed.
int ga_hatcher_op(GaHatcher& h,int op,double v){
 int i=int(v);occ::handle<Geom2d_Line> line=new Geom2d_Line(gp_Pnt2d(0.,v),gp_Dir2d(1.,0.));
 GaMute m;
 switch(op){case 0:h.Trim();break;case 1:h.ComputeDomains();break;case 2:h.Trim(i);break;case 3:h.ComputeDomains(i);break;
  case 4:h.RemElement(i);break;case 5:h.RemHatching(i);break;case 6:h.Confusion2d(v);break;case 7:h.Confusion3d(v);break;
  case 8:h.KeepPoints(v!=0.);break;case 9:h.KeepSegments(v!=0.);break;case 10:h.Intersector(Geom2dHatch_Intersector(v,v));break;
  case 11:h.Clear();break;case 12:h.ClrElements();break;case 13:h.ClrHatchings();break;
  case 14:return h.AddHatching(Geom2dAdaptor_Curve(line));case 15:return h.Trim(Geom2dAdaptor_Curve(line));
  case 16:h.Element(i).Orientation(TopAbs_REVERSED);break;default:h.Hatching(i).TrimDone(false);}
 return -1;
}
// `conf2d conf3d keepP keepS ne (orientation curve u0 u1)*ne nh (curve u0 u1)*nh nops (op value)*`:
// a hatcher with the elements (AddElement of the Geom2d curve when u0 > u1) and hatchings,
// the operations in turn, then the report.
void ga_g2hatch_hatcher_case(std::istream& input){
 GaReader r{input};
 GA_TRY(double c2=r.real(),c3=r.real();bool kp=r.flag(),ks=r.flag();
  GaHatcher h(Geom2dHatch_Intersector(1e-7,1e-7),c2,c3,kp,ks);int ne=r.integer();
  for(int k=0;k<ne;k++){TopAbs_Orientation o=ga_orient(r);occ::handle<Geom2d_Curve> c=ga_curve2d(r);double u0=r.real(),u1=r.real();
   if(u0>u1)ga_int(h.AddElement(c,o));else ga_int(h.AddElement(u0<u1?Geom2dAdaptor_Curve(c,u0,u1):Geom2dAdaptor_Curve(c),o));}
  int nh=r.integer();for(int k=0;k<nh;k++)ga_int(h.AddHatching(ga_hatch_adaptor(r)));
  int nops=r.integer();for(int k=0;k<nops;k++){int op=r.integer();double v=r.real();GA_TRY(int k=ga_hatcher_op(h,op,v);if(k>=0)ga_int(k));}
  ga_real(h.Intersector().ConfusionTolerance());ga_real(h.ChangeIntersector().TangencyTolerance());ga_hatcher_report(h,ne,nh));
 ga_end();
}
// `curve u0 u1 conf n (param position before after begin end ne (index param)*ne)*n rem`: a
// hatching with the points added under `conf`, its flags and status, domains made of its
// points, RemPoint(rem), RemDomain(1), ChangePoint(1), ClrPoints and ClassificationPoint.
void ga_g2hatch_hatching_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dHatch_Hatching h(ga_hatch_adaptor(r));double conf=r.real();int n=r.integer();
  for(int k=0;k<n;k++){HatchGen_PointOnHatching p;p.SetParameter(r.real());p.SetPosition(ga_orient(r));
   p.SetStateBefore(TopAbs_State(r.integer()));p.SetStateAfter(TopAbs_State(r.integer()));p.SetSegmentBeginning(r.flag());
   p.SetSegmentEnd(r.flag());int ne=r.integer();
   for(int j=0;j<ne;j++){HatchGen_PointOnElement e;e.SetIndex(r.integer());e.SetParameter(r.real());p.AddPoint(e,conf);}
   h.AddPoint(p,conf);}
  int rem=r.integer();ga_flag(h.TrimDone());ga_flag(h.TrimFailed());ga_flag(h.IsDone());ga_int(h.Status());
  int np=h.NbPoints();ga_int(np);for(int k=0;k<=np+1;k++)GA_TRY(ga_hgen_ph(h.Point(k)));
  h.TrimDone(true);h.TrimFailed(true);ga_int(h.Status());h.TrimFailed(false);ga_int(h.Status());h.Status(HatchGen_IncoherentParity);
  ga_int(h.Status());h.IsDone(true);
  for(int k=1;k<=np;k++){GA_TRY(h.AddDomain(HatchGen_Domain(h.Point(k),k%2==1)));}
  h.AddDomain(HatchGen_Domain());int nd=h.NbDomains();ga_int(nd);for(int k=0;k<=nd+1;k++)GA_TRY(ga_hgen_domain_out(h.Domain(k)));
  GA_TRY(h.RemDomain(1));GA_TRY(h.RemDomain(nd+5));ga_int(h.NbDomains());GA_TRY(h.RemPoint(rem));ga_flag(h.IsDone());
  ga_int(h.NbDomains());ga_int(h.NbPoints());GA_TRY(h.ChangePoint(1).SetParameter(-7.);ga_hgen_ph(h.Point(1)));
  h.IsDone(true);h.AddDomain(HatchGen_Domain());h.ClrPoints();ga_flag(h.TrimDone());ga_flag(h.TrimFailed());ga_flag(h.IsDone());
  ga_int(h.NbDomains());ga_int(h.NbPoints());ga_int(h.Status());GA_TRY(ga_pnt2d(h.ClassificationPoint()));
  ga_adaptor_out(h.Curve());ga_adaptor_out(Geom2dHatch_Hatching().Curve()));
 ga_end();
}
