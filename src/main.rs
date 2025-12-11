//! WrldBldr Player - TTRPG gameplay client
//!
//! The Player app provides two views:
//! - PC View: Visual novel gameplay experience for players
//! - DM View: Directorial control panel for running the game

mod application;
mod domain;
mod infrastructure;
mod presentation;

use dioxus::prelude::*;
use presentation::state::{ConnectionStatus, DialogueState, GameState, SessionState};

use application::services::{SessionService, DEFAULT_ENGINE_URL};
use infrastructure::websocket::ParticipantRole;

#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging (desktop only - WASM uses tracing-wasm)
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wrldbldr_player=debug,dioxus=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }

    tracing::info!("Starting WrldBldr Player");

    // Launch the Dioxus application
    dioxus::launch(App);
}

/// Root application component with state providers
#[component]
fn App() -> Element {
    // Provide global state via context
    use_context_provider(GameState::new);
    use_context_provider(SessionState::new);
    use_context_provider(DialogueState::new);

    // Local UI state
    let mut current_view = use_signal(|| AppView::MainMenu);
    let mut pending_server_url = use_signal(|| None::<String>);

    // Get state from context
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();
    let connection_status = session_state.connection_status;
    let error_message = session_state.error_message;

    rsx! {
        div {
            class: "app-container",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; background: #0f0f23;",

            // Header with connection status
            AppHeader {
                connection_status: *connection_status.read(),
            }

            // Main content
            main {
                style: "flex: 1; overflow: hidden; position: relative;",

                match *current_view.read() {
                    AppView::MainMenu => rsx! {
                        presentation::views::main_menu::MainMenu {
                            on_connect: move |server_url: String| {
                                // Store the server URL and go to role select
                                pending_server_url.set(Some(server_url));
                                current_view.set(AppView::RoleSelect);
                            }
                        }
                    },

                    AppView::RoleSelect => {
                        let session_state = session_state.clone();
                        let game_state = game_state.clone();
                        let dialogue_state = dialogue_state.clone();
                        rsx! {
                            presentation::views::role_select::RoleSelect {
                                on_select_role: move |role: UserRole| {
                                    // Connect to Engine with selected role
                                    let server_url = pending_server_url.read().clone()
                                        .unwrap_or_else(|| DEFAULT_ENGINE_URL.to_string());

                                    let participant_role = match role {
                                        UserRole::DungeonMaster => ParticipantRole::DungeonMaster,
                                        UserRole::Player => ParticipantRole::Player,
                                        UserRole::Spectator => ParticipantRole::Spectator,
                                    };

                                    // Generate a user ID (in production, this would come from auth)
                                    let user_id = format!("user-{}", uuid::Uuid::new_v4());

                                    // Initiate connection
                                    initiate_connection(
                                        server_url,
                                        user_id,
                                        participant_role,
                                        session_state.clone(),
                                        game_state.clone(),
                                        dialogue_state.clone(),
                                    );

                                    // Navigate to the appropriate view
                                    match role {
                                        UserRole::DungeonMaster => current_view.set(AppView::DungeonMasterView),
                                        UserRole::Player => current_view.set(AppView::PlayerView),
                                        UserRole::Spectator => current_view.set(AppView::SpectatorView),
                                    }
                                }
                            }
                        }
                    },

                    AppView::PlayerView => {
                        let session_state = session_state.clone();
                        let game_state = game_state.clone();
                        let dialogue_state = dialogue_state.clone();
                        rsx! {
                            presentation::views::pc_view::PCView {
                                on_back: move |_| {
                                    // Disconnect and go back
                                    handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                                    current_view.set(AppView::RoleSelect);
                                }
                            }
                        }
                    },

                    AppView::DungeonMasterView => {
                        let session_state = session_state.clone();
                        let game_state = game_state.clone();
                        let dialogue_state = dialogue_state.clone();
                        rsx! {
                            presentation::views::dm_view::DMView {
                                on_back: move |_| {
                                    handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                                    current_view.set(AppView::RoleSelect);
                                }
                            }
                        }
                    },

                    AppView::SpectatorView => {
                        let session_state = session_state.clone();
                        let game_state = game_state.clone();
                        let dialogue_state = dialogue_state.clone();
                        rsx! {
                            presentation::views::spectator_view::SpectatorView {
                                on_back: move |_| {
                                    handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                                    current_view.set(AppView::RoleSelect);
                                }
                            }
                        }
                    },
                }
            }

            // Error overlay
            if let Some(error) = error_message.read().as_ref() {
                ErrorOverlay {
                    message: error.clone(),
                    on_dismiss: move |_| {
                        error_message.clone().set(None);
                    }
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
    use application::services::handle_session_event;

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

/// Application header with title and connection status
#[derive(Props, Clone, PartialEq)]
struct AppHeaderProps {
    connection_status: ConnectionStatus,
}

#[component]
fn AppHeader(props: AppHeaderProps) -> Element {
    let indicator_color = props.connection_status.indicator_color();
    let status_text = props.connection_status.display_text();

    rsx! {
        header {
            class: "app-header",
            style: "padding: 0.75rem 1rem; background: #1a1a2e; color: white; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #2d2d44;",

            h1 {
                style: "margin: 0; font-size: 1.25rem; font-family: 'Cinzel', serif; color: #d4af37;",
                "WrldBldr"
            }

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

/// Current view in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppView {
    MainMenu,
    RoleSelect,
    PlayerView,
    DungeonMasterView,
    SpectatorView,
}

/// User role in the game session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    DungeonMaster,
    Player,
    Spectator,
}
