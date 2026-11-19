#version 300 es
precision highp float;

out vec4 fragColor;
uniform sampler2D u_scene;
uniform vec2 u_origin;
uniform vec2 u_captureSize;
uniform vec2 u_center;
uniform float u_radius;
uniform float u_strength;

// The twist tapers smoothly to zero at the outer edge of the lens.
const float TWIST_RADIANS = 5.3;
const float LENS_COMPRESSION = 0.21;
const float CORE_DARKNESS = 0.44;

void main() {
    vec2 p = gl_FragCoord.xy;
    vec2 offset = p - u_center;
    float dist = length(offset);
    float r = dist / max(u_radius, 1.0);
    vec2 sampleP = p;

    if (r < 1.0) {
        // Twist the scene around the cursor, not the shape of the visible ring.
        // All displacements stay within the captured radius.
        float interior = 1.0 - smoothstep(0.025, 0.97, r);
        float angle = TWIST_RADIANS * u_strength * pow(interior, 1.25);
        float s = sin(angle);
        float c = cos(angle);
        vec2 rotated = vec2(c * offset.x - s * offset.y,
                            s * offset.x + c * offset.y);

        // A modest radial pull adds depth to the swirling distortion.
        float pull = 1.0 - LENS_COMPRESSION * u_strength * pow(1.0 - r, 2.0);
        sampleP = u_center + rotated * max(pull, 0.6);
    }

    vec2 uv = clamp(sampleP - u_origin,
                    vec2(0.5),
                    u_captureSize - vec2(0.5)) / u_captureSize;
    vec4 scene = texture(u_scene, uv);

    // Iridescent accretion ring, without a second scene texture lookup.
    float ring = exp(-pow((r - 0.59) / 0.105, 2.0));
    float halo = exp(-pow((r - 0.85) / 0.055, 2.0));
    float theta = atan(offset.y, offset.x);
    vec3 violet = vec3(0.55, 0.32, 1.00);
    vec3 cyan = vec3(0.25, 0.83, 1.00);
    vec3 accent = mix(violet, cyan, 0.5 + 0.5 * sin(2.0 * theta + 0.8));
    scene.rgb += accent * (0.29 * ring + 0.065 * halo) * u_strength;

    // Darker than the previous version, but still translucent.
    float core = 1.0 - smoothstep(0.08, 0.27, r);
    float opacity = clamp(CORE_DARKNESS * u_strength * core, 0.0, 0.76);
    scene.rgb = mix(scene.rgb, vec3(0.014, 0.018, 0.035), opacity);

    fragColor = vec4(scene.rgb, scene.a);
}
