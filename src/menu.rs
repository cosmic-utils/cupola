use crate::{fl, key_binds::MenuAction, message::Message};
use cosmic::{
    Element,
    widget::menu::{self, ItemHeight, ItemWidth, MenuBar, Tree},
};
use std::collections::HashMap;

pub fn menu_bar(key_binds: &HashMap<menu::KeyBind, MenuAction>) -> MenuBar<Message> {
    MenuBar::new(vec![
        file_menu(key_binds),
        view_menu(key_binds),
        nav_menu(key_binds),
        help_menu(key_binds),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.)
}

fn file_menu(key_binds: &HashMap<menu::KeyBind, MenuAction>) -> Tree<Message> {
    Tree::with_children(
        Element::from(menu::root(fl!("menu-file"))),
        menu::items(
            key_binds,
            vec![
                menu::Item::Button(fl!("menu-open"), None, MenuAction::Open),
                menu::Item::Button(fl!("menu-open-folder"), None, MenuAction::OpenFolder),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-close"), None, MenuAction::Close),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-quit"), None, MenuAction::Quit),
            ],
        ),
    )
}

fn view_menu(key_binds: &HashMap<menu::KeyBind, MenuAction>) -> Tree<Message> {
    Tree::with_children(
        Element::from(menu::root(fl!("menu-view"))),
        menu::items(
            key_binds,
            vec![
                menu::Item::Button(fl!("menu-zoom-in"), None, MenuAction::ZoomIn),
                menu::Item::Button(fl!("menu-zoom-out"), None, MenuAction::ZoomOut),
                menu::Item::Button(fl!("menu-zoom-reset"), None, MenuAction::ZoomReset),
                menu::Item::Button(fl!("menu-zoom-fit"), None, MenuAction::ZoomFit),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-fullscreen"), None, MenuAction::Fullscreen),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-single"), None, MenuAction::SingleView),
                menu::Item::Button(fl!("menu-gallery"), None, MenuAction::GalleryView),
            ],
        ),
    )
}

fn nav_menu(key_binds: &HashMap<menu::KeyBind, MenuAction>) -> Tree<Message> {
    Tree::with_children(
        Element::from(menu::root(fl!("menu-nav"))),
        menu::items(
            key_binds,
            vec![
                menu::Item::Button(fl!("menu-next"), None, MenuAction::Next),
                menu::Item::Button(fl!("menu-prev"), None, MenuAction::Prev),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-first"), None, MenuAction::First),
                menu::Item::Button(fl!("menu-last"), None, MenuAction::Last),
            ],
        ),
    )
}

fn help_menu(key_binds: &HashMap<menu::KeyBind, MenuAction>) -> Tree<Message> {
    Tree::with_children(
        Element::from(menu::root(fl!("menu-help"))),
        menu::items(
            key_binds,
            vec![menu::Item::Button(
                fl!("menu-about"),
                None,
                MenuAction::About,
            )],
        ),
    )
}
