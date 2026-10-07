// HLRBRep_ShapeToHLR and HLRBRep_Data, the Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_data.rs.
#include <HLRBRep_Data.hxx>
#include <HLRBRep_ShapeToHLR.hxx>
#include <HLRBRep_Hider.hxx>
#include <HLRTopoBRep_OutLiner.hxx>
#include <HLRAlgo.hxx>
#include <HLRAlgo_WiresBlock.hxx>
#include <NCollection_List.hxx>
#include <gp_Dir2d.hxx>
void hb_minmax(const HLRAlgo_EdgesBlock::MinMaxIndices& m){for(int k=0;k<8;++k)ga_int(m.Min[k]);for(int k=0;k<8;++k)ga_int(m.Max[k]);}
// An edge record after Load (full) or after Update.
void hb_data_edge(HLRBRep_EdgeData& d,bool boxes){
 hb_edge_flags(d);if(!boxes)return;hb_minmax(d.MinMax());}
void hb_data_face(HLRBRep_FaceData& f){
 for(bool b:{f.Selected(),f.Back(),f.Side(),f.Closed(),f.Hiding(),f.Simple(),f.Cut(),f.WithOutL(),f.Plane(),
  f.Cylinder(),f.Cone(),f.Sphere(),f.Torus()})ga_flag(b);
 ga_real(f.Size());ga_int(f.Orientation());ga_real(f.Tolerance());
 const occ::handle<HLRAlgo_WiresBlock>& wb=f.Wires();ga_int(wb->NbWires());hb_minmax(wb->MinMax());
 for(int w=1;w<=wb->NbWires();++w){const occ::handle<HLRAlgo_EdgesBlock>& eb=wb->Wire(w);ga_int(eb->NbEdges());hb_minmax(eb->MinMax());
  for(int e=1;e<=eb->NbEdges();++e){ga_int(eb->Edge(e));ga_int(eb->Orientation(e));
   for(bool b:{eb->OutLine(e),eb->Internal(e),eb->Double(e),eb->IsoLine(e)})ga_flag(b);}}}
void hb_local2d(const gp_Dir2d& t,const gp_Dir2d& n,double c){ga_real(t.X());ga_real(t.Y());ga_real(n.X());ga_real(n.Y());ga_real(c);}
void hb_interference(HLRBRep_Data& ds,const HLRAlgo_Interference& i){
 const HLRAlgo_Intersection& x=i.Intersection();ga_int(x.Orientation());ga_int(x.Level());ga_int(x.SegIndex());
 ga_int(x.Index());ga_real(x.Parameter());ga_real(x.Tolerance());ga_int(x.State());
 int fe;double p;i.Boundary().Value2D(fe,p);ga_int(fe);ga_real(p);
 if(x.State()==TopAbs_ON){TopAbs_State a,b;i.Boundary().State3D(a,b);ga_int(a);ga_int(b);}
 ga_int(i.Orientation());ga_int(i.Transition());ga_int(i.BoundaryTransition());
 gp_Dir2d t,n;double c;GA_TRY(ds.LocalLEGeometry2D(x.Parameter(),t,n,c);hb_local2d(t,n,c));
 GA_TRY(ds.LocalFEGeometry2D(fe,p,t,n,c);hb_local2d(t,n,c));}
// The hiding walk of one face, as the Hider drives it, without the edge building.
void hb_data_hide(HLRBRep_Data& ds,int f,NCollection_DataMap<TopoDS_Shape,BRepTopAdaptor_Tool,TopTools_ShapeMapHasher>& mst){
 ds.InitEdge(f,mst);ga_flag(ds.IsBadFace());ga_flag(ds.HidingTheFace());ga_flag(ds.SimpleHidingFace());
 for(int n=0;ds.MoreEdge()&&n<200;ds.NextEdge(),++n){int e=ds.Edge();ga_int(e);
  GA_TRY(NCollection_List<HLRAlgo_Interference> il;
   for(ds.InitInterference();ds.MoreInterference();ds.NextInterference()){bool rej=ds.RejectedInterference();ga_flag(rej);
    ga_flag(ds.AboveInterference());if(!rej){hb_interference(ds,ds.Interference());il.Append(ds.Interference());}}
   HLRBRep_EdgeData& ed=ds.EDataArray()(e);ga_int(ds.Compare(e,ed));double a,b;float ta,tb;ed.Status().Bounds(a,ta,b,tb);
   int level=0;GA_TRY(ga_int(ds.Classify(e,ed,true,level,(a+b)/2));ga_int(level));
   GA_TRY(ga_int(ds.SimplClassify(e,ed,3,a,b)));GA_TRY(ga_int(ds.HidingStartLevel(e,ed,il)));
   ga_flag(ds.EdgeOfTheHidingFace(e,ed)));}
 ga_flag(ds.MoreEdge());}
// InitBoundSort on the box of the edges and faces (as HLRBRep_InternalAlgo::Update).
void hb_bound_sort(HLRBRep_Data& ds){
 int ne=ds.NbEdges(),nf=ds.NbFaces();
 HLRAlgo_EdgesBlock::MinMaxIndices smin,smax,tmin,tmax,tot;
 for(int e=1;e<=ne;++e){HLRAlgo::DecodeMinMax(ds.EDataArray()(e).MinMax(),tmin,tmax);
  if(e==1)HLRAlgo::CopyMinMax(tmin,tmax,smin,smax);else HLRAlgo::AddMinMax(tmin,tmax,smin,smax);}
 for(int f=1;f<=nf;++f){HLRAlgo::DecodeMinMax(ds.FDataArray()(f).Wires()->MinMax(),tmin,tmax);HLRAlgo::AddMinMax(tmin,tmax,smin,smax);}
 HLRAlgo::EncodeMinMax(smin,smax,tot);ds.InitBoundSort(tot,1,ne);}
// The visible parts of every edge.
void hb_edge_status(HLRBRep_Data& ds){
 for(int e=1;e<=ds.NbEdges();++e){HLRAlgo_EdgeStatus& st=ds.EDataArray()(e).Status();
  ga_flag(st.AllHidden());ga_flag(st.AllVisible());int n=st.NbVisiblePart();ga_int(n);
  for(int k=1;k<=n;++k){double a,b;float ta,tb;st.VisiblePart(k,a,ta,b,tb);ga_real(a);ga_real(ta);ga_real(b);ga_real(tb);}}}
// `projector shape nbIso`: Load, Update, InitBoundSort, then HLRBRep_Hider::Hide by
// every hiding face, and the visible parts of the edges.
void hb_hider_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 try{TopoDS_Shape s=hb_shape(r);int nb_iso=r.integer();
  occ::handle<HLRTopoBRep_OutLiner> o=new HLRTopoBRep_OutLiner(s);
  NCollection_DataMap<TopoDS_Shape,BRepTopAdaptor_Tool,TopTools_ShapeMapHasher> mst;
  occ::handle<HLRBRep_Data> ds=HLRBRep_ShapeToHLR::Load(o,projector,mst,nb_iso);ds->Update(projector);
  if(ds->NbEdges()==0){ga_end();return;}
  hb_bound_sort(*ds);HLRBRep_Hider h(ds);
  for(int f=1;f<=ds->NbFaces();++f)if(ds->FDataArray()(f).Hiding())GA_TRY(h.Hide(f,mst));
  hb_edge_status(*ds);
 }catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
 ga_end();}
// `projector shape nbIso`: ShapeToHLR::Load, the records, Update and the records, then
// InitBoundSort on the box of the shape and the hiding walk of every hiding face.
void hb_data_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 try{TopoDS_Shape s=hb_shape(r);int nb_iso=r.integer();
  occ::handle<HLRTopoBRep_OutLiner> o=new HLRTopoBRep_OutLiner(s);
  NCollection_DataMap<TopoDS_Shape,BRepTopAdaptor_Tool,TopTools_ShapeMapHasher> mst;
  occ::handle<HLRBRep_Data> ds=HLRBRep_ShapeToHLR::Load(o,projector,mst,nb_iso);
  int ne=ds->NbEdges(),nf=ds->NbFaces();ga_int(ds->NbVertices());ga_int(ne);ga_int(nf);
  for(int e=1;e<=ne;++e)hb_data_edge(ds->EDataArray()(e),false);
  ds->Update(projector);
  for(int e=1;e<=ne;++e)hb_data_edge(ds->EDataArray()(e),true);
  for(int f=1;f<=nf;++f)hb_data_face(ds->FDataArray()(f));
  if(ne==0){ga_end();return;}
  HLRAlgo_EdgesBlock::MinMaxIndices smin,smax,tmin,tmax,tot;bool first=true;
  for(int e=1;e<=ne;++e){HLRAlgo::DecodeMinMax(ds->EDataArray()(e).MinMax(),tmin,tmax);
   if(first)HLRAlgo::CopyMinMax(tmin,tmax,smin,smax);else HLRAlgo::AddMinMax(tmin,tmax,smin,smax);first=false;}
  for(int f=1;f<=nf;++f){HLRAlgo::DecodeMinMax(ds->FDataArray()(f).Wires()->MinMax(),tmin,tmax);HLRAlgo::AddMinMax(tmin,tmax,smin,smax);}
  HLRAlgo::EncodeMinMax(smin,smax,tot);ds->InitBoundSort(tot,1,ne);
  for(int f=1;f<=nf;++f)if(ds->FDataArray()(f).Hiding())GA_TRY(hb_data_hide(*ds,f,mst));
 }catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
 ga_end();}
