pub mod app;
pub mod config;
pub mod image;
pub mod key_binds;
pub mod localize;
pub mod menu;
pub mod message;
pub mod nav;
pub mod views;
pub mod watcher;

use app::ImageViewer;
use std::path::PathBuf;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(400.0)
            .min_height(300.0),
    );

    let optional_image = std::env::args().nth(1).map(PathBuf::from);

    cosmic::app::run::<ImageViewer>(settings, optional_image)
}
