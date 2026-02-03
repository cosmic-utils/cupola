use crate::{fl, key_binds::MenuAction, message::Message};
use cosmic::{
    Core, Element,
    widget::{
        menu::{self, ItemHeight, ItemWidth, KeyBind},
        responsive_menu_bar,
    },
};
use std::{collections::HashMap, path::Path, sync::LazyLock};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

fn build_file_menu(recent_folders: &[String]) -> Vec<menu::Item<MenuAction, String>> {
    let mut items = vec![
        menu::Item::button(fl!("menu-open"), None, MenuAction::Open),
        menu::Item::button(fl!("menu-open-folder"), None, MenuAction::OpenFolder),
    ];

    if !recent_folders.is_empty() {
        items.push(menu::Item::divider());
        let mut folder_items: Vec<menu::Item<MenuAction, String>> = recent_folders
            .iter()
            .enumerate()
            .map(|(idx, folder)| {
                let display_name = Path::new(folder)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(folder)
                    .to_string();
                menu::Item::button(display_name, None, MenuAction::OpenRecentFolder(idx))
            })
            .collect();

        folder_items.push(menu::Item::divider());
        folder_items.push(menu::Item::button(
            fl!("menu-clear-recent"),
            None,
            MenuAction::ClearRecentFolders,
        ));

        items.push(menu::Item::folder(fl!("menu-recent-folders"), folder_items).width(280));
    }

    items.push(menu::Item::divider());
    items.push(menu::Item::button(fl!("menu-save"), None, MenuAction::Save));
    items.push(menu::Item::button(
        fl!("menu-save-as"),
        None,
        MenuAction::SaveAs,
    ));
    items.push(menu::Item::divider());
    items.push(menu::Item::button(
        fl!("menu-set-wallpaper"),
        None,
        MenuAction::SetWallpaper,
    ));
    items.push(menu::Item::button(
        fl!("menu-delete"),
        None,
        MenuAction::DeleteImage,
    ));
    items.push(menu::Item::divider());
    items.push(menu::Item::button(
        fl!("menu-settings"),
        None,
        MenuAction::Settings,
    ));
    items.push(menu::Item::divider());
    items.push(menu::Item::button(fl!("menu-quit"), None, MenuAction::Quit));

    items
}

pub fn menu_bar<'a>(
    core: &Core,
    key_binds: &HashMap<KeyBind, MenuAction>,
    is_slideshow_active: bool,
    recent_folders: &[String],
) -> Element<'a, Message> {
    let file_menu = build_file_menu(recent_folders);

    responsive_menu_bar()
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(250))
        .spacing(4.)
        .into_element(
            core,
            key_binds,
            MENU_ID.clone(),
            Message::Surface,
            vec![
                (fl!("menu-file"), file_menu),
                (
                    fl!("menu-edit"),
                    vec![
                        menu::Item::button(fl!("menu-rotate-90"), None, MenuAction::Rotate90),
                        menu::Item::button(fl!("menu-rotate-180"), None, MenuAction::Rotate180),
                        menu::Item::divider(),
                        menu::Item::button(
                            fl!("menu-flip-horizontal"),
                            None,
                            MenuAction::FlipHorizontal,
                        ),
                        menu::Item::button(
                            fl!("menu-flip-vertical"),
                            None,
                            MenuAction::FlipVertical,
                        ),
                        menu::Item::divider(),
                        menu::Item::button(fl!("menu-crop"), None, MenuAction::StartCrop),
                        menu::Item::divider(),
                        menu::Item::button(fl!("menu-undo"), None, MenuAction::Undo),
                    ],
                ),
                (
                    fl!("menu-view"),
                    vec![
                        menu::Item::button(fl!("menu-zoom-in"), None, MenuAction::ZoomIn),
                        menu::Item::button(fl!("menu-zoom-out"), None, MenuAction::ZoomOut),
                        menu::Item::button(fl!("menu-zoom-reset"), None, MenuAction::ZoomReset),
                        menu::Item::button(fl!("menu-zoom-fit"), None, MenuAction::ZoomFit),
                        menu::Item::divider(),
                        menu::Item::button(fl!("menu-fullscreen"), None, MenuAction::Fullscreen),
                        menu::Item::button(
                            if is_slideshow_active {
                                fl!("menu-slideshow-stop")
                            } else {
                                fl!("menu-slideshow-start")
                            },
                            None,
                            MenuAction::ToggleSlideshow,
                        ),
                        menu::Item::divider(),
                        menu::Item::button(fl!("menu-sort-name"), None, MenuAction::SortByName),
                        menu::Item::button(fl!("menu-sort-date"), None, MenuAction::SortByDate),
                        menu::Item::button(fl!("menu-sort-size"), None, MenuAction::SortBySize),
                        menu::Item::divider(),
                        menu::Item::button(
                            fl!("menu-sort-ascending"),
                            None,
                            MenuAction::SortAscending,
                        ),
                        menu::Item::button(
                            fl!("menu-sort-descending"),
                            None,
                            MenuAction::SortDescending,
                        ),
                    ],
                ),
                (
                    fl!("menu-nav"),
                    vec![
                        menu::Item::button(fl!("menu-next"), None, MenuAction::Next),
                        menu::Item::button(fl!("menu-prev"), None, MenuAction::Prev),
                        menu::Item::divider(),
                        menu::Item::button(fl!("menu-first"), None, MenuAction::First),
                        menu::Item::button(fl!("menu-last"), None, MenuAction::Last),
                    ],
                ),
                (
                    fl!("menu-help"),
                    vec![menu::Item::button(
                        fl!("menu-about"),
                        None,
                        MenuAction::About,
                    )],
                ),
            ],
        )
}
