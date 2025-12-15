//! Keyboard shortcuts and menu action definitions
use crate::message::{ImageMessage, Message, NavMessage, ViewMessage};
use cosmic::{
    iced::keyboard::{Key, key::Named},
    widget::menu::{
        Action,
        key_bind::{KeyBind, Modifier},
    },
};
use std::collections::HashMap;

/// Menu and keyboard bound actions
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
    GalleryView,
    SingleView,
    CloseModal,
    About,
    Settings,
    ImageInfo,
}

impl MenuAction {
    /// Convert action to app message
    pub fn message(self) -> Message {
        match self {
            MenuAction::Open => Message::OpenFileDialog,
            MenuAction::OpenFolder => Message::OpenFolderDialog,
            MenuAction::Close | MenuAction::Quit => Message::Image(ImageMessage::Clear),
            MenuAction::ZoomIn => Message::View(ViewMessage::ZoomIn),
            MenuAction::ZoomOut => Message::View(ViewMessage::ZoomOut),
            MenuAction::ZoomReset => Message::View(ViewMessage::ZoomReset),
            MenuAction::ZoomFit => Message::View(ViewMessage::ZoomFit),
            MenuAction::Fullscreen => Message::View(ViewMessage::ToggleFullScreen),
            MenuAction::Next => Message::Nav(NavMessage::Next),
            MenuAction::Prev => Message::Nav(NavMessage::Prev),
            MenuAction::First => Message::Nav(NavMessage::First),
            MenuAction::Last => Message::Nav(NavMessage::Last),
            MenuAction::GalleryView => Message::View(ViewMessage::ShowGallery),
            MenuAction::SingleView => Message::View(ViewMessage::ShowSingle),
            MenuAction::CloseModal => Message::View(ViewMessage::CloseModal),
            MenuAction::About => Message::ToggleContextPage(crate::message::ContextPage::About),
            MenuAction::Settings => {
                Message::ToggleContextPage(crate::message::ContextPage::Settings)
            }
            MenuAction::ImageInfo => {
                Message::ToggleContextPage(crate::message::ContextPage::ImageInfo)
            }
        }
    }
}

impl Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Message {
        (*self).message()
    }
}

/// Init default keybindings
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

    // View modes
    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Character("g".into()),
        },
        MenuAction::GalleryView,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Character("s".into()),
        },
        MenuAction::SingleView,
    );

    binds.insert(
        KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Escape),
        },
        MenuAction::CloseModal,
    );

    // Info
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("i".into()),
        },
        MenuAction::ImageInfo,
    );

    binds
}
