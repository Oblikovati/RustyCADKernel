// The operations of the `dsx` probe (included by tkbool_probe_topopebrepds_ops.cxx); the
// Rust twin is crates/tkbool/examples/bl_probe/top_ope_brep_ds_ops_run.rs.

// TKI iterations of a list on geometry then on support.
static void bl_dsx_tki(const BlLoi& l){
 for(int s=0;s<2;s++){TopOpeBRepDS_TKI tki;if(s==0)tki.FillOnGeometry(l);else tki.FillOnSupport(l);
  for(tki.Init();tki.More();tki.Next()){TopOpeBRepDS_Kind k;int g;const BlLoi& x=tki.Value(k,g);ga_int((int)k);ga_int(g);ga_int(x.Extent());}
  ga_int(-1);}}

// Every selector on a copy of the list with the integer `arg`.
static void bl_dsx_select(const BlLoi& l,int arg){
 for(int s=0;s<9;s++){BlLoi l1;l1.Assign(l);BlLoi l2;int n=0;
  GA_TRY(
  if(s==0)n=FUN_selectTRASHAinterference(l1,arg%2?TopAbs_EDGE:TopAbs_FACE,l2);
  else if(s==1)n=FUN_selectITRASHAinterference(l1,arg,l2);
  else if(s==2)n=FUN_selectTRAUNKinterference(l1,l2);
  else if(s==3)n=FUN_selectTRAORIinterference(l1,(TopAbs_Orientation)(arg%4),l2);
  else if(s==4)n=FUN_selectGKinterference(l1,(TopOpeBRepDS_Kind)(arg%9),l2);
  else if(s==5)n=FUN_selectSKinterference(l1,(TopOpeBRepDS_Kind)(arg%9),l2);
  else if(s==6)n=FUN_selectGIinterference(l1,arg,l2);
  else if(s==7)n=FUN_selectSIinterference(l1,arg,l2);
  else n=FUN_selectTRAINTinterference(l,l2)?1:0;
  ga_int(n);bl_ds_loi(l1);bl_ds_loi(l2))}}

// The selections of the edge interference reducer.
static void bl_dsx_select3d(BlDs& d,int six,const BlLoi& l){
 TopOpeBRepDS_DataStructure& ds=d.hds->ChangeDS();
 BlLoi li;li.Assign(l);BlLoi lF,lFE,lE;
 FUN_selectTRASHAinterference(li,TopAbs_FACE,lF);FUN_selectSKinterference(lF,TopOpeBRepDS_EDGE,lFE);
 FUN_selectTRASHAinterference(li,TopAbs_EDGE,lE);
 BlLoi lp;lp.Assign(lFE);BlLoi l2p;GA_TRY(ga_int(FUN_selectpure2dI(lF,lp,l2p));bl_ds_loi(lp);bl_ds_loi(l2p))
 BlLoi r,t3,f3,t3r,t2;GA_TRY(FUN_select3dinterference(six,ds,lF,f3,lFE,r,t3,t3r,t2);
  bl_ds_loi(lF);bl_ds_loi(f3);bl_ds_loi(lFE);bl_ds_loi(r);bl_ds_loi(t3);bl_ds_loi(t3r);bl_ds_loi(t2))
 BlLoi e1;GA_TRY(ga_int(FUN_select2dI(six,ds,TopAbs_EDGE,lE,e1));bl_ds_loi(lE);bl_ds_loi(e1))
 BlLoi esd;GA_TRY(ga_int(FUN_select1dI(six,ds,lE,esd));bl_ds_loi(lE);bl_ds_loi(esd))}

// The reductions of a list: doubles, unknown, symmetric, repvg, reclSE, scan, GetTr.
static void bl_dsx_reduce(BlDs& d,int six,const BlLoi& l,int arg){
 TopOpeBRepDS_DataStructure& ds=d.hds->ChangeDS();
 BlLoi a;a.Assign(l);GA_TRY(FUN_reducedoublons(a,ds,six);bl_ds_loi(a))
 BlLoi b;b.Assign(l);GA_TRY(FUN_unkeepUNKNOWN(b,ds,six);bl_ds_loi(b))
 BlLoi c;c.Assign(l);GA_TRY(FUN_unkeepEsymetrictransitions(c,ds,six);bl_ds_loi(c))
 BlLoi f;f.Assign(l);GA_TRY(FUN_unkeepFdoubleGBoundinterferences(f,ds,six);bl_ds_loi(f))
 BlLoi e;e.Assign(l);BlLoi re;GA_TRY(FDS_repvg(ds,six,arg%2?TopOpeBRepDS_VERTEX:TopOpeBRepDS_POINT,e,re);bl_ds_loi(e);bl_ds_loi(re))
 BlLoi g;g.Assign(l);BlLoi rg;GA_TRY(FUN_reclSE(six,ds,g,rg);bl_ds_loi(g);bl_ds_loi(rg))
 BlLoi lf,lr,lin,lex;GA_TRY(int nf,nr,ni,ne;FUN_scanloi(l,lf,nf,lr,nr,lin,ni,lex,ne);ga_int(nf);ga_int(nr);ga_int(ni);ga_int(ne);bl_ds_loi(lex))
 GA_TRY(TopAbs_State sb,sa;int ib,bd,ia,ad;FUN_ds_GetTr(ds,six,0,l,sb,ib,bd,sa,ia,ad);
  ga_int((int)sb);ga_int(ib);ga_int(bd);ga_int((int)sa);ga_int(ia);ga_int(ad))
 BlLoi h;GA_TRY(ga_int(FUN_ds_hasI2d(six,l,h));bl_ds_loi(h))
 BlLoi l1d,rest;rest.Assign(l);FUN_selectTRASHAinterference(rest,TopAbs_EDGE,l1d);
 for(BlLoi::Iterator it(l);it.More();it.Next()){TopOpeBRepDS_Transition t;GA_TRY(ga_flag(FUN_ds_redu2d1d(ds,six,it.Value(),l1d,t));
  ga_int((int)t.Before());ga_int((int)t.After());ga_int(t.IndexBefore());ga_int(t.IndexAfter()))}}

// The queries of EXPORT, TOOL and connex on the shape `six` and the reference `other`.
static void bl_dsx_query(BlDs& d,int six,int other){
 const TopOpeBRepDS_DataStructure& ds=d.hds->DS();const TopoDS_Shape& s=ds.Shape(six);const TopoDS_Shape& o=ds.Shape(other);
 NCollection_List<TopoDS_Shape> l3,l2;GA_TRY(ga_flag(FDS_HasSameDomain3d(ds,s,&l3));ga_int(l3.Extent());ga_flag(FDS_HasSameDomain2d(ds,s,&l2));ga_int(l2.Extent()))
 GA_TRY(TopoDS_Shape oov;ga_flag(FUN_ds_getoov(s,ds,oov));ga_int(ds.Shape(oov));ga_flag(FUN_ds_getoov(s,d.hds,oov)))
 GA_TRY(int iv;ga_flag(FUN_ds_getVsdm(ds,six,iv));ga_int(iv);ga_flag(FUN_ds_sdm(ds,s,o)))
 GA_TRY(TopOpeBRepDS_Config c=TopOpeBRepDS_UNSHGEOMETRY;ga_flag(FDS_Config3d(s,o,c));ga_int((int)c))
 GA_TRY(ga_flag(FDS_EdgeIsConnexToSameDomainFaces(s,d.hds)))
 GA_TRY(TopAbs_Orientation ori;ga_int(FUN_ds_oriEinF(ds,TopoDS::Edge(s),o,ori));ga_int((int)ori))
 GA_TRY(int ie=0;ga_flag(FUN_ds_ONesd(ds,six,o,ie));ga_int(ie))
 GA_TRY(NCollection_List<TopoDS_Shape> ls;ga_int(TopOpeBRepDS_TOOL::EShareG(d.hds,TopoDS::Edge(s),ls)))
 GA_TRY(int ie=0;ga_flag(TopOpeBRepDS_TOOL::GetEsd(d.hds,o,six,ie));ga_int(ie);ga_flag(TopOpeBRepDS_TOOL::ShareG(d.hds,six,other)))
 GA_TRY(ga_int(FDSCNX_EdgeConnexitySameShape(s,d.hds).Extent());ga_flag(FDSCNX_HasConnexFace(s,d.hds)))
 GA_TRY(double p1,p2;FDS_getupperlower(d.hds,six,0.5,p1,p2);ga_real(p1);ga_real(p2))
 for(BlLoi::Iterator it(ds.ShapeInterferences(six));it.More();it.Next())GA_TRY(ga_flag(FDS_SIisGIofIofSBAofTofI(ds,other,it.Value())))}

// The checks and the dump texts.
static void bl_dsx_check(BlDs& d){
 occ::handle<TopOpeBRepDS_Check> c=new TopOpeBRepDS_Check(d.hds);
 GA_TRY(ga_flag(c->ChkIntg()))GA_TRY(ga_flag(c->ChkIntgSamDom()))GA_TRY(ga_flag(c->OneVertexOnPnt()))
 ga_captured([&]{c->PrintIntg(std::cout);});
 TopOpeBRepDS_Dumper du(d.hds);const TopOpeBRepDS_DataStructure& ds=d.hds->DS();
 for(int i=1;i<=ds.NbShapes();i++){GA_TRY(ga_text(du.SPrintShape(i).ToCString()))GA_TRY(ga_text(du.SPrintShapeRefOri(ds.Shape(i),"").ToCString()))}}

// The operation `op` on the shape `six`.
static void bl_dsx_run(BlDs& d,int op,int six,int arg){
 TopOpeBRepDS_DataStructure& ds=d.hds->ChangeDS();
 FDSCNX_Prepare(ds.Shape(1),ds.Shape(2),d.hds);
 BlLoi l;l.Assign(ds.ShapeInterferences(six));
 NCollection_DataMap<TopoDS_Shape,TopOpeBRepDS_ListOfShapeOn1State,TopTools_ShapeMapHasher> m;
 TopOpeBRepTool_ShapeClassifier sc;
 switch(op){
 case 0:bl_dsx_tki(l);break;
 case 1:bl_dsx_select(l,arg);break;
 case 2:bl_dsx_select3d(d,six,l);break;
 case 3:bl_dsx_reduce(d,six,l,arg);break;
 case 4:{TopOpeBRepDS_EIR eir(d.hds);eir.ProcessEdgeInterferences(six);break;}
 case 5:{TopOpeBRepDS_EIR eir(d.hds);eir.ProcessEdgeInterferences();break;}
 case 6:{TopOpeBRepDS_FIR fir(d.hds);fir.ProcessFaceInterferences(six,m);break;}
 case 7:{TopOpeBRepDS_FIR fir(d.hds);fir.ProcessFaceInterferences(m);break;}
 case 8:{TopOpeBRepDS_Filter f(d.hds,&sc);f.ProcessEdgeInterferences(six);break;}
 case 9:{TopOpeBRepDS_Filter f(d.hds,&sc);f.ProcessFaceInterferences(six,m);break;}
 case 10:{TopOpeBRepDS_Filter f(d.hds,&sc);f.ProcessInterferences();break;}
 case 11:bl_dsx_query(d,six,bl_ds_ref(d,arg));break;
 case 12:bl_dsx_check(d);break;
 default:{int k=op-13;
  if(k==0)FUN_ds_PURGEforE9(d.hds);else if(k==1)FUN_ds_completeforSE1(d.hds);
  else if(k==2)FUN_ds_completeforSE2(d.hds);else if(k==3)FUN_ds_completeforSE3(d.hds);
  else if(k==4)FUN_ds_completeforSE4(d.hds);else if(k==5)FUN_ds_completeforSE5(d.hds);
  else if(k==6)FUN_ds_completeforSE6(d.hds);else if(k==7)FUN_ds_completeforE7(d.hds);
  else if(k==8)FUN_ds_completeforSE8(d.hds);else if(k==9)FUN_ds_completeforSE9(d.hds);
  else if(k==10)FUN_ds_PointToVertex(d.hds);else if(k==11)FUN_ds_redusamsha(d.hds);
  else if(k==12)FUN_ds_FillSDMFaces(d.hds);else if(k==13)FUN_ds_addSEsdm1d(d.hds);
  else if(k==14)FUN_ds_FEIGb1TO0(d.hds,m);else FUN_ds_complete1dForSESDM(d.hds);}}}
