#version 460
#extension GL_EXT_buffer_reference : require

const vec2 VERTICES[] = vec2[](
  vec2(0.0, 0.0),
  vec2(1.0, 0.0),
  vec2(1.0, 1.0),
  vec2(1.0, 1.0),
  vec2(0.0, 1.0),
  vec2(0.0, 0.0)
);

struct Mesh {
  vec2 position;
  vec2 size;
  vec2 texPosition;
  uint color;
  float pad;
};

layout(std430, buffer_reference, buffer_reference_align = 8) readonly buffer MeshBuffer {
  Mesh meshes[];
};

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

layout(std430, push_constant) uniform PushConstant {
  vec2 cameraSize;
  MeshBuffer meshBuffer;
} pushConstant;

vec2 map(const vec2 from, const vec2 minFrom, const vec2 maxFrom, const vec2 minTo, const vec2 maxTo) {
  return minTo + (from - minFrom) * (maxTo - minTo) / (maxFrom - minFrom);
}

void main() {
  const Mesh mesh = pushConstant.meshBuffer.meshes[gl_VertexIndex / VERTICES.length()];
  const vec2 translation = map(mesh.position, vec2(0.0), pushConstant.cameraSize, vec2(-1.0), vec2(1.0));
  const vec2 scale = map(mesh.size, vec2(0.0), pushConstant.cameraSize, vec2(0.0), vec2(2.0));
  const vec2 position = VERTICES[gl_VertexIndex % VERTICES.length()];

  gl_Position = vec4(position * scale + translation, 0.0, 1.0);
  fragTexCoord = position * mesh.size + mesh.texPosition;

  fragColor = vec3(
    float(mesh.color >> 24) / 255.0,
    float((mesh.color >> 16) & 0xFF) / 255.0,
    float((mesh.color >> 8) & 0xFF) / 255.0
  );
}
