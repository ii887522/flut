#version 460

// Per vertex
layout(location = 0) in vec2 position;

// Per instance
layout(location = 1) in vec2 translation;
layout(location = 2) in vec3 color;

layout(location = 0) out vec3 fragColor;

layout(push_constant) uniform PushConstant {
  vec2 cameraSize;
} pushConstant;

vec2 map(const vec2 from, const vec2 minFrom, const vec2 maxFrom, const vec2 minTo, const vec2 maxTo) {
  return minTo + (from - minFrom) * (maxTo - minTo) / (maxFrom - minFrom);
}

void main() {
  const vec2 translation = map(translation, vec2(0.0), pushConstant.cameraSize, vec2(-1.0), vec2(1.0));
  gl_Position = vec4(position + translation, 0.0, 1.0);
  fragColor = color;
}
