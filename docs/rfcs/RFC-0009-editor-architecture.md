# RFC-0009: Editor Architecture for Real-Time Asset Preview

**Status:** Draft
**Created:** 2026-01-17
**Author:** Claude
**Related:** RFC-0008 (LLM-Native Asset Authoring)

## Abstract

This RFC proposes an integrated editor for SpecCade that provides real-time preview of assets during authoring. The editor combines a Monaco-based code editor with domain-specific visualizers (waveform, 3D viewport, texture preview) and optional LLM assistance.

## Motivation

Currently, the SpecCade workflow requires:
1. Write Starlark spec
2. Run CLI to generate assets
3. Open assets in external viewers
4. Iterate

This creates a slow feedback loop that impacts both human and LLM authors. An integrated editor with hot-reload preview would:
- Reduce iteration time from minutes to milliseconds
- Enable real-time parameter tweaking
- Provide immediate visual/auditory feedback
- Support LLM-in-the-loop refinement workflows

## Goals

1. Sub-100ms preview latency for parameter changes
2. Support all major asset types (audio, texture, mesh, animation)
3. Preserve determinism guarantees
4. Work standalone and as IDE extension
5. Optional LLM integration for assisted authoring

## Non-Goals

- Replacing the CLI for production builds
- Supporting non-Starlark input formats
- Real-time multiplayer collaboration (v1)

## Design

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Shell | Tauri 2.x | Native performance, small binary, Rust backend |
| Editor | Monaco | VSCode compatibility, LSP support, syntax highlighting |
| 3D Viewport | three.js | WebGL2, glTF native, mature ecosystem |
| Audio | Web Audio API | Low-latency, waveform visualization |
| Texture | Canvas 2D / WebGL | Tiling preview, mipmap visualization |

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Shell                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │    Monaco    │  │   Preview    │  │   LLM Panel     │  │
│  │    Editor    │  │   Viewport   │  │   (Optional)    │  │
│  │              │  │              │  │                  │  │
│  │  - Starlark  │  │  - 3D/WebGL  │  │  - Chat         │  │
│  │  - JSON      │  │  - Waveform  │  │  - Suggestions  │  │
│  │  - LSP       │  │  - Texture   │  │  - Refinement   │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         └────────────┬────┴────────────────────┘            │
│                      │                                      │
│              ┌───────▼───────┐                              │
│              │  IPC Bridge   │                              │
│              └───────┬───────┘                              │
├──────────────────────┼──────────────────────────────────────┤
│                      │           Rust Backend               │
│              ┌───────▼───────┐                              │
│              │  Spec Parser  │                              │
│              │  (Starlark)   │                              │
│              └───────┬───────┘                              │
│                      │                                      │
│              ┌───────▼───────┐                              │
│              │   Validator   │                              │
│              └───────┬───────┘                              │
│                      │                                      │
│         ┌────────────┼────────────┐                         │
│         ▼            ▼            ▼                         │
│   ┌──────────┐ ┌──────────┐ ┌──────────┐                   │
│   │  Audio   │ │ Texture  │ │   Mesh   │                   │
│   │ Preview  │ │ Preview  │ │ Preview  │                   │
│   │ Backend  │ │ Backend  │ │ Backend  │                   │
│   └──────────┘ └──────────┘ └──────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

### Preview Backends

Preview backends are optimized versions of the full generation backends:

1. **Audio Preview**: Generate short segments (0.5s) with reduced sample rate (22kHz) for parameter tweaking, full quality on explicit request
2. **Texture Preview**: Generate at 256x256 with mipmaps, upscale on zoom
3. **Mesh Preview**: Generate low-poly proxy, refine on demand

### Hot Reload Pipeline

```
Editor Change
     │
     ▼
  Debounce (50ms)
     │
     ▼
  Parse Starlark
     │
     ├─── Syntax Error ──► Inline Diagnostic
     │
     ▼
  Validate Spec
     │
     ├─── Validation Error ──► Inline Diagnostic + Suggestions
     │
     ▼
  Preview Generation
     │
     ├─── Audio ──► Waveform + Playback
     ├─── Texture ──► Canvas Render
     └─── Mesh ──► Three.js Scene
```

### LSP Extensions

The editor extends the Language Server Protocol with SpecCade-specific features:

| Extension | Description |
|-----------|-------------|
| `speccade/presetHover` | Show preset preview on hover |
| `speccade/parameterSlider` | Inline parameter adjustment widgets |
| `speccade/waveformPreview` | Inline audio waveform for sound specs |
| `speccade/colorPreview` | Inline color swatches for texture specs |

### LLM Integration (Optional)

The editor can connect to an LLM for assisted authoring:

1. **Describe-to-Generate**: User describes desired asset, LLM generates spec
2. **Refinement**: User describes changes, LLM modifies spec
3. **Error Explanation**: LLM explains validation errors with fix suggestions
4. **Parameter Guidance**: LLM suggests parameter values based on intent

LLM integration uses a well-defined protocol:
```json
{
  "request": "refine",
  "current_spec": "...",
  "instruction": "make the sound more metallic",
  "context": {
    "asset_type": "audio",
    "validation_state": "valid",
    "current_preview_hash": "abc123"
  }
}
```

### IDE Extension Mode

The editor can run as a VSCode/Cursor extension:
- Monaco component replaced by native editor
- Preview panels as webview
- IPC via extension host
- Same Rust backend via WASM or sidecar

## Determinism Considerations

Preview generation may use approximations for speed. The editor must clearly indicate:
- Preview mode (fast, approximate)
- Production mode (deterministic, full quality)
- Hash mismatch between preview and production output

A "Verify Determinism" button generates full-quality output and compares against preview.

## Tracking

All implementation work and open questions for this RFC are tracked in `docs/ROADMAP.md` under **Editor / Real-Time Preview (RFC-0009)**.

## Alternatives Considered

### Electron
- Pros: Familiar, large ecosystem
- Cons: Large binary size (150MB+), higher memory usage
- Decision: Tauri preferred for Rust integration and smaller footprint

### Custom Editor
- Pros: Full control, no dependencies
- Cons: Massive effort, missing features users expect
- Decision: Monaco provides 90% of needed features

### Web-Only (No Desktop Shell)
- Pros: Universal access, no install
- Cons: Limited file system access, no native performance
- Decision: Desktop-first with potential web viewer later

### Blender Add-on
- Pros: Mature 3D environment, existing user base
- Cons: Only covers mesh/animation, complex integration
- Decision: Standalone editor covers all asset types

## Security Considerations

- Starlark sandbox prevents arbitrary code execution
- LLM integration uses user-provided API keys (not stored)
- Preview generation runs in separate process with resource limits
- No network access except explicit LLM API calls

## References

- [Tauri Documentation](https://tauri.app/v2/)
- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
- [Three.js](https://threejs.org/)
- [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
- [Shadertoy](https://www.shadertoy.com/) - Inspiration for real-time shader editing
