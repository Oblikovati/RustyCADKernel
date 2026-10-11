// Native OCCT 3d097a0 rubber-band state, geometry and retained cache oracle.
#include <AIS_RubberBand.hxx>
class V3RubberBandProbe : public AIS_RubberBand {
public:
  using AIS_RubberBand::AIS_RubberBand;
  using AIS_RubberBand::Compute;
  using AIS_RubberBand::fillTriangles;
  const auto& Borders() const { return myBorders; }
  const auto& Triangles() const { return myTriangles; }
};
static void rubber_band_snapshot(const occ::handle<V3RubberBandProbe>& band) {
  ga_int(band->Signature());ga_int(band->Type());ga_int(band->ZLayer());
  ga_flag(band->HasColor());ga_flag(band->IsFilling());ga_flag(band->IsPolygonClosed());
  ga_real(band->LineWidth());ga_int(band->LineType());ga_real(band->FillTransparency());
  ais_line_color(band->LineColor());ais_line_color(band->FillColor());
  ga_int(band->Points().Length());
  for(const auto& point:band->Points()){ga_int(point.x());ga_int(point.y());}
}
static void rubber_band_caches(const occ::handle<V3RubberBandProbe>& band) {
  ga_flag(band->Borders().IsNull());if(!band->Borders().IsNull())sv_array_state(band->Borders());
  ga_flag(band->Triangles().IsNull());if(!band->Triangles().IsNull())sv_array_state(band->Triangles());
}
static void rubber_band_mutate(const occ::handle<V3RubberBandProbe>& band,int op,double value) {
  switch(op) {
    case 0: band->SetRectangle(10,20,30,40);break;
    case 1: band->SetRectangle(30,40,10,20);break;
    case 2: band->AddPoint(NCollection_Vec2<int>(20,10));break;
    case 3: band->RemoveLastPoint();break;
    case 4: band->ClearPoints();break;
    case 5: band->SetPolygonClosed(false);break;
    case 6: band->SetPolygonClosed(true);break;
    case 7: band->SetFilling(false);break;
    case 8: band->SetFilling(true);break;
    case 9: band->SetFilling(Quantity_NOC_RED,value);break;
    case 10: band->SetFillTransparency(value);break;
    case 11: band->SetFillColor(Quantity_NOC_RED);break;
    case 12: band->SetLineColor(Quantity_NOC_RED);break;
    case 13: band->SetLineWidth(value);break;
    case 14: band->SetLineType(Aspect_TOL_DOT);break;
    case 15: ga_flag(band->fillTriangles());rubber_band_caches(band);break;
    case 18:
      band->ClearPoints();
      for(const auto& p:{NCollection_Vec2<int>(8,8),NCollection_Vec2<int>(8,36),NCollection_Vec2<int>(20,20),NCollection_Vec2<int>(36,36),NCollection_Vec2<int>(36,8)})band->AddPoint(p);
      break;
    case 19:
      if(!band->Triangles().IsNull())for(int index=1;index<=band->Triangles()->VertexNumber();++index)band->Triangles()->SetVertexNormal(index,0.,1.,0.);
      break;
  }
}
void v3_ais_rubber_band(std::istream& input) {
  GaReader r{input};int ctor=r.integer();double width=r.real();bool closed=r.flag();
  double transparency=r.real();int count=r.integer();double value=r.real();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  GA_TRY(
    occ::handle<V3RubberBandProbe> band;
    if(ctor==0)band=new V3RubberBandProbe();
    else if(ctor==1)band=new V3RubberBandProbe(Quantity_NOC_WHITE,Aspect_TOL_DASH,width,closed);
    else band=new V3RubberBandProbe(Quantity_NOC_WHITE,Aspect_TOL_DASH,Quantity_NOC_BLUE,transparency,width,closed);
    rubber_band_snapshot(band);
    for(int index=0;index<count;++index) {
      int op=r.integer();
      GA_TRY(
        if(op==16||op==17) {
          auto borders=band->Borders();auto triangles=band->Triangles();
          band->Compute(manager,prs,op==16?0:1);
          ga_flag(borders==band->Borders());ga_flag(triangles==band->Triangles());rubber_band_caches(band);
        } else rubber_band_mutate(band,op,value);
      )
      rubber_band_snapshot(band);
    }
    ga_int(prs->Groups().Size());
  )
  ga_end();
}
