// Prs3d numeric parts, the Rust twin of crates/tkv3d/examples/v3_probe/prs3d.rs.
#include <Bnd_Box.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Poly_Triangulation.hxx>
#include <Prs3d.hxx>
#include <Prs3d_Arrow.hxx>
#include <Prs3d_BndBox.hxx>
#include <Prs3d_ToolCylinder.hxx>
#include <Prs3d_ToolDisk.hxx>
#include <Prs3d_ToolSector.hxx>
#include <Prs3d_ToolSphere.hxx>
#include <Prs3d_ToolTorus.hxx>
#include <gp_Ax1.hxx>
#include <gp_Trsf.hxx>
// Reads `Tx Ty Tz Ax Ay Az Angle`: a rotation about the axis through the origin, then a
// translation part.
gp_Trsf v3_trsf(GaReader& r){
 gp_XYZ t=r.xyz();gp_Dir a=r.dir();double angle=r.real();gp_Trsf trsf;
 trsf.SetRotation(gp_Ax1(gp_Pnt(0,0,0),a),angle);trsf.SetTranslationPart(gp_Vec(t));return trsf;}
// The tool of a kind with its five parameters (unused ones are ignored).
std::unique_ptr<Prs3d_ToolQuadric> v3_tool(int kind,const double* p,int slices,int stacks){
 switch(kind){
  case 0:return std::make_unique<Prs3d_ToolCylinder>(p[0],p[1],p[2],slices,stacks);
  case 1:{auto d=std::make_unique<Prs3d_ToolDisk>(p[0],p[1],slices,stacks);d->SetAngleRange(p[2],p[3]);return d;}
  case 2:return std::make_unique<Prs3d_ToolSphere>(p[0],slices,stacks);
  case 3:return std::make_unique<Prs3d_ToolTorus>(p[0],p[1],p[2],p[3],p[4],slices,stacks);
  default:return std::make_unique<Prs3d_ToolSector>(p[0],slices,stacks);}}
// `Kind P0..P4 Slices Stacks Trsf`: the counts, the indexed array of CreateTriangulation, a
// non-indexed array filled by FillArray, and the nodes and triangles of CreatePolyTriangulation.
void v3_quadric_case(std::istream& input){
 GaReader r{input};int kind=r.integer();double p[5];for(double& v:p)v=r.real();int slices=r.integer(),stacks=r.integer();
 gp_Trsf trsf=v3_trsf(r);std::unique_ptr<Prs3d_ToolQuadric> tool=v3_tool(kind,p,slices,stacks);
 ga_int(tool->TrianglesNb());ga_int(tool->VerticesNb());ga_int(tool->VerticesNb(false));
 GA_TRY(occ::handle<Graphic3d_ArrayOfTriangles> a=tool->CreateTriangulation(trsf);sv_array_state(a))
 GA_TRY(occ::handle<Graphic3d_ArrayOfTriangles> b=new Graphic3d_ArrayOfTriangles(tool->VerticesNb(false),0,Graphic3d_ArrayFlags_VertexNormal);
  tool->FillArray(b,trsf);sv_array_state(b))
 GA_TRY(occ::handle<Poly_Triangulation> t=tool->CreatePolyTriangulation(trsf);ga_int(t->NbNodes());ga_int(t->NbTriangles());
  for(int i=1;i<=t->NbNodes();++i)ga_pnt(t->Node(i));
  for(int i=1;i<=t->NbTriangles();++i){int n1,n2,n3;t->Triangle(i).Get(n1,n2,n3);ga_int(n1);ga_int(n2);ga_int(n3);})
 ga_end();}
// `Loc(3) Dir(3) Angle Length NbSegments TubeRadius AxisLength ConeRadius ConeLength NbFacettes`:
// DrawSegments and DrawShaded.
void v3_arrow_case(std::istream& input){
 GaReader r{input};gp_Pnt loc=r.pnt();gp_Dir dir=r.dir();double angle=r.real(),length=r.real();int nb=r.integer();
 double tube=r.real(),axisLength=r.real(),cone=r.real(),coneLength=r.real();int facettes=r.integer();
 GA_TRY(sv_array_state(Prs3d_Arrow::DrawSegments(loc,dir,angle,length,nb)))
 GA_TRY(occ::handle<Graphic3d_ArrayOfTriangles> s=Prs3d_Arrow::DrawShaded(gp_Ax1(loc,dir),tube,axisLength,cone,coneLength,facettes);
  ga_flag(!s.IsNull());if(!s.IsNull())sv_array_state(s))
 ga_end();}
// `Min(3) Max(3) Coef MaxChord Open X Y Z Distance P1(3) P2(3)`: GetDeflection of the corners
// and of the box (void when Min.X > Max.X, opened in -X when Open), MatchSegment and the box
// segments.
void v3_prs3d_case(std::istream& input){
 GaReader r{input};gp_Pnt mn=r.pnt(),mx=r.pnt();double coef=r.real(),chord=r.real();bool open=r.flag();
 double x=r.real(),y=r.real(),z=r.real(),distance=r.real();gp_Pnt p1=r.pnt(),p2=r.pnt();
 ga_real(Prs3d::GetDeflection(NCollection_Vec3<double>(mn.X(),mn.Y(),mn.Z()),NCollection_Vec3<double>(mx.X(),mx.Y(),mx.Z()),coef));
 Bnd_Box box;if(mn.X()<=mx.X()){box.Update(mn.X(),mn.Y(),mn.Z(),mx.X(),mx.Y(),mx.Z());if(open)box.OpenXmin();}
 ga_real(Prs3d::GetDeflection(box,coef,chord));
 double dist=-1.0;ga_flag(Prs3d::MatchSegment(x,y,z,distance,p1,p2,dist));ga_real(dist);
 occ::handle<Graphic3d_ArrayOfSegments> segs=Prs3d_BndBox::FillSegments(box);ga_flag(!segs.IsNull());if(!segs.IsNull())sv_array_state(segs);
 ga_end();}
