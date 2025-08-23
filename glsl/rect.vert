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
  vec3 texPosition;
  uint color;
  vec2 texSize;
  vec2 position;
  vec2 size;
  vec2 pad;
};

layout(buffer_reference, std430, buffer_reference_align = 8) readonly buffer RectBuffer {
  Rect rects[];
};

layout(push_constant, std430) uniform PushConst {
  RectBuffer rectBuffer;
  vec2 camPosition;
  vec2 camSize;
} pushConst;

layout(location = 0) out vec4 color;
layout(location = 1) out vec3 texCoord;

void main() {
  const vec2 position = POSITIONS[gl_VertexIndex % POSITIONS.length()];
  const Rect rect = pushConst.rectBuffer.rects[gl_VertexIndex / POSITIONS.length()];

  gl_Position = vec4(
    (position.x * rect.size.x + rect.position.x - pushConst.camPosition.x) / pushConst.camSize.x * 2.0 - 1.0,
    (position.y * rect.size.y + rect.position.y - pushConst.camPosition.y) / pushConst.camSize.y * 2.0 - 1.0,
    0.0,
    1.0
  );

  color = vec4(
    (rect.color >> 24) / 255.0,
    ((rect.color >> 16) & 0xFF) / 255.0,
    ((rect.color >> 8) & 0xFF) / 255.0,
    (rect.color & 0xFF) / 255.0
  );

  texCoord = vec3(
    position.x * rect.texSize.x + rect.texPosition.x,
    position.y * rect.texSize.y + rect.texPosition.y,
    rect.texPosition.z
  );
}
