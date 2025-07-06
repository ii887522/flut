#version 460 core
#extension GL_EXT_buffer_reference : require

const vec2 POSITIONS[] = vec2[](
  vec2(0.0, 0.0),
  vec2(1.0, 0.0),
  vec2(1.0, 1.0),
  vec2(1.0, 1.0),
  vec2(0.0, 1.0),
  vec2(0.0, 0.0)
);

struct Rect {
  vec2 position;
  vec2 size;
  uint color;
  float pad;
};

layout(buffer_reference, std430, buffer_reference_align = 8) readonly buffer RectBuffer {
  Rect rects[];
};

layout(push_constant, std430) uniform PushConst {
  RectBuffer rectBuffer;
  vec2 camSize;
} pushConst;

layout(location = 0) out vec3 color;

void main() {
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];
  const Rect rect = pushConst.rectBuffer.rects[gl_VertexIndex / POSITIONS.length()];

  gl_Position = vec4(
    (position.x * rect.size.x + rect.position.x) / pushConst.camSize.x * 2.0 - 1.0,
    (position.y * rect.size.y + rect.position.y) / pushConst.camSize.y * 2.0 - 1.0,
    0.0,
    1.0
  );

  color = vec3((rect.color >> 24) / 255.0, ((rect.color >> 16) & 0xFF) / 255.0, ((rect.color >> 8) & 0xFF) / 255.0);
}
