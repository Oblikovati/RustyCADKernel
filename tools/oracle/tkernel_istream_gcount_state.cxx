// State-only operations retain gcount after OCCT's std::istream read protocol.
#include <Standard_ReadLineBuffer.hxx>
#include <iostream>
#include <sstream>

struct ExposedReadHook : Standard_ReadLineBuffer {
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  using Standard_ReadLineBuffer::readStream;
};

int main()
{
  ExposedReadHook reader(4);
  std::istringstream source("ab");
  std::size_t count = 77;
  reader.readStream(source, 4, count);
  std::cout << count << '/' << source.gcount();
  try { source.exceptions(std::ios::failbit); }
  catch (const std::ios_base::failure&) { std::cout << "/mask:" << source.gcount(); }
  try { source.clear(std::ios::failbit); }
  catch (const std::ios_base::failure&) { std::cout << "/clear:" << source.gcount(); }
  source.clear();
  std::cout << "/good:" << source.gcount();
  reader.readStream(source, 0, count);
  std::cout << "/zero:" << count << '/' << source.gcount() << '\n';
}
