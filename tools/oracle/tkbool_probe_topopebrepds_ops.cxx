// TopOpeBRepDS interference processing probe operations (TKI, ProcessInterferencesTool,
// EXPORT, repvg, redu, EIR, FIR, Filter, Check, Dumper); the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_ds_ops.rs.
// A request builds a data structure: `solid1 solid2` (bl_tt_solid records) with the solids,
// faces, edges and vertices of each added with its rank; `np` points `edge t tol` (the
// point of an edge at the fraction `t` of its range); `nsd` same domain pairs `a b`; `nse`
// section edges `e`; `ni` interferences `target type sb sa shb sha ib ia sk s gk g t gb cf`
// appended to the shape `target`. A reference `k*1000+n` is the shape `n` (modulo their
// count) of kind `k` (0 solid, 1 face, 2 edge, 3 vertex, 4 point index); the parameter is
// the fraction `t` of the range of the target edge. Then `op` runs and prints its results
// and the interferences of every shape, the sorted section edges and the kept points.
#include <algorithm>
#include <BRepAdaptor_Curve.hxx>
#include <TopExp_Explorer.hxx>
#include <TopOpeBRepDS_HDataStructure.hxx>
#include <TopOpeBRepDS_DataStructure.hxx>
#include <TopOpeBRepDS_CurvePointInterference.hxx>
#include <TopOpeBRepDS_EdgeVertexInterference.hxx>
#include <TopOpeBRepDS_FaceEdgeInterference.hxx>
#include <TopOpeBRepDS_ShapeShapeInterference.hxx>
#include <TopOpeBRepDS_SurfaceCurveInterference.hxx>
#include <TopOpeBRepDS_SolidSurfaceInterference.hxx>
#include <TopOpeBRepDS_TKI.hxx>
#include <TopOpeBRepDS_EXPORT.hxx>
#include <TopOpeBRepDS_ProcessInterferencesTool.hxx>
#include <TopOpeBRepDS_repvg.hxx>
#include <TopOpeBRepDS_EIR.hxx>
#include <TopOpeBRepDS_FIR.hxx>
#include <TopOpeBRepDS_Filter.hxx>
#include <TopOpeBRepDS_Check.hxx>
#include <TopOpeBRepDS_Dumper.hxx>
#include <TopOpeBRepDS_TOOL.hxx>
#include <TopOpeBRepDS_connex.hxx>
#include <TopOpeBRepTool_ShapeClassifier.hxx>
TopoDS_Shape bl_tt_solid(GaReader& r);
// Defined in TopOpeBRepDS_redu.cxx and TopOpeBRepDS_connex.cxx (the header declares a
// FUN_scanloi without the EXT list).
Standard_EXPORT void FUN_scanloi(const NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,int&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,int&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,int&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,int&);
void FDSCNX_Close();
Standard_EXPORT void FUN_resolveEUNKNOWN(NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,TopOpeBRepDS_DataStructure&,const int);
Standard_EXPORT void FUN_unkeepEsymetrictransitions(NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,const TopOpeBRepDS_DataStructure&,const int);
Standard_EXPORT void FUN_unkeepFdoubleGBoundinterferences(NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,const TopOpeBRepDS_DataStructure&,const int);
Standard_EXPORT void FUN_reclSE(const int,const TopOpeBRepDS_DataStructure&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&,NCollection_List<occ::handle<TopOpeBRepDS_Interference>>&);

// The built structure with the indices of its solids, faces, edges and vertices.
struct BlDs{occ::handle<TopOpeBRepDS_HDataStructure> hds;std::vector<int> idx[4];int np=0;};
typedef NCollection_List<occ::handle<TopOpeBRepDS_Interference>> BlLoi;

// The index of a reference.
static int bl_ds_ref(const BlDs& d,int ref){
 int k=ref/1000,n=ref%1000;if(k==4)return d.np==0?0:n%d.np+1;
 const std::vector<int>& v=d.idx[k];return v.empty()?0:v[n%v.size()];}

// The dynamic type code of an interference.
static int bl_ds_type(const occ::handle<TopOpeBRepDS_Interference>& i){
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_CurvePointInterference)))return 0;
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_EdgeVertexInterference)))return 1;
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_FaceEdgeInterference)))return 2;
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_ShapeShapeInterference)))return 3;
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_SurfaceCurveInterference)))return 4;
 if(i->IsInstance(STANDARD_TYPE(TopOpeBRepDS_SolidSurfaceInterference)))return 5;
 return 6;}

// An interference: type, transition, kinds and indices, parameter, GBound and config.
static void bl_ds_i(const occ::handle<TopOpeBRepDS_Interference>& i){
 if(i.IsNull()){ga_int(-1);return;}
 int ty=bl_ds_type(i);ga_int(ty);const TopOpeBRepDS_Transition& t=i->Transition();
 ga_int((int)t.Before());ga_int((int)t.After());ga_int((int)t.ShapeBefore());ga_int((int)t.ShapeAfter());
 ga_int(t.IndexBefore());ga_int(t.IndexAfter());
 ga_int((int)i->SupportType());ga_int(i->Support());ga_int((int)i->GeometryType());ga_int(i->Geometry());
 if(ty==0)ga_real(occ::down_cast<TopOpeBRepDS_CurvePointInterference>(i)->Parameter());
 if(ty==1)ga_real(occ::down_cast<TopOpeBRepDS_EdgeVertexInterference>(i)->Parameter());
 if(ty>=1&&ty<=3){occ::handle<TopOpeBRepDS_ShapeShapeInterference> s=occ::down_cast<TopOpeBRepDS_ShapeShapeInterference>(i);
  ga_flag(s->GBound());ga_int((int)s->Config());}}

static void bl_ds_loi(const BlLoi& l){ga_int(l.Extent());for(BlLoi::Iterator it(l);it.More();it.Next())bl_ds_i(it.Value());}

// The interferences of every shape, the section edges and the kept points.
static void bl_ds_dump(const BlDs& d){
 const TopOpeBRepDS_DataStructure& ds=d.hds->DS();ga_int(ds.NbShapes());
 for(int i=1;i<=ds.NbShapes();i++){const BlLoi& l=ds.ShapeInterferences(i);if(l.IsEmpty())continue;ga_int(i);bl_ds_loi(l);}
 // the section edges sorted: TOOL::EShareG adds them in the order of a shape map
 std::vector<int> se;for(int i=1;i<=ds.NbSectionEdges();i++)se.push_back(ds.Shape(ds.SectionEdge(i)));
 std::sort(se.begin(),se.end());ga_int(ds.NbSectionEdges());for(int i:se)ga_int(i);
 for(int i=1;i<=ds.NbPoints();i++)ga_flag(ds.KeepPoint(i));}

// One interference record.
static occ::handle<TopOpeBRepDS_Interference> bl_ds_make(BlDs& d,GaReader& r,int& target){
 TopOpeBRepDS_DataStructure& ds=d.hds->ChangeDS();
 target=bl_ds_ref(d,r.integer());int ty=r.integer();
 TopAbs_State sb=(TopAbs_State)r.integer(),sa=(TopAbs_State)r.integer();
 TopAbs_ShapeEnum shb=(TopAbs_ShapeEnum)r.integer(),sha=(TopAbs_ShapeEnum)r.integer();
 int ib=bl_ds_ref(d,r.integer()),ia=bl_ds_ref(d,r.integer());
 TopOpeBRepDS_Kind sk=(TopOpeBRepDS_Kind)r.integer();int s=bl_ds_ref(d,r.integer());
 TopOpeBRepDS_Kind gk=(TopOpeBRepDS_Kind)r.integer();int g=bl_ds_ref(d,r.integer());
 double t=r.real();bool gb=r.integer()!=0;TopOpeBRepDS_Config cf=(TopOpeBRepDS_Config)r.integer();
 double p=t;const TopoDS_Shape& ts=ds.Shape(target);
 if(!ts.IsNull()&&ts.ShapeType()==TopAbs_EDGE){double f,l;BRep_Tool::Range(TopoDS::Edge(ts),f,l);p=f+t*(l-f);}
 TopOpeBRepDS_Transition tr(sb,sa,shb,sha);tr.IndexBefore(ib);tr.IndexAfter(ia);
 if(ty==0)return new TopOpeBRepDS_CurvePointInterference(tr,sk,s,gk,g,p);
 if(ty==1)return new TopOpeBRepDS_EdgeVertexInterference(tr,sk,s,g,gb,cf,p);
 if(ty==2)return new TopOpeBRepDS_FaceEdgeInterference(tr,s,g,gb,cf);
 return new TopOpeBRepDS_ShapeShapeInterference(tr,sk,s,gk,g,gb,cf);}

// The structure of a request.
static BlDs bl_ds_build(GaReader& r){
 BlDs d;d.hds=new TopOpeBRepDS_HDataStructure();TopOpeBRepDS_DataStructure& ds=d.hds->ChangeDS();
 TopoDS_Shape s[2];s[0]=bl_tt_solid(r);s[1]=bl_tt_solid(r);
 for(int k=0;k<2;k++)d.idx[0].push_back(ds.AddShape(s[k],k+1));
 TopAbs_ShapeEnum ty[3]={TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX};
 for(int k=0;k<2;k++)for(int j=0;j<3;j++)for(TopExp_Explorer ex(s[k],ty[j]);ex.More();ex.Next()){
  bool had=ds.HasShape(ex.Current());int i=ds.AddShape(ex.Current(),k+1);if(!had)d.idx[j+1].push_back(i);}
 int np=r.integer();for(int k=0;k<np;k++){int e=bl_ds_ref(d,r.integer());double t=r.real(),tol=r.real();
  BRepAdaptor_Curve bc(TopoDS::Edge(ds.Shape(e)));double p=bc.FirstParameter()+t*(bc.LastParameter()-bc.FirstParameter());
  ds.AddPoint(TopOpeBRepDS_Point(bc.Value(p),tol));}
 d.np=np;
 int nsd=r.integer();for(int k=0;k<nsd;k++){int a=bl_ds_ref(d,r.integer()),b=bl_ds_ref(d,r.integer());
  ds.FillShapesSameDomain(ds.Shape(a),ds.Shape(b));}
 int nse=r.integer();for(int k=0;k<nse;k++)ds.AddSectionEdge(TopoDS::Edge(ds.Shape(bl_ds_ref(d,r.integer()))));
 int ni=r.integer();for(int k=0;k<ni;k++){int target;occ::handle<TopOpeBRepDS_Interference> i=bl_ds_make(d,r,target);
  ds.ChangeShapeInterferences(target).Append(i);}
 return d;}

#include "tkbool_probe_topopebrepds_ops_run.cxx"

// `dsx solids... op target arg`: the structure built, the operation `op` on the shape
// `target` (a reference) with the integer `arg`, then the interferences of every shape.
void bl_dsx_case(std::istream& input){
 GaReader r{input};BlDs d;
 try{d=bl_ds_build(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int op=r.integer(),target=bl_ds_ref(d,r.integer()),arg=r.integer();
 GA_TRY(bl_dsx_run(d,op,target,arg))
 GA_TRY(bl_ds_dump(d))
 FDSCNX_Close();
 ga_end();}
