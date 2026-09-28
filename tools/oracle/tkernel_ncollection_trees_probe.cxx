// NCollection trees probe operations (twin of oracle_probe/ncollection/trees.rs).
// UBTree/EBTree shapes after Add, Remove, Gemmate and Kill; UBTreeFiller's mt19937 order
// and CheckTree text; CellFilter visits, purges and the Reset quirks; KDTree Array1 builds.
#include <NCollection_CellFilter.hxx>
#include <NCollection_EBTree.hxx>
#include <NCollection_KDTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <Standard_Failure.hxx>

#include <cstdint>
#include <cstring>
#include <istream>
#include <ostream>
#include <sstream>
#include <string>
#include <vector>

namespace trees_probe
{
static uint64_t bits(double theValue)
{
  uint64_t aBits;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  return aBits;
}

// 2D box with the minimal UBTree bound interface.
struct Box
{
  double Min[2] = {0.0, 0.0};
  double Max[2] = {0.0, 0.0};

  void Add(const Box& theOther)
  {
    for (int i = 0; i < 2; ++i)
    {
      Min[i] = theOther.Min[i] < Min[i] ? theOther.Min[i] : Min[i];
      Max[i] = theOther.Max[i] > Max[i] ? theOther.Max[i] : Max[i];
    }
  }

  bool IsOut(const Box& theOther) const
  {
    return theOther.Min[0] > Max[0] || theOther.Max[0] < Min[0] || theOther.Min[1] > Max[1]
           || theOther.Max[1] < Min[1];
  }

  double SquareExtent() const
  {
    const double aDx = Max[0] - Min[0];
    const double aDy = Max[1] - Min[1];
    return aDx * aDx + aDy * aDy;
  }
};

typedef NCollection_UBTree<int, Box> Tree;

static Box readBox(std::istream& theIn)
{
  Box aBox;
  theIn >> aBox.Min[0] >> aBox.Min[1] >> aBox.Max[0] >> aBox.Max[1];
  return aBox;
}

static std::vector<std::pair<int, Box>> readItems(std::istream& theIn)
{
  size_t aCount = 0;
  theIn >> aCount;
  std::vector<std::pair<int, Box>> anItems;
  for (size_t i = 0; i < aCount; ++i)
  {
    int anObj = 0;
    theIn >> anObj;
    anItems.emplace_back(anObj, readBox(theIn));
  }
  return anItems;
}

static void printShape(std::ostream& theOut, const Tree::TreeNode& theNode)
{
  theOut << ' ' << (theNode.IsLeaf() ? 'L' : 'B') << theNode.Object() << '['
         << bits(theNode.Bnd().Min[0]) << ',' << bits(theNode.Bnd().Min[1]) << ','
         << bits(theNode.Bnd().Max[0]) << ',' << bits(theNode.Bnd().Max[1]) << ']';
  if (!theNode.IsLeaf())
  {
    printShape(theOut, theNode.Child(0));
    printShape(theOut, theNode.Child(1));
  }
}

static void printTree(std::ostream& theOut, const Tree& theTree)
{
  if (theTree.IsEmpty())
    theOut << " empty";
  else
    printShape(theOut, theTree.Root());
}

class Selector : public Tree::Selector
{
public:
  Selector(const Box& theQuery, int theLimit)
      : myQuery(theQuery),
        myLimit(theLimit)
  {
  }

  bool Reject(const Box& theBnd) const override { return myQuery.IsOut(theBnd); }

  bool Accept(const int& theObj) override
  {
    Accepted.push_back(theObj);
    myStop = myLimit > 0 && static_cast<int>(Accepted.size()) >= myLimit;
    return theObj % 3 != 0; // objects divisible by 3 are inspected but not counted
  }

  std::vector<int> Accepted;

private:
  Box myQuery;
  int myLimit;
};

static void select(std::istream& theIn, std::ostream& theOut, const Tree& theTree)
{
  const Box aQuery = readBox(theIn);
  int       aLimit = 0;
  theIn >> aLimit;
  Selector  aSelector(aQuery, aLimit);
  const int aCount = theTree.Select(aSelector);
  theOut << " select " << aCount;
  for (int anObj : aSelector.Accepted)
    theOut << ' ' << anObj;
}

// ncoll_trees_ubtree <n> {obj box} <query box> <stop>
static void ubtree(std::istream& theIn, std::ostream& theOut)
{
  Tree aTree;
  for (const auto& anItem : readItems(theIn))
    theOut << aTree.Add(anItem.first, anItem.second);
  printTree(theOut, aTree);
  select(theIn, theOut, aTree);
}

// Walks `path` child indices from the root, stopping at a leaf; the probe casts away Root()'s constness because
// Gemmate and Kill are only reachable on a node reference.
static Tree::TreeNode& walk(std::istream& theIn, const Tree& theTree)
{
  Tree::TreeNode* aNode = const_cast<Tree::TreeNode*>(&theTree.Root());
  int             aDepth = 0;
  theIn >> aDepth;
  for (int i = 0; i < aDepth; ++i)
  {
    int aChild = 0;
    theIn >> aChild;
    if (!aNode->IsLeaf()) // a leaf has no children to descend into
      aNode = &aNode->ChangeChild(aChild);
  }
  return *aNode;
}

// ncoll_trees_kill <n> {obj box} <depth> {child} <index>
static void kill(std::istream& theIn, std::ostream& theOut)
{
  Tree aTree;
  for (const auto& anItem : readItems(theIn))
    aTree.Add(anItem.first, anItem.second);
  Tree::TreeNode& aNode = walk(theIn, aTree);
  int             anIndex = 0;
  theIn >> anIndex;
  aNode.Kill(anIndex, aTree.Allocator());
  printTree(theOut, aTree);
}

// ncoll_trees_gemmate <n> {obj box} <depth> {child} <new box> <obj> <box>
static void gemmate(std::istream& theIn, std::ostream& theOut)
{
  Tree aTree;
  for (const auto& anItem : readItems(theIn))
    aTree.Add(anItem.first, anItem.second);
  Tree::TreeNode& aNode   = walk(theIn, aTree);
  const Box       aNewBnd = readBox(theIn);
  int             anObj   = 0;
  theIn >> anObj;
  aNode.Gemmate(aNewBnd, anObj, readBox(theIn), aTree.Allocator());
  theOut << (&aNode.Child(1).Parent() == &aNode ? " linked" : " broken");
  printTree(theOut, aTree);
}

// ncoll_trees_ebtree <n> {obj box} <m> {obj} <query box> <stop>
static void ebtree(std::istream& theIn, std::ostream& theOut)
{
  NCollection_EBTree<int, Box> aTree;
  for (const auto& anItem : readItems(theIn))
    theOut << aTree.Add(anItem.first, anItem.second);
  size_t aRemovals = 0;
  theIn >> aRemovals;
  theOut << " remove";
  std::vector<int> aRemoved;
  for (size_t i = 0; i < aRemovals; ++i)
  {
    int anObj = 0;
    theIn >> anObj;
    aRemoved.push_back(anObj);
    theOut << aTree.Remove(anObj);
  }
  printTree(theOut, aTree);
  for (int anObj : aRemoved)
    theOut << (aTree.Contains(anObj) ? " +" : " -");
  select(theIn, theOut, aTree);
}

// ncoll_trees_filler <fullRandom> <n> {obj box}
static void filler(std::istream& theIn, std::ostream& theOut)
{
  int aFull = 1;
  theIn >> aFull;
  Tree aTree;
  {
    NCollection_UBTreeFiller<int, Box> aFiller(aTree, nullptr, aFull != 0);
    for (const auto& anItem : readItems(theIn))
      aFiller.Add(anItem.first, anItem.second);
    theOut << aFiller.Fill();
    std::ostringstream aCheck;
    theOut << ' ' << aFiller.CheckTree(aCheck) << ' ';
    for (char aChar : aCheck.str())
      theOut << (aChar == '\n' ? '|' : aChar);
    // Items queued after Fill are added by the destructor.
    aFiller.Add(-1, Box());
  }
  printTree(theOut, aTree);
}
} // namespace trees_probe

static bool ncollection_trees_probe_rest(const std::string& op, std::istream& in, std::ostream& out);

static bool ncollection_trees_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_trees_ubtree")
    trees_probe::ubtree(in, out);
  else if (op == "ncoll_trees_kill")
    trees_probe::kill(in, out);
  else if (op == "ncoll_trees_gemmate")
    trees_probe::gemmate(in, out);
  else if (op == "ncoll_trees_ebtree")
    trees_probe::ebtree(in, out);
  else if (op == "ncoll_trees_filler")
    trees_probe::filler(in, out);
  else
    return ncollection_trees_probe_rest(op, in, out);
  return true;
}

#include "tkernel_ncollection_trees_cell_probe.cxx"
