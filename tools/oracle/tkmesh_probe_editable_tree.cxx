#include <NCollection_EBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <Bnd_Box2d.hxx>
static Bnd_Box2d editable_bound(int i){Bnd_Box2d b;double x=i;b.Add(gp_Pnt2d(x,x*.5));b.Add(gp_Pnt2d(x+1,x*.5+2));b.Enlarge(1e-7);return b;}
class EditableBoxSelector:public NCollection_UBTree<int,Bnd_Box2d>::Selector{public:Bnd_Box2d bounds;std::vector<int> objects;bool Reject(const Bnd_Box2d& b)const override{return bounds.IsOut(b);}bool Accept(const int& object)override{objects.push_back(object);return true;}};
void mesh_editable_tree(std::istream& input){
 int count,full,duplicate;input>>count>>full>>duplicate;NCollection_EBTree<int,Bnd_Box2d> tree;
 {NCollection_UBTreeFiller<int,Bnd_Box2d> filler(tree,nullptr,full);for(int i=0;i<count;++i)filler.Add(duplicate?i%3:i,editable_bound(i));ga_int(filler.Fill());std::ostringstream out;ga_int(filler.CheckTree(out));filler.Add(-1,editable_bound(-1));}
 for(int i=-1;i<count;++i){ga_flag(tree.Contains(i));if(tree.Contains(i))ga_real(tree.FindNode(i).Bnd().SquareExtent());}
 ga_flag(tree.Remove(1));ga_flag(tree.Contains(1));EditableBoxSelector selector;selector.bounds.Add(gp_Pnt2d(-100,-100));selector.bounds.Add(gp_Pnt2d(1000,1000));ga_int(tree.Select(selector));for(int i:selector.objects)ga_int(i);ga_end();
}
