// Graphic3d_ArrayOfPrimitives and its buffers, the Rust twin of
// crates/tkservice/examples/sv_probe/array.rs.
#include <Graphic3d_ArrayOfPrimitives.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_ArrayOfPolygons.hxx>
#include <Graphic3d_ArrayOfPolylines.hxx>
#include <Graphic3d_ArrayOfQuadrangles.hxx>
#include <Graphic3d_ArrayOfQuadrangleStrips.hxx>
#include <Graphic3d_ArrayOfSegments.hxx>
#include <Graphic3d_ArrayOfTriangleFans.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_ArrayOfTriangleStrips.hxx>
// One scripted operation; prints its result or the raised exception type.
void sv_array_op(const occ::handle<Graphic3d_ArrayOfPrimitives>& a,GaReader& r){
 int op=r.integer();
 switch(op){
  case 1:{double x=r.real(),y=r.real(),z=r.real();GA_TRY(ga_int(a->AddVertex(x,y,z)))break;}
  case 2:{double v[6];for(double& c:v)c=r.real();GA_TRY(ga_int(a->AddVertex(v[0],v[1],v[2],v[3],v[4],v[5])))break;}
  case 3:{double v[5];for(double& c:v)c=r.real();GA_TRY(ga_int(a->AddVertex(v[0],v[1],v[2],v[3],v[4])))break;}
  case 4:{double v[8];for(double& c:v)c=r.real();GA_TRY(ga_int(a->AddVertex(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7])))break;}
  case 5:{int i=r.integer();double cr=r.real(),cg=r.real(),cb=r.real();GA_TRY(a->SetVertexColor(i,cr,cg,cb))break;}
  case 6:{int i=r.integer(),c=r.integer();GA_TRY(a->SetVertexColor(i,c))break;}
  case 7:{int v=r.integer();GA_TRY(ga_int(a->AddEdge(v)))break;}
  case 8:{int n=r.integer();GA_TRY(ga_int(a->AddBound(n)))break;}
  case 9:{int n=r.integer();double cr=r.real(),cg=r.real(),cb=r.real();GA_TRY(ga_int(a->AddBound(n,cr,cg,cb)))break;}
  case 10:{int lo=r.integer(),hi=r.integer();GA_TRY(a->AddTriangleStripEdges(lo,hi))break;}
  case 11:{int lo=r.integer(),hi=r.integer();bool c=r.flag();GA_TRY(a->AddTriangleFanEdges(lo,hi,c))break;}
  case 12:{int lo=r.integer(),hi=r.integer();bool c=r.flag();GA_TRY(a->AddPolylineEdges(lo,hi,c))break;}
  case 13:{int i=r.integer();double cr=r.real(),cg=r.real(),cb=r.real();GA_TRY(a->SetBoundColor(i,cr,cg,cb))break;}
  case 14:{int v[4];for(int& c:v)c=r.integer();GA_TRY(ga_int(a->AddQuadTriangleEdges(v[0],v[1],v[2],v[3])))break;}
  case 15:{ga_flag(a->IsValid());break;}
  case 16:{int i=r.integer();double x=r.real(),y=r.real(),z=r.real();GA_TRY(a->SetVertexNormal(i,x,y,z))break;}
  case 18:{int i=r.integer();double c[3]={0,0,0};GA_TRY(a->BoundColor(i,c[0],c[1],c[2]);ga_real(c[0]);ga_real(c[1]);ga_real(c[2]))break;}
  case 17:{int i=r.integer();double x=r.real(),y=r.real();GA_TRY(a->SetVertexTexel(i,x,y))break;}
 }}
// The buffers: vertex layout and bytes, index stride and values, bounds and colors.
void sv_array_state(const occ::handle<Graphic3d_ArrayOfPrimitives>& a){
 ga_text(a->StringType());ga_int(a->ItemNumber());ga_int(a->EdgeNumber());ga_int(a->EdgeNumberAllocated());
 ga_int(a->BoundNumber());ga_int(a->BoundNumberAllocated());
 ga_flag(a->HasVertexNormals());ga_flag(a->HasVertexColors());ga_flag(a->HasVertexTexels());ga_flag(a->HasBoundColors());
 const occ::handle<Graphic3d_Buffer>& b=a->Attributes();ga_flag(!b.IsNull());
 if(!b.IsNull()){ga_int(a->VertexNumber());ga_int(a->VertexNumberAllocated());ga_int(b->Stride);ga_int(b->NbAttributes);
  ga_flag(b->IsInterleaved());ga_flag(b->IsMutable());
  for(size_t i=0;i<b->Size();++i)ga_int(b->Data()[i]);}
 const occ::handle<Graphic3d_IndexBuffer>& ix=a->Indices();ga_flag(!ix.IsNull());
 if(!ix.IsNull()){ga_int(ix->Stride);ga_flag(ix->IsMutable());for(int i=0;i<ix->NbElements;++i)ga_int(ix->Index(i));}
 // Bound colors are printed by op 18 only: the native allocation leaves unset colors
 // uninitialised.
 for(int i=1;i<=a->BoundNumber();++i)ga_int(a->Bound(i));
 for(int i=1;i<=(b.IsNull()?0:a->VertexNumber());++i){double v[3];a->Vertice(i,v[0],v[1],v[2]);ga_real(v[0]);ga_real(v[1]);ga_real(v[2]);
  a->VertexColor(i,v[0],v[1],v[2]);ga_real(v[0]);ga_real(v[1]);ga_real(v[2]);a->VertexNormal(i,v[0],v[1],v[2]);ga_real(v[0]);ga_real(v[1]);ga_real(v[2]);}}
// `Kind MaxVertexs MaxBounds MaxEdges Flags NbOps Op...`: CreateArray, the scripted operations,
// then the buffers.
void sv_array_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),maxV=r.integer(),maxB=r.integer(),maxE=r.integer(),flags=r.integer(),nbOps=r.integer();
 occ::handle<Graphic3d_ArrayOfPrimitives> a=Graphic3d_ArrayOfPrimitives::CreateArray((Graphic3d_TypeOfPrimitiveArray)kind,maxV,maxB,maxE,flags);
 ga_flag(!a.IsNull());
 if(!a.IsNull()){ga_text(a->DynamicType()->Name());for(int i=0;i<nbOps;++i)sv_array_op(a,r);sv_array_state(a);}
 ga_end();}
