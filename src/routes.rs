//! Application routing - URL-based navigation for all views

use dioxus::prelude::*;
use crate::presentation::state::{ConnectionStatus, DialogueState, GameState, GenerationState, SessionState};
use crate::presentation::views::dm_view::DMMode;
use crate::infrastructure::websocket::ParticipantRole;
use crate::application::services::{SessionService, DEFAULT_ENGINE_URL};

/// Application routes - each URL maps to a view
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    MainMenuRoute {},

    #[route("/roles")]
    RoleSelectRoute {},

    #[route("/worlds")]
    WorldSelectRoute {},

    #[route("/worlds/:world_id/dm")]
    DMViewRoute { world_id: String },

    #[route("/worlds/:world_id/play")]
    PCViewRoute { world_id: String },

    #[route("/worlds/:world_id/watch")]
    SpectatorViewRoute { world_id: String },

    #[route("/:..route")]
    NotFoundRoute { route: Vec<String> },
}

/// Route components - each route has a component that wraps the actual view

#[component]
pub fn MainMenuRoute() -> Element {
    let navigator = use_navigator();

    rsx! {
        crate::presentation::views::main_menu::MainMenu {
            on_connect: move |_server_url: String| {
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

#[component]
pub fn RoleSelectRoute() -> Element {
    let navigator = use_navigator();

    rsx! {
        crate::presentation::views::role_select::RoleSelect {
            on_select_role: move |role: crate::UserRole| {
                // Store the selected role in a context or signal for use in WorldSelect
                // For now, we'll navigate and let WorldSelect handle the role
                navigator.push(Route::WorldSelectRoute {});
            }
        }
    }
}

#[component]
pub fn WorldSelectRoute() -> Element {
    let navigator = use_navigator();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    // Get selected role from context (TODO: implement context storage)
    let role = crate::UserRole::Player;

    rsx! {
        crate::presentation::views::world_select::WorldSelectView {
            role: role,
            on_world_selected: move |world_id: String| {
                // Initiate connection to the Engine
                let server_url = DEFAULT_ENGINE_URL.to_string();
                let user_id = format!("user-{}", uuid::Uuid::new_v4());
                let participant_role = ParticipantRole::Player;

                initiate_connection(
                    server_url,
                    user_id,
                    participant_role,
                    session_state.clone(),
                    game_state.clone(),
                    dialogue_state.clone(),
                );

                // Navigate to the appropriate view based on role
                navigator.push(Route::PCViewRoute { world_id });
            },
            on_back: move |_| {
                navigator.push(Route::RoleSelectRoute {});
            },
        }
    }
}

#[component]
pub fn PCViewRoute(world_id: String) -> Element {
    let navigator = use_navigator();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    rsx! {
        crate::presentation::views::pc_view::PCView {
            on_back: move |_| {
                handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

#[component]
pub fn DMViewRoute(world_id: String) -> Element {
    let navigator = use_navigator();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    // Track DM mode state
    let mut dm_mode = use_signal(|| DMMode::Director);

    // Build the layout with header and DM view
    let current_dm_mode = *dm_mode.read();

    rsx! {
        div {
            class: "app-container",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; background: #0f0f23;",

            // Header with DM tabs
            DMViewHeader {
                connection_status: *session_state.connection_status.read(),
                dm_mode: current_dm_mode,
                on_dm_mode_change: move |mode: DMMode| {
                    dm_mode.set(mode);
                },
                on_back: {
                    let session_state = session_state.clone();
                    let game_state = game_state.clone();
                    let dialogue_state = dialogue_state.clone();
                    move |_| {
                        handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                        navigator.push(Route::RoleSelectRoute {});
                    }
                },
            }

            // Main content
            main {
                style: "flex: 1; overflow: hidden; position: relative; z-index: 1;",

                crate::presentation::views::dm_view::DMView {
                    active_mode: current_dm_mode,
                }
            }

            // Error overlay
            if let Some(error) = session_state.error_message.read().as_ref() {
                ErrorOverlay {
                    message: error.clone(),
                    on_dismiss: move |_| {
                        session_state.error_message.clone().set(None);
                    }
                }
            }
        }
    }
}

#[component]
pub fn SpectatorViewRoute(world_id: String) -> Element {
    let navigator = use_navigator();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    rsx! {
        crate::presentation::views::spectator_view::SpectatorView {
            on_back: move |_| {
                handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

#[component]
pub fn NotFoundRoute(route: Vec<String>) -> Element {
    let navigator = use_navigator();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: white; background: #0f0f23;",

            h1 {
                style: "font-size: 4rem; color: #ef4444; margin: 0;",
                "404"
            }
            p {
                style: "color: #9ca3af; margin: 1rem 0 2rem 0;",
                "Page not found: /{route.join(\"/\")}"
            }

            button {
                onclick: move |_| {
                    navigator.push(Route::MainMenuRoute {});
                },
                style: "padding: 0.75rem 1.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; text-decoration: none; cursor: pointer; font-size: 1rem;",
                "← Back to Main Menu"
            }
        }
    }
}

/// Header component for DM View (separate from main App header)
#[derive(Props, Clone, PartialEq)]
struct DMViewHeaderProps {
    connection_status: ConnectionStatus,
    dm_mode: DMMode,
    on_dm_mode_change: EventHandler<DMMode>,
    on_back: EventHandler<()>,
}

#[component]
fn DMViewHeader(props: DMViewHeaderProps) -> Element {
    let indicator_color = props.connection_status.indicator_color();
    let status_text = props.connection_status.display_text();

    rsx! {
        header {
            class: "app-header",
            style: "padding: 0.75rem 1rem; background: #1a1a2e; color: white; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #2d2d44; position: relative; z-index: 100;",

            // Left side: title and DM tabs
            div {
                style: "display: flex; align-items: center; gap: 1.5rem; position: relative; z-index: 101;",

                h1 {
                    style: "margin: 0; font-size: 1.25rem; font-family: 'Cinzel', serif; color: #d4af37;",
                    "WrldBldr"
                }

                // DM tabs
                div {
                    style: "display: flex; gap: 0.25rem; position: relative; z-index: 102;",

                    DMHeaderTab {
                        label: "Director",
                        active: props.dm_mode == DMMode::Director,
                        on_click: move |_| {
                            props.on_dm_mode_change.call(DMMode::Director);
                        },
                    }
                    DMHeaderTab {
                        label: "Creator",
                        active: props.dm_mode == DMMode::Creator,
                        on_click: move |_| {
                            props.on_dm_mode_change.call(DMMode::Creator);
                        },
                    }
                    DMHeaderTab {
                        label: "Settings",
                        active: props.dm_mode == DMMode::Settings,
                        on_click: move |_| {
                            props.on_dm_mode_change.call(DMMode::Settings);
                        },
                    }
                }
            }

            // Right side: back button and connection status
            div {
                style: "display: flex; align-items: center; gap: 1rem;",

                button {
                    onclick: move |e| {
                        e.stop_propagation();
                        props.on_back.call(());
                    },
                    style: "padding: 0.4rem 0.75rem; background: transparent; color: #9ca3af; border: 1px solid #374151; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; transition: all 0.15s;",
                    "← Back"
                }

                // Connection status
                div {
                    class: "connection-status",
                    style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem;",

                    span {
                        class: "status-indicator",
                        style: "width: 8px; height: 8px; border-radius: 50%; background: {indicator_color};",
                    }
                    span {
                        style: "color: #9ca3af;",
                        "{status_text}"
                    }
                }
            }
        }
    }
}

/// Header tab button for DM View
#[component]
fn DMHeaderTab(label: &'static str, active: bool, on_click: EventHandler<()>) -> Element {
    let bg_color = if active { "#3b82f6" } else { "transparent" };
    let text_color = if active { "white" } else { "#9ca3af" };

    rsx! {
        button {
            onclick: move |e| {
                e.stop_propagation();
                on_click.call(());
            },
            style: format!(
                "padding: 0.4rem 0.75rem; background: {}; color: {}; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: {}; transition: all 0.15s; position: relative; z-index: 103; pointer-events: auto;",
                bg_color,
                text_color,
                if active { "500" } else { "400" }
            ),
            "{label}"
        }
    }
}

/// Error overlay component
#[derive(Props, Clone, PartialEq)]
struct ErrorOverlayProps {
    message: String,
    on_dismiss: EventHandler<()>,
}

#[component]
fn ErrorOverlay(props: ErrorOverlayProps) -> Element {
    rsx! {
        div {
            class: "error-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.75); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| props.on_dismiss.call(()),

            div {
                class: "error-card",
                style: "background: #1f2937; border: 1px solid #ef4444; border-radius: 0.5rem; padding: 1.5rem; max-width: 400px; margin: 1rem;",
                onclick: move |e| e.stop_propagation(),

                h3 {
                    style: "color: #ef4444; margin: 0 0 0.5rem 0; font-size: 1.125rem;",
                    "Connection Error"
                }
                p {
                    style: "color: #d1d5db; margin: 0 0 1rem 0; font-size: 0.875rem;",
                    "{props.message}"
                }
                button {
                    onclick: move |_| props.on_dismiss.call(()),
                    style: "background: #374151; color: white; border: none; padding: 0.5rem 1rem; border-radius: 0.375rem; cursor: pointer;",
                    "Dismiss"
                }
            }
        }
    }
}

/// Initiate WebSocket connection (platform-specific)
#[cfg(target_arch = "wasm32")]
fn initiate_connection(
    server_url: String,
    user_id: String,
    role: ParticipantRole,
    session_state: SessionState,
    game_state: GameState,
    dialogue_state: DialogueState,
) {
    let session_service = SessionService::new(&server_url);

    if let Err(e) = session_service.connect_and_join(
        user_id,
        role,
        session_state.clone(),
        game_state,
        dialogue_state,
    ) {
        web_sys::console::error_1(&format!("Connection failed: {}", e).into());
        session_state.clone().set_failed(e.to_string());
    }
}

/// Initiate WebSocket connection (desktop version using channels)
#[cfg(not(target_arch = "wasm32"))]
fn initiate_connection(
    server_url: String,
    user_id: String,
    role: ParticipantRole,
    mut session_state: SessionState,
    mut game_state: GameState,
    mut dialogue_state: DialogueState,
) {
    use crate::application::services::handle_session_event;
    use dioxus::prelude::*;

    // Update session state to connecting
    session_state.start_connecting(&server_url);
    session_state.set_user(user_id.clone(), role);

    // Spawn async task to handle connection
    spawn(async move {
        let session_service = SessionService::new(&server_url);

        match session_service.connect(user_id, role).await {
            Ok(mut rx) => {
                // Store client reference
                session_state.set_connected(std::sync::Arc::clone(session_service.client()));

                // Process events from the channel
                while let Some(event) = rx.recv().await {
                    handle_session_event(
                        event,
                        &mut session_state,
                        &mut game_state,
                        &mut dialogue_state,
                    );
                }

                tracing::info!("Event channel closed");
            }
            Err(e) => {
                tracing::error!("Connection failed: {}", e);
                session_state.set_failed(e.to_string());
            }
        }
    });
}

/// Handle disconnection and cleanup
fn handle_disconnect(
    mut session_state: SessionState,
    mut game_state: GameState,
    mut dialogue_state: DialogueState,
) {
    // Disconnect client if present
    if let Some(client) = session_state.engine_client.read().as_ref() {
        #[cfg(target_arch = "wasm32")]
        {
            // WASM client disconnect is synchronous
            let _ = client;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = std::sync::Arc::clone(client);
            spawn(async move {
                client.disconnect().await;
            });
        }
    }

    // Clear all state
    session_state.clear();
    game_state.clear();
    dialogue_state.clear();
}
