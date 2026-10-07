// ShapeProcess and ShapeProcessAPI probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_process.rs.
//
// Resources come from the fixed tables below (requests are numbers only): a parameter is
// an index into SP_PARAMS (name, value) set with Resource_Manager::SetResource.
//  `sp_context nops (op)...`: a ShapeProcess_ShapeContext("", "") and the ops: 0 k set
//    parameter k; 1 s SetScope(SP_SCOPES[s]); 2 UnSetScope; 3..9 n: on SP_NAMES[n]
//    IsParamSet, GetReal, GetInteger, GetBoolean, GetString, StringVal(def "def"),
//    GetContinuity (a flag, then the value when found; exceptions printed).
//  `sp_process (prelude) program s mode trace nparams (k)...`: a ShapeProcess_ShapeContext
//    initialized on shape s, a messenger without printers, the trace level and the
//    parameters; mode >= 0 runs ShapeProcess::Perform(ctx, "seq"), mode < 0 the operation
//    flags -mode (bit i for Operation i). Prints the return, the result digest, the map
//    extent and the number of shapes with messages.
//  `sp_apply (prelude) program s nparams (k)...`: ShapeProcessAPI_ApplySequence("", "seq")
//    with the parameters set on its context, PrepareShape(s) and the same digests.
#include <Message_Messenger.hxx>
#include <ShapeProcess.hxx>
#include <ShapeProcessAPI_ApplySequence.hxx>
#include <ShapeProcess_OperLibrary.hxx>
#include <ShapeProcess_ShapeContext.hxx>
#include <Resource_Manager.hxx>
const char* const SP_PARAMS[][2]={
 {"seq.exec.op","FixShape"},{"seq.exec.op","DirectFaces,SameParameter;SetTolerance"},
 {"seq.exec.op","SplitAngle ToBezier"},{"seq.exec.op","Unknown,FixShape"},
 {"seq.exec.op","BSplineRestriction"},{"seq.exec.op","SplitContinuity SplitClosedFaces SplitClosedEdges"},
 {"seq.exec.op","FixWireGaps FixFaceSize DropSmallSolids DropSmallEdges SplitCommonVertex"},
 {"seq.exec.op","ElementaryToRevolution SweptToElementary SurfaceToBSpline"},
 {"seq.FixShape.Tolerance3d","1.e-4"},{"seq.FixShape.MaxTolerance3d","1."},
 {"seq.SetTolerance.Mode","1"},{"seq.SetTolerance.Value","& seq.Base"},{"seq.Base","0.001"},
 {"seq.SplitAngle.Angle","1.5"},{"seq.BSplineRestriction.RequiredDegree","3"},
 {"seq.BSplineRestriction.PlaneMode","1"},{"seq.BSplineRestriction.Continuity3d","c2"},
 {"seq.SetTolerance.Regularity","0.01"},{"seq.ToBezier.SurfaceMode","1"},
 {"seq.ToBezier.Curve3dMode","1"},{"seq.FixShape.FixTailMode","1"},{"seq.SetTolerance.Ratio","2"},
 {"Blank","  "},{"Word"," c1 "},{"Int","12abc"},{"seq.SurfaceToBSpline.OffsetMode","0"},
 {"FixShape.Tolerance3d","1e-3"},{"SplitAngle.Angle","1.0"},{"seq.Alias","&Missing"},
 {"seq.Real","2.5e-1"},{"ToBezier.SurfaceMode","1"},{"ToBezier.Curve3dMode","1"},
 {"ToBezier.Curve2dMode","1"}};
const char* const SP_SCOPES[]={"seq","FixShape","SetTolerance"};
const char* const SP_NAMES[]={"exec.op","FixShape.Tolerance3d","Base","Blank","Word","Int",
 "SetTolerance.Value","Missing","Alias","Real"};
void sp_set(ShapeProcess_Context& c,int k){c.ResourceManager()->SetResource(SP_PARAMS[k][0],SP_PARAMS[k][1]);}
void sp_query(const ShapeProcess_ShapeContext& c,int op,const char* n){
 if(op==3){ga_flag(c.IsParamSet(n));return;}
 if(op==4){double v=0.;GA_TRY(bool f=c.GetReal(n,v);ga_flag(f);if(f)ga_real(v))return;}
 if(op==5){int v=0;GA_TRY(bool f=c.GetInteger(n,v);ga_flag(f);if(f)ga_int(v))return;}
 if(op==6){bool v=false;bool f=c.GetBoolean(n,v);ga_flag(f);if(f)ga_flag(v);return;}
 if(op==7){TCollection_AsciiString v;bool f=c.GetString(n,v);ga_flag(f);if(f)ga_text(v.ToCString());return;}
 if(op==8){ga_text(c.StringVal(n,"def"));return;}
 GeomAbs_Shape v=GeomAbs_C0;bool f=c.GetContinuity(n,v);ga_flag(f);if(f)ga_int(v);}
void sp_context_case(std::istream& input){
 GaReader r{input};occ::handle<ShapeProcess_ShapeContext> c=new ShapeProcess_ShapeContext("","");
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0)sp_set(*c,r.integer());else if(op==1)c->SetScope(SP_SCOPES[r.integer()]);
  else if(op==2)c->UnSetScope();else sp_query(*c,op,SP_NAMES[r.integer()]);}
 ga_end();}
void sp_digest(const occ::handle<ShapeProcess_ShapeContext>& c){
 tp_shape(c->Result());ga_int(c->Map().Extent());
 ga_int(c->Messages().IsNull()?-1:c->Messages()->MapShape().Extent());}
occ::handle<Message_Messenger> sp_silent(){
 occ::handle<Message_Messenger> m=new Message_Messenger();m->ChangePrinters().Clear();return m;}
void sp_process_case(std::istream& input){
 ShapeProcess_OperLibrary::Init();
 GaReader r{input};TpShapes t=shc_shapes(r);TopoDS_Shape s=sh_shape(t,r.integer());
 int mode=r.integer(),trace=r.integer();
 occ::handle<ShapeProcess_ShapeContext> c=new ShapeProcess_ShapeContext(s,"","");
 c->SetMessenger(sp_silent());c->SetTraceLevel(trace);
 int n=r.integer();for(int k=0;k<n;++k)sp_set(*c,r.integer());
 GA_TRY(if(mode>=0)ga_flag(ShapeProcess::Perform(c,"seq"));
  else{ShapeProcess::OperationsFlags f;for(int i=0;i<=ShapeProcess::Last;++i)f.set(i,((-mode)>>i)&1);
   ga_flag(ShapeProcess::Perform(c,f));})
 sp_digest(c);ga_end();}
void sp_apply_case(std::istream& input){
 GaReader r{input};TpShapes t=shc_shapes(r);TopoDS_Shape s=sh_shape(t,r.integer());
 ShapeProcessAPI_ApplySequence api("","seq");api.Context()->SetMessenger(sp_silent());
 int n=r.integer();for(int k=0;k<n;++k)sp_set(*api.Context(),r.integer());
 GA_TRY(tp_shape(api.PrepareShape(s)))sp_digest(api.Context());ga_end();}
