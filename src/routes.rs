//! Application routing - URL-based navigation for all views
//!
//! This module handles all routing for the WrldBldr Player application.
//!
//! ## Browser History & Deep Linking
//!
//! The application uses Dioxus Router which provides automatic browser history support:
//! - URL updates automatically on navigation (browser address bar updates)
//! - Back/forward buttons work correctly (router handles state restoration)
//! - Deep links work: users can share or bookmark direct URLs to specific views
//! - Missing state redirects: if a user navigates directly to a view without required context,
//!   the application redirects to the appropriate setup step (MainMenu → RoleSelect → WorldSelect)
//!
//! ## Page Titles
//!
//! Each route component sets a dynamic page title visible in the browser tab.
//! This helps users distinguish between different views when multiple tabs are open.
//!
//! ## localStorage Persistence
//!
//! On web platforms (WASM), the application persists user preferences:
//! - Server URL: Remembers the last connected server
//! - Selected Role: Saves the player's role choice
//! - Last World: Remembers the last world they accessed
//!
//! These are loaded on application startup and saved when changed.

use dioxus::prelude::*;
use crate::presentation::state::{ConnectionStatus, DialogueState, GameState, GenerationState, SessionState};
use crate::presentation::views::dm_view::DMMode;
// Use port type for ParticipantRole instead of infrastructure type
use crate::application::services::{ParticipantRolePort as ParticipantRole, DEFAULT_ENGINE_URL};
#[cfg(not(target_arch = "wasm32"))]
use crate::application::services::SessionService;
use crate::application::ports::outbound::{Platform, storage_keys};

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

    // DM view with tab parameter - defaults to "director"
    #[route("/worlds/:world_id/dm")]
    DMViewRoute { world_id: String },

    #[route("/worlds/:world_id/dm/:tab")]
    DMViewTabRoute { world_id: String, tab: String },

    // Creator mode sub-tabs
    #[route("/worlds/:world_id/dm/creator/:subtab")]
    DMCreatorSubTabRoute { world_id: String, subtab: String },

    // Settings sub-tabs
    #[route("/worlds/:world_id/dm/settings/:subtab")]
    DMSettingsSubTabRoute { world_id: String, subtab: String },

    // Story Arc sub-tabs
    #[route("/worlds/:world_id/dm/story-arc/:subtab")]
    DMStoryArcSubTabRoute { world_id: String, subtab: String },

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
    let platform = use_context::<Platform>();

    // Clone platform for different closures
    let platform_title = platform.clone();
    let platform_storage = platform.clone();

    // Set page title for this view
    use_effect(move || {
        platform_title.set_page_title("Main Menu");
    });

    rsx! {
        crate::presentation::views::main_menu::MainMenu {
            on_connect: move |server_url: String| {
                // Save server URL preference
                platform_storage.storage_save(storage_keys::SERVER_URL, &server_url);
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

#[component]
pub fn RoleSelectRoute() -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();

    // Clone platform for different closures
    let platform_title = platform.clone();
    let platform_storage = platform.clone();

    // Set page title for this view
    use_effect(move || {
        platform_title.set_page_title("Select Role");
    });

    rsx! {
        crate::presentation::views::role_select::RoleSelect {
            on_select_role: move |role: crate::UserRole| {
                // Save selected role preference
                let role_str = format!("{:?}", role);
                platform_storage.storage_save(storage_keys::ROLE, &role_str);
                navigator.push(Route::WorldSelectRoute {});
            }
        }
    }
}

#[component]
pub fn WorldSelectRoute() -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    // Clone platform for different closures
    let platform_title = platform.clone();
    let platform_handler = platform.clone();

    // Set page title for this view
    use_effect(move || {
        platform_title.set_page_title("Select World");
    });

    // Load selected role from localStorage
    let role = load_role_from_storage(&platform);

    rsx! {
        crate::presentation::views::world_select::WorldSelectView {
            role: role,
            on_world_selected: {
                let role = role;
                let platform_connection = platform_handler.clone();
                move |world_id: String| {
                    // Save last accessed world
                    platform_connection.storage_save(storage_keys::LAST_WORLD, &world_id);

                    // Map UserRole to ParticipantRole
                    let participant_role = match role {
                        crate::UserRole::DungeonMaster => ParticipantRole::DungeonMaster,
                        crate::UserRole::Player => ParticipantRole::Player,
                        crate::UserRole::Spectator => ParticipantRole::Spectator,
                    };

                    // Initiate connection to the Engine
                    let server_url = DEFAULT_ENGINE_URL.to_string();
                    let user_id = format!("user-{}", uuid::Uuid::new_v4());

                    initiate_connection(
                        server_url,
                        user_id,
                        participant_role,
                        session_state.clone(),
                        game_state.clone(),
                        dialogue_state.clone(),
                        platform_connection.clone(),
                    );

                    // Navigate to the appropriate view based on role
                    match role {
                        crate::UserRole::DungeonMaster => {
                            navigator.push(Route::DMViewRoute { world_id });
                        }
                        crate::UserRole::Player => {
                            navigator.push(Route::PCViewRoute { world_id });
                        }
                        crate::UserRole::Spectator => {
                            navigator.push(Route::SpectatorViewRoute { world_id });
                        }
                    }
                }
            },
            on_back: move |_| {
                navigator.push(Route::RoleSelectRoute {});
            },
        }
    }
}

/// Load user role from localStorage, defaults to Player
fn load_role_from_storage(platform: &Platform) -> crate::UserRole {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(role_str) = platform.storage_load(storage_keys::ROLE) {
            match role_str.as_str() {
                "DungeonMaster" => return crate::UserRole::DungeonMaster,
                "Player" => return crate::UserRole::Player,
                "Spectator" => return crate::UserRole::Spectator,
                _ => {}
            }
        }
    }
    crate::UserRole::Player
}

#[component]
pub fn PCViewRoute(world_id: String) -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    // Clone platform for different closures
    let platform_title = platform.clone();
    let platform_storage = platform.clone();

    // Set page title for this view
    use_effect(move || {
        platform_title.set_page_title("Playing");
    });

    rsx! {
        crate::presentation::views::pc_view::PCView {
            on_back: move |_| {
                handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                // Clear world preference when disconnecting
                platform_storage.storage_remove(storage_keys::LAST_WORLD);
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

/// DMViewRoute - renders Director tab directly (no redirect needed)
#[component]
pub fn DMViewRoute(world_id: String) -> Element {
    let platform = use_context::<Platform>();

    // Set page title
    use_effect(move || {
        platform.set_page_title("Director");
    });

    // Render Director mode directly instead of redirecting
    rsx! {
        DMViewLayout {
            world_id: world_id,
            dm_mode: DMMode::Director,
            creator_subtab: None,
            settings_subtab: None,
            story_arc_subtab: None,
        }
    }
}

/// DMViewTabRoute - DM view with specific tab
/// For tabs with subtabs (creator, settings, story-arc), render with default subtab
/// This avoids use_effect redirect race conditions
#[component]
pub fn DMViewTabRoute(world_id: String, tab: String) -> Element {
    let platform = use_context::<Platform>();

    // Determine mode and default subtab based on tab parameter
    let (dm_mode, creator_subtab, settings_subtab, story_arc_subtab, title) = match tab.as_str() {
        "director" => (DMMode::Director, None, None, None, "Director"),
        "creator" => (DMMode::Creator, Some("characters".to_string()), None, None, "Creator - Characters"),
        "settings" => (DMMode::Settings, None, Some("workflows".to_string()), None, "Settings - Workflows"),
        "story-arc" => (DMMode::StoryArc, None, None, Some("timeline".to_string()), "Story Arc - Timeline"),
        _ => (DMMode::Director, None, None, None, "Director"),
    };

    use_effect(move || {
        platform.set_page_title(title);
    });

    rsx! {
        DMViewLayout {
            world_id: world_id,
            dm_mode: dm_mode,
            creator_subtab: creator_subtab,
            settings_subtab: settings_subtab,
            story_arc_subtab: story_arc_subtab,
        }
    }
}

/// DMCreatorSubTabRoute - Creator mode with specific sub-tab
#[component]
pub fn DMCreatorSubTabRoute(world_id: String, subtab: String) -> Element {
    let platform = use_context::<Platform>();

    // Set page title based on subtab
    let title = match subtab.as_str() {
        "characters" => "Creator - Characters",
        "locations" => "Creator - Locations",
        "items" => "Creator - Items",
        "maps" => "Creator - Maps",
        _ => "Creator",
    };

    use_effect(move || {
        platform.set_page_title(title);
    });

    rsx! {
        DMViewLayout {
            world_id: world_id,
            dm_mode: DMMode::Creator,
            creator_subtab: Some(subtab),
            settings_subtab: None,
            story_arc_subtab: None,
        }
    }
}

/// DMSettingsSubTabRoute - Settings with specific sub-tab
#[component]
pub fn DMSettingsSubTabRoute(world_id: String, subtab: String) -> Element {
    let platform = use_context::<Platform>();

    // Set page title based on subtab
    let title = match subtab.as_str() {
        "workflows" => "Settings - Workflows",
        "skills" => "Settings - Skills",
        _ => "Settings",
    };

    use_effect(move || {
        platform.set_page_title(title);
    });

    rsx! {
        DMViewLayout {
            world_id: world_id,
            dm_mode: DMMode::Settings,
            creator_subtab: None,
            settings_subtab: Some(subtab),
            story_arc_subtab: None,
        }
    }
}

/// DMStoryArcSubTabRoute - Story Arc with specific sub-tab
#[component]
pub fn DMStoryArcSubTabRoute(world_id: String, subtab: String) -> Element {
    let platform = use_context::<Platform>();

    // Set page title based on subtab
    let title = match subtab.as_str() {
        "timeline" => "Story Arc - Timeline",
        "events" => "Story Arc - Narrative Events",
        "chains" => "Story Arc - Event Chains",
        _ => "Story Arc",
    };

    use_effect(move || {
        platform.set_page_title(title);
    });

    rsx! {
        DMViewLayout {
            world_id: world_id,
            dm_mode: DMMode::StoryArc,
            creator_subtab: None,
            settings_subtab: None,
            story_arc_subtab: Some(subtab),
        }
    }
}

/// Shared DM View layout component
#[derive(Props, Clone, PartialEq)]
struct DMViewLayoutProps {
    world_id: String,
    dm_mode: DMMode,
    creator_subtab: Option<String>,
    settings_subtab: Option<String>,
    story_arc_subtab: Option<String>,
}

#[component]
fn DMViewLayout(props: DMViewLayoutProps) -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    rsx! {
        div {
            class: "app-container",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; background: #0f0f23;",

            // Header with DM tabs
            DMViewHeader {
                world_id: props.world_id.clone(),
                connection_status: *session_state.connection_status.read(),
                dm_mode: props.dm_mode,
                on_back: {
                    let session_state = session_state.clone();
                    let game_state = game_state.clone();
                    let dialogue_state = dialogue_state.clone();
                    move |_| {
                        handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                        // Clear world preference when disconnecting
                        platform.storage_remove(storage_keys::LAST_WORLD);
                        navigator.push(Route::RoleSelectRoute {});
                    }
                },
            }

            // Main content
            main {
                style: "flex: 1; overflow: hidden; position: relative; z-index: 1;",

                crate::presentation::views::dm_view::DMView {
                    world_id: props.world_id.clone(),
                    active_mode: props.dm_mode,
                    creator_subtab: props.creator_subtab.clone(),
                    settings_subtab: props.settings_subtab.clone(),
                    story_arc_subtab: props.story_arc_subtab.clone(),
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
    let platform = use_context::<Platform>();
    let session_state = use_context::<SessionState>();
    let game_state = use_context::<GameState>();
    let dialogue_state = use_context::<DialogueState>();

    // Clone platform for different closures
    let platform_title = platform.clone();
    let platform_storage = platform.clone();

    // Set page title for this view
    use_effect(move || {
        platform_title.set_page_title("Watching");
    });

    rsx! {
        crate::presentation::views::spectator_view::SpectatorView {
            on_back: move |_| {
                handle_disconnect(session_state.clone(), game_state.clone(), dialogue_state.clone());
                // Clear world preference when disconnecting
                platform_storage.storage_remove(storage_keys::LAST_WORLD);
                navigator.push(Route::RoleSelectRoute {});
            }
        }
    }
}

#[component]
pub fn NotFoundRoute(route: Vec<String>) -> Element {
    let navigator = use_navigator();
    let platform = use_context::<Platform>();

    // Set page title for this view
    use_effect(move || {
        platform.set_page_title("Page Not Found");
    });

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
    world_id: String,
    connection_status: ConnectionStatus,
    dm_mode: DMMode,
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

                // DM tabs - use router Links for navigation
                div {
                    style: "display: flex; gap: 0.25rem; position: relative; z-index: 102;",

                    DMHeaderTabLink {
                        label: "Director",
                        tab: "director",
                        world_id: props.world_id.clone(),
                        active: props.dm_mode == DMMode::Director,
                    }
                    DMHeaderTabLink {
                        label: "Creator",
                        tab: "creator",
                        world_id: props.world_id.clone(),
                        active: props.dm_mode == DMMode::Creator,
                    }
                    DMHeaderTabLink {
                        label: "Story Arc",
                        tab: "story-arc",
                        world_id: props.world_id.clone(),
                        active: props.dm_mode == DMMode::StoryArc,
                    }
                    DMHeaderTabLink {
                        label: "Settings",
                        tab: "settings",
                        world_id: props.world_id.clone(),
                        active: props.dm_mode == DMMode::Settings,
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

/// Header tab link for DM View - uses router navigation
/// Links directly to the appropriate subtab route to avoid redirect race conditions
#[component]
fn DMHeaderTabLink(label: &'static str, tab: &'static str, world_id: String, active: bool) -> Element {
    let bg_color = if active { "#3b82f6" } else { "transparent" };
    let text_color = if active { "white" } else { "#9ca3af" };

    // Determine the correct route based on tab - link directly to subtab routes
    // to avoid use_effect redirect race conditions
    let route = match tab {
        "director" => Route::DMViewTabRoute {
            world_id: world_id.clone(),
            tab: "director".to_string(),
        },
        "creator" => Route::DMCreatorSubTabRoute {
            world_id: world_id.clone(),
            subtab: "characters".to_string(),
        },
        "story-arc" => Route::DMStoryArcSubTabRoute {
            world_id: world_id.clone(),
            subtab: "timeline".to_string(),
        },
        "settings" => Route::DMSettingsSubTabRoute {
            world_id: world_id.clone(),
            subtab: "workflows".to_string(),
        },
        _ => Route::DMViewTabRoute {
            world_id: world_id.clone(),
            tab: tab.to_string(),
        },
    };

    rsx! {
        Link {
            to: route,
            style: format!(
                "padding: 0.4rem 0.75rem; background: {}; color: {}; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: {}; transition: all 0.15s; position: relative; z-index: 103; pointer-events: auto; text-decoration: none;",
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
    platform: Platform,
) {
    // Update session state to connecting
    let mut session_state = session_state.clone();
    session_state.start_connecting(&server_url);
    session_state.set_user(user_id.clone(), role);

    let mut session_state_for_message = session_state.clone();
    let mut game_state = game_state;
    let mut dialogue_state = dialogue_state;
    let platform_for_handler = platform.clone();

    // Forward incoming server messages into the presentation handler.
    let on_message = Box::new(move |message| {
        crate::presentation::handlers::handle_server_message(
            message,
            &mut session_state_for_message,
            &mut game_state,
            &mut dialogue_state,
            &platform_for_handler,
        );
    });

    // Keep connection status in sync; send JoinSession once connected.
    let mut session_state_for_state = session_state.clone();
    let user_id_for_join = user_id.clone();
    let on_state_change = Box::new(move |state| {
        let status = match state {
            crate::infrastructure::websocket::ConnectionState::Disconnected => ConnectionStatus::Disconnected,
            crate::infrastructure::websocket::ConnectionState::Connecting => ConnectionStatus::Connecting,
            crate::infrastructure::websocket::ConnectionState::Connected => ConnectionStatus::Connected,
            crate::infrastructure::websocket::ConnectionState::Reconnecting => ConnectionStatus::Reconnecting,
            crate::infrastructure::websocket::ConnectionState::Failed => ConnectionStatus::Failed,
        };
        session_state_for_state.connection_status.set(status);

        if matches!(state, crate::infrastructure::websocket::ConnectionState::Connected) {
            // Send JoinSession when connected (WASM)
            if let Some(client) = session_state_for_state.engine_client.read().as_ref() {
                if let Err(e) = client.join_session(&user_id_for_join, role) {
                    web_sys::console::error_1(&format!("Failed to send JoinSession: {}", e).into());
                }
            }
        }

        if matches!(
            state,
            crate::infrastructure::websocket::ConnectionState::Disconnected
                | crate::infrastructure::websocket::ConnectionState::Failed
        ) {
            session_state_for_state.engine_client.set(None);
        }
    });

    // WASM uses the infrastructure EngineClient directly for connection establishment.
    // This is acceptable at the composition boundary; message handling remains in presentation.
    let client = crate::infrastructure::websocket::EngineClient::new(&server_url);
    client.set_on_state_change(on_state_change);
    client.set_on_message(on_message);
    // Store a send-capable connection handle for UI commands.
    let connection = crate::infrastructure::websocket::EngineGameConnection::new(client.clone());
    session_state.set_connection_handle(std::sync::Arc::new(connection));

    if let Err(e) = client.connect() {
        web_sys::console::error_1(&format!("Connection failed: {}", e).into());
        session_state.set_failed(e.to_string());
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
    platform: Platform,
) {
    use crate::presentation::handlers::handle_session_event;
    use dioxus::prelude::*;

    // Update session state to connecting
    session_state.start_connecting(&server_url);
    session_state.set_user(user_id.clone(), role);

    // Spawn async task to handle connection
    spawn(async move {
        // Build a concrete connection (infrastructure) and pass it to the application service.
        let client = crate::infrastructure::websocket::EngineClient::new(&server_url);
        let connection = std::sync::Arc::new(crate::infrastructure::websocket::EngineGameConnection::new(client));
        session_state.set_connection_handle(connection.clone());
        let session_service = SessionService::new(connection.clone());

        match session_service.connect(user_id, role).await {
            Ok(mut rx) => {
                // Process events from the channel
                while let Some(event) = rx.recv().await {
                    handle_session_event(
                        event,
                        &mut session_state,
                        &mut game_state,
                        &mut dialogue_state,
                        &platform,
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
        client.disconnect();
    }

    // Clear all state
    session_state.clear();
    game_state.clear();
    dialogue_state.clear();
}
