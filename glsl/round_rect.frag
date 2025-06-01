#version 460 core

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragControlPoint;
layout(location = 2) in float fragControlRadius;
layout(location = 3) in vec2 fragBearing;

layout(location = 0) out vec4 color;

void main() {
  color = gl_FragCoord.x * fragBearing.x <= fragControlPoint.x * fragBearing.x ||
    gl_FragCoord.y * fragBearing.y <= fragControlPoint.y * fragBearing.y ||
    distance(gl_FragCoord.xy, fragControlPoint) <= fragControlRadius ? fragColor : vec4(fragColor.rgb, 0.0);
}
