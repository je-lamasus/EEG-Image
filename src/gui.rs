use crate::export::save_svg_as_jpeg;
use crate::points::{draw_eeg_svg, parse_points};
use iced::border::Radius;
use iced::mouse::Cursor;
use iced::theme::Style;
use iced::widget::canvas::{Cache, Geometry, Path, Program, Stroke};
use iced::widget::rule::FillMode;
use iced::widget::text::{LineHeight, Wrapping};
use iced::widget::text_editor::{Action, Content};
use iced::widget::{Canvas, button, column, container, row, rule, stack, svg, text, text_editor};
use iced::{
    Alignment, Background, Border, Color, ContentFit, Element, Fill, Font, Point, Rectangle,
    Renderer, Shadow, Subscription, Task, Theme,
};
use std::time::{Duration, Instant};

const BACKGROUND: Color = Color::from_rgb8(242, 247, 248);
const SURFACE: Color = Color::from_rgb8(251, 254, 254);
const SURFACE_MUTED: Color = Color::from_rgb8(242, 247, 248);
const MAP_BACKGROUND: Color = Color::from_rgb8(249, 252, 252);
const MAP_GRID: Color = Color::from_rgb8(225, 234, 235);
const BORDER: Color = Color::from_rgb8(207, 221, 223);
const BORDER_STRONG: Color = Color::from_rgb8(185, 203, 206);
const TEXT: Color = Color::from_rgb8(32, 50, 53);
const TEXT_MUTED: Color = Color::from_rgb8(100, 119, 122);
const PLACEHOLDER: Color = Color::from_rgb8(156, 164, 173);
const ACCENT: Color = Color::from_rgb8(40, 114, 122);
const ACCENT_HOVER: Color = Color::from_rgb8(32, 95, 102);
const ACCENT_SOFT: Color = Color::from_rgb8(226, 240, 241);
const FOCUS: Color = Color::from_rgb8(130, 183, 187);
const DANGER: Color = Color::from_rgb8(168, 75, 72);
const DANGER_SOFT: Color = Color::from_rgb8(250, 238, 238);
const SVG_DEBOUNCE_DELAY: Duration = Duration::from_millis(250);
const SVG_DEBOUNCE_TICK: Duration = Duration::from_millis(50);
const INITIAL_POINTS: &str = "C1 CZ C2 FCZ FPZ";

#[derive(Debug, Clone)]
pub enum Message {
    Loaded,
    Clear,
    Save,
    Saved(Result<(), String>),
    PointsChanged(Action),
    DebounceTick(Instant),
}

pub struct Application {
    points_text: Content,
    points_svg: svg::Handle,
    points_svg_bytes: Vec<u8>,
    pending_points: Vec<String>,
    last_edit_at: Option<Instant>,
    svg_dirty: bool,
    error_message: String,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    const MAP_IMAGE: &'static [u8] = include_bytes!("../images/map.svg");
    pub const ROBOTO_FONT: &'static [u8] = include_bytes!("../fonts/Roboto-Variable.ttf");
    pub const LIBERATION_MONO_FONT: &'static [u8] =
        include_bytes!("../fonts/LiberationMono-Regular.ttf");

    pub fn new() -> Self {
        let (pending_points, _) = parse_points(INITIAL_POINTS.to_lowercase());
        let (points_svg_bytes, error_message) =
            match draw_eeg_svg(Self::MAP_IMAGE.to_vec(), pending_points.clone()) {
                Ok(xml_bytes) => (xml_bytes, String::new()),
                Err(error) => (Self::MAP_IMAGE.to_vec(), error.to_string()),
            };
        let points_svg = svg::Handle::from_memory(points_svg_bytes.clone());

        Self {
            points_text: Content::with_text(INITIAL_POINTS),
            points_svg,
            points_svg_bytes,
            pending_points,
            last_edit_at: None,
            svg_dirty: false,
            error_message,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let grid = Canvas::new(GridBackground::new(24.0, 1.0, MAP_GRID))
            .height(Fill)
            .width(Fill);

        let map = svg(self.points_svg.clone())
            .content_fit(ContentFit::Contain)
            .width(Fill)
            .height(Fill);

        let map_area = stack![grid, map].width(Fill).height(Fill);

        let map_container = container(map_area)
            .width(Fill)
            .height(Fill)
            .style(|_| container::Style::from(MAP_BACKGROUND));

        let has_error = !self.error_message.is_empty();
        let field_header = row![
            text("Список электродов").size(12).font(semibold_font()),
            iced::widget::Space::new().width(Fill),
            text("через пробел").size(10).color(TEXT_MUTED),
        ]
        .width(Fill)
        .align_y(Alignment::Center);

        let mut controls_content = column![
            field_header,
            text_editor(&self.points_text)
                .on_action(Message::PointsChanged)
                .height(106)
                .padding([10, 11])
                .size(13)
                .line_height(LineHeight::Relative(1.65))
                .wrapping(Wrapping::WordOrGlyph)
                .style(move |theme, status| text_input_style(theme, status, has_error))
                .font(Font::with_name("Liberation Mono")),
            row![
                button(text("Очистить").size(12).font(semibold_font()).center())
                    .on_press(Message::Clear)
                    .width(92)
                    .height(34)
                    .style(clear_button_style),
                button(
                    text("Сохранить результат")
                        .size(12)
                        .font(semibold_font())
                        .width(Fill)
                        .center()
                )
                .on_press(Message::Save)
                .width(Fill)
                .height(34)
                .style(save_button_style),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .height(Fill);

        if !self.error_message.is_empty() {
            let error_indicator = container(text(""))
                .width(6)
                .height(6)
                .style(error_indicator_style);

            let error = container(
                row![
                    error_indicator,
                    text(&self.error_message)
                        .size(11)
                        .width(Fill)
                        .wrapping(Wrapping::WordOrGlyph),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .width(Fill)
            .padding([8, 10])
            .style(error_container_style);

            controls_content = controls_content
                .push(iced::widget::Space::new().height(Fill))
                .push(error);
        }

        let controls = container(controls_content)
            .width(320)
            .height(Fill)
            .padding(16)
            .style(|_| container::Style::from(SURFACE));

        let vertical_divider = rule::vertical(1).style(|_| rule::Style {
            color: BORDER_STRONG,
            fill_mode: FillMode::Full,
            radius: Radius::new(0),
            snap: true,
        });

        row![map_container, vertical_divider, controls]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PointsChanged(editor_action) => {
                match editor_action {
                    Action::Edit(_) => {
                        self.points_text.perform(editor_action);
                        self.error_message.clear();

                        let (existing_points, not_found_points) =
                            parse_points(self.points_text.text().to_lowercase());

                        self.pending_points = existing_points;
                        self.last_edit_at = Some(Instant::now());
                        self.svg_dirty = true;

                        if !not_found_points.is_empty() {
                            self.error_message = format!(
                                "Не получилось найти электроды: {}",
                                not_found_points.join(", ")
                            )
                        }
                    }
                    _ => self.points_text.perform(editor_action),
                }

                Task::none()
            }
            Message::Clear => {
                self.points_text = Content::new();
                self.error_message = String::new();
                self.set_svg(Self::MAP_IMAGE.to_vec());
                self.pending_points.clear();
                self.last_edit_at = None;
                self.svg_dirty = false;

                Task::none()
            }
            Message::Save => {
                if self.svg_dirty {
                    self.svg_dirty = false;
                    self.last_edit_at = None;

                    if let Err(error) = self.rebuild_svg() {
                        self.error_message = error;
                        return Task::none();
                    }
                }

                Task::perform(
                    save_svg_as_jpeg(self.points_svg_bytes.clone()),
                    Message::Saved,
                )
            }
            Message::Saved(result) => {
                if let Err(error) = result {
                    self.error_message = error;
                }

                Task::none()
            }
            Message::DebounceTick(now) => {
                if self.svg_dirty
                    && self.last_edit_at.is_some_and(|last_edit| {
                        now.duration_since(last_edit) >= SVG_DEBOUNCE_DELAY
                    })
                {
                    self.svg_dirty = false;
                    self.last_edit_at = None;

                    if let Err(error) = self.rebuild_svg() {
                        self.error_message = error;
                    }
                }

                Task::none()
            }
            Message::Loaded => Task::none(),
        }
    }

    fn rebuild_svg(&mut self) -> Result<(), String> {
        let xml_bytes = draw_eeg_svg(Self::MAP_IMAGE.to_vec(), self.pending_points.clone())
            .map_err(|error| error.to_string())?;
        self.set_svg(xml_bytes);

        Ok(())
    }

    fn set_svg(&mut self, xml_bytes: Vec<u8>) {
        self.points_svg = svg::Handle::from_memory(xml_bytes.clone());
        self.points_svg_bytes = xml_bytes;
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.svg_dirty {
            iced::time::every(SVG_DEBOUNCE_TICK).map(Message::DebounceTick)
        } else {
            Subscription::none()
        }
    }

    pub fn style(&self, _: &Theme) -> Style {
        Style {
            background_color: BACKGROUND,
            text_color: TEXT,
        }
    }
}

fn clear_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = match status {
        button::Status::Active => (SURFACE, TEXT, BORDER_STRONG),
        button::Status::Hovered => (SURFACE_MUTED, TEXT, BORDER_STRONG),
        button::Status::Pressed => (ACCENT_SOFT, TEXT, ACCENT),
        button::Status::Disabled => (SURFACE_MUTED, TEXT_MUTED, BORDER),
    };

    button::Style {
        text_color,
        border: Border::default().rounded(2).width(1).color(border_color),
        background: Some(Background::Color(background)),
        snap: true,
        shadow: Shadow::default(),
    }
}

fn save_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = match status {
        button::Status::Active => (ACCENT, Color::WHITE, ACCENT),
        button::Status::Hovered | button::Status::Pressed => {
            (ACCENT_HOVER, Color::WHITE, ACCENT_HOVER)
        }
        button::Status::Disabled => (ACCENT_SOFT, TEXT_MUTED, BORDER),
    };

    button::Style {
        text_color,
        border: Border::default().rounded(2).width(1).color(border_color),
        background: Some(Background::Color(background)),
        snap: true,
        shadow: Shadow::default(),
    }
}

fn text_input_style(_: &Theme, status: text_editor::Status, has_error: bool) -> text_editor::Style {
    let (background, border_color, value) = match status {
        text_editor::Status::Active => (SURFACE, FOCUS, TEXT),
        text_editor::Status::Disabled => (SURFACE_MUTED, BORDER, TEXT_MUTED),
        text_editor::Status::Hovered => (SURFACE, BORDER_STRONG, TEXT),
        _ => (SURFACE, FOCUS, TEXT),
    };
    let border_color = if has_error { DANGER } else { border_color };

    text_editor::Style {
        background: Background::Color(background),
        border: Border::default().rounded(2).width(1).color(border_color),
        placeholder: PLACEHOLDER,
        value,
        selection: ACCENT_SOFT,
    }
}

fn semibold_font() -> Font {
    Font {
        weight: iced::font::Weight::Semibold,
        ..Font::DEFAULT
    }
}

fn error_container_style(_: &Theme) -> container::Style {
    container::Style::default()
        .color(DANGER)
        .background(DANGER_SOFT)
        .border(Border::default().rounded(2).width(1).color(DANGER))
}

fn error_indicator_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(DANGER)
        .border(Border::default().rounded(3))
}

struct GridBackground {
    size: f32,
    thickness: f32,
    color: Color,
}

impl GridBackground {
    fn new(size: f32, thickness: f32, color: Color) -> Self {
        Self {
            size,
            thickness,
            color,
        }
    }
}

impl Program<Message> for GridBackground {
    type State = Cache;

    fn draw(
        &self,
        state: &Cache,
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: Cursor,
    ) -> Vec<Geometry> {
        let grid = Path::new(|builder| {
            let mut x = 0.0;
            while x <= bounds.width {
                builder.move_to(Point::new(x, 0.0));
                builder.line_to(Point::new(x, bounds.height));

                x += self.size
            }

            let mut y = 0.0;
            while y <= bounds.height {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(bounds.width, y));

                y += self.size;
            }
        });

        let geometry = state.draw(renderer, bounds.size(), |frame| {
            frame.stroke(
                &grid,
                Stroke::default()
                    .with_color(self.color)
                    .with_width(self.thickness),
            );
        });

        vec![geometry]
    }
}
