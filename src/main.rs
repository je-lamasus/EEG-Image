#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use crate::gui::Application;
use iced::Font;

pub mod export;
pub mod gui;
pub mod points;

fn application_icon() -> Option<iced::window::Icon> {
    let icon = image::load_from_memory_with_format(
        include_bytes!("../packaging/icons/app-icon-window.png"),
        image::ImageFormat::Png,
    )
    .ok()?
    .into_rgba8();
    let (width, height) = icon.dimensions();

    iced::window::icon::from_rgba(icon.into_raw(), width, height).ok()
}

fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view)
        .title("Точки ЭЭГ")
        .window(iced::window::Settings {
            icon: application_icon(),
            ..iced::window::Settings::default()
        })
        .window_size((1040.0, 680.0))
        .resizable(false)
        .centered()
        .font(Application::ROBOTO_FONT)
        .font(Application::LIBERATION_MONO_FONT)
        .default_font(Font::with_name("Roboto"))
        .style(Application::style)
        .subscription(Application::subscription)
        .run()
}
