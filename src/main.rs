use crate::gui::Application;
use iced::Font;

pub mod export;
pub mod gui;
pub mod points;

fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view)
        .title("Точки ЭЭГ")
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
