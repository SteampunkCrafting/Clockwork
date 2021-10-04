#version 450
precision highp float;

#include "../glsl_lib/light.glsl"

/* ---- INPUT ---- */
// Vertex Data
layout(location = 0) in vec2 vert_texture;

// Uniform Data
layout(set = 1, binding = 0) uniform DataWorld { AmbientLight ambient_light; }
world_uniforms;

layout(set = 2, binding = 0) uniform DataMesh {
  PhongMaterial material;
  bool is_textured;
}
mesh_uniforms;

layout(set = 3, binding = 0) uniform sampler2D material_texture;

/* ---- OUTPUT ---- */
layout(location = 0) out vec4 frag_color;

/* ---- MAIN ---- */
void process_colored_mesh() {
  /* -- INITIALIZATION -- */
  Vertex vertex = {vec3(0), vec3(0)};
  frag_color = vec4(0.0);

  /* -- LIGHT APPLICATION -- */
  // AMBIENT
  frag_color +=
      light_apply(world_uniforms.ambient_light, mesh_uniforms.material, vertex);

  /* -- CLAMPING -- */
  frag_color = clamp(frag_color, 0, 1);
}

void process_textured_mesh() {
  /* -- INITIALIZATION -- */
  TexturedVertex vertex = {vec3(0), vec3(0), vert_texture};
  frag_color = vec4(0.0);

  /* -- LIGHT APPLICATION -- */
  // AMBIENT
  frag_color += light_apply(world_uniforms.ambient_light,
                            mesh_uniforms.material, material_texture, vertex);
}

void main() {
  if (mesh_uniforms.is_textured)
    process_textured_mesh();
  else
    process_colored_mesh();
}
