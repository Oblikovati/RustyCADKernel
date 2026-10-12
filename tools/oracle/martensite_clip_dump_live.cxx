// Native DumpJson skips null children/depth-zero and advances actual nodes after callbacks.
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; valid external plane anchors stay alive.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <iostream>
#include <sstream>
#include <vector>
class ClipDumpLivePlane : public Graphic3d_ClipPlane {
public:
  int tag;
  Graphic3d_SequenceOfHClipPlane* sequence;
  std::vector<std::string>* rows;
  const occ::handle<Graphic3d_ClipPlane>* remove = nullptr;
  const occ::handle<Graphic3d_ClipPlane>* append = nullptr;
  mutable bool editOnce = false;
  ClipDumpLivePlane(int identity, Graphic3d_SequenceOfHClipPlane& owner,
                    std::vector<std::string>& trace) : tag(identity), sequence(&owner), rows(&trace) {}
  void DumpJson(Standard_OStream& stream, int depth = -1) const override {
    std::ostringstream row;
    row << "visit " << tag << ' ' << depth << ' ' << sequence->Size() << ' ' << GetRefCount();
    rows->push_back(row.str());
    if (editOnce) {
      editOnce = false;
      sequence->Remove(*remove);
      sequence->Append(*append);
    }
    stream << "\"tag\": " << tag;
  }
};
static void capture_null_depth() {
  Graphic3d_SequenceOfHClipPlane sequence;
  sequence.Append(occ::handle<Graphic3d_ClipPlane>());
  for (int depth : {-1, 0}) {
    std::ostringstream stream;
    sequence.DumpJson(stream, depth);
    std::cout << "null " << depth << ' ' << sequence.Size() << ' ' << stream.str() << '\n';
  }
}
static void capture_live_dump() {
  occ::handle<Graphic3d_SequenceOfHClipPlane> sequence = new Graphic3d_SequenceOfHClipPlane;
  std::vector<std::string> rows;
  occ::handle<Graphic3d_ClipPlane> first = new ClipDumpLivePlane(1, *sequence, rows);
  occ::handle<Graphic3d_ClipPlane> second = new ClipDumpLivePlane(2, *sequence, rows);
  occ::handle<Graphic3d_ClipPlane> third = new ClipDumpLivePlane(3, *sequence, rows);
  auto* callback = static_cast<ClipDumpLivePlane*>(first.get());
  callback->remove = &second;
  callback->append = &third;
  callback->editOnce = true;
  sequence->Append(first);
  sequence->Append(second);
  sequence->SetOverrideGlobal(true);
  std::ostringstream stream;
  sequence->DumpJson(stream, -1);
  for (const auto& row : rows) std::cout << row << '\n';
  std::cout << "json " << stream.str() << '\n';
  std::cout << "counts " << sequence->GetRefCount() << ' ' << first->GetRefCount() << ' '
            << second->GetRefCount() << ' ' << third->GetRefCount() << '\n';
}
int main() {
  capture_null_depth();
  capture_live_dump();
  return 0;
}
