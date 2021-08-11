#ifndef VERTEX_GLSL
#define VERTEX_GLSL

/* ---- STRUCTS ---- */
struct Vertex {
  vec3 position;
  vec3 normal;
};

struct TexturedVertex {
  vec3 position;
  vec3 normal;
  vec2 texture_coordinate;
};

#endif