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

struct RoundRect {
  vec3 position;
  float radius;
  vec2 size;
  uint color;
};

layout(buffer_reference, std430) readonly buffer RoundRectBuffer {
  RoundRect round_rects[];
};

layout(push_constant) uniform PushConsts {
  RoundRectBuffer round_rect_buffer;
  vec2 cam_size;
} push_consts;

layout(location = 0) out vec3 color;
layout(location = 1) out vec2 local_position;
layout(location = 2) flat out vec2 half_size;
layout(location = 3) flat out float radius;

void main() {
  const RoundRect round_rect = push_consts.round_rect_buffer.round_rects[gl_VertexIndex / POSITIONS.length()];
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];

  gl_Position = vec4(
    (position * round_rect.size + round_rect.position.xy) / push_consts.cam_size * vec2(2.0) - vec2(1.0),
    round_rect.position.z,
    1.0
  );

  color = vec3((round_rect.color >> 24) & 0xFF, (round_rect.color >> 16) & 0xFF, (round_rect.color >> 8) & 0xFF) / 255.0;
  local_position = (position - vec2(0.5)) * round_rect.size;
  half_size = round_rect.size * 0.5;
  radius = round_rect.radius;
}
