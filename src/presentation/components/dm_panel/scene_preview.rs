//! Scene preview component for DM view
//!
//! Shows a smaller version of what players see including backdrop and character sprites.

use dioxus::prelude::*;

/// Character data for scene preview
#[derive(Clone, PartialEq)]
pub struct CharacterData {
    /// Character ID
    pub id: String,
    /// Character name
    pub name: String,
    /// Character sprite image URL
    pub sprite_url: Option<String>,
    /// Position on stage (left, center, right)
    pub position: SpritePosition,
    /// Current emotion/expression
    pub emotion: String,
}

/// Position of a character sprite on stage
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpritePosition {
    Left,
    Center,
    Right,
}

impl SpritePosition {
    fn as_style(&self) -> &'static str {
        match self {
            SpritePosition::Left => "left: 10%; transform: translateX(0);",
            SpritePosition::Center => "left: 50%; transform: translateX(-50%);",
            SpritePosition::Right => "right: 10%; transform: translateX(0);",
        }
    }
}

/// Scene data for preview
#[derive(Clone, PartialEq)]
pub struct SceneData {
    /// Scene name/title
    pub name: String,
    /// Backdrop image URL
    pub backdrop_url: Option<String>,
    /// Current dialogue text being shown
    pub dialogue_text: String,
    /// Speaker of the dialogue
    pub speaker_name: String,
}

/// Props for the ScenePreview component
#[derive(Props, Clone, PartialEq)]
pub struct ScenePreviewProps {
    /// Optional scene data to display
    #[props(default)]
    pub scene: Option<SceneData>,
    /// Characters currently visible in the scene
    #[props(default)]
    pub characters: Vec<CharacterData>,
}

/// ScenePreview component - Shows smaller version of PC view
///
/// Displays a compact version of the current scene with backdrop and character sprites,
/// along with current dialogue. Useful for DMs to see what players are experiencing.
#[component]
pub fn ScenePreview(props: ScenePreviewProps) -> Element {
    let bg_style = match &props.scene {
        Some(scene) => match &scene.backdrop_url {
            Some(url) => format!(
                "background-image: url('{}'); background-size: cover; background-position: center;",
                url
            ),
            None => "background: linear-gradient(to bottom, #1a1a2e, #2d1b3d);".to_string(),
        },
        None => "background: linear-gradient(to bottom, #1a1a2e, #2d1b3d);".to_string(),
    };

    let has_dialogue = props
        .scene
        .as_ref()
        .map(|s| !s.dialogue_text.is_empty())
        .unwrap_or(false);

    rsx! {
        div {
            class: "scene-preview",
            style: "height: 100%; width: 100%; position: relative; overflow: hidden; border-radius: 0.5rem;",

            // Backdrop
            div {
                style: format!(
                    "position: absolute; inset: 0; {}",
                    bg_style
                ),
            }

            // Vignette effect
            div {
                style: "position: absolute; inset: 0; box-shadow: inset 0 0 100px rgba(0, 0, 0, 0.4); pointer-events: none;",
            }

            // Character sprites
            div {
                style: "position: absolute; bottom: 20%; width: 100%; display: flex; justify-content: space-around; align-items: flex-end; padding: 0 2rem;",

                for character in props.characters.iter() {
                    CharacterSpritePreview {
                        character: character.clone(),
                    }
                }
            }

            // Dialogue box overlay (small version)
            if has_dialogue {
                if let Some(scene) = &props.scene {
                    div {
                        style: "position: absolute; bottom: 0; left: 0; right: 0; background: linear-gradient(to top, #1a1a2e 0%, rgba(26, 26, 46, 0.8) 100%); padding: 1rem; border-top: 1px solid #374151;",

                        // Speaker name
                        if !scene.speaker_name.is_empty() {
                            div {
                                style: "color: #3b82f6; font-size: 0.875rem; font-weight: 600; margin-bottom: 0.25rem;",
                                "{scene.speaker_name}"
                            }
                        }

                        // Dialogue text
                        p {
                            style: "color: white; font-size: 0.875rem; line-height: 1.4; margin: 0;",
                            "{scene.dialogue_text}"
                        }
                    }
                }
            }

            // Empty state
            if props.scene.is_none() {
                div {
                    style: "position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; color: #6b7280; font-size: 0.875rem;",
                    "No scene loaded"
                }
            }
        }
    }
}

/// Character sprite preview component
#[component]
fn CharacterSpritePreview(character: CharacterData) -> Element {
    let sprite_content = match &character.sprite_url {
        Some(url) => rsx! {
            img {
                src: "{url}",
                alt: "{character.name}",
                style: "width: 100%; height: 100%; object-fit: contain;",
            }
        },
        None => rsx! {
            div {
                style: "width: 100%; height: 100%; background: rgba(59, 130, 246, 0.2); border-radius: 0.25rem; display: flex; align-items: center; justify-content: center; font-size: 0.75rem; color: #3b82f6;",
                "[{character.name}]"
            }
        },
    };

    rsx! {
        div {
            style: format!(
                "position: relative; width: 80px; height: 120px; {} display: flex; flex-direction: column; align-items: center;",
                character.position.as_style()
            ),

            // Sprite container
            div {
                style: "width: 100%; height: 100%; position: relative; overflow: visible;",
                {sprite_content}
            }

            // Name label
            div {
                style: "position: absolute; bottom: -1.5rem; left: 50%; transform: translateX(-50%); white-space: nowrap; font-size: 0.75rem; color: #9ca3af; background: rgba(0, 0, 0, 0.5); padding: 0.25rem 0.5rem; border-radius: 0.25rem;",
                "{character.name}"
            }

            // Emotion indicator
            if !character.emotion.is_empty() {
                div {
                    style: "position: absolute; top: -0.5rem; right: -0.5rem; background: #8b5cf6; color: white; font-size: 0.625rem; padding: 0.25rem 0.5rem; border-radius: 0.375rem; white-space: nowrap;",
                    "{character.emotion}"
                }
            }
        }
    }
}
