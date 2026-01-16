use crate::{fl, key_binds::MenuAction, message::Message};
use cosmic::{
    Core, Element,
    widget::{
        menu::{self, ItemHeight, ItemWidth, KeyBind},
        responsive_menu_bar,
    },
};
use std::{collections::HashMap, sync::LazyLock};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

/// Returns a ResponsiveMenuBar as an Element
pub fn menu_bar<'a>(
    core: &Core,
    key_binds: &HashMap<KeyBind, MenuAction>,
    is_slideshow_active: bool,
) -> Element<'a, Message> {
    responsive_menu_bar()
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(260))
        .spacing(4.)
        .into_element(
            core,
            key_binds,
            MENU_ID.clone(),
            Message::Surface,
            vec![
                (
                    fl!("menu-file"),
                    vec![
                        menu::Item::Button(fl!("menu-open"), None, MenuAction::Open),
                        menu::Item::Button(fl!("menu-open-folder"), None, MenuAction::OpenFolder),
                        menu::Item::Divider,
                        menu::Item::Button(
                            fl!("menu-set-wallpaper"),
                            None,
                            MenuAction::SetWallpaper,
                        ),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-settings"), None, MenuAction::Settings),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-quit"), None, MenuAction::Quit),
                    ],
                ),
                (
                    fl!("menu-view"),
                    vec![
                        menu::Item::Button(fl!("menu-zoom-in"), None, MenuAction::ZoomIn),
                        menu::Item::Button(fl!("menu-zoom-out"), None, MenuAction::ZoomOut),
                        menu::Item::Button(fl!("menu-zoom-reset"), None, MenuAction::ZoomReset),
                        menu::Item::Button(fl!("menu-zoom-fit"), None, MenuAction::ZoomFit),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-fullscreen"), None, MenuAction::Fullscreen),
                        menu::Item::Button(
                            if is_slideshow_active {
                                fl!("menu-slideshow-stop")
                            } else {
                                fl!("menu-slideshow-start")
                            },
                            None,
                            MenuAction::ToggleSlideshow,
                        ),
                    ],
                ),
                (
                    fl!("menu-nav"),
                    vec![
                        menu::Item::Button(fl!("menu-next"), None, MenuAction::Next),
                        menu::Item::Button(fl!("menu-prev"), None, MenuAction::Prev),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-first"), None, MenuAction::First),
                        menu::Item::Button(fl!("menu-last"), None, MenuAction::Last),
                    ],
                ),
                (
                    fl!("menu-help"),
                    vec![menu::Item::Button(
                        fl!("menu-about"),
                        None,
                        MenuAction::About,
                    )],
                ),
            ],
        )
}
