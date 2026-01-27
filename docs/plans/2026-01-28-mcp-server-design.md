# MCP Server for LLM-Assisted Asset Editing

**Date:** 2026-01-28
**Status:** Implemented

## Problem

SpecCade's declarative spec system is ideal for LLM-assisted authoring, but there's no
standard interface for LLM coding tools (Claude Code, Codex, OpenCode, Copilot) to
interact with the asset pipeline programmatically. Users must manually copy-paste CLI
output and describe results to the LLM.

## Solution

Build `speccade-mcp`, a Model Context Protocol server that exposes SpecCade's asset
pipeline as tools any MCP-compatible LLM client can call. The editor serves as the
live preview surface via its existing file watcher.

## Architecture

```
┌──────────────────┐     stdio (MCP)     ┌──────────────┐     subprocess     ┌──────────────┐
│  Claude Code /   │ ◄────────────────► │  speccade-mcp │ ────────────────► │  speccade    │
│  Codex / etc.    │   JSON-RPC 2.0     │  (new crate)  │   CLI commands    │  CLI         │
└──────────────────┘                     └──────────────┘                    └──────────────┘
                                                                                  │
       User sees preview in editor ◄──── file watcher hot-reload ◄───────────────┘
```

The MCP server is a thin wrapper around existing CLI commands. No new asset logic.
The editor's file watcher provides the live preview loop for free.

## MCP Tools

### Discovery & Context

| Tool | Input | Output | CLI Command |
|------|-------|--------|-------------|
| `stdlib_reference` | (none) | Starlark function catalog (JSON) | `stdlib dump --format json` |
| `list_templates` | `asset_type?` | Template IDs + descriptions | `template list --json` |
| `get_template` | `template_id` | Starlark source | `template show <id>` |
| `list_specs` | `directory?` | File paths + asset types | Glob `**/*.star` + `eval` |
| `read_spec` | `path` | Starlark source code | File read |

### Authoring

| Tool | Input | Output | CLI Command |
|------|-------|--------|-------------|
| `write_spec` | `path`, `content` | Success + path | File write (triggers editor reload) |
| `eval_spec` | `path` | Canonical JSON IR | `eval --spec <path> --json` |

### Validation & Generation

| Tool | Input | Output | CLI Command |
|------|-------|--------|-------------|
| `validate_spec` | `path`, `budget?` | Pass/fail + errors | `validate --spec <path> --json` |
| `generate_preview` | `path` | Preview asset path | `generate --spec <path> --preview 2` |
| `generate_full` | `path`, `out_dir?` | Generated asset paths | `generate --spec <path>` |

### Analysis

| Tool | Input | Output | CLI Command |
|------|-------|--------|-------------|
| `analyze_asset` | `path` | Quality metrics JSON | `analyze --input <path> --json` |
| `compare_assets` | `path_a`, `path_b` | Comparison report | `compare --a <path_a> --b <path_b>` |

## Crate Structure

```
crates/speccade-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs           # MCP server setup via rmcp, tool registration
│   ├── tools/
│   │   ├── mod.rs        # Tool registry + dispatch
│   │   ├── discovery.rs  # stdlib_reference, list_templates, get_template, list_specs, read_spec
│   │   ├── authoring.rs  # write_spec, eval_spec
│   │   ├── generation.rs # validate_spec, generate_preview, generate_full
│   │   └── analysis.rs   # analyze_asset, compare_assets
```

## MCP Protocol Implementation

Uses the official [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) crate
(the Rust MCP SDK maintained by the MCP organization). This provides:

- Correct protocol handling (initialization, capability negotiation, error codes)
- Tool definition macros/builders
- stdio + SSE transport support
- Forward compatibility as the MCP spec evolves

No need to hand-roll JSON-RPC -- the SDK handles the protocol layer.

## Configuration

Users add to `.mcp.json` in their project root or Claude Code settings:

```json
{
  "mcpServers": {
    "speccade": {
      "command": "speccade-mcp",
      "args": ["--project-dir", "."]
    }
  }
}
```

Or with cargo: `cargo run -p speccade-mcp --`

## Typical LLM Workflow

1. LLM calls `stdlib_reference` → understands available Starlark functions
2. LLM calls `list_templates` → picks a starting point
3. LLM calls `get_template` → gets starter code
4. LLM calls `write_spec` → creates/modifies `.star` file (editor auto-previews)
5. LLM calls `validate_spec` → checks for errors
6. User reviews preview in editor, asks for changes
7. LLM reads spec, modifies, writes back → iterate
8. LLM calls `generate_full` → produce final assets

## Out of Scope (Future Work)

- Screenshot/vision feedback (capture editor preview for VLM analysis)
- Editor-embedded MCP (keep as separate process for simplicity)
- Streaming generation progress
- Multi-user / remote server mode
- Resource subscriptions (spec change notifications)

## Implementation Sequence

1. Scaffold `crates/speccade-mcp` with Cargo.toml and binary target
2. Implement `protocol.rs` (MCP JSON-RPC types)
3. Implement discovery tools (most useful for LLM context)
4. Implement authoring tools (write_spec, eval_spec)
5. Implement generation tools (validate, preview, full)
6. Implement analysis tools
7. Add `.mcp.json` example to repo root
8. Test end-to-end with Claude Code
