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

struct Rect {
  vec2 position;
  vec2 size;
  vec4 color;
};

layout(std430, buffer_reference) readonly buffer RectBuffer {
  Rect rects[];
};

layout(push_constant) uniform PushConsts {
  RectBuffer rect_buffer;
  vec2 cam_position;
  vec2 cam_size;
} push_consts;

layout(location = 0) out vec4 frag_color;

void main() {
  const Rect rect = push_consts.rect_buffer.rects[gl_VertexIndex / POSITIONS.length()];

  gl_Position = vec4(
    (POSITIONS[gl_VertexIndex % POSITIONS.length()] * rect.size + rect.position + push_consts.cam_position) / push_consts.cam_size * 2.0 - 1.0,
    0.0,
    1.0
  );

  frag_color = rect.color;
}
