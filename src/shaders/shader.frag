#version 460 core

layout(location = 0) in vec3 color;
layout(location = 1) in vec2 local_position;
layout(location = 2) flat in vec2 half_size;
layout(location = 3) flat in float radius;

layout(location = 0) out vec4 out_color;

// Signed distance function for rounded rectangle
// - position: point relative to rect center
// - half_size: half of rect dimensions
// - radius: corner radius
float sd_round_rect(const vec2 position, const vec2 half_size, const float radius) {
  const vec2 q = abs(position) - half_size + vec2(radius);
  return length(max(q, vec2(0.0))) - radius;
}

void main() {
  const float d = sd_round_rect(local_position, half_size, radius);
  out_color = vec4(color, 1.0 - d);
}
