// Four public Pick overloads, including repeated switches on the same selector.
static void v3_run_public_pick(V3TraversalSelector& selector,const occ::handle<V3d_View>& view,int kind){
  switch(kind){
    case 1:selector.Pick(40,30,view);break;
    case 2:selector.Pick(20,15,60,45,view);break;
    case 3:selector.Pick(gp_Ax1(gp_Pnt(0.,0.,20.),gp_Dir(0.,0.,-1.)),view);break;
    case 4:{NCollection_Array1<gp_Pnt2d> points(1,4);points(1)=gp_Pnt2d(20.,15.);points(2)=gp_Pnt2d(60.,15.);points(3)=gp_Pnt2d(60.,45.);points(4)=gp_Pnt2d(20.,45.);selector.Pick(points,view);break;}
  }
}
void v3_selector_pick(std::istream& input){
  GaReader r{input};V3TraversalConfig config{1,r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer()};
  V3TraversalScene scene(config);occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());auto view=viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(80,60);view->SetWindow(window);
  view->SetCamera(v3_traversal_volume(config).Camera());if(config.clipping==1||config.clipping==3)view->SetClipPlanes(v3_traversal_planes(1.,0.));
  scene.selector->SetPixelTolerance(r.integer());int count=r.integer();
  for(int i=0;i<count;++i){int kind=r.integer();GA_TRY(v3_run_public_pick(*scene.selector,view,kind))scene.state();
    const auto& volume=scene.selector->volume();ga_int(volume.GetActiveSelectionType());ga_flag(volume.Camera().IsNull());int width=0,height=0;volume.WindowSize(width,height);ga_int(width);ga_int(height);
    ga_flag(volume.ViewClipping().IsNull());ga_flag(volume.ObjectClipping().IsNull());ga_int(scene.selector->layerCount());
  }ga_end();
}
