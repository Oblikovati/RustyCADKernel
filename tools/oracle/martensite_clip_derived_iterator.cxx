// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; borrowed derived collection iteration.
#define main source_protocol_baseline_main
#include "martensite_clip_source_protocol.cxx"
#undef main
int main() {
  occ::handle<ClipProtocolSequence> sequence = new ClipProtocolSequence;
  occ::handle<Graphic3d_ClipPlane> plane = new ClipProtocolPlane;
  sequence->Append(plane);
  Graphic3d_SequenceOfHClipPlane::Iterator cursor(sequence);
  std::cout << "initial " << sequence->GetRefCount() << ' '
            << (cursor.Value() == plane) << ' ' << plane->GetRefCount() << '\n';
  cursor.Next();
  std::cout << "next " << cursor.More() << '\n';
  cursor.Init(sequence);
  std::cout << "restart " << sequence->GetRefCount() << ' ' << cursor.More()
            << ' ' << plane->GetRefCount() << '\n';
  sequence->Clear();
  return 0;
}
