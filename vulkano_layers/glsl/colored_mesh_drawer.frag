#version 450
precision highp float;

#include "../glsl_lib/light.glsl"

/* ---- INPUT ---- */
// Vertex Data
layout(location = 0) in vec3 view_position;
layout(location = 1) in vec3 view_normal;
layout(location = 2) in vec4 vert_color;

// Uniform Data
layout(set = 1, binding = 0) uniform DataWorld {
  AmbientLight ambient_light;

  uint num_dir_lights;
  DirectionalLight dir_lights[32];

  uint num_point_lights;
  PointLight point_lights[32];

  uint num_spot_lights;
  SpotLight spot_lights[32];
}
world_uniforms;

layout(set = 2, binding = 0) uniform DataMesh { PhongMaterial material; }
mesh_uniforms;

/* ---- OUTPUT ---- */
layout(location = 0) out vec4 frag_color;

/* ---- MAIN ---- */
void main() {
  /* -- INITIALIZATION -- */
  Vertex vertex = {view_position, view_normal};
  frag_color = vec4(0.0);

  /* -- LIGHT APPLICATION -- */
  // AMBIENT
  frag_color +=
      light_apply(world_uniforms.ambient_light, mesh_uniforms.material, vertex);

  // DIRECTIONAL
  for (uint i = 0; i < world_uniforms.num_dir_lights; ++i)
    frag_color += light_apply(world_uniforms.dir_lights[i],
                              mesh_uniforms.material, vertex);

  // POINT
  for (uint i = 0; i < world_uniforms.num_point_lights; ++i)
    frag_color += light_apply(world_uniforms.point_lights[i],
                              mesh_uniforms.material, vertex);

  // SPOT
  for (uint i = 0; i < world_uniforms.num_spot_lights; ++i)
    frag_color += light_apply(world_uniforms.spot_lights[i],
                              mesh_uniforms.material, vertex);

  /* -- CLAMPING -- */
  frag_color = clamp(frag_color, 0, 1);
}
