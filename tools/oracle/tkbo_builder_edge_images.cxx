#include <BOPAlgo_Builder.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>

using ImageMap = NCollection_DataMap<TopoDS_Shape,
                                     NCollection_List<TopoDS_Shape>,
                                     TopTools_ShapeMapHasher>;

class FillEdgeImagesProbe : public BOPAlgo_Builder {
public:
  void Run(BOPDS_DS& ds) { myDS = &ds; FillImagesEdges(Message_ProgressRange()); }
};

static TopoDS_Edge edge(double y, double lastX) {
  return BRepBuilderAPI_MakeEdge(gp_Pnt(0.0, y, 0.0), gp_Pnt(lastX, y, 0.0));
}

static void empty_reference_case() {
  const TopoDS_Edge source = edge(0.0, 2.0);
  const TopoDS_Edge empty = edge(1.0, 2.0);
  const TopoDS_Edge unreferenced = edge(2.0, 2.0);
  BOPDS_DS ds;
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(source);
  arguments.Append(empty);
  arguments.Append(unreferenced);
  ds.SetArguments(arguments);
  ds.Init(1.0e-7);
  NCollection_List<occ::handle<BOPDS_PaveBlock>>& blocks = ds.ChangePaveBlocks(ds.Index(source));
  const TopoDS_Edge image = edge(0.0, 1.0);
  const int imageIndex = ds.Append(image);
  blocks.First()->SetEdge(imageIndex);
  ds.ChangePaveBlocks(ds.Index(empty)).Clear();

  FillEdgeImagesProbe builder;
  builder.Run(ds);
  const NCollection_List<TopoDS_Shape>* sourceImages = builder.Images().Seek(source);
  const NCollection_List<TopoDS_Shape>* emptyImages = builder.Images().Seek(empty);
  std::cout << "images=" << builder.Images().Extent() << " origins=" << builder.Origins().Extent()
            << " source_count=" << (sourceImages ? sourceImages->Length() : -1)
            << " empty_bound=" << (emptyImages != nullptr)
            << " empty_count=" << (emptyImages ? emptyImages->Length() : -1)
            << " unreferenced_bound=" << (builder.Images().Seek(unreferenced) != nullptr) << '\n';
}

static void common_block_case() {
  const TopoDS_Edge first = edge(0.0, 2.0);
  const TopoDS_Edge second = edge(0.0, 2.0);
  BOPDS_DS ds;
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(first);
  arguments.Append(second);
  ds.SetArguments(arguments);
  ds.Init(1.0e-7);
  NCollection_List<occ::handle<BOPDS_PaveBlock>>& firstBlocks = ds.ChangePaveBlocks(ds.Index(first));
  NCollection_List<occ::handle<BOPDS_PaveBlock>>& secondBlocks = ds.ChangePaveBlocks(ds.Index(second));
  const TopoDS_Edge firstImage = edge(0.0, 2.0);
  const TopoDS_Edge secondImage = edge(0.0, 2.0);
  const int firstImageIndex = ds.Append(firstImage);
  const int secondImageIndex = ds.Append(secondImage);
  firstBlocks.First()->SetEdge(firstImageIndex);
  secondBlocks.First()->SetEdge(secondImageIndex);
  NCollection_List<occ::handle<BOPDS_PaveBlock>> grouped;
  grouped.Append(firstBlocks.First());
  grouped.Append(secondBlocks.First());
  occ::handle<BOPDS_CommonBlock> common = new BOPDS_CommonBlock;
  common->SetPaveBlocks(grouped);
  ds.SetCommonBlock(firstBlocks.First(), common);
  ds.SetCommonBlock(secondBlocks.First(), common);

  FillEdgeImagesProbe builder;
  builder.Run(ds);
  const NCollection_List<TopoDS_Shape>* firstSources = builder.Origins().Seek(firstImage);
  const TopoDS_Shape* sameDomain = builder.ShapesSD().Seek(secondImage);
  std::cout << "images=" << builder.Images().Extent() << " origins=" << builder.Origins().Extent()
            << " first_count=" << (firstSources ? firstSources->Length() : -1)
            << " second_maps_to_first=" << (sameDomain && sameDomain->IsSame(firstImage)) << '\n';
}

int main() {
  empty_reference_case();
  common_block_case();
}
