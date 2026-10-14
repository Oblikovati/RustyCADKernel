// AIS_Triangulation presentations through the recording display driver, the twin of
// crates/tkv3d/examples/v3_probe/ais_triangulation.rs. A request is
// `v3_ais_triangulation Normals Colors Link Count Op...`.
#include <AIS_Triangulation.hxx>
#include <Poly_Triangulation.hxx>

static occ::handle<Poly_Triangulation> v3_tri_mesh(bool normals){
  occ::handle<Poly_Triangulation> t=new Poly_Triangulation(4,2,false,false);
  const gp_Pnt nodes[4]={gp_Pnt(0,0,0),gp_Pnt(2,0,0),gp_Pnt(2,1,0.5),gp_Pnt(0,1,0)};
  for(int i=1;i<=4;++i)t->SetNode(i,nodes[i-1]);
  t->SetTriangle(1,Poly_Triangle(1,2,3));t->SetTriangle(2,Poly_Triangle(1,3,4));
  if(normals){t->AddNormals();for(int i=1;i<=4;++i)t->SetNormal(i,gp_Dir(0.1*i,0.2,1.));}
  return t;
}
void v3_ais_triangulation(std::istream& input){
  GaReader r{input};const bool normals=r.flag(),colors=r.flag();const int link=r.integer(),count=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<AIS_Triangulation> o=new AIS_Triangulation(v3_tri_mesh(normals));
  if(link==0)o->Attributes()->SetupOwnDefaults();
  if(link==1){occ::handle<Prs3d_Drawer> d=new Prs3d_Drawer();d->SetupOwnDefaults();o->Attributes()->SetLink(d);}
  if(colors){occ::handle<NCollection_HArray1<int>> c=new NCollection_HArray1<int>(1,4);
    for(int i=1;i<=4;++i)c->SetValue(i,int(0x00102030u*unsigned(i)+0x40));o->SetColors(c);}
  ga_flag(o->HasVertexColors());ga_int(o->Type());ga_int(o->Signature());
  for(int i=0;i<count;++i){const int op=r.integer();
    switch(op){
      case 0:GA_TRY(if(manager->HasPresentation(o,0))manager->Update(o,0);else manager->Presentation(o,0,true);)break;
      case 1:GA_TRY(o->SetTransparency(0.4))break;
      case 2:GA_TRY(o->UnsetTransparency())break;
      case 3:GA_TRY(o->SetColor(Quantity_NOC_RED))break;
      case 4:GA_TRY(manager->Presentation(o,1,true))break;
      case 5:{occ::handle<SelectMgr_Selection> s=new SelectMgr_Selection(0);static_cast<SelectMgr_SelectableObject*>(o.get())->ComputeSelection(s,0);ga_int(s->Entities().Size());break;}
    }
    ga_flag(o->IsTransparent());ga_real(o->Transparency());ga_flag(o->Attributes()->HasOwnShadingAspect());ga_int(o->Presentations().Size());}
  v3_display_trace(trace);ga_end();
}
