use crate::message::{ContextPage, EditMessage, Message, NavMessage, SettingsMessage, ViewMessage};
use cosmic::{
    iced::keyboard::{Key, key::Named},
    widget::menu::{
        Action,
        key_bind::{KeyBind, Modifier},
    },
};
use std::collections::HashMap;
use viewer_config::{SortMode, SortOrder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuAction {
    Open,
    OpenFolder,
    Close,
    Quit,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    Fullscreen,
    Next,
    Prev,
    First,
    Last,
    CloseModal,
    About,
    Settings,
    ImageInfo,
    FocusUp,
    FocusDown,
    SelectFocused,
    ToggleSlideshow,
    SetWallpaper,
    DeleteImage,
    Rotate90,
    Rotate180,
    FlipHorizontal,
    FlipVertical,
    StartCrop,
    Save,
    SaveAs,
    Undo,
    SortByName,
    SortByDate,
    SortBySize,
    SortAscending,
    SortDescending,
    OpenRecentFolder(usize),
    ClearRecentFolders,
}

impl MenuAction {
    pub fn message(self) -> Message {
        match self {
            MenuAction::Open => Message::OpenFileDialog,
            MenuAction::OpenFolder => Message::OpenFolderDialog,
            MenuAction::Close | MenuAction::Quit => Message::Quit,
            MenuAction::ZoomIn => Message::View(ViewMessage::ZoomIn),
            MenuAction::ZoomOut => Message::View(ViewMessage::ZoomOut),
            MenuAction::ZoomReset => Message::View(ViewMessage::ZoomReset),
            MenuAction::ZoomFit => Message::View(ViewMessage::ZoomFit),
            MenuAction::Fullscreen => Message::View(ViewMessage::ToggleFullScreen),
            MenuAction::Next => Message::Nav(NavMessage::Next),
            MenuAction::Prev => Message::Nav(NavMessage::Prev),
            MenuAction::First => Message::Nav(NavMessage::First),
            MenuAction::Last => Message::Nav(NavMessage::Last),
            MenuAction::CloseModal => Message::View(ViewMessage::CloseModal),
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::ImageInfo => Message::ToggleContextPage(ContextPage::ImageInfo),
            MenuAction::FocusUp => Message::View(ViewMessage::FocusUp),
            MenuAction::FocusDown => Message::View(ViewMessage::FocusDown),
            MenuAction::SelectFocused => Message::View(ViewMessage::SelectFocused),
            MenuAction::ToggleSlideshow => Message::View(ViewMessage::ToggleSlideshow),
            MenuAction::SetWallpaper => Message::SetWallpaper,
            MenuAction::DeleteImage => Message::DeleteImage,
            MenuAction::Rotate90 => Message::Edit(EditMessage::Rotate90),
            MenuAction::Rotate180 => Message::Edit(EditMessage::Rotate180),
            MenuAction::FlipHorizontal => Message::Edit(EditMessage::FlipHorizontal),
            MenuAction::FlipVertical => Message::Edit(EditMessage::FlipVertical),
            MenuAction::StartCrop => Message::Edit(EditMessage::StartCrop),
            MenuAction::Save => Message::Edit(EditMessage::Save),
            MenuAction::SaveAs => Message::Edit(EditMessage::SaveAs),
            MenuAction::Undo => Message::Edit(EditMessage::Undo),
            MenuAction::SortByName => Message::Settings(SettingsMessage::SortMode(SortMode::Name)),
            MenuAction::SortByDate => Message::Settings(SettingsMessage::SortMode(SortMode::Date)),
            MenuAction::SortBySize => Message::Settings(SettingsMessage::SortMode(SortMode::Size)),
            MenuAction::SortAscending => {
                Message::Settings(SettingsMessage::SortOrder(SortOrder::Ascending))
            }
            MenuAction::SortDescending => {
                Message::Settings(SettingsMessage::SortOrder(SortOrder::Descending))
            }
            MenuAction::OpenRecentFolder(idx) => Message::OpenRecentFolder(idx),
            MenuAction::ClearRecentFolders => Message::ClearRecentFolders,
        }
    }
}

impl Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Message {
        (*self).message()
    }
}

pub fn init_key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut binds = HashMap::new();

    // File ops
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("o".into()),
        },
        MenuAction::Open,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("o".into()),
        },
        MenuAction::OpenFolder,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Super],
            key: Key::Character("q".into()),
        },
        MenuAction::Quit,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Alt],
            key: Key::Named(Named::F4),
        },
        MenuAction::Quit,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("=".into()),
        },
        MenuAction::ZoomIn,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("-".into()),
        },
        MenuAction::ZoomOut,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("0".into()),
        },
        MenuAction::ZoomReset,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("f".into()),
        },
        MenuAction::ZoomFit,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::F11),
        },
        MenuAction::Fullscreen,
    );

    // Navigation arrow keys
    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::ArrowLeft),
        },
        MenuAction::Prev,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::ArrowRight),
        },
        MenuAction::Next,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::ArrowUp),
        },
        MenuAction::FocusUp,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::ArrowDown),
        },
        MenuAction::FocusDown,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Enter),
        },
        MenuAction::SelectFocused,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Home),
        },
        MenuAction::First,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::End),
        },
        MenuAction::Last,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Escape),
        },
        MenuAction::CloseModal,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::F5),
        },
        MenuAction::ToggleSlideshow,
    );

    // Info
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("i".into()),
        },
        MenuAction::ImageInfo,
    );

    // Set wallpaper
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("w".into()),
        },
        MenuAction::SetWallpaper,
    );

    // Delete image
    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Delete),
        },
        MenuAction::DeleteImage,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("r".into()),
        },
        MenuAction::Rotate90,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Shift],
            key: Key::Character("r".into()),
        },
        MenuAction::Rotate180,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("x".into()),
        },
        MenuAction::StartCrop,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("s".into()),
        },
        MenuAction::Save,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Shift],
            key: Key::Character("s".into()),
        },
        MenuAction::SaveAs,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("z".into()),
        },
        MenuAction::Undo,
    );

    binds
}
