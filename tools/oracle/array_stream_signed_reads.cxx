#include <Standard_ArrayStreamBuffer.hxx>
#include <iostream>

class SignedReadBuffer final : public Standard_ArrayStreamBuffer
{
public:
  SignedReadBuffer() : Standard_ArrayStreamBuffer("ABCD", 4) {}
  std::streamsize xsgetn(char* output, std::streamsize count) override
  {
    ++calls;
    if (count < 0) return count - 4;
    if (count == 0) return -11;
    return Standard_ArrayStreamBuffer::xsgetn(output, count);
  }
  int calls = 0;
};

int main()
{
  SignedReadBuffer buffer;
  std::streambuf& dispatch = buffer;
  char output[4] = {};
  std::cout << dispatch.sgetn(output, -3) << ' ' << buffer.calls << '\n';
  std::cout << dispatch.sgetn(output, 0) << ' ' << buffer.calls << '\n';
  const auto copied = dispatch.sgetn(output, 2);
  std::cout << copied << ' ' << buffer.calls << ' ' << std::string(output, copied) << '\n';
  std::cout << dispatch.sgetc() << '\n';
}
