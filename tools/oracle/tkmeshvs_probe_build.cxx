// The presentation builders and the mesh, the twin of crates/tkmeshvs/examples/mv_probe/build.rs.
class MvMesh:public MeshVS_Mesh{
public:
  MvMesh(bool overlapped):MeshVS_Mesh(overlapped){}
  bool accepts(int mode) const{return AcceptDisplayMode(mode);}
};
// Drawer changes of a request, bit by bit (see mv_drawer in build.rs).
static void mv_drawer(const occ::handle<MeshVS_Drawer>& d,int bits){
  if(bits&1)d->SetBoolean(MeshVS_DA_Reflection,false);
  if(bits&2)d->SetBoolean(MeshVS_DA_SmoothShading,true);
  if(bits&4)d->SetBoolean(MeshVS_DA_ShowEdges,false);
  if(bits&8)d->SetBoolean(MeshVS_DA_IsAllowOverlapped,true);
  if(bits&16)d->SetBoolean(MeshVS_DA_ColorReflection,true);
  if(bits&32)d->SetBoolean(MeshVS_DA_SupressBackFaces,true);
  if(bits&64)d->SetBoolean(MeshVS_DA_DisplayNodes,false);
  if(bits&128)d->SetDouble(MeshVS_DA_ShrinkCoeff,.5);
  if(bits&256)d->SetInteger(MeshVS_DA_MaxFaceNodes,6);
  if(bits&512){Graphic3d_MaterialAspect m(Graphic3d_NameOfMaterial_Gold);m.SetTransparency(.5f);d->SetMaterial(MeshVS_DA_FrontMaterial,m);}
  if(bits&1024){d->SetInteger(MeshVS_DA_EdgeType,1);d->SetInteger(MeshVS_DA_BeamType,2);}
  if(bits&2048){d->SetAsciiString(MeshVS_DA_TextFont,"Arial");d->SetInteger(MeshVS_DA_TextFontAspect,0);d->SetInteger(MeshVS_DA_TextStyle,1);}
  if(bits&4096){d->SetDouble(MeshVS_DA_EdgeWidth,2.5);d->SetDouble(MeshVS_DA_BeamWidth,4.);}
  if(bits&8192)d->SetInteger(MeshVS_DA_MaxFaceNodes,4);
  if(bits&16384)d->SetBoolean(MeshVS_DA_SelectableAuto,true);
}
static occ::handle<TColStd_HPackedMapOfInteger> mv_ids(int mask,int first,int count){
  if(mask<0)return nullptr;occ::handle<TColStd_HPackedMapOfInteger> m=new TColStd_HPackedMapOfInteger;
  for(int i=0;i<count;++i)if(mask&(1<<i))m->ChangeMap().Add(first+i);return m;}
static Quantity_Color mv_node_color(int i){return Quantity_Color(i/14.,1.-i/14.,.5,Quantity_TOC_RGB);}
static occ::handle<MeshVS_PrsBuilder> mv_builder(const occ::handle<MeshVS_Mesh>& mesh,int kind,int variant){
  switch(kind){
    case 0:return new MeshVS_MeshPrsBuilder(mesh);
    case 1:{occ::handle<MeshVS_NodalColorPrsBuilder> b=new MeshVS_NodalColorPrsBuilder(mesh);
      if(variant==1||variant==3){b->UseTexture(true);NCollection_Sequence<Quantity_Color> map;map.Append(Quantity_NOC_RED);
        if(variant==1){map.Append(Quantity_NOC_GREEN);map.Append(Quantity_NOC_BLUE);}b->SetColorMap(map);
        b->SetInvalidColor(Quantity_NOC_BLACK);for(int i=1;i<=14;++i)b->SetTextureCoord(i,i/14.-.1);}
      else for(int i=1;i<=14;++i)if(variant!=2||i!=5)b->SetColor(i,mv_node_color(i));
      return b;}
    case 2:{occ::handle<MeshVS_ElementalColorPrsBuilder> b=new MeshVS_ElementalColorPrsBuilder(mesh);
      if(variant!=1){const Quantity_NameOfColor c[8]={Quantity_NOC_RED,Quantity_NOC_RED,Quantity_NOC_BLUE1,Quantity_NOC_RED,Quantity_NOC_GREEN,Quantity_NOC_BLUE1,Quantity_NOC_RED,Quantity_NOC_YELLOW};
        for(int i=0;i<8;++i)if(variant==0||MV_ELEMENTS[i].type!=MeshVS_ET_Face)b->SetColor1(101+i,c[i]);}
      if(variant!=0){b->SetColor2(101,Quantity_NOC_RED,Quantity_NOC_BLUE1);b->SetColor2(102,Quantity_NOC_RED,Quantity_NOC_BLUE1);
        b->SetColor2(103,Quantity_NOC_GREEN,Quantity_NOC_GREEN);b->SetColor2(108,BindTwoColors(Quantity_NOC_YELLOW,Quantity_NOC_CYAN1));}
      return b;}
    case 3:{occ::handle<MeshVS_TextPrsBuilder> b=new MeshVS_TextPrsBuilder(mesh,12.,Quantity_NOC_RED);
      for(int i=1;i<=14;++i)b->SetText(false,i,TCollection_AsciiString("N")+i);
      for(int i=101;i<=108;++i)if(variant!=1||i!=103)b->SetText(true,i,TCollection_AsciiString("E")+i);
      return b;}
    case 4:{occ::handle<MeshVS_VectorPrsBuilder> b=new MeshVS_VectorPrsBuilder(mesh,2.,Quantity_NOC_GREEN);
      b->SetSimplePrsMode((variant&1)!=0);if(variant&2)b->SetSimplePrsParams(3.,.7,.9);
      for(int i=1;i<=14;++i)b->SetVector(false,i,i==4?gp_Vec(0,0,0):gp_Vec(.1*i,-.05*i,.2));
      for(int i=101;i<=108;++i)b->SetVector(true,i,gp_Vec(0,0,.1*(i-100)));
      return b;}
    default:return new MeshVS_MeshPrsBuilder(mesh,MeshVS_DMF_OCCMask|MeshVS_DMF_DeformedMask);
  }}
// `mv_build Kind Variant Mode IsElement IdMask DrawerBits HiddenNodes HiddenElems ExclMask Excluding`.
void mv_build(std::istream& input){
  GaReader r{input};const int kind=r.integer(),variant=r.integer(),mode=r.integer();const bool isElement=r.flag();
  const int idMask=r.integer(),bits=r.integer(),hiddenNodes=r.integer(),hiddenElems=r.integer(),excl=r.integer(),excluding=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<MvMesh> mesh=new MvMesh(false);mesh->SetDataSource(mv_source(3));mv_drawer(mesh->GetDrawer(),bits);
  mesh->SetHiddenNodes(mv_ids(hiddenNodes,1,14));mesh->SetHiddenElems(mv_ids(hiddenElems,101,8));
  occ::handle<MeshVS_PrsBuilder> b=mv_builder(mesh,kind,variant);if(excluding>=0)b->SetExcluding(excluding!=0);
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  TColStd_PackedMapOfInteger ids=mv_ids(idMask,isElement?101:1,isElement?8:14)->Map();
  TColStd_PackedMapOfInteger excluded=mv_ids(excl,isElement?101:1,isElement?8:14)->Map();
  ga_int(b->GetId());ga_int(b->GetPriority());ga_int(b->GetFlags());ga_flag(b->IsExcludingOn());ga_flag(b->TestFlags(mode));
  GA_TRY(b->Build(prs,ids,excluded,isElement,mode))
  mv_trace(trace);mv_map(excluded);mv_structure(prs);
  if(kind==2){auto e=occ::down_cast<MeshVS_ElementalColorPrsBuilder>(b);ga_int(e->GetColors1().Extent());ga_int(e->GetColors2().Extent());
    Quantity_Color c;ga_flag(e->GetColor1(103,c));mv_color(c);Quantity_Color c1,c2;ga_flag(e->GetColor2(101,c1,c2));mv_color(c1);mv_color(c2);}
  if(kind==1){auto n=occ::down_cast<MeshVS_NodalColorPrsBuilder>(b);ga_flag(n->IsUseTexture());ga_int(n->GetColors().Extent());ga_int(n->GetColorMap().Length());
    mv_color(n->GetInvalidColor());ga_real(n->GetTextureCoord(3));ga_real(n->GetTextureCoord(99));
    auto shading=mesh->Attributes()->ShadingAspect();ga_flag(shading.IsNull());if(!shading.IsNull())mv_aspect(shading->Aspect());}
  if(kind==4){auto v=occ::down_cast<MeshVS_VectorPrsBuilder>(b);double mn=-1,mx=-1;v->GetMinMaxVectorValue(false,mn,mx);ga_real(mn);ga_real(mx);
    NCollection_Array1<gp_Pnt> a(3,10);ga_real(MeshVS_VectorPrsBuilder::calculateArrow(a,2.,.1));for(const gp_Pnt& p:a)ga_pnt(p);}
  ga_end();}
// `mv_compute Mode BuilderSet DrawerBits Overlapped`: MeshVS_Mesh::Compute over several builders.
void mv_compute(std::istream& input){
  GaReader r{input};const int mode=r.integer(),set=r.integer(),bits=r.integer();const bool overlapped=r.flag();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<MvMesh> mesh=new MvMesh(overlapped);mesh->SetDataSource(mv_source(3));mv_drawer(mesh->GetDrawer(),bits|0);
  for(int k=0;k<5;++k)if(set&(1<<k))mesh->AddBuilder(mv_builder(mesh,k,0),k==0);
  for(int m:{-1,0,1,2,3,4,8,16,32,64,128,1024})ga_flag(mesh->accepts(m));
  for(int i=1;i<=mesh->GetBuildersCount();++i){ga_int(mesh->GetBuilder(i)->GetId());ga_int(mesh->GetBuilder(i)->GetPriority());}
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  GA_TRY(mesh->Compute(nullptr,prs,mode))
  mv_trace(trace);mv_structure(prs);ga_end();}
static void mv_builders_state(const occ::handle<MeshVS_Mesh>& mesh){
  ga_int(mesh->GetBuildersCount());for(int i=1;i<=mesh->GetBuildersCount();++i){ga_int(mesh->GetBuilder(i)->GetId());ga_int(mesh->GetBuilder(i)->GetPriority());}
  ga_int(mesh->GetHilighter().IsNull()?-1:mesh->GetHilighter()->GetId());ga_int(mesh->GetFreeId());
  auto text=mesh->FindBuilder(STANDARD_TYPE(MeshVS_TextPrsBuilder));ga_int(text.IsNull()?-1:text->GetId());
  auto any=mesh->FindBuilder(STANDARD_TYPE(MeshVS_PrsBuilder));ga_int(any.IsNull()?-1:any->GetId());
  ga_flag(mesh->GetBuilder(0).IsNull());ga_flag(mesh->GetBuilderById(7).IsNull());
  for(int i:{1,2,7,14})ga_flag(mesh->IsHiddenNode(i)),ga_flag(mesh->IsSelectableNode(i));
  for(int i:{101,104,106})ga_flag(mesh->IsHiddenElem(i)),ga_flag(mesh->IsSelectableElem(i));
  mv_hmap(mesh->GetSelectableNodes());ga_int(int(mesh->GetMeshSelMethod()));}
// `mv_mesh_api Count Op...`: builder sequence, highlighter, hidden and selectable maps.
void mv_mesh_api(std::istream& input){
  GaReader r{input};const int count=r.integer();occ::handle<MvMesh> mesh=new MvMesh(false);mesh->SetDataSource(mv_source(0));
  ga_int(mesh->DisplayMode());ga_int(mesh->HilightMode());ga_flag(mesh->IsAutoHilight());mv_color(mesh->Attributes()->Color());ga_flag(mesh->HasColor());ga_flag(mesh->HasMaterial());
  for(int i=0;i<count;++i){const int op=r.integer();
    switch(op){
      case 0:mesh->AddBuilder(new MeshVS_MeshPrsBuilder(mesh));break;
      case 1:mesh->AddBuilder(new MeshVS_NodalColorPrsBuilder(mesh));break;
      case 2:mesh->AddBuilder(new MeshVS_TextPrsBuilder(mesh,10.,Quantity_NOC_RED),true);break;
      case 3:mesh->AddBuilder(new MeshVS_MeshPrsBuilder(mesh,MeshVS_DMF_OCCMask,nullptr,7,MeshVS_BP_ElemColor));break;
      case 4:mesh->RemoveBuilder(1);break;
      case 5:mesh->RemoveBuilderById(7);break;
      case 6:ga_flag(mesh->SetHilighter(2));break;
      case 7:ga_flag(mesh->SetHilighterById(0));break;
      case 8:mesh->RemoveBuilder(5);break;
      case 9:mesh->SetHilighter(occ::handle<MeshVS_PrsBuilder>());break;
      case 10:mesh->SetHiddenNodes(mv_ids(0x7,1,14));break;
      case 11:mesh->SetHiddenElems(mv_ids(0x9,101,8));break;
      case 12:mesh->UpdateSelectableNodes();break;
      case 13:mesh->SetSelectableNodes(nullptr);break;
      case 14:mesh->GetDrawer()->SetBoolean(MeshVS_DA_SelectableAuto,true);break;
      case 15:mesh->SetHiddenNodes(nullptr);break;
      case 16:mesh->SetMeshSelMethod(MeshVS_MSM_BOX);break;
      case 17:mesh->AddBuilder(nullptr);break;
      case 18:mesh->SetHiddenNodes(mv_ids(0,1,14));break;
      case 19:mesh->GetDrawer()->SetInteger(MeshVS_DA_MaxFaceNodes,2);break;
    }
    mv_builders_state(mesh);}
  ga_end();}
