#version 460 core
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
  vec3 position;
  uint color;
  vec3 texPosition;
  float pad;
  vec2 size;
  vec2 texSize;
};

layout(std430, buffer_reference, buffer_reference_align = 16) readonly restrict buffer MeshBuffer {
  Mesh meshes[];
};

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragTexCoord;

layout(std430, push_constant) uniform PushConstant {
  vec2 cameraPosition;
  vec2 cameraSize;
  float pixelSize;
  float pad;
  MeshBuffer meshBuffer;
} pushConstant;

vec2 map(const vec2 from, const vec2 minFrom, const vec2 maxFrom, const vec2 minTo, const vec2 maxTo) {
  return minTo + (from - minFrom) * (maxTo - minTo) / (maxFrom - minFrom);
}

void main() {
  const Mesh mesh = pushConstant.meshBuffer.meshes[gl_VertexIndex / VERTICES.length()];

  const vec2 translation = map(
    mesh.position.xy,
    pushConstant.cameraPosition,
    pushConstant.cameraPosition + pushConstant.cameraSize * pushConstant.pixelSize,
    vec2(-1.0),
    vec2(1.0)
  );

  const vec2 scale = map(mesh.size, vec2(0.0), pushConstant.cameraSize * pushConstant.pixelSize, vec2(0.0), vec2(2.0));
  const vec2 position = VERTICES[gl_VertexIndex % VERTICES.length()];

  gl_Position = vec4(position * scale + translation, mesh.position.z, 1.0);
  fragTexCoord = vec3(position * mesh.texSize + mesh.texPosition.xy, mesh.texPosition.z);

  fragColor = vec4(
    float(mesh.color >> 24) / 255.0,
    float((mesh.color >> 16) & 0xFF) / 255.0,
    float((mesh.color >> 8) & 0xFF) / 255.0,
    float(mesh.color & 0xFF) / 255.0
  );
}
