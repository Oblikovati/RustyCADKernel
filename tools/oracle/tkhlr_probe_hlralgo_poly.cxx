// HLRAlgo polyhedral classes, the Rust twin of crates/tkhlr/examples/hl_probe/hlr_algo_poly.rs:
// PolyData with HideByPolyData, PolyShellData and PolyAlgo (Update, the hide and show walks).
// Every array entry a request prints was written first: the native arrays start
// uninitialized.
#include <Bnd_Box.hxx>
#include <HLRAlgo_PolyAlgo.hxx>
#include <HLRAlgo_PolyData.hxx>
#include <HLRAlgo_PolyInternalData.hxx>
#include <HLRAlgo_PolyShellData.hxx>
#include <HLRAlgo_PolyHidingData.hxx>
#include <HLRAlgo_TriangleData.hxx>
// `nbNodes (x y z)* nbTri (n1 n2 n3 flags)* nbHide (index min max a b c d)* index min max`:
// one face; nbHide < 0 leaves the hiding array null.
occ::handle<HLRAlgo_PolyData> hl_read_face(GaReader& r){
 occ::handle<HLRAlgo_PolyData> pd=new HLRAlgo_PolyData();
 int nn=r.integer();occ::handle<NCollection_HArray1<gp_XYZ>> nodes=new NCollection_HArray1<gp_XYZ>(1,nn);
 for(int i=1;i<=nn;++i)nodes->ChangeValue(i)=r.xyz();
 int nt=r.integer();occ::handle<NCollection_HArray1<HLRAlgo_TriangleData>> tris=new NCollection_HArray1<HLRAlgo_TriangleData>(1,nt);
 for(int i=1;i<=nt;++i){HLRAlgo_TriangleData& t=tris->ChangeValue(i);t.Node1=r.integer();t.Node2=r.integer();t.Node3=r.integer();t.Flags=r.integer();}
 int nh=r.integer();
 if(nh>=0){occ::handle<NCollection_HArray1<HLRAlgo_PolyHidingData>> hid=new NCollection_HArray1<HLRAlgo_PolyHidingData>(1,nh);
  for(int i=1;i<=nh;++i){int a=r.integer(),b=r.integer(),c=r.integer();double x=r.real(),y=r.real(),z=r.real();hid->ChangeValue(i).Set(a,b,c,x,y,z,r.real());}
  pd->HPHDat(hid);}
 pd->HNodes(nodes);pd->HTData(tris);pd->FaceIndex(r.integer());
 pd->Indices().Min=r.integer();pd->Indices().Max=r.integer();return pd;}
void hl_print_face(const occ::handle<HLRAlgo_PolyData>& pd){
 ga_int(pd->FaceIndex());ga_int(pd->Indices().Min);ga_int(pd->Indices().Max);ga_flag(pd->Hiding());
 for(const HLRAlgo_TriangleData& t:pd->TData())ga_int(t.Flags);
 if(pd->Hiding())for(HLRAlgo_PolyHidingData& h:pd->PHDat()){
  ga_int(h.Indices().Index);ga_int(h.Indices().Min);ga_int(h.Indices().Max);ga_xyz(h.Plane().Normal);ga_real(h.Plane().D);}}
void hl_print_hide_status(HLRAlgo_EdgeStatus& st){
 ga_flag(st.AllHidden());ga_flag(st.AllVisible());int nb=st.NbVisiblePart();ga_int(nb);
 double s,e;float ts,te;for(int i=1;i<=nb;++i){st.VisiblePart(i,s,ts,e,te);hl_print_parts(s,ts,e,te);}}
void hl_print_triangle(const HLRAlgo_PolyData::Triangle& t){
 ga_xy(t.V1);ga_xy(t.V2);ga_xy(t.V3);ga_real(t.Param);ga_real(t.TolParam);ga_real(t.TolAng);ga_real(t.Tolerance);}
// `face n (P1 P2 indices[10] hidingShell V1 V2 V3 param tolParam tolAng tolerance)*n`: the
// face, UpdateGlobalMinMax of a void box, then per segment HideByPolyData on a status
// [0, 1] at the parameter tolerance, with the status and the working triangle after it.
void hl_ha_polydata_case(std::istream& input){
 GaReader r{input};occ::handle<HLRAlgo_PolyData> pd=hl_read_face(r);hl_print_face(pd);
 Bnd_Box box;GA_TRY(pd->UpdateGlobalMinMax(box);if(box.IsVoid())ga_int(-1);else{auto [a,b,c,d,e,f]=box.Get();for(double v:{a,b,c,d,e,f})ga_real(v);})
 for(int n=r.integer();n>0;--n){
  HLRAlgo_BiPoint::PointsT pts;pts.PntP1=r.xyz();pts.PntP2=r.xyz();pts.Pnt1=pts.PntP1;pts.Pnt2=pts.PntP2;
  HLRAlgo_BiPoint::IndicesT ind;int* f=&ind.ShapeIndex;for(int k=0;k<10;++k)f[k]=r.integer();
  bool hidingShell=r.flag();HLRAlgo_PolyData::Triangle tri;tri.V1=r.pnt2d().XY();tri.V2=r.pnt2d().XY();tri.V3=r.pnt2d().XY();
  tri.Param=r.real();tri.TolParam=r.real();tri.TolAng=r.real();tri.Tolerance=r.real();
  HLRAlgo_EdgeStatus st(0.0,(float)tri.TolParam,1.0,(float)tri.TolParam);
  GA_TRY(pd->HideByPolyData(pts,tri,ind,hidingShell,st))hl_print_hide_status(st);hl_print_triangle(tri);}
 ga_end();}
// `nbShells (nbFaces face* nbSeg (c[12] index i1 i1p1 i1p2 i2 i2p1 i2p2 flag)*)*`: the shells
// of a PolyAlgo, then Update, the packed boxes, the hide walk and the show walk.
void hl_ha_polyalgo_case(std::istream& input){
 GaReader r{input};occ::handle<HLRAlgo_PolyAlgo> algo=new HLRAlgo_PolyAlgo();int ns=r.integer();algo->Init(ns);
 for(int s=1;s<=ns;++s){int nf=r.integer();occ::handle<HLRAlgo_PolyShellData> psd=new HLRAlgo_PolyShellData(nf);
  for(int f=1;f<=nf;++f)psd->PolyData().ChangeValue(f)=hl_read_face(r);
  for(int nb=r.integer();nb>0;--nb){double c[12];for(double& x:c)x=r.real();
   int index=r.integer(),i1=r.integer(),i1p1=r.integer(),i1p2=r.integer(),i2=r.integer(),i2p1=r.integer(),i2p2=r.integer();
   psd->Edges().Append(HLRAlgo_BiPoint(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],index,i1,i1p1,i1p2,i2,i2p1,i2p2,r.integer()));}
  algo->ChangePolyShell().ChangeValue(s)=psd;}
 GA_TRY(algo->Update();
  for(const occ::handle<HLRAlgo_PolyShellData>& psd:algo->PolyShell()){
   ga_int(psd->Indices().Min);ga_int(psd->Indices().Max);ga_flag(psd->Hiding());ga_int(psd->HidingPolyData().Length());
   for(HLRAlgo_BiPoint& bp:psd->Edges()){ga_int(bp.Indices().MinSeg);ga_int(bp.Indices().MaxSeg);}
   for(const occ::handle<HLRAlgo_PolyData>& pd:psd->PolyData())hl_print_face(pd);
   for(const occ::handle<HLRAlgo_PolyData>& pd:psd->HidingPolyData())ga_int(pd->FaceIndex());}
  HLRAlgo_EdgeStatus st;int index;bool r1,rn,ol,il;
  for(algo->InitHide();algo->MoreHide();algo->NextHide()){
   HLRAlgo_BiPoint::PointsT& p=algo->Hide(st,index,r1,rn,ol,il);ga_int(index);ga_flag(r1);ga_flag(rn);ga_flag(ol);ga_flag(il);
   ga_xyz(p.PntP1);hl_print_hide_status(st);}
  ga_int(-3);
  for(algo->InitShow();algo->MoreShow();algo->NextShow()){
   HLRAlgo_BiPoint::PointsT& p=algo->Show(index,r1,rn,ol,il);ga_int(index);ga_flag(r1);ga_flag(rn);ga_flag(ol);ga_flag(il);ga_xyz(p.Pnt2);})
 ga_end();}
