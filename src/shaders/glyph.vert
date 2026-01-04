#version 460 core
#extension GL_EXT_buffer_reference2 : require

const vec2 POSITIONS[] = vec2[](
  vec2(0.0, 0.0), // Top-left
  vec2(1.0, 0.0), // Top-right
  vec2(1.0, 1.0), // Bottom-right
  vec2(1.0, 1.0), // Bottom-right
  vec2(0.0, 1.0), // Bottom-left
  vec2(0.0, 0.0)  // Top-left
);

struct Glyph {
  vec2 position;
  vec2 size;
  vec2 atlas_position;
  vec2 atlas_size;
  uint color;
  float pad;
};

layout(std430, buffer_reference) readonly buffer GlyphBuffer {
  Glyph glyphs[];
};

layout(push_constant) uniform PushConsts {
  GlyphBuffer glyph_buffer;
  vec2 cam_position;
  vec2 cam_size;
  vec2 atlas_size;
} push_consts;

layout(location = 0) out vec2 frag_uv;
layout(location = 1) out vec4 frag_color;

void main() {
  const Glyph glyph = push_consts.glyph_buffer.glyphs[gl_VertexIndex / POSITIONS.length()];
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];

  gl_Position = vec4(
    (position * glyph.size + glyph.position + push_consts.cam_position) / push_consts.cam_size * 2.0 - 1.0,
    0.0,
    1.0
  );

  frag_uv = (position * glyph.atlas_size + glyph.atlas_position) / push_consts.atlas_size;

  frag_color = vec4(
    (glyph.color >> 24) / 255.0,
    ((glyph.color >> 16) & 0xFF) / 255.0,
    ((glyph.color >> 8) & 0xFF) / 255.0,
    (glyph.color & 0xFF) / 255.0
  );
}
