// Differential storage probe for pinned math_DoubleTab; no production linkage.
#include <math_DoubleTab.hxx>
void table_values(const math_DoubleTab& table) {
  std::cout<<table.LowerRow()<<' '<<table.UpperRow()<<' '<<table.LowerCol()<<' '<<table.UpperCol()<<' '
           <<table.NbRows()<<' '<<table.NbColumns()<<' '<<table.IsDeletable()<<' ';
  for(size_t row=0;row<table.RowSize();++row)
    for(size_t col=0;col<table.ColSize();++col)std::cout<<bits(table.At(row,col))<<' ';
}
void table_case(std::istream& input) {
  int rows,cols,rl,cl;double value;input>>rows>>cols>>rl>>cl>>value;
  math_DoubleTab original(rl,rl+rows-1,cl,cl+cols-1);original.Init(value);
  original.ChangeAt(rows-1,cols-1)=-value;table_values(original);
  math_DoubleTab copy(original);table_values(copy);
  math_DoubleTab moved(std::move(original));table_values(moved);table_values(original);
  moved.SetLowerRow(cl);moved.SetLowerCol(rl);table_values(moved);
  math_DoubleTab destination(cl,cl+rows-1,rl,rl+cols-1);
  destination=std::move(moved);table_values(destination);table_values(moved);
  std::vector<double> external(rows*cols,42.0);
  math_DoubleTab borrowed(external.data(),rl,rl+rows-1,cl,cl+cols-1);
  borrowed=copy;table_values(borrowed);
  borrowed.Value(rl,cl)=value+1.0;std::cout<<bits(external[0])<<' ';
  math_DoubleTab borrowedMove(std::move(borrowed));table_values(borrowedMove);table_values(borrowed);
  copy.Copy(destination);table_values(destination);std::cout<<'\n';
}
