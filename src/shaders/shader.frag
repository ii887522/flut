#version 460 core

// Model types
const int ROUND_RECT = 0;
const int GLYPH = 1;

layout(location = 0) flat in int model_type;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 local_position;
layout(location = 3) flat in vec2 half_size;
layout(location = 4) flat in float radius;
layout(location = 5) in vec2 atlas_position;

layout(binding = 0) uniform sampler2D glyph_atlas_sampler;

layout(location = 0) out vec4 out_color;

/// Signed distance function for rounded rectangle
/// - `position`: point relative to rect center
/// - `half_size`: half of rect dimensions
/// - `radius`: corner radius
float sd_round_rect(const vec2 position, const vec2 half_size, const float radius) {
  const vec2 q = abs(position) - half_size + vec2(radius);
  return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

void main() {
  float a;

  switch (model_type) {
    case ROUND_RECT:
      const float d = sd_round_rect(local_position, half_size, radius);
      const float w = fwidth(d) * 0.65;
      a = 1.0 - smoothstep(-w, w, d);
      break;

    case GLYPH:
      a = texture(glyph_atlas_sampler, atlas_position).r;
      break;
  }

  out_color = vec4(color, a);
}
