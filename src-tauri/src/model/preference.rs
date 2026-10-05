use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    Save,
    CycleMode,
    ToggleRelocation,
    PreviousUnit,
    NextUnit,
    PreviousPage,
    NextPage,
    #[serde(alias = "cycle_marker_opacity")]
    ToggleProofreadPreview,
    InsertRecentSymbol,
    InsertFavoriteOne,
    InsertFavoriteTwo,
    InsertFavoriteThree,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutScope {
    Editor,
    CanvasNavigation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShortcutBinding {
    Unbound,
    Chord {
        key: String,
        control: bool,
        meta: bool,
        alt: bool,
        shift: bool,
        scope: ShortcutScope,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct Shortcut {
    pub action: ShortcutAction,
    pub binding: ShortcutBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct SpecialCharacter {
    pub id: String,
    pub text: String,
    pub is_favorite: bool,
}

fn shortcut(action: ShortcutAction, key: &str, control: bool, alt: bool, shift: bool) -> Shortcut {
    Shortcut {
        action,
        binding: ShortcutBinding::Chord {
            key: key.to_owned(),
            control,
            meta: false,
            alt,
            shift,
            scope: ShortcutScope::Editor,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ApplicationPreference {
    pub shortcut: Vec<Shortcut>,
    pub special_character: Vec<SpecialCharacter>,
    pub relocation_enabled: bool,
    pub marker_opacity: f64,
}

impl Default for ApplicationPreference {
    fn default() -> Self {
        let bindings = vec![
            shortcut(ShortcutAction::CycleMode, "KeyM", true, false, false),
            shortcut(ShortcutAction::ToggleRelocation, "KeyL", true, false, false),
            shortcut(
                ShortcutAction::ToggleProofreadPreview,
                "KeyX",
                true,
                false,
                false,
            ),
            shortcut(ShortcutAction::NextUnit, "Tab", false, false, false),
            shortcut(ShortcutAction::PreviousUnit, "Tab", false, false, true),
            shortcut(ShortcutAction::PreviousPage, "KeyU", true, false, false),
            shortcut(ShortcutAction::NextPage, "KeyD", true, false, false),
            shortcut(
                ShortcutAction::InsertRecentSymbol,
                "KeyQ",
                true,
                false,
                false,
            ),
            shortcut(
                ShortcutAction::InsertFavoriteOne,
                "Digit1",
                false,
                true,
                false,
            ),
            shortcut(
                ShortcutAction::InsertFavoriteTwo,
                "Digit2",
                false,
                true,
                false,
            ),
            shortcut(
                ShortcutAction::InsertFavoriteThree,
                "Digit3",
                false,
                true,
                false,
            ),
            shortcut(ShortcutAction::Save, "KeyS", true, false, false),
        ];

        let special_character = ["♪", "「」", "『』", "❤", "●", "★", "☆", "♡", "○", "※"]
            .into_iter()
            .enumerate()
            .map(|(index, text)| SpecialCharacter {
                id: (index + 1).to_string(),
                text: text.to_owned(),
                is_favorite: true,
            })
            .collect();

        Self {
            shortcut: bindings,
            special_character,
            relocation_enabled: false,
            marker_opacity: 1.0,
        }
    }
}
