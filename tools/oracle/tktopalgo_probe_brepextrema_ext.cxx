// BRepExtrema probe operations, the Ext wrappers (docs/topalgo-design.md section 13): each
// wrapper built, initialized and performed in several ways, then every accessor for the
// indices 0 to NbExt+1. Needs tktopalgo_probe_brepextrema.cxx (sub-shape readers). The Rust
// twin is crates/tktopalgo/examples/tp_probe/brep_extrema_ext.rs.
// The accessors of an ExtPC.
void tp_bxx_pc_state(const BRepExtrema_ExtPC& e){
 ga_flag(e.IsDone());GA_TRY(ga_int(e.NbExt()));int n=0;GA_TRY(n=e.IsDone()?e.NbExt():0);
 for(int i=0;i<=n+1;++i){GA_TRY(ga_real(e.SquareDistance(i)));GA_TRY(ga_flag(e.IsMin(i)));
  GA_TRY(ga_real(e.Parameter(i)));GA_TRY(ga_pnt(e.Point(i)));}
 double d1,d2;gp_Pnt p1,p2;e.TrimmedSquareDistances(d1,d2,p1,p2);ga_real(d1);ga_real(d2);ga_pnt(p1);ga_pnt(p2);}
// `shape edge mode n (x y z tol)...`: mode 0 BRepExtrema_ExtPC(V, E) per vertex, 1 one object
// with Initialize(E) then Perform(V) per vertex, 2 Perform(V) without Initialize, 3 one
// object initialized on each edge of the shape in turn.
void tp_bxx_pc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Shape e=tp_bxe_sub(r,s);int mode=r.integer();int n=r.integer();
  std::vector<TopoDS_Vertex> vs;for(int i=0;i<n;++i)vs.push_back(tp_bxe_vertex(r));
  TopoDS_Edge edge=TopoDS::Edge(e);BRepExtrema_ExtPC one;
  if(mode==1){GA_TRY(one.Initialize(edge));}
  for(const TopoDS_Vertex& v:vs){
   if(mode==0){GA_TRY(BRepExtrema_ExtPC x(v,edge);tp_bxx_pc_state(x));}
   else if(mode==3){for(const TopoDS_Edge& k:s.edges){GA_TRY(one.Initialize(k));GA_TRY(one.Perform(v));tp_bxx_pc_state(one);}}
   else{GA_TRY(one.Perform(v));tp_bxx_pc_state(one);}});
 ga_end();}
// The accessors of an ExtPF.
void tp_bxx_pf_state(const BRepExtrema_ExtPF& e){
 ga_flag(e.IsDone());ga_int(e.NbExt());
 for(int i=0;i<=e.NbExt()+1;++i){GA_TRY(ga_real(e.SquareDistance(i)));
  GA_TRY(double u,v;e.Parameter(i,u,v);ga_real(u);ga_real(v));GA_TRY(ga_pnt(e.Point(i)));}}
// `shape face flag algo mode n (x y z tol)...`: mode 0 the constructor per vertex, 1 one
// object initialized once, 2 without Initialize, 3 initialized once with the flag and the
// algorithm changed (SetFlag MIN, SetAlgo Tree, then back) before each Perform.
void tp_bxx_pf_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Face f=TopoDS::Face(tp_bxe_sub(r,s));Extrema_ExtFlag fl=tp_bxe_flag(r.integer());
  Extrema_ExtAlgo al=tp_bxe_algo(r.integer());int mode=r.integer();int n=r.integer();
  std::vector<TopoDS_Vertex> vs;for(int i=0;i<n;++i)vs.push_back(tp_bxe_vertex(r));BRepExtrema_ExtPF one;
  if(mode==1||mode==3){GA_TRY(one.Initialize(f,fl,al));}
  int k=0;
  for(const TopoDS_Vertex& v:vs){
   if(mode==0){GA_TRY(BRepExtrema_ExtPF x(v,f,fl,al);tp_bxx_pf_state(x));continue;}
   if(mode==3){one.SetFlag(k%3==1?Extrema_ExtFlag_MIN:fl);one.SetAlgo(k%3==2?Extrema_ExtAlgo_Tree:al);}
   ++k;GA_TRY(one.Perform(v,f));tp_bxx_pf_state(one);});
 ga_end();}
// The accessors of an ExtCC.
void tp_bxx_cc_state(const BRepExtrema_ExtCC& e){
 ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsParallel()));GA_TRY(ga_int(e.NbExt()));int n=0;GA_TRY(n=e.IsDone()?e.NbExt():0);
 for(int i=0;i<=n+1;++i){GA_TRY(ga_real(e.SquareDistance(i)));GA_TRY(ga_real(e.ParameterOnE1(i)));
  GA_TRY(ga_pnt(e.PointOnE1(i)));GA_TRY(ga_real(e.ParameterOnE2(i)));GA_TRY(ga_pnt(e.PointOnE2(i)));}
 double d[4];gp_Pnt p[4];e.TrimmedSquareDistances(d[0],d[1],d[2],d[3],p[0],p[1],p[2],p[3]);
 for(int i=0;i<4;++i){ga_real(d[i]);ga_pnt(p[i]);}}
// `shape1 edge1 shape2 edge2 mode`: mode 0 BRepExtrema_ExtCC(E1, E2), 1 Initialize(E2) then
// Perform(E1) and Perform(E2) on one object, 2 Perform(E1) without Initialize, 3
// Initialize(E1), Initialize(E2) then Perform(E1) twice.
void tp_bxx_cc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TopoDS_Edge e1=TopoDS::Edge(tp_bxe_sub(r,a));TpShape b=tp_shape(r);
  TopoDS_Edge e2=TopoDS::Edge(tp_bxe_sub(r,b));int mode=r.integer();BRepExtrema_ExtCC one;
  if(mode==0){GA_TRY(BRepExtrema_ExtCC x(e1,e2);tp_bxx_cc_state(x));}
  else if(mode==1){GA_TRY(one.Initialize(e2));GA_TRY(one.Perform(e1));tp_bxx_cc_state(one);GA_TRY(one.Perform(e2));tp_bxx_cc_state(one);}
  else if(mode==2){GA_TRY(one.Perform(e1));tp_bxx_cc_state(one);}
  else{GA_TRY(one.Initialize(e1));GA_TRY(one.Initialize(e2));GA_TRY(one.Perform(e1));tp_bxx_cc_state(one);
   GA_TRY(one.Perform(e1));tp_bxx_cc_state(one);});
 ga_end();}
// The accessors of an ExtCF.
void tp_bxx_cf_state(const BRepExtrema_ExtCF& e){
 ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsParallel()));ga_int(e.NbExt());
 for(int i=0;i<=e.NbExt()+1;++i){GA_TRY(ga_real(e.SquareDistance(i)));GA_TRY(ga_real(e.ParameterOnEdge(i)));
  GA_TRY(double u,v;e.ParameterOnFace(i,u,v);ga_real(u);ga_real(v));GA_TRY(ga_pnt(e.PointOnEdge(i)));
  GA_TRY(ga_pnt(e.PointOnFace(i)));}}
// `shape1 edge shape2 face mode`: mode 0 BRepExtrema_ExtCF(E, F), 1 Initialize(E, F) then
// Perform(E, F) twice, 2 Perform without Initialize, 3 Initialize then Perform with each
// edge of the first shape.
void tp_bxx_cf_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TopoDS_Edge e=TopoDS::Edge(tp_bxe_sub(r,a));TpShape b=tp_shape(r);
  TopoDS_Face f=TopoDS::Face(tp_bxe_sub(r,b));int mode=r.integer();BRepExtrema_ExtCF one;
  if(mode==0){GA_TRY(BRepExtrema_ExtCF x(e,f);tp_bxx_cf_state(x));}
  else if(mode==1){GA_TRY(one.Initialize(e,f));for(int i=0;i<2;++i){GA_TRY(one.Perform(e,f));tp_bxx_cf_state(one);}}
  else if(mode==2){GA_TRY(one.Perform(e,f));tp_bxx_cf_state(one);}
  else{GA_TRY(one.Initialize(e,f));for(const TopoDS_Edge& k:a.edges){GA_TRY(one.Perform(k,f));tp_bxx_cf_state(one);}});
 ga_end();}
// The accessors of an ExtFF.
void tp_bxx_ff_state(const BRepExtrema_ExtFF& e){
 ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsParallel()));ga_int(e.NbExt());
 for(int i=0;i<=e.NbExt()+1;++i){GA_TRY(ga_real(e.SquareDistance(i)));
  GA_TRY(double u,v;e.ParameterOnFace1(i,u,v);ga_real(u);ga_real(v));
  GA_TRY(double u,v;e.ParameterOnFace2(i,u,v);ga_real(u);ga_real(v));
  GA_TRY(ga_pnt(e.PointOnFace1(i)));GA_TRY(ga_pnt(e.PointOnFace2(i)));}}
// `shape1 face1 shape2 face2 mode`: mode 0 BRepExtrema_ExtFF(F1, F2), 1 Initialize(F2) then
// Perform(F1, F2) twice, 2 Perform without Initialize, 3 Initialize(F2) then Perform with
// each face of the first shape.
void tp_bxx_ff_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TopoDS_Face f1=TopoDS::Face(tp_bxe_sub(r,a));TpShape b=tp_shape(r);
  TopoDS_Face f2=TopoDS::Face(tp_bxe_sub(r,b));int mode=r.integer();BRepExtrema_ExtFF one;
  if(mode==0){GA_TRY(BRepExtrema_ExtFF x(f1,f2);tp_bxx_ff_state(x));}
  else if(mode==1){GA_TRY(one.Initialize(f2));for(int i=0;i<2;++i){GA_TRY(one.Perform(f1,f2));tp_bxx_ff_state(one);}}
  else if(mode==2){GA_TRY(one.Perform(f1,f2));tp_bxx_ff_state(one);}
  else{GA_TRY(one.Initialize(f2));for(const TopoDS_Face& k:a.faces){GA_TRY(one.Perform(k,f2));tp_bxx_ff_state(one);}});
 ga_end();}
