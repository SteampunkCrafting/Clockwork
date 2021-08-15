#ifndef LIGHT_GLSL
#define LIGHT_GLSL
#include "phong_material.glsl"
#include "vertex.glsl"

/* ---- STRUCTS ---- */

struct Attenuation {
  float constant_component;
  float linear_component;
  float quadratic_component;
};

struct AmbientLight {
  vec3 color;
};

struct DirectionalLight {
  vec3 view_direction;
  vec3 color;
};

struct PointLight {
  vec3 view_position;
  vec3 color;
  Attenuation attenuation;
};

struct SpotLight {
  float opening_angle_rad;
  vec3 view_position;
  vec3 view_direction;
  vec3 color;
  Attenuation attenuation;
};

/* ---- PUBLIC FUNCTION DECLARATIONS ---- */

vec4 light_apply(AmbientLight, PhongMaterial, Vertex);

vec4 light_apply(DirectionalLight, PhongMaterial, Vertex);

vec4 light_apply(PointLight, PhongMaterial, Vertex);

vec4 light_apply(SpotLight, PhongMaterial, Vertex);

/* ---- FUNCTION DEFINITIONS ---- */

vec4 light_apply(AmbientLight light, PhongMaterial material, Vertex) {
  return vec4(light.color, 1.0) * material.ambient;
}

vec4 light_apply(DirectionalLight light, PhongMaterial material,
                 Vertex vertex) {
  // diffuse
  float diffuse_intensity = max(dot(-light.view_direction, vertex.normal), 0.0);
  vec4 diffuse_color_component =
      diffuse_intensity * (material.diffuse * vec4(light.color, 1.0));
  // specular
  vec3 reflect_direction =
      normalize(reflect(light.view_direction, vertex.normal));
  vec3 fragment_direcion = -normalize(vertex.position);
  float specular_intensity =
      pow(max(dot(fragment_direcion, reflect_direction), 0.0),
          material.specular_power);
  vec4 specular_color_component =
      specular_intensity * (material.specular * vec4(light.color, 1.0));

  vec4 result = diffuse_color_component + specular_color_component;
  return clamp(result, 0, 1);
}

vec4 light_apply(PointLight light, PhongMaterial material, Vertex vertex) {
  // variables
  vec3 view_direction = vertex.position - light.view_position;
  float dist = length(view_direction);
  view_direction = normalize(view_direction);

  // diffuse
  float diffuse_intensity = max(dot(-view_direction, vertex.normal), 0.0);
  vec4 diffuse_color_component =
      diffuse_intensity * (material.diffuse * vec4(light.color, 1.0));
  // specular
  vec3 reflect_direction = normalize(reflect(view_direction, vertex.normal));
  vec3 fragment_direcion = -normalize(vertex.position);
  float specular_intensity =
      pow(max(dot(fragment_direcion, reflect_direction), 0.0),
          material.specular_power);
  vec4 specular_color_component =
      specular_intensity * (material.specular * vec4(light.color, 1.0));
  // result
  vec4 result = diffuse_color_component + specular_color_component;

  // attenuation
  result /= light.attenuation.constant_component +
            light.attenuation.linear_component * dist +
            light.attenuation.quadratic_component * dist * dist;

  return clamp(result, 0, 1);
}

#endif