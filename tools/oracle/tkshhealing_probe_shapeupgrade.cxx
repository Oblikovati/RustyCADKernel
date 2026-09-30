// ShapeUpgrade package functions, ShapeUpgrade_Tool, ShapeUpgrade_RemoveLocations and
// ShapeUpgrade_SplitCurve probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_upgrade.rs.
//
// `su_c0split dim null (B-spline)`: C0BSplineToSequenceOfC1BSplineCurve on a 3d (dim 3) or
// 2d (dim 2) B-spline `degree periodic n (pole weight)... nk (knot mult)...` (a pole is x y
// z or u v; weights all 1 give a non-rational curve), or on a null handle; prints the
// result, then each piece (degree, rational flag, knots and multiplicities, poles and
// weights, curve digest) and, for 3d, the periodic flag and the pole count of the input.
// `su_tool nops (op)...`: 0 p: SetPrecision; 1 t: SetMinTolerance; 2 t: SetMaxTolerance;
// 3 t: LimitTolerance printed; 4 p min max ctx: Set(tool) from a tool with these values
// and a new context when ctx; 5: SetContext(new ShapeBuild_ReShape); then the precision,
// the tolerances, whether the context is null and the dynamic type.
// `su_removeloc (prelude) program nops (op)...`: 0 level: SetRemoveLevel, RemoveLevel
// printed; 1 s: Remove(s), then the digest of GetResult(); 2 s: ModifiedShape(s): its
// IsSame index in the table and digest; 3: the extent of the map and IsBound of each
// table shape; 4: the digest of GetResult().
// `su_splitcurve nops (op)...`: 0 first last: Init; 1 n v...: SetSplitValues of a new
// sequence; 2: SetSplitValues(null); 3: SetSplitValues(SplitValues()); 4 segment:
// Perform; 5: Compute; 6 segment: Build; then the split values (null flag, count,
// values), the 19 status flags and the dynamic type.
#include <ShapeUpgrade.hxx>
#include <ShapeUpgrade_RemoveLocations.hxx>
#include <ShapeUpgrade_SplitCurve.hxx>
#include <ShapeUpgrade_Tool.hxx>
template<class P> void su_poles(GaReader& r,int n,NCollection_Array1<P>& poles,NCollection_Array1<double>& w);
template<> void su_poles(GaReader& r,int n,NCollection_Array1<gp_Pnt>& poles,NCollection_Array1<double>& w){
 for(int i=1;i<=n;++i){poles(i)=r.pnt();w(i)=r.real();}}
template<> void su_poles(GaReader& r,int n,NCollection_Array1<gp_Pnt2d>& poles,NCollection_Array1<double>& w){
 for(int i=1;i<=n;++i){poles(i)=r.pnt2d();w(i)=r.real();}}
template<class C,class P> occ::handle<C> su_bspline(GaReader& r){
 int degree=r.integer();bool periodic=r.flag();int n=r.integer();
 NCollection_Array1<P> poles(1,n);NCollection_Array1<double> w(1,n);su_poles(r,n,poles,w);
 NCollection_Array1<double> k(1,1);NCollection_Array1<int> m(1,1);tp_curve_knots(r,k,m);
 return new C(poles,w,k,m,degree,periodic);}
void su_pole(const gp_Pnt& p){ga_pnt(p);}
void su_pole(const gp_Pnt2d& p){ga_pnt2d(p);}
template<class C> void su_piece(const occ::handle<C>& bs){
 ga_int(bs->Degree());ga_flag(bs->IsRational());ga_int(bs->NbKnots());
 for(int i=1;i<=bs->NbKnots();++i){ga_real(bs->Knot(i));ga_int(bs->Multiplicity(i));}
 ga_int(bs->NbPoles());for(int i=1;i<=bs->NbPoles();++i){su_pole(bs->Pole(i));ga_real(bs->Weight(i));}}
void su_c0split_case(std::istream& input){
 GaReader r{input};int dim=r.integer();bool null=r.flag();
 if(dim==3){occ::handle<Geom_BSplineCurve> bs;if(!null)bs=su_bspline<Geom_BSplineCurve,gp_Pnt>(r);
  occ::handle<NCollection_HSequence<occ::handle<Geom_BoundedCurve>>> seq;
  GA_TRY(ga_flag(ShapeUpgrade::C0BSplineToSequenceOfC1BSplineCurve(bs,seq));ga_flag(seq.IsNull());
   if(!seq.IsNull()){ga_int(seq->Length());for(int i=1;i<=seq->Length();++i){
    occ::handle<Geom_BSplineCurve> piece=occ::down_cast<Geom_BSplineCurve>(seq->Value(i));su_piece(piece);shc_curve(piece);}})
  if(!bs.IsNull()){ga_flag(bs->IsPeriodic());ga_int(bs->NbPoles());}}
 else{occ::handle<Geom2d_BSplineCurve> bs;if(!null)bs=su_bspline<Geom2d_BSplineCurve,gp_Pnt2d>(r);
  occ::handle<NCollection_HSequence<occ::handle<Geom2d_BoundedCurve>>> seq;
  GA_TRY(ga_flag(ShapeUpgrade::C0BSplineToSequenceOfC1BSplineCurve(bs,seq));ga_flag(seq.IsNull());
   if(!seq.IsNull()){ga_int(seq->Length());for(int i=1;i<=seq->Length();++i){
    occ::handle<Geom2d_BSplineCurve> piece=occ::down_cast<Geom2d_BSplineCurve>(seq->Value(i));su_piece(piece);shc_curve2d(piece);}})
  if(!bs.IsNull()){ga_flag(bs->IsPeriodic());ga_int(bs->NbPoles());}}
 ga_end();}
void su_tool_case(std::istream& input){
 GaReader r{input};occ::handle<ShapeUpgrade_Tool> tool=new ShapeUpgrade_Tool();int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0)tool->SetPrecision(r.real());
  else if(op==1)tool->SetMinTolerance(r.real());
  else if(op==2)tool->SetMaxTolerance(r.real());
  else if(op==3)ga_real(tool->LimitTolerance(r.real()));
  else if(op==4){occ::handle<ShapeUpgrade_Tool> other=new ShapeUpgrade_Tool();other->SetPrecision(r.real());
   other->SetMinTolerance(r.real());other->SetMaxTolerance(r.real());if(r.flag())other->SetContext(new ShapeBuild_ReShape());
   tool->Set(other);ga_flag(tool->Context()==other->Context());}
  else tool->SetContext(new ShapeBuild_ReShape());}
 ga_real(tool->Precision());ga_real(tool->MinTolerance());ga_real(tool->MaxTolerance());
 ga_flag(tool->Context().IsNull());ga_text(tool->DynamicType()->Name());ga_end();}
void su_removeloc_case(std::istream& input){
 GaReader r{input};TpShapes t=shc_shapes(r);occ::handle<ShapeUpgrade_RemoveLocations> rl=new ShapeUpgrade_RemoveLocations();
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0){rl->SetRemoveLevel(TopAbs_ShapeEnum(r.integer()));ga_int(rl->RemoveLevel());}
  else if(op==1){TopoDS_Shape s=sh_shape(t,r.integer());GA_TRY(ga_flag(rl->Remove(s)))tp_shape(rl->GetResult());}
  else if(op==2){TopoDS_Shape m=rl->ModifiedShape(sh_shape(t,r.integer()));ga_int(tp_same_index(t,m));tp_shape(m);}
  else if(op==3){ga_int(rl->GetModifiedShapesMap().Extent());for(const TopoDS_Shape& s:t.shapes)ga_flag(rl->GetModifiedShapesMap().IsBound(s));}
  else tp_shape(rl->GetResult());}
 ga_end();}
void su_splitcurve_case(std::istream& input){
 GaReader r{input};occ::handle<ShapeUpgrade_SplitCurve> sc=new ShapeUpgrade_SplitCurve();int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0){double f=r.real();sc->Init(f,r.real());}
  else if(op==1){occ::handle<NCollection_HSequence<double>> v=new NCollection_HSequence<double>;int n=r.integer();
   for(int i=0;i<n;++i)v->Append(r.real());sc->SetSplitValues(v);}
  else if(op==2)sc->SetSplitValues(occ::handle<NCollection_HSequence<double>>());
  else if(op==3)sc->SetSplitValues(sc->SplitValues());
  else if(op==4)sc->Perform(r.flag());
  else if(op==5)sc->Compute();
  else sc->Build(r.flag());}
 const occ::handle<NCollection_HSequence<double>>& v=sc->SplitValues();ga_flag(v.IsNull());
 if(!v.IsNull()){ga_int(v->Length());for(int i=1;i<=v->Length();++i)ga_real(v->Value(i));}
 for(int k=0;k<19;++k)ga_flag(sc->Status(ShapeExtend_Status(k)));
 ga_text(sc->DynamicType()->Name());ga_end();}
