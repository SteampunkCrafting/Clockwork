#ifndef PHONG_MATERIAL_GLSL
#define PHONG_MATERIAL_GLSL

/* ---- STRUCTS ---- */
struct PhongMaterial {
  vec4 ambient;
  vec4 diffuse;
  vec4 specular;
  float specular_power;
};

#endif