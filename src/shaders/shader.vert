#version 460 core

const vec2 POSITIONS[6] = vec2[](
  vec2(0.0, 0.0),
  vec2(1.0, 0.0),
  vec2(1.0, 1.0),
  vec2(1.0, 1.0),
  vec2(0.0, 1.0),
  vec2(0.0, 0.0)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexIndex], 0.0, 1.0);
}
