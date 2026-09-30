// HLRAlgo value classes, the Rust twin of crates/tkhlr/examples/hl_probe/hlr_algo.rs: the
// package box functions, BiPoint, the edge and wire blocks, EdgeStatus with EdgeIterator,
// and the interference records. Values a native class leaves
// uninitialized are never printed (see docs/tkhlr-progress.md).
#include <HLRAlgo.hxx>
#include <HLRAlgo_BiPoint.hxx>
#include <HLRAlgo_EdgeIterator.hxx>
#include <HLRAlgo_EdgeStatus.hxx>
#include <HLRAlgo_EdgesBlock.hxx>
#include <HLRAlgo_Interference.hxx>
#include <HLRAlgo_Projector.hxx>
#include <HLRAlgo_WiresBlock.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Lin.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
using HlMinMax=HLRAlgo_EdgesBlock::MinMaxIndices;
HlMinMax hl_read_minmax(GaReader& r){HlMinMax v;for(int& x:v.Min)x=r.integer();for(int& x:v.Max)x=r.integer();return v;}
void hl_print_minmax(const HlMinMax& v){for(int x:v.Min)ga_int(x);for(int x:v.Max)ga_int(x);}
void hl_print_parts(double s,float ts,double e,float te){ga_real(s);ga_real(ts);ga_real(e);ga_real(te);}
// `big tol n (x y z)*n`: InitMinMax, UpdateMinMax per point, then EnlargeMinMax; both boxes.
void hl_ha_minmax_case(std::istream& input){
 GaReader r{input};double big=r.real(),tol=r.real();int n=r.integer();double mn[16],mx[16];
 HLRAlgo::InitMinMax(big,mn,mx);
 for(int i=0;i<n;++i){double x=r.real(),y=r.real();HLRAlgo::UpdateMinMax(x,y,r.real(),mn,mx);}
 for(int i=0;i<16;++i){ga_real(mn[i]);ga_real(mx[i]);}
 HLRAlgo::EnlargeMinMax(tol,mn,mx);
 for(int i=0;i<16;++i){ga_real(mn[i]);ga_real(mx[i]);}
 ga_end();}
// `min max other`: EncodeMinMax, DecodeMinMax of the code, SizeBox, Minimize and Maximize by
// other, AddMinMax and CopyMinMax.
void hl_ha_encode_case(std::istream& input){
 GaReader r{input};HlMinMax mn=hl_read_minmax(r),mx=hl_read_minmax(r),other=hl_read_minmax(r);
 HlMinMax code{},d1{},d2{};HLRAlgo::EncodeMinMax(mn,mx,code);hl_print_minmax(code);
 HLRAlgo::DecodeMinMax(code,d1,d2);hl_print_minmax(d1);hl_print_minmax(d2);
 ga_real(HLRAlgo::SizeBox(mn,mx));
 HlMinMax a=mn;a.Minimize(other);hl_print_minmax(a);HlMinMax b=mn;b.Maximize(other);hl_print_minmax(b);
 HlMinMax o1=mn,o2=mx;HLRAlgo::AddMinMax(other,other,o1,o2);hl_print_minmax(o1);hl_print_minmax(o2);
 HLRAlgo::CopyMinMax(other,mn,o1,o2);hl_print_minmax(o1);hl_print_minmax(o2);
 ga_end();}
void hl_print_bipoint(HLRAlgo_BiPoint& bp){
 const HLRAlgo_BiPoint::IndicesT& i=bp.Indices();
 for(int v:{i.ShapeIndex,i.FaceConex1,i.Face1Pt1,i.Face1Pt2,i.FaceConex2,i.Face2Pt1,i.Face2Pt2,i.MinSeg,i.MaxSeg,i.SegFlags})ga_int(v);
 const HLRAlgo_BiPoint::PointsT& p=bp.Points();ga_xyz(p.Pnt1);ga_xyz(p.Pnt2);ga_xyz(p.PntP1);ga_xyz(p.PntP2);
 ga_xy(p.PntP12D());ga_xy(p.PntP22D());
 ga_flag(bp.Rg1Line());ga_flag(bp.RgNLine());ga_flag(bp.OutLine());ga_flag(bp.IntLine());ga_flag(bp.Hidden());}
// `variant c[12] index i1 i1p1 i1p2 i2 i2p1 i2p2 bits n (which value)*n`: variant 0..5 the six
// constructors (booleans from bits 0..3, or the raw flag word), 6 the default one; then the
// flag setters (which 0 Rg1Line .. 4 Hidden); then every field.
void hl_ha_bipoint_case(std::istream& input){
 GaReader r{input};int variant=r.integer();double c[12];for(double& x:c)x=r.real();
 int index=r.integer(),i1=r.integer(),i1p1=r.integer(),i1p2=r.integer(),i2=r.integer(),i2p1=r.integer(),i2p2=r.integer(),bits=r.integer();
 bool b0=bits&1,b1=bits&2,b2=bits&4,b3=bits&8;HLRAlgo_BiPoint bp;
 switch(variant){
  case 0:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,b0,b1,b2,b3);break;
  case 1:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,bits);break;
  case 2:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,i1,i1p1,i1p2,b0,b1,b2,b3);break;
  case 3:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,i1,i1p1,i1p2,bits);break;
  case 4:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,i1,i1p1,i1p2,i2,i2p1,i2p2,b0,b1,b2,b3);break;
  case 5:bp=HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,i1,i1p1,i1p2,i2,i2p1,i2p2,bits);break;
  default:break;}
 for(int n=r.integer();n>0;--n){int which=r.integer();bool v=r.flag();
  switch(which){case 0:bp.Rg1Line(v);break;case 1:bp.RgNLine(v);break;case 2:bp.OutLine(v);break;case 3:bp.IntLine(v);break;default:bp.Hidden(v);break;}}
 hl_print_bipoint(bp);ga_end();}
// `nbEdges n ops`: 0 Edge(I, EI), 1 Orientation(I, Or), 2..5 OutLine/Internal/Double/IsoLine(I, B),
// 6 read everything at I, 7 UpdateMinMax(16 ints) then MinMax(); NbEdges first. Reads only
// follow the setters of the same rank (the native arrays start uninitialized).
void hl_ha_edges_case(std::istream& input){
 GaReader r{input};occ::handle<HLRAlgo_EdgesBlock> eb=new HLRAlgo_EdgesBlock(r.integer());ga_int(eb->NbEdges());
 for(int n=r.integer();n>0;--n){int op=r.integer();
  if(op==7){HlMinMax v=hl_read_minmax(r);eb->UpdateMinMax(v);hl_print_minmax(eb->MinMax());continue;}
  int i=r.integer();
  if(op==6){GA_TRY(ga_int(eb->Edge(i)))GA_TRY(ga_int(eb->Orientation(i)))GA_TRY(ga_flag(eb->OutLine(i)))
   GA_TRY(ga_flag(eb->Internal(i)))GA_TRY(ga_flag(eb->Double(i)))GA_TRY(ga_flag(eb->IsoLine(i)))continue;}
  int v=r.integer();
  GA_TRY(switch(op){case 0:eb->Edge(i,v);break;case 1:eb->Orientation(i,TopAbs_Orientation(v));break;
   case 2:eb->OutLine(i,v!=0);break;case 3:eb->Internal(i,v!=0);break;case 4:eb->Double(i,v!=0);break;default:eb->IsoLine(i,v!=0);break;})}
 ga_end();}
// `nbWires n ops`: 0 Set(I, new block of `v` edges), 1 Wire(I) (its NbEdges, -1 when null),
// 2 UpdateMinMax(16 ints) then MinMax(); NbWires first.
void hl_ha_wires_case(std::istream& input){
 GaReader r{input};occ::handle<HLRAlgo_WiresBlock> wb=new HLRAlgo_WiresBlock(r.integer());ga_int(wb->NbWires());
 for(int n=r.integer();n>0;--n){int op=r.integer();
  if(op==2){HlMinMax v=hl_read_minmax(r);wb->UpdateMinMax(v);hl_print_minmax(wb->MinMax());continue;}
  int i=r.integer();
  if(op==0){int v=r.integer();GA_TRY(wb->Set(i,new HLRAlgo_EdgesBlock(v)))}
  else GA_TRY(const occ::handle<HLRAlgo_EdgesBlock>& w=wb->Wire(i);ga_int(w.IsNull()?-1:w->NbEdges()))}
 ga_end();}
void hl_print_status(HLRAlgo_EdgeStatus& st){
 double s,e;float ts,te;st.Bounds(s,ts,e,te);hl_print_parts(s,ts,e,te);
 ga_flag(st.AllHidden());ga_flag(st.AllVisible());int nb=st.NbVisiblePart();ga_int(nb);
 for(int i=1;i<=nb;++i){st.VisiblePart(i,s,ts,e,te);hl_print_parts(s,ts,e,te);}
 HLRAlgo_EdgeIterator it;
 for(it.InitVisible(st);it.MoreVisible();it.NextVisible()){it.Visible(s,ts,e,te);hl_print_parts(s,ts,e,te);}
 // A hidden walk over no visible interval reads outside the native sequence: skipped.
 if(nb==0&&!st.AllHidden()){ga_int(-1);return;}
 for(it.InitHidden(st);it.MoreHidden();it.NextHidden()){it.Hidden(s,ts,e,te);hl_print_parts(s,ts,e,te);}
 ga_int(-2);}
// `mode [s ts e te] n ops`: mode 0 the default status, 1 the status of the bounds; ops 0
// Initialize, 1 Hide(s ts e te onFace onBoundary), 2 HideAll, 3 ShowAll, 4 AllHidden(B), 5
// AllVisible(B); the status with its visible and hidden walks after each op.
void hl_ha_status_case(std::istream& input){
 GaReader r{input};HLRAlgo_EdgeStatus st;
 if(r.integer()==1){double s=r.real();float ts=(float)r.real();double e=r.real();st=HLRAlgo_EdgeStatus(s,ts,e,(float)r.real());}
 hl_print_status(st);
 for(int n=r.integer();n>0;--n){int op=r.integer();
  if(op<=1){double s=r.real();float ts=(float)r.real();double e=r.real();float te=(float)r.real();
   if(op==0)st.Initialize(s,ts,e,te);else{bool f=r.flag();st.Hide(s,ts,e,te,f,r.flag());}}
  else if(op==2)st.HideAll();else if(op==3)st.ShowAll();else if(op==4)st.AllHidden(r.flag());else st.AllVisible(r.flag());
  hl_print_status(st);}
 ga_end();}
void hl_print_intersection(const HLRAlgo_Intersection& x,bool withEnums){
 if(withEnums){ga_int(x.Orientation());ga_int(x.State());}
 ga_int(x.Level());ga_int(x.SegIndex());ga_int(x.Index());ga_real(x.Parameter());ga_real(x.Tolerance());}
void hl_print_coincidence(const HLRAlgo_Coincidence& c){
 int fe;double p;c.Value2D(fe,p);ga_int(fe);ga_real(p);TopAbs_State a,b;c.State3D(a,b);ga_int(a);ga_int(b);}
// `o lev seg ind p tol s fe param sb sa or tr btr x`: the default records (without their
// uninitialized enumerations), the records of the values, their setters, and an Interference
// edited through ChangeIntersection and ChangeBoundary.
void hl_ha_records_case(std::istream& input){
 GaReader r{input};TopAbs_Orientation o=TopAbs_Orientation(r.integer());int lev=r.integer(),seg=r.integer(),ind=r.integer();
 double p=r.real();float tol=(float)r.real();TopAbs_State s=TopAbs_State(r.integer());int fe=r.integer();double param=r.real();
 TopAbs_State sb=TopAbs_State(r.integer()),sa=TopAbs_State(r.integer());
 TopAbs_Orientation orr=TopAbs_Orientation(r.integer()),tr=TopAbs_Orientation(r.integer()),btr=TopAbs_Orientation(r.integer());double x=r.real();
 HLRAlgo_Intersection d;hl_print_intersection(d,false);HLRAlgo_Coincidence dc;hl_print_coincidence(dc);
 HLRAlgo_Intersection v(o,lev,seg,ind,p,tol,s);hl_print_intersection(v,true);
 HLRAlgo_Intersection w;w.Orientation(orr);w.Level(ind);w.SegIndex(lev);w.Index(seg);w.Parameter(x);w.Tolerance(tol*2);w.State(sb);hl_print_intersection(w,true);
 HLRAlgo_Coincidence c;c.Set2D(fe,param);c.SetState3D(sb,sa);hl_print_coincidence(c);
 HLRAlgo_Interference f(v,c,orr,tr,btr);hl_print_intersection(f.Intersection(),true);hl_print_coincidence(f.Boundary());
 ga_int(f.Orientation());ga_int(f.Transition());ga_int(f.BoundaryTransition());
 f.ChangeIntersection().Parameter(x);f.ChangeBoundary().Set2D(fe+1,x);f.Orientation(tr);f.Transition(btr);f.BoundaryTransition(orr);
 f.Intersection(w);hl_print_intersection(f.Intersection(),true);hl_print_coincidence(f.Boundary());
 f.Boundary(dc);hl_print_coincidence(f.Boundary());ga_int(f.Orientation());ga_int(f.Transition());ga_int(f.BoundaryTransition());
 ga_end();}
