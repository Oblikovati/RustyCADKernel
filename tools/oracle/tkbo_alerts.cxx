// Oracle for every pinned BOPAlgo_Alerts.hxx macro declaration.
#include <BOPAlgo_Alerts.hxx>
#include <TopoDS_Vertex.hxx>
#include <BRep_Builder.hxx>
#include <cassert>
#include <cstring>

template <class T> void checkSimple(const char* name)
{
  occ::handle<T> alert = new T;
  assert(std::strcmp(alert->GetMessageKey(), name) == 0);
  assert(std::strcmp(alert->DynamicType()->Parent()->Name(), "Message_Alert") == 0);
  assert(alert->SupportsMerge());
  assert(alert->Merge(new T));
}

template <class T> void checkShape(const char* name)
{
  TopoDS_Vertex vertex;
  BRep_Builder().MakeVertex(vertex, gp_Pnt(1, 2, 3), 1e-7);
  occ::handle<T> alert = new T(vertex);
  assert(std::strcmp(alert->GetMessageKey(), name) == 0);
  assert(std::strcmp(alert->DynamicType()->Parent()->Name(), "TopoDS_AlertWithShape") == 0);
  assert(!alert->SupportsMerge());
  assert(alert->GetShape().IsSame(vertex));
  assert(!alert->Merge(new T(vertex)));
  alert->SetShape(TopoDS_Shape());
  assert(alert->GetShape().IsNull());
}

void checkGroup0()
{
  checkSimple<BOPAlgo_AlertUserBreak>("BOPAlgo_AlertUserBreak");
  checkSimple<BOPAlgo_AlertBOPNotAllowed>("BOPAlgo_AlertBOPNotAllowed");
  checkSimple<BOPAlgo_AlertBOPNotSet>("BOPAlgo_AlertBOPNotSet");
  checkSimple<BOPAlgo_AlertBuilderFailed>("BOPAlgo_AlertBuilderFailed");
  checkSimple<BOPAlgo_AlertIntersectionFailed>("BOPAlgo_AlertIntersectionFailed");
  checkSimple<BOPAlgo_AlertMultipleArguments>("BOPAlgo_AlertMultipleArguments");
  checkSimple<BOPAlgo_AlertNoFiller>("BOPAlgo_AlertNoFiller");
  checkSimple<BOPAlgo_AlertNullInputShapes>("BOPAlgo_AlertNullInputShapes");
  checkSimple<BOPAlgo_AlertPostTreatFF>("BOPAlgo_AlertPostTreatFF");
  checkSimple<BOPAlgo_AlertSolidBuilderFailed>("BOPAlgo_AlertSolidBuilderFailed");
}

void checkGroup1()
{
  checkSimple<BOPAlgo_AlertTooFewArguments>("BOPAlgo_AlertTooFewArguments");
  checkShape<BOPAlgo_AlertBadPositioning>("BOPAlgo_AlertBadPositioning");
  checkShape<BOPAlgo_AlertEmptyShape>("BOPAlgo_AlertEmptyShape");
  checkShape<BOPAlgo_AlertNotSplittableEdge>("BOPAlgo_AlertNotSplittableEdge");
  checkShape<BOPAlgo_AlertRemovalOfIBForEdgesFailed>("BOPAlgo_AlertRemovalOfIBForEdgesFailed");
  checkShape<BOPAlgo_AlertRemovalOfIBForFacesFailed>("BOPAlgo_AlertRemovalOfIBForFacesFailed");
  checkShape<BOPAlgo_AlertRemovalOfIBForMDimShapes>("BOPAlgo_AlertRemovalOfIBForMDimShapes");
  checkShape<BOPAlgo_AlertRemovalOfIBForSolidsFailed>("BOPAlgo_AlertRemovalOfIBForSolidsFailed");
  checkShape<BOPAlgo_AlertSelfInterferingShape>("BOPAlgo_AlertSelfInterferingShape");
  checkShape<BOPAlgo_AlertShellSplitterFailed>("BOPAlgo_AlertShellSplitterFailed");
}

void checkGroup2()
{
  checkShape<BOPAlgo_AlertTooSmallEdge>("BOPAlgo_AlertTooSmallEdge");
  checkShape<BOPAlgo_AlertIntersectionOfPairOfShapesFailed>("BOPAlgo_AlertIntersectionOfPairOfShapesFailed");
  checkShape<BOPAlgo_AlertBuildingPCurveFailed>("BOPAlgo_AlertBuildingPCurveFailed");
  checkShape<BOPAlgo_AlertAcquiredSelfIntersection>("BOPAlgo_AlertAcquiredSelfIntersection");
  checkShape<BOPAlgo_AlertUnsupportedType>("BOPAlgo_AlertUnsupportedType");
  checkSimple<BOPAlgo_AlertNoFacesToRemove>("BOPAlgo_AlertNoFacesToRemove");
  checkShape<BOPAlgo_AlertUnableToRemoveTheFeature>("BOPAlgo_AlertUnableToRemoveTheFeature");
  checkSimple<BOPAlgo_AlertRemoveFeaturesFailed>("BOPAlgo_AlertRemoveFeaturesFailed");
  checkShape<BOPAlgo_AlertSolidBuilderUnusedFaces>("BOPAlgo_AlertSolidBuilderUnusedFaces");
  checkShape<BOPAlgo_AlertFaceBuilderUnusedEdges>("BOPAlgo_AlertFaceBuilderUnusedEdges");
}

void checkGroup3()
{
  checkShape<BOPAlgo_AlertUnableToOrientTheShape>("BOPAlgo_AlertUnableToOrientTheShape");
  checkShape<BOPAlgo_AlertUnknownShape>("BOPAlgo_AlertUnknownShape");
  checkSimple<BOPAlgo_AlertNoPeriodicityRequired>("BOPAlgo_AlertNoPeriodicityRequired");
  checkShape<BOPAlgo_AlertUnableToTrim>("BOPAlgo_AlertUnableToTrim");
  checkShape<BOPAlgo_AlertUnableToMakeIdentical>("BOPAlgo_AlertUnableToMakeIdentical");
  checkShape<BOPAlgo_AlertUnableToRepeat>("BOPAlgo_AlertUnableToRepeat");
  checkSimple<BOPAlgo_AlertMultiDimensionalArguments>("BOPAlgo_AlertMultiDimensionalArguments");
  checkShape<BOPAlgo_AlertUnableToMakePeriodic>("BOPAlgo_AlertUnableToMakePeriodic");
  checkShape<BOPAlgo_AlertUnableToGlue>("BOPAlgo_AlertUnableToGlue");
  checkShape<BOPAlgo_AlertShapeIsNotPeriodic>("BOPAlgo_AlertShapeIsNotPeriodic");
}

void checkGroup4()
{
  checkShape<BOPAlgo_AlertUnableToMakeClosedEdgeOnFace>("BOPAlgo_AlertUnableToMakeClosedEdgeOnFace");
}

int main()
{
  checkGroup0();
  checkGroup1();
  checkGroup2();
  checkGroup3();
  checkGroup4();
}
