// BRepMesh_DelabellaMeshAlgoFactory and the bundled DelaBella library at OCCT
// 3d097a0328e71b826377d4814ab05ec3c3d23871. The library is not an installed header, so
// its exported C interface is declared here (delabella.pxx).
#include <cstdint>

struct DelaBella_Vertex
{
  int               i;
  double            x, y;
  DelaBella_Vertex* next;
};

struct DelaBella_Triangle
{
  DelaBella_Vertex*   v[3];
  DelaBella_Triangle* f[3];
  DelaBella_Triangle* next;
};

void*                     DelaBella_Create();
void                      DelaBella_Destroy(void* db);
int                       DelaBella_TriangulateFloat(void* db, int points, float* x, float* y, int advance_bytes);
int                       DelaBella_TriangulateDouble(void* db, int points, double* x, double* y, int advance_bytes);
const DelaBella_Triangle* GetFirstDelaunayTriangle(void* db);
const DelaBella_Vertex*   GetFirstHullVertex(void* db);

namespace
{
struct DelabellaCloud
{
  uint64_t state;
  uint64_t next()
  {
    state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    return state >> 11;
  }
};

// The deterministic clouds of mesh_delabella_points (mirrored by the Rust probe).
std::vector<double> delabella_cloud(int kind, int count)
{
  DelabellaCloud      cloud{0x9E3779B97F4A7C15ULL ^ static_cast<uint64_t>(kind * 100000 + count)};
  std::vector<double> xy;
  for (int k = 0; k < count; ++k)
  {
    double x = 0., y = 0.;
    switch (kind)
    {
      case 0: x = double(cloud.next() % 4096) / 64.; y = double(cloud.next() % 4096) / 64.; break;
      case 1:
      case 5: x = double(cloud.next()) * 0x1p-53 - 0.5; y = double(cloud.next()) * 0x1p-53 - 0.5; break;
      case 2: {
        const double t = (k + 0.5) / count * 4. - 2.;
        x = k == 0 ? 0.1 : (1. - t * t) / (1. + t * t);
        y = k == 0 ? 0.05 : 2. * t / (1. + t * t);
        break;
      }
      case 3: x = k % 7; y = k / 7; break;
      default: x = k; y = 2. * k + 1.; break;
    }
    xy.push_back(x);
    xy.push_back(y);
  }
  return xy;
}
} // namespace

// mesh_delabella_points <kind 0..5> <count>: the raw library; the result, every
// Delaunay triangle by original indices, then at most count+1 hull vertices.
void mesh_delabella_points(std::istream& input)
{
  int kind, count;
  input >> kind >> count;
  std::vector<double> xy = delabella_cloud(kind, count);
  void*               db = DelaBella_Create();
  int                 result;
  if (kind == 5)
  {
    std::vector<float> xyf(xy.begin(), xy.end());
    result = DelaBella_TriangulateFloat(db, count, xyf.data(), xyf.data() + 1, 0);
  }
  else
    result = DelaBella_TriangulateDouble(db, count, xy.data(), xy.data() + 1, 0);
  std::cout << "OK ";
  ga_int(result);
  for (const DelaBella_Triangle* t = result > 0 ? GetFirstDelaunayTriangle(db) : nullptr; t; t = t->next)
  {
    ga_int(t->v[0]->i); ga_int(t->v[1]->i); ga_int(t->v[2]->i);
  }
  int hull = 0;
  for (const DelaBella_Vertex* v = GetFirstHullVertex(db); v && hull <= count; v = v->next, ++hull)
    ga_int(v->i);
  DelaBella_Destroy(db);
  ga_end();
}

// mesh_delabella_shape <kind 0..7> <deflection 0..1> <mode 0..3>: the shapes of
// mesh_incremental_shape meshed through BRepMesh_Context(IMeshTools_MeshAlgoType_Delabella).
void mesh_delabella_shape(std::istream& input)
{
  int kind, deflection, mode;
  input >> kind >> deflection >> mode;
  const TopoDS_Shape    shape = incremental_shape(kind);
  IMeshTools_Parameters parameters;
  parameters.MeshAlgo                                  = IMeshTools_MeshAlgoType_Delabella;
  parameters.Deflection                                = deflection == 0 ? 0.5 : 0.1;
  parameters.InternalVerticesMode                      = mode == 1;
  parameters.EnableControlSurfaceDeflectionAllSurfaces = mode == 2;
  parameters.Relative                                  = mode == 3;
  BRepMesh_IncrementalMesh mesher(shape, parameters);
  std::cout << "OK ";
  ga_flag(mesher.IsDone());
  ga_int(mesher.GetStatusFlags());
  int faces = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More(); explorer.Next(), ++faces)
    incremental_face_report(TopoDS::Face(explorer.Current()));
  ga_int(faces);
  ga_end();
}
