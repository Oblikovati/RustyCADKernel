// HLRBRep_Algo, HLRBRep_InternalAlgo, HLRBRep_HLRToShape and HLRAppli_ReflectLines, the
// Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_algo.rs.
#include <HLRAppli_ReflectLines.hxx>
#include <HLRBRep_Algo.hxx>
#include <HLRBRep_HLRToShape.hxx>
#include <HLRBRep_TypeOfResultingEdge.hxx>
// The edges of a result compound: null flag, count, then each edge.
void hb_compound(const TopoDS_Shape& c){
 ga_flag(c.IsNull());if(c.IsNull())return;int n=0;
 for(TopExp_Explorer ex(c,TopAbs_EDGE);ex.More();ex.Next())++n;ga_int(n);
 for(TopExp_Explorer ex(c,TopAbs_EDGE);ex.More();ex.Next())GA_TRY(hb_edge_summary(TopoDS::Edge(ex.Current())));}
// Every compound of a HLRToShape: the 2D visible and hidden edges of each type, the 3D
// visible outlines, and the visible and hidden sharp edges of the first shape.
void hb_to_shape(const occ::handle<HLRBRep_Algo>& a,const TopoDS_Shape& first){
 HLRBRep_HLRToShape t(a);
 for(int ty=HLRBRep_IsoLine;ty<=HLRBRep_Sharp;++ty)for(bool v:{true,false})
  GA_TRY(hb_compound(t.CompoundOfEdges((HLRBRep_TypeOfResultingEdge)ty,v,false)));
 GA_TRY(hb_compound(t.OutLineVCompound3d()));GA_TRY(hb_compound(t.VCompound(first)));GA_TRY(hb_compound(t.HCompound(first)));}
// `projector nshapes shape* nbIso mode`: Algo::Add of each shape, Update, then Hide
// (mode 0), PartialHide (1), Hide(1) and Hide(1, 2) (2), and the results.
void hb_algo_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 try{int n=r.integer();std::vector<TopoDS_Shape> shapes;for(int k=0;k<n;++k)shapes.push_back(hb_shape(r));
  int nb_iso=r.integer(),mode=r.integer();occ::handle<HLRBRep_Algo> a=new HLRBRep_Algo();
  for(const TopoDS_Shape& s:shapes)a->Add(s,nb_iso);a->Projector(projector);a->Update();ga_int(a->NbShapes());
  occ::handle<HLRBRep_Data> ds=a->DataStructure();ga_flag(ds.IsNull());
  if(!ds.IsNull()){ga_int(ds->NbEdges());ga_int(ds->NbFaces());}
  for(int k=1;k<=a->NbShapes();++k){int v1,v2,e1,e2,f1,f2;a->ShapeBounds(k).Bounds(v1,v2,e1,e2,f1,f2);
   for(int x:{v1,v2,e1,e2,f1,f2})ga_int(x);hb_minmax(a->ShapeBounds(k).MinMax());ga_int(a->Index(shapes[k-1]));}
  if(mode==0){GA_TRY(a->Hide());}
  else if(mode==1){GA_TRY(a->PartialHide());}
  else{GA_TRY(a->Hide(1));if(n>1){GA_TRY(a->Hide(1,2));}a->Select();}
  hb_to_shape(a,shapes[0]);
 }catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
 ga_end();}
// `shape N(3) At(3) Up(3)`: ReflectLines::SetAxes, Perform, GetResult and the visible
// and hidden 3D sharp edges.
void hb_reflect_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=hb_shape(r);double v[9];for(double& x:v)x=r.real();
  HLRAppli_ReflectLines l(s);l.SetAxes(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8]);l.Perform();
  GA_TRY(hb_compound(l.GetResult()));
  for(bool vis:{true,false})GA_TRY(hb_compound(l.GetCompoundOf3dEdges(HLRBRep_Sharp,vis,true)));
 }catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
 ga_end();}
