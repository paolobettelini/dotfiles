// Hyprland 0.57: bounded, pointer-following swirling gravitational lens.
#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/event/EventBus.hpp>
#include <hyprland/src/render/Renderer.hpp>
#include <hyprland/src/render/pass/PassElement.hpp>
#include <hyprland/src/managers/input/InputManager.hpp>
#include <hyprland/src/output/Monitor.hpp>

#include <GLES3/gl3.h>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>

namespace {
HANDLE handle = nullptr;
bool enabled = false;
constexpr float radius = 235.0F;    // logical pixels: larger lens
constexpr float strength = 1.10F;   // pronounced swirl and lensing
constexpr float margin = 18.0F;     // room for sampling beyond the lens
CHyprSignalListener pointerListener, renderListener;

std::string shaderSource() {
    std::ifstream input(BLACKHOLE_SHADER_PATH);
    if (!input) throw std::runtime_error("Cannot open " BLACKHOLE_SHADER_PATH);
    std::ostringstream output;
    output << input.rdbuf();
    return output.str();
}

constexpr auto VERTEX = R"(#version 300 es
precision highp float;
void main() {
    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
})";

GLuint compileShader(GLenum type, const std::string& source) {
    const GLuint shader = glCreateShader(type);
    const char* text = source.c_str();
    glShaderSource(shader, 1, &text, nullptr);
    glCompileShader(shader);
    GLint ok = GL_FALSE;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &ok);
    if (ok != GL_TRUE) {
        GLint size = 0;
        glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &size);
        std::string error(static_cast<size_t>(std::max(size, 1)), '\0');
        glGetShaderInfoLog(shader, size, nullptr, error.data());
        glDeleteShader(shader);
        throw std::runtime_error("Shader compilation failed: " + error);
    }
    return shader;
}

// Save all GL state that the custom pass changes. Hyprland owns the context.
struct GLState {
    GLint program = 0, vao = 0, activeTexture = 0, tex0 = 0;
    GLint readFbo = 0, drawFbo = 0;
    GLint viewport[4] = {}, scissorBox[4] = {};
    GLboolean colorMask[4] = {};
    GLboolean blend, scissor, depth, stencil, cull;

    GLState() {
        glGetIntegerv(GL_CURRENT_PROGRAM, &program);
        glGetIntegerv(GL_VERTEX_ARRAY_BINDING, &vao);
        glGetIntegerv(GL_ACTIVE_TEXTURE, &activeTexture);
        glActiveTexture(GL_TEXTURE0);
        glGetIntegerv(GL_TEXTURE_BINDING_2D, &tex0);
        glGetIntegerv(GL_READ_FRAMEBUFFER_BINDING, &readFbo);
        glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING, &drawFbo);
        glGetIntegerv(GL_VIEWPORT, viewport);
        glGetIntegerv(GL_SCISSOR_BOX, scissorBox);
        glGetBooleanv(GL_COLOR_WRITEMASK, colorMask);
        blend = glIsEnabled(GL_BLEND);
        scissor = glIsEnabled(GL_SCISSOR_TEST);
        depth = glIsEnabled(GL_DEPTH_TEST);
        stencil = glIsEnabled(GL_STENCIL_TEST);
        cull = glIsEnabled(GL_CULL_FACE);
    }
    ~GLState() {
        glUseProgram(static_cast<GLuint>(program));
        glBindVertexArray(static_cast<GLuint>(vao));
        glBindTexture(GL_TEXTURE_2D, static_cast<GLuint>(tex0));
        glActiveTexture(static_cast<GLenum>(activeTexture));
        glBindFramebuffer(GL_READ_FRAMEBUFFER, static_cast<GLuint>(readFbo));
        glBindFramebuffer(GL_DRAW_FRAMEBUFFER, static_cast<GLuint>(drawFbo));
        glViewport(viewport[0], viewport[1], viewport[2], viewport[3]);
        glScissor(scissorBox[0], scissorBox[1], scissorBox[2], scissorBox[3]);
        glColorMask(colorMask[0], colorMask[1], colorMask[2], colorMask[3]);
        if (blend) glEnable(GL_BLEND); else glDisable(GL_BLEND);
        if (scissor) glEnable(GL_SCISSOR_TEST); else glDisable(GL_SCISSOR_TEST);
        if (depth) glEnable(GL_DEPTH_TEST); else glDisable(GL_DEPTH_TEST);
        if (stencil) glEnable(GL_STENCIL_TEST); else glDisable(GL_STENCIL_TEST);
        if (cull) glEnable(GL_CULL_FACE); else glDisable(GL_CULL_FACE);
    }
};

class GPU {
public:
    GLuint program = 0, texture = 0, vao = 0;
    int textureWidth = 0, textureHeight = 0;
    bool failed = false;

    bool init() {
        if (failed) return false;
        if (program != 0) return true;
        try {
            const GLuint vs = compileShader(GL_VERTEX_SHADER, VERTEX);
            GLuint fs = 0;
            try {
                fs = compileShader(GL_FRAGMENT_SHADER, shaderSource());
            } catch (...) {
                glDeleteShader(vs);
                throw;
            }
            program = glCreateProgram();
            glAttachShader(program, vs);
            glAttachShader(program, fs);
            glLinkProgram(program);
            glDeleteShader(vs);
            glDeleteShader(fs);
            GLint ok = GL_FALSE;
            glGetProgramiv(program, GL_LINK_STATUS, &ok);
            if (ok != GL_TRUE) throw std::runtime_error("Program linking failed");
            glGenTextures(1, &texture);
            glGenVertexArrays(1, &vao);
            glBindTexture(GL_TEXTURE_2D, texture);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
            return true;
        } catch (const std::exception& e) {
            // Do not trigger Hyprland notifications recursively from within a render pass.
            std::fprintf(stderr, "hypr-blackhole: %s\n", e.what());
            failed = true;
            return false;
        }
    }

    void release() {
        if (texture) glDeleteTextures(1, &texture);
        if (vao) glDeleteVertexArrays(1, &vao);
        if (program) glDeleteProgram(program);
        program = texture = vao = 0;
        textureWidth = textureHeight = 0;
    }

    void draw(int screenW, int screenH, float cx, float cy, float r, float power) {
        if (screenW <= 0 || screenH <= 0) return;
        // A small bounding region; never copy/read back a whole monitor frame.
        const int extra = static_cast<int>(std::ceil(margin));
        const int x0 = std::clamp(static_cast<int>(std::floor(cx - r)) - extra, 0, screenW);
        const int y0 = std::clamp(static_cast<int>(std::floor(cy - r)) - extra, 0, screenH);
        const int x1 = std::clamp(static_cast<int>(std::ceil(cx + r)) + extra, 0, screenW);
        const int y1 = std::clamp(static_cast<int>(std::ceil(cy + r)) + extra, 0, screenH);
        const int copyW = x1 - x0, copyH = y1 - y0;
        if (copyW <= 0 || copyH <= 0) return;

        GLState state;
        if (!init()) return;

        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture);
        if (copyW != textureWidth || copyH != textureHeight) {
            textureWidth = copyW;
            textureHeight = copyH;
            glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, copyW, copyH, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
        }

        // The scene is in Hyprland's DRAW framebuffer; read from that exact FB.
        // Copy before drawing, so the shader never samples its own output.
        glBindFramebuffer(GL_READ_FRAMEBUFFER, static_cast<GLuint>(state.drawFbo));
        glCopyTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, x0, y0, copyW, copyH);
        glBindFramebuffer(GL_READ_FRAMEBUFFER, static_cast<GLuint>(state.readFbo));

        glDisable(GL_BLEND);
        glDisable(GL_DEPTH_TEST);
        glDisable(GL_STENCIL_TEST);
        glDisable(GL_CULL_FACE);
        glEnable(GL_SCISSOR_TEST);
        glScissor(x0, y0, copyW, copyH);
        glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
        glViewport(0, 0, screenW, screenH);
        glUseProgram(program);
        glUniform1i(glGetUniformLocation(program, "u_scene"), 0);
        glUniform2f(glGetUniformLocation(program, "u_origin"), float(x0), float(y0));
        glUniform2f(glGetUniformLocation(program, "u_captureSize"), float(copyW), float(copyH));
        glUniform2f(glGetUniformLocation(program, "u_center"), cx, cy);
        glUniform1f(glGetUniformLocation(program, "u_radius"), r);
        glUniform1f(glGetUniformLocation(program, "u_strength"), power);
        glBindVertexArray(vao);
        glDrawArrays(GL_TRIANGLES, 0, 3);
    }
};
GPU gpu;

class BlackholePass final : public IPassElement {
public:
    BlackholePass(int w, int h, float x, float y, float r, float power)
        : width(w), height(h), x(x), y(y), radiusPx(r), strengthPx(power) {}

    std::vector<UP<IPassElement>> draw(Render::CRenderContext&) override {
        gpu.draw(width, height, x, y, radiusPx, strengthPx);
        return {};
    }
    bool needsLiveBlur(Render::CRenderContext&) override { return false; }
    bool needsPrecomputeBlur(Render::CRenderContext&) override { return false; }
    bool requiresFullDamage(Render::CRenderContext&) override { return false; }
    // Ensure old and new damaged pixels are repainted from the original scene.
    bool disableSimplification(Render::CRenderContext&) override { return true; }
    const char* passName() override { return "BlackholePass"; }
    ePassElementType type() override { return EK_CUSTOM; }

private:
    int width, height;
    float x, y, radiusPx, strengthPx;
};

void damageAt(const Vector2D& p) {
    if (!g_pHyprRenderer) return;
    const double pad = radius + margin + 3.0;
    g_pHyprRenderer->damageBox(CBox{p.x - pad, p.y - pad, 2.0 * pad, 2.0 * pad});
}

void damageCurrent() {
    if (g_pInputManager) damageAt(g_pInputManager->getMouseCoordsInternal());
}

int luaToggle(lua_State*) {
    enabled = !enabled;
    damageCurrent();
    return 0;
}
int luaOn(lua_State*) {
    enabled = true;
    damageCurrent();
    return 0;
}
int luaOff(lua_State*) {
    enabled = false;
    damageCurrent();
    return 0;
}
} // namespace

APICALL EXPORT std::string PLUGIN_API_VERSION() { return HYPRLAND_API_VERSION; }

APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE h) {
    handle = h;
    if (std::string(__hyprland_api_get_hash()) != __hyprland_api_get_client_hash())
        throw std::runtime_error("hypr-blackhole: Hyprland ABI mismatch");
    if (!g_pHyprRenderer || g_pHyprRenderer->type() != Render::IHyprRenderer::RT_GL)
        throw std::runtime_error("hypr-blackhole: OpenGL renderer required");

    if (!HyprlandAPI::addLuaFunction(handle, "blackhole", "toggle", luaToggle) ||
        !HyprlandAPI::addLuaFunction(handle, "blackhole", "on", luaOn) ||
        !HyprlandAPI::addLuaFunction(handle, "blackhole", "off", luaOff))
        throw std::runtime_error("hypr-blackhole: unable to register Lua functions");

    pointerListener = Event::bus()->m_events.input.mouse.move.listen([](Vector2D p, Event::SCallbackInfo&) {
        // p is the absolute logical cursor coordinate, not a delta.
        static Vector2D previous = p;
        if (enabled) {
            damageAt(previous); // erase the old image (no trails)
            damageAt(p);        // draw only at the new cursor position
        }
        previous = p;
    });

    renderListener = Event::bus()->m_events.render.stage.listen([](Event::SRenderStageEvent event) {
        if (!enabled || event.stage != RENDER_LAST_MOMENT || !event.context || !event.monitor) return;
        const auto p = g_pInputManager->getMouseCoordsInternal();
        const auto& monitor = event.monitor;
        const auto local = p - monitor->m_position;
        const float scale = monitor->m_scale;
        const int w = static_cast<int>(monitor->m_pixelSize.x);
        const int h = static_cast<int>(monitor->m_pixelSize.y);
        const float x = static_cast<float>(local.x * scale);
        // The previous version used h - local.y * scale; on this GL pass it
        // mirrored the lens vertically. Use the native framebuffer Y instead.
        const float y = static_cast<float>(local.y * scale);
        const float pixelRadius = radius * scale;
        if (x < -pixelRadius || x > w + pixelRadius || y < -pixelRadius || y > h + pixelRadius)
            return;
        Render::IHyprRenderer::addPassElement(event.context->get(),
            makeUnique<BlackholePass>(w, h, x, y, pixelRadius, strength));
    });
    return {"hypr-blackhole", "Localized cursor-following lens (experimental)", "local", "0.4.0"};
}

APICALL EXPORT void PLUGIN_EXIT() {
    pointerListener = {};
    renderListener = {};
    enabled = false;
    damageCurrent();
    if (g_pHyprRenderer && Render::GL::g_pHyprOpenGL) {
        Render::GL::g_pHyprOpenGL->makeEGLCurrent();
        gpu.release();
    }
}
