// ShapeAnalysis_Geom, ShapeAnalysis_ShapeTolerance, ShapeAnalysis_Shell,
// ShapeAnalysis_WireOrder and ShapeAnalysis_FreeBoundData probe operations; the Rust twin
// is crates/tkshhealing/examples/sh_probe/analysis_basic.rs.
//
// `sh_geom 0 n (x y z)...`: NearestPlane; prints the flag, then the plane (location,
// direction, X direction) and Dmax when it succeeds.
// `sh_geom 1 null rl ru cl cu (value)... unit prec`: PositionTrsf on a matrix of bounds
// rl..ru x cl..cu (a null handle when null is 1); prints the flag, the form, the scale
// factor and the 12 values of the transformation, or the exception.
//
// `sh_shapetol program nops (op)...` over the shapes of a TKTopAlgo shape program, each op
// in GA_TRY: 0 s mode type: Tolerance; 1 s value type: OverTolerance; 2 s min max type:
// InTolerance (a list is printed as its length and the IsSame index and orientation of each shape);
// 3: InitTolerance; 4 s type: AddTolerance; 5: GlobalTolerance of the modes -1, 0 and 1.
// A tool made with ShapeAnalysis_ShapeTolerance() is used only after InitTolerance or
// Tolerance: its sum is uninitialized natively before.
//
// `sh_shell program nops (op)...`: 0 s: LoadShells; 1 s alsofree internal:
// CheckOrientedShells (prints the result); 2: Clear; 3 s: IsLoaded; 4 num: Loaded; then
// NbLoaded, the loaded shells, HasBadEdges, BadEdges, HasFreeEdges, FreeEdges and
// HasConnectedEdges (a compound as the IsSame index and orientation of each child).
//
// `sh_wireorder mode3d tol modeboth nops (op)...`: 0 (xyz xyz): Add 3D; 1 (xy xy): Add 2D;
// 2 (xyz xyz xy xy): Add both; 3 closed: Perform; 4 mode3d tol both: SetMode; 5: Clear;
// 6 gap: SetChains; 7 gap: SetCouples; 8 keep: KeepLoopsMode() = keep; then the digest
// (sh_wo_digest).
//
// `sh_fbdata program w nops (op)... probe`: ShapeAnalysis_FreeBoundData() (w = -2) or of
// the wire w; 0 v: SetArea; 1 v: SetPerimeter; 2 v: SetRatio; 3 v: SetWidth; 4 w v:
// AddNotch; 5: Clear (the handle of Notches taken before is printed after it); 6 w:
// SetFreeBound; then the digest, with NotchWidth of the wire `probe`.
#include <ShapeAnalysis_FreeBoundData.hxx>
#include <ShapeAnalysis_Geom.hxx>
#include <ShapeAnalysis_ShapeTolerance.hxx>
#include <ShapeAnalysis_Shell.hxx>
#include <ShapeAnalysis_WireOrder.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
void sh_geom_case(std::istream& input){
 GaReader r{input};int op=r.integer();
 if(op==0){int n=r.integer();NCollection_Array1<gp_Pnt> pts(1,n);for(int i=1;i<=n;++i)pts(i)=r.pnt();
  gp_Pln pln;double dmax=0.;GA_TRY(bool ok=ShapeAnalysis_Geom::NearestPlane(pts,pln,dmax);ga_flag(ok);
  if(ok){ga_pnt(pln.Location());ga_dir(pln.Axis().Direction());ga_dir(pln.XAxis().Direction());ga_real(dmax);})
  ga_end();return;}
 bool null=r.flag();int rl=r.integer(),ru=r.integer(),cl=r.integer(),cu=r.integer();
 occ::handle<NCollection_HArray2<double>> m;if(!null)m=new NCollection_HArray2<double>(rl,ru,cl,cu);
 for(int i=rl;i<=ru;++i)for(int j=cl;j<=cu;++j){double v=r.real();if(!null)m->SetValue(i,j,v);}
 double unit=r.real(),prec=r.real();gp_Trsf T;
 GA_TRY(bool ok=ShapeAnalysis_Geom::PositionTrsf(m,T,unit,prec);ga_flag(ok);ga_int(T.Form());ga_real(T.ScaleFactor());
 for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(T.Value(i,j)))
 ga_end();}
// A shape of a list: its IsSame index and its orientation.
void sh_item(const TpShapes& t,const TopoDS_Shape& s){ga_int(tp_same_index(t,s));ga_int(s.IsNull()?-1:int(s.Orientation()));}
// A list of shapes: its length and each shape (sh_item).
void sh_list(const TpShapes& t,const occ::handle<NCollection_HSequence<TopoDS_Shape>>& l){
 ga_int(l->Length());for(int i=1;i<=l->Length();++i)sh_item(t,l->Value(i));}
void sh_children(const TpShapes& t,const TopoDS_Shape& c){
 ga_int(c.NbChildren());for(TopoDS_Iterator it(c);it.More();it.Next())sh_item(t,it.Value());}
void sh_st_op(GaReader& r,const TpShapes& t,ShapeAnalysis_ShapeTolerance& st,int op){
 if(op==0){TopoDS_Shape s=sh_shape(t,r.integer());int mode=r.integer();TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());
  GA_TRY(ga_real(st.Tolerance(s,mode,ty)))}
 else if(op==1){TopoDS_Shape s=sh_shape(t,r.integer());double v=r.real();TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());
  GA_TRY(sh_list(t,st.OverTolerance(s,v,ty)))}
 else if(op==2){TopoDS_Shape s=sh_shape(t,r.integer());double a=r.real(),b=r.real();TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());
  GA_TRY(sh_list(t,st.InTolerance(s,a,b,ty)))}
 else if(op==3)st.InitTolerance();
 else if(op==4){TopoDS_Shape s=sh_shape(t,r.integer());TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());GA_TRY(st.AddTolerance(s,ty))}
 else{ga_real(st.GlobalTolerance(-1));ga_real(st.GlobalTolerance(0));ga_real(st.GlobalTolerance(1));}}
void sh_shapetol_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);ShapeAnalysis_ShapeTolerance st;int nops=r.integer();
 for(int k=0;k<nops;++k)sh_st_op(r,t,st,r.integer());
 ga_end();}
void sh_shell_op(GaReader& r,const TpShapes& t,ShapeAnalysis_Shell& sh,int op){
 if(op==0){TopoDS_Shape s=sh_shape(t,r.integer());GA_TRY(sh.LoadShells(s))}
 else if(op==1){TopoDS_Shape s=sh_shape(t,r.integer());bool f=r.flag(),in=r.flag();GA_TRY(ga_flag(sh.CheckOrientedShells(s,f,in)))}
 else if(op==2)sh.Clear();
 else if(op==3)ga_flag(sh.IsLoaded(sh_shape(t,r.integer())));
 else{int num=r.integer();GA_TRY(ga_int(tp_shape_index(t,sh.Loaded(num))))}}
void sh_shell_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);ShapeAnalysis_Shell sh;int nops=r.integer();
 for(int k=0;k<nops;++k)sh_shell_op(r,t,sh,r.integer());
 int n=sh.NbLoaded();ga_int(n);for(int i=1;i<=n;++i)ga_int(tp_shape_index(t,sh.Loaded(i)));
 ga_flag(sh.HasBadEdges());sh_children(t,sh.BadEdges());ga_flag(sh.HasFreeEdges());sh_children(t,sh.FreeEdges());
 ga_flag(sh.HasConnectedEdges());ga_end();}
void sh_wo_op(GaReader& r,ShapeAnalysis_WireOrder& wo,int op){
 if(op==0){gp_XYZ a=r.xyz();wo.Add(a,r.xyz());}
 else if(op==1){gp_XY a=r.pnt2d().XY();wo.Add(a,r.pnt2d().XY());}
 else if(op==2){gp_XYZ a=r.xyz(),b=r.xyz();gp_XY c=r.pnt2d().XY();wo.Add(a,b,c,r.pnt2d().XY());}
 else if(op==3){bool c=r.flag();GA_TRY(wo.Perform(c))}
 else if(op==4){bool m=r.flag();double tol=r.real();wo.SetMode(m,tol,r.flag());}
 else if(op==5)wo.Clear();
 else if(op==6){double g=r.real();GA_TRY(wo.SetChains(g))}
 else if(op==7)wo.SetCouples(r.real());
 else wo.KeepLoopsMode()=r.flag();}
// The digest: Tolerance, NbEdges, IsDone, Status, KeepLoopsMode, Ordered of each rank (and
// 0 and n + 1), XYZ and XY of each ordered edge, Gap of 0..n, the chains (Chain of -1..n+1)
// and the couples.
void sh_wo_digest(const ShapeAnalysis_WireOrder& wo){
 ga_real(wo.Tolerance());int n=wo.NbEdges();ga_int(n);ga_flag(wo.IsDone());ga_int(wo.Status());
 for(int i=0;i<=n+1;++i)GA_TRY(ga_int(wo.Ordered(i)))
 for(int i=1;i<=n;++i){gp_XYZ a,b;gp_XY c,d;GA_TRY(int o=wo.Ordered(i);wo.XYZ(o,a,b);ga_xyz(a);ga_xyz(b);wo.XY(o,c,d);ga_xy(c);ga_xy(d))}
 GA_TRY(gp_XYZ a,b;wo.XYZ(0,a,b))
 for(int i=0;i<=n;++i)GA_TRY(ga_real(wo.Gap(i)))
 int nc=wo.NbChains();ga_int(nc);
 for(int i=-1;i<=nc+1;++i){int n1=-7,n2=-7;GA_TRY(wo.Chain(i,n1,n2);ga_int(n1);ga_int(n2))}
 ga_int(wo.NbCouples());int n1=-7,n2=-7;wo.Couple(1,n1,n2);ga_int(n1);ga_int(n2);}
void sh_wireorder_case(std::istream& input){
 GaReader r{input};bool m3=r.flag();double tol=r.real();bool both=r.flag();
 ShapeAnalysis_WireOrder wo(m3,tol,both);int nops=r.integer();
 for(int k=0;k<nops;++k)sh_wo_op(r,wo,r.integer());
 ga_flag(wo.KeepLoopsMode());sh_wo_digest(wo);ga_end();}
void sh_fbdata_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);int w=r.integer();
 occ::handle<ShapeAnalysis_FreeBoundData> d=w==-2?new ShapeAnalysis_FreeBoundData():new ShapeAnalysis_FreeBoundData(sh_wire(t,w));
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op<=3){double v=r.real();if(op==0)d->SetArea(v);else if(op==1)d->SetPerimeter(v);else if(op==2)d->SetRatio(v);else d->SetWidth(v);}
  else if(op==4){TopoDS_Wire n=sh_wire(t,r.integer());d->AddNotch(n,r.real());}
  else if(op==5){occ::handle<NCollection_HSequence<TopoDS_Shape>> kept=d->Notches();d->Clear();ga_int(kept->Length());}
  else d->SetFreeBound(sh_wire(t,r.integer()));}
 TopoDS_Wire probe=sh_wire(t,r.integer());
 ga_int(tp_shape_index(t,d->FreeBound()));ga_real(d->Area());ga_real(d->Perimeter());ga_real(d->Ratio());ga_real(d->Width());
 int n=d->NbNotches();ga_int(n);sh_list(t,d->Notches());
 for(int i=0;i<=n+1;++i)GA_TRY(ga_int(tp_shape_index(t,d->Notch(i)));ga_real(d->NotchWidth(i)))
 GA_TRY(ga_real(d->NotchWidth(probe)))
 ga_end();}
