#version 460 core
#extension GL_EXT_buffer_reference : require

const vec2 POSITIONS[6] = vec2[](
  vec2(0.0, 0.0),
  vec2(1.0, 0.0),
  vec2(1.0, 1.0),
  vec2(1.0, 1.0),
  vec2(0.0, 1.0),
  vec2(0.0, 0.0)
);

// Model types
const int ROUND_RECT = 0;
const int GLYPH = 1;

// Glyph settings
const float GLYPH_RESOLUTION_SCALE = 2.0;

struct RoundRect {
  vec3 position;
  float radius;
  vec2 size;
  uint color;
};

struct Glyph {
  vec3 position;
  uint color;
  vec2 size;
  vec2 atlas_position;
};

layout(buffer_reference, std430) readonly buffer RoundRectBuffer {
  RoundRect round_rects[];
};

layout(buffer_reference, std430) readonly buffer GlyphBuffer {
  Glyph glyphs[];
};

layout(push_constant) uniform PushConsts {
  RoundRectBuffer round_rect_buffer;
  GlyphBuffer glyph_buffer;
  vec2 cam_size;
  vec2 glyph_atlas_size;
  float window_scale_factor;
} push_consts;

layout(location = 0) flat out int model_type;
layout(location = 1) out vec3 color;
layout(location = 2) out vec2 local_position;
layout(location = 3) flat out vec2 half_size;
layout(location = 4) flat out float radius;
layout(location = 5) out vec2 atlas_position;

void main() {
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];
  const uint model_index = gl_VertexIndex / POSITIONS.length();
  vec3 model_position;
  vec2 model_size;
  uint model_color;

  switch (gl_BaseInstance) {
    case ROUND_RECT:
      const RoundRect round_rect = push_consts.round_rect_buffer.round_rects[model_index];
      model_position = round_rect.position;
      model_size = round_rect.size;
      model_color = round_rect.color;

      model_type = ROUND_RECT;
      local_position = (position - vec2(0.5)) * round_rect.size;
      half_size = round_rect.size * 0.5;
      radius = round_rect.radius;
      break;

    case GLYPH:
      const Glyph glyph = push_consts.glyph_buffer.glyphs[model_index];
      model_position = glyph.position;
      model_size = glyph.size;
      model_color = glyph.color;

      model_type = GLYPH;
      atlas_position = (position * glyph.size * push_consts.window_scale_factor * GLYPH_RESOLUTION_SCALE + glyph.atlas_position) / push_consts.glyph_atlas_size;
      break;
  }

  gl_Position = vec4(
    (position * model_size + model_position.xy) / push_consts.cam_size * vec2(2.0) - vec2(1.0),
    model_position.z,
    1.0
  );

  color = vec3((model_color >> 24) & 0xFF, (model_color >> 16) & 0xFF, (model_color >> 8) & 0xFF) / 255.0;
}
