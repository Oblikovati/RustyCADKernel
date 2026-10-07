// Graphic3d_ClipPlane, the Rust twin of crates/tkservice/examples/sv_probe/clip_plane.rs.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_Texture2D.hxx>
void sv_vec4(const NCollection_Vec4<double>& v){ga_real(v.x());ga_real(v.y());ga_real(v.z());ga_real(v.w());}
NCollection_Vec4<double> sv_read_vec4(GaReader& r){double x=r.real(),y=r.real(),z=r.real(),w=r.real();return NCollection_Vec4<double>(x,y,z,w);}
// `Kind A B C D`: 0 default + SetEquation(Vec4), 1 Vec4 constructor, 2 gp_Pln constructor,
// 3 default + SetEquation(gp_Pln), 4 default plane.
occ::handle<Graphic3d_ClipPlane> sv_clip_make(int kind,const NCollection_Vec4<double>& e){
 if(kind==1)return new Graphic3d_ClipPlane(e);
 if(kind==2)return new Graphic3d_ClipPlane(gp_Pln(e.x(),e.y(),e.z(),e.w()));
 occ::handle<Graphic3d_ClipPlane> p=new Graphic3d_ClipPlane();
 if(kind==0)p->SetEquation(e);
 if(kind==3)p->SetEquation(gp_Pln(e.x(),e.y(),e.z(),e.w()));
 return p;}
void sv_clip_state(const occ::handle<Graphic3d_ClipPlane>& p){
 sv_vec4(p->GetEquation());sv_vec4(p->ReversedEquation());const gp_Pln& pl=p->ToPlane();
 ga_pnt(pl.Location());ga_dir(pl.Axis().Direction());ga_dir(pl.XAxis().Direction());ga_dir(pl.YAxis().Direction());
 ga_int((int)p->MCountEquation());ga_flag(p->IsOn());ga_flag(p->IsChain());ga_int(p->NbChainNextPlanes());}
// `Kind A B C D NChain (A B C D)* NPoints (X Y Z W)* NBoxes (MinXYZ MaxXYZ)*`: a plane, a chain
// of planes after it, the point and box tests of the chain, then the chain cut.
void sv_clip_case(std::istream& input){
 GaReader r{input};int kind=r.integer();NCollection_Vec4<double> e=sv_read_vec4(r);
 occ::handle<Graphic3d_ClipPlane> head;
 try{head=sv_clip_make(kind,e);}catch(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';ga_end();return;}
 sv_clip_state(head);
 occ::handle<Graphic3d_ClipPlane> last=head;int nchain=r.integer();
 for(int i=0;i<nchain;++i){NCollection_Vec4<double> ce=sv_read_vec4(r);
  GA_TRY(occ::handle<Graphic3d_ClipPlane> c=new Graphic3d_ClipPlane(ce);last->SetChainNextPlane(c);last=c;)}
 ga_int(head->NbChainNextPlanes());ga_flag(head->IsChain());ga_int(last->NbChainNextPlanes());
 ga_flag(last->ChainPreviousPlane().IsNull());GA_TRY(last->SetOn(true))
 int npts=r.integer();
 for(int i=0;i<npts;++i){NCollection_Vec4<double> pt=sv_read_vec4(r);
  ga_int(head->ProbePoint(pt));ga_int(head->ProbePointHalfspace(pt));ga_flag(head->IsPointOutHalfspace(pt));}
 int nbox=r.integer();
 for(int i=0;i<nbox;++i){double v[6];for(double& x:v)x=r.real();
  Graphic3d_BndBox3d box(NCollection_Vec3<double>(v[0],v[1],v[2]),NCollection_Vec3<double>(v[3],v[4],v[5]));
  ga_int(head->ProbeBox(box));ga_flag(head->ProbeBoxTouch(box));ga_int(head->ProbeBoxHalfspace(box));
  ga_flag(head->IsBoxFullOutHalfspace(box));ga_flag(head->IsBoxFullInHalfspace(box));ga_int(head->ProbeBoxMaxPointHalfspace(box));}
 occ::handle<Graphic3d_ClipPlane> second=head->ChainNextPlane();head->SetChainNextPlane(occ::handle<Graphic3d_ClipPlane>());
 ga_int(head->NbChainNextPlanes());ga_flag(head->IsChain());ga_int((int)head->MCountEquation());
 if(!second.IsNull()){ga_flag(second->ChainPreviousPlane().IsNull());GA_TRY(second->SetOn(false))ga_flag(second->IsOn());}
 ga_end();}
void sv_clip_capping_state(const occ::handle<Graphic3d_ClipPlane>& p){
 Quantity_Color c=p->CappingColor();ga_real(c.Red());ga_real(c.Green());ga_real(c.Blue());
 ga_int(p->CappingMaterial().Name());ga_flag(p->IsHatchOn());
 bool hasHatch=!p->CappingCustomHatch().IsNull();ga_flag(hasHatch);if(hasHatch)ga_int(p->CappingHatch());
 ga_flag(!p->CappingTexture().IsNull());ga_flag(p->CappingAspect()->ToMapTexture());
 ga_flag(p->ToUseObjectMaterial());ga_flag(p->ToUseObjectTexture());ga_flag(p->ToUseObjectShader());ga_flag(p->ToUseObjectProperties());
 ga_flag(p->IsCapping());ga_int((int)p->MCountAspect());ga_int((int)p->MCountEquation());}
// `N (Op Args)*`, then the capping state after each operation: 1 R G B color, 2 Material
// name, 3 Style hatch, 4 hatch on, 5 hatch off, 6 Flag On object flag, 7 On capping, 8 clone,
// 9 With texture (1 a file texture, 0 none), 10 A B C D SetEquation.
void sv_clip_capping_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_ClipPlane> p=new Graphic3d_ClipPlane();int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();
  if(op==1){double cr=r.real(),cg=r.real(),cb=r.real();GA_TRY(p->SetCappingColor(Quantity_Color(cr,cg,cb,Quantity_TOC_RGB)))}
  else if(op==2)p->SetCappingMaterial(Graphic3d_MaterialAspect((Graphic3d_NameOfMaterial)r.integer()));
  else if(op==3)p->SetCappingHatch((Aspect_HatchStyle)r.integer());
  else if(op==4)p->SetCappingHatchOn();
  else if(op==5)p->SetCappingHatchOff();
  else if(op==6){int flag=r.integer();bool on=r.flag();
   if(flag==0)p->SetUseObjectMaterial(on);else if(flag==1)p->SetUseObjectTexture(on);else p->SetUseObjectShader(on);}
  else if(op==7)p->SetCapping(r.flag());
  else if(op==8)p=p->Clone();
  else if(op==9){bool with=r.flag();
   p->SetCappingTexture(with?occ::handle<Graphic3d_TextureMap>(new Graphic3d_Texture2D(TCollection_AsciiString("capping.png"))):occ::handle<Graphic3d_TextureMap>());}
  else if(op==10){NCollection_Vec4<double> e=sv_read_vec4(r);GA_TRY(p->SetEquation(e))}
  sv_clip_capping_state(p);}
 ga_end();}
