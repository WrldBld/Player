# Player - Claude Code Instructions

This is the **Player** component of WrldBldr - the frontend application built with Dioxus (Rust) that runs in the browser (WASM) or as a desktop app.

## Environment

This project runs on **NixOS**. Use `nix-shell` for development dependencies:

```bash
nix-shell -p rustc cargo gcc pkg-config openssl.dev webkitgtk_4_1.dev glib.dev gtk3.dev libsoup_3.dev --run "cargo check"
```

For WASM builds:
```bash
nix-shell -p rustc cargo wasm-pack --run "dx build --release"
```

## Architecture

The Player follows a layered architecture:

```
src/
├── domain/           # Domain entities (mirrored from Engine)
├── application/      # Services (session, actions)
├── infrastructure/   # External adapters
│   ├── asset_loader/ # Data types matching Engine API responses
│   ├── websocket.rs  # WebSocket client
│   ├── api.rs        # REST API client
│   └── storage.rs    # localStorage wrapper
├── presentation/     # UI layer
│   ├── views/        # Page-level components
│   ├── components/   # Reusable UI components
│   └── state.rs      # Application state (signals)
├── routes.rs         # URL routing
└── main.rs
```

## Key Conventions

### Routing (CRITICAL)

**All navigation MUST use proper URL routing:**

- Top-level tabs: `/worlds/{world_id}/dm/{tab}` (director, creator, story-arc, settings)
- Sub-tabs: `/worlds/{world_id}/dm/{tab}/{subtab}`
- Use `Link` component from Dioxus router, NOT onClick handlers for navigation
- Tab state should come from URL parameters, not local signals

Example for adding a new tabbed view:

1. Add route in `routes.rs`:
```rust
#[route("/worlds/:world_id/dm/my-feature/:subtab")]
MyFeatureSubTabRoute { world_id: String, subtab: String },
```

2. Add route component:
```rust
#[component]
pub fn MyFeatureSubTabRoute(world_id: String, subtab: String) -> Element {
    // ...
}
```

3. Use `Link` for tab navigation:
```rust
Link {
    to: Route::MyFeatureSubTabRoute {
        world_id: world_id.clone(),
        subtab: "tab-name".to_string(),
    },
    // ...
}
```

### API Response Handling

- Engine API responses often have wrapper structures (e.g., `PaginatedResponse`)
- Always check the API response format in Engine before writing client code
- Add response wrapper structs in the fetch functions when needed

### Dioxus RSX Patterns

- Use `Signal<T>` for reactive state
- Pre-compute format values before RSX blocks (can't use `"{}", expr` inline)
- For Props with complex types, implement `PartialEq` manually if needed
- Read signals into variables before using in closures to avoid borrow issues

### Components

- Components go in `src/presentation/components/`
- Group related components in subdirectories (e.g., `story_arc/`)
- Export from module's `mod.rs`
- Props structs should derive `Props, Clone, PartialEq`

### Platform-Specific Code

Use cfg attributes for platform differences:
```rust
#[cfg(target_arch = "wasm32")]
{
    // WASM-specific code (gloo_net for HTTP)
}
#[cfg(not(target_arch = "wasm32"))]
{
    // Desktop-specific code (reqwest for HTTP)
}
```

## Running

```bash
# Check compilation
cargo check

# Run desktop app
dx serve

# Build for WASM
dx build --release
```
