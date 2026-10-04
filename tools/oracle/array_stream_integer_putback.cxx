// Supplemental probe of the protected pbackfail hook at OCCT 3d097a0328e71b826.
#include <Standard_ArrayStreamBuffer.hxx>
#include <climits>
#include <iostream>

class IntegerPutbackProbe : public Standard_ArrayStreamBuffer
{
public:
  using Standard_ArrayStreamBuffer::Standard_ArrayStreamBuffer;
  using Standard_ArrayStreamBuffer::pbackfail;
  using Standard_ArrayStreamBuffer::showmanyc;
  using Standard_ArrayStreamBuffer::uflow;
};

static void probe(const unsigned char byte, const int requested)
{
  const char storage[] = {static_cast<char>(byte)};
  IntegerPutbackProbe buffer(storage, 1);
  const int consumed = buffer.uflow();
  const int restored = buffer.pbackfail(requested);
  std::cout << int(byte) << ' ' << requested << ' ' << consumed << ' ' << restored
            << ' ' << buffer.showmanyc() << ' ' << buffer.pbackfail(-1) << '\n';
}

int main()
{
  for (const unsigned char byte : {0, 65, 127, 128, 129, 254, 255})
  {
    probe(byte, static_cast<char>(byte));
  }
  for (const int requested : {INT_MIN, -129, -1, 128, 256, INT_MAX})
  {
    probe(128, requested);
  }
}
