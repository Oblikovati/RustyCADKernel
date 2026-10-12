// Included by the alias oracle: old owners are retained to avoid native dangling-reference UB.
static void inheritedShadingSlots() {
  AppearanceScene scene;auto cube=scene.cube;auto parent=occ::handle<Prs3d_Drawer>(new Prs3d_Drawer());
  auto original=cube->BoxSideStyle();auto next=occ::handle<Prs3d_ShadingAspect>(new Prs3d_ShadingAspect());
  parent->SetShadingAspect(original);auto drawer=occ::handle<Prs3d_Drawer>(new Prs3d_Drawer());drawer->SetLink(parent);cube->SetAttributes(drawer);
  const auto& alias=cube->BoxSideStyle();shadeAliasRow(7,alias,cube->BoxSideStyle(),original,next);
  drawer->SetShadingAspect(next);shadeAliasRow(8,alias,cube->BoxSideStyle(),original,next);
  parent->SetShadingAspect(next);shadeAliasRow(9,alias,cube->BoxSideStyle(),original,next);
  parent->SetShadingAspect({});shadeAliasRow(10,alias,cube->BoxSideStyle(),original,next);
  parent->SetShadingAspect(original);shadeAliasRow(11,alias,cube->BoxSideStyle(),original,next);
}
static void retainedResetOwners() {
  for(int reset=0;reset<3;++reset) {
    AppearanceScene scene;auto cube=scene.cube;prepareCustomAppearance(cube);
    auto drawer=cube->Attributes();auto side=cube->BoxSideStyle();auto edge=cube->BoxEdgeStyle();auto corner=cube->BoxCornerStyle();
    auto dynamic=cube->DynamicHilightAttributes();auto oldHighlight=dynamic->ShadingAspect();auto animation=cube->ViewAnimation();auto persistence=cube->TransformPersistence();auto start=cube->start();auto end=cube->end();
    cube->SetHilightAttributes(new Prs3d_Drawer());auto highlight=cube->HilightAttributes();
    if(reset==0)cube->UnsetAttributes();else if(reset==1)cube->UnsetHilightAttributes();else cube->ResetStyles();
    std::cout<<"reset-owners "<<reset<<' '<<(cube->Attributes()==drawer)<<' '<<(cube->BoxSideStyle()==side)<<' '<<(cube->BoxEdgeStyle()==edge)<<' '<<(cube->BoxCornerStyle()==corner)<<' '<<(cube->DynamicHilightAttributes()==dynamic)<<' '<<(dynamic->ShadingAspect()==oldHighlight)<<' '<<(cube->HilightAttributes()==highlight)<<' '<<cube->HilightAttributes().IsNull()<<' '<<(cube->ViewAnimation()==animation)<<' '<<(cube->TransformPersistence()==persistence)<<' '<<(cube->start()==start)<<' '<<(cube->end()==end)<<' '<<cube->frames<<' '<<cube->finished<<std::endl;
  }
}

class ResetOverrideCube:public AIS_ViewCube {
public:
  int attributes=0,highlights=0;
  void UnsetAttributes() override {++attributes;AIS_ViewCube::UnsetAttributes();}
  void UnsetHilightAttributes() override {++highlights;AIS_ViewCube::UnsetHilightAttributes();}
};
static void resetVirtualDispatch() {
  auto cube=occ::handle<ResetOverrideCube>(new ResetOverrideCube());cube->ResetStyles();
  std::cout<<"reset-dispatch 0 "<<cube->attributes<<' '<<cube->highlights<<std::endl;
}
