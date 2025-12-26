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

struct RoundRect {
  vec2 position;
  vec2 size;
  vec4 color;
  float radius;
};

layout(std430, buffer_reference) readonly buffer RoundRectBuffer {
  RoundRect round_rects[];
};

layout(push_constant) uniform PushConsts {
  RoundRectBuffer round_rect_buffer;
  vec2 cam_position;
  vec2 cam_size;
} push_consts;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec2 frag_local_pos;
layout(location = 2) out vec2 frag_half_size;
layout(location = 3) out float frag_radius;

void main() {
  const RoundRect round_rect = push_consts.round_rect_buffer.round_rects[gl_VertexIndex / POSITIONS.length()];
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];

  gl_Position = vec4(
    (position * round_rect.size + round_rect.position + push_consts.cam_position) / push_consts.cam_size * 2.0 - 1.0,
    0.0,
    1.0
  );

  frag_color = round_rect.color;
  frag_local_pos = (position - vec2(0.5)) * round_rect.size;
  frag_half_size = round_rect.size * 0.5;
  frag_radius = round_rect.radius;
}
