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
  uint color;
  float controlRadius;
  vec2 controlPoint;
};

layout(std430, buffer_reference, buffer_reference_align = 8) readonly buffer MeshBuffer {
  Mesh meshes[];
};

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragControlPoint;
layout(location = 2) out float fragControlRadius;
layout(location = 3) out vec2 fragBearing;

layout(std430, push_constant) uniform PushConstant {
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
    mesh.position, vec2(0.0), pushConstant.cameraSize * pushConstant.pixelSize, vec2(-1.0), vec2(1.0)
  );
  const vec2 scale = map(mesh.size, vec2(0.0), pushConstant.cameraSize * pushConstant.pixelSize, vec2(0.0), vec2(2.0));
  const vec2 position = VERTICES[gl_VertexIndex % VERTICES.length()];

  gl_Position = vec4(position * scale + translation, 0.0, 1.0);

  fragColor = vec4(
    float(mesh.color >> 24) / 255.0,
    float((mesh.color >> 16) & 0xFF) / 255.0,
    float((mesh.color >> 8) & 0xFF) / 255.0,
    float(mesh.color & 0xFF) / 255.0
  );

  fragControlPoint = mesh.controlPoint / pushConstant.pixelSize;
  fragControlRadius = mesh.controlRadius / pushConstant.pixelSize;

  fragBearing = vec2(
    normalize(mesh.controlPoint.x - (mesh.position.x + mesh.size.x * 0.5)),
    normalize(mesh.controlPoint.y - (mesh.position.y + mesh.size.y * 0.5))
  );
}
