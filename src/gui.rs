use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, style::*, Color32, ColorImage, Frame, Layout, Response, TextureHandle, Ui},
    EguiContext, EguiPlugin,
};
use egui::{Align2, FontId, Mesh, Pos2, Rect, Shape, Vec2};
use image::{DynamicImage, GenericImageView};

const ICON_SIZE: f32 = 64.0;

use crate::{
    element::*,
    language::{element_names, get_text, Language},
    pseudo_random::PseudoRandom,
    render::cell_color,
    sandbox::SandBox,
    settings::Settings,
    simulation::Simulation,
    spawn_sandbox,
    toolbox::{Tool, ToolBox},
    SystemOrderLabel,
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_system(gui_system.before(SystemOrderLabel::PointerInput))
            .add_startup_system(setup_gui);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GuiMode {
    MainGui,
    ElementSelect,
    ToolSelect,
    SandboxSettings,
    MoveView,
}

#[derive(Resource)]
pub struct SandboxGui {
    pub mode: GuiMode,
    pub last_element: Element,
    pub bucket_icon_handle: TextureHandle,
    pub icon_circle_handle: TextureHandle,
    pub icon_square_handle: TextureHandle,
    pub icon_pencil_handle: TextureHandle,
    pub icon_spray_handle: TextureHandle,
    pub icon_bucket_handle: TextureHandle,
    pub icon_play_handle: TextureHandle,
    pub icon_pause_handle: TextureHandle,
    pub icon_zoom_in_handle: TextureHandle,
    pub icon_zoom_out_handle: TextureHandle,
    pub icon_move_handle: TextureHandle,
    pub icon_settings_handle: TextureHandle,
    pub icon_eraser_handle: TextureHandle,
    pub icon_step_handle: TextureHandle,
    pub element_icons: [TextureHandle; ELEMENT_COUNT as usize],
    pub element_names: HashMap<Element, String>,
}

pub fn gui_system(
    mut egui_context: ResMut<EguiContext>,
    camera: Query<&mut Transform, With<Camera>>,
    mut gui: ResMut<SandboxGui>,
    mut settings: ResMut<Settings>,
    mut toolbox: ResMut<ToolBox>,
    mut simulation: ResMut<Simulation>,
    mut sandbox: Query<(Entity, &mut SandBox)>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    egui::SidePanel::right("right_panel")
        .frame(Frame::none())
        .show_separator_line(false)
        .resizable(false)
        .min_width(ICON_SIZE)
        .show(egui_context.ctx_mut(), |ui| {
            let settings_button =
                egui::widgets::ImageButton::new(&gui.icon_settings_handle, [ICON_SIZE, ICON_SIZE])
                    .frame(false);
            let settings_button = if gui.mode == GuiMode::SandboxSettings {
                settings_button.tint(Color32::LIGHT_GREEN)
            } else {
                settings_button
            };
            if ui.add(settings_button).clicked() {
                gui.mode = if gui.mode == GuiMode::SandboxSettings {
                    GuiMode::MainGui
                } else {
                    GuiMode::SandboxSettings
                }
            };
            if ui
                .add(
                    egui::widgets::ImageButton::new(
                        if simulation.running {
                            &gui.icon_play_handle
                        } else {
                            &gui.icon_pause_handle
                        },
                        [ICON_SIZE, ICON_SIZE],
                    )
                    .frame(false),
                )
                .clicked()
            {
                simulation.running = !simulation.running;
            };
            if !simulation.running {
                if ui
                    .add(
                        egui::widgets::ImageButton::new(
                            &gui.icon_step_handle,
                            [ICON_SIZE, ICON_SIZE],
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    simulation.step = true;
                };
            }

            view_gui(ui, gui.as_mut(), camera);
        });

    egui::TopBottomPanel::bottom("bottom_panel")
        .frame(Frame::none())
        .show_separator_line(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                tool_gui(ui, gui.as_mut(), toolbox.as_mut());
            });
        });

    if gui.mode == GuiMode::SandboxSettings {
        egui::SidePanel::left("settings").show(egui_context.ctx_mut(), |ui| {
            let (entity, sandbox) = sandbox.single_mut();
            egui::ComboBox::from_label(get_text("size", settings.language))
                .selected_text(format!(
                    "{}x{}",
                    settings.sandbox_size, settings.sandbox_size
                ))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.sandbox_size, 64, "64x64");
                    ui.selectable_value(&mut settings.sandbox_size, 128, "128x128");
                    ui.selectable_value(&mut settings.sandbox_size, 256, "256x256");
                    ui.selectable_value(&mut settings.sandbox_size, 512, "512x512");
                    ui.selectable_value(&mut settings.sandbox_size, 1024, "1024x1024");
                });
            if ui.button(get_text("new", settings.language)).clicked() {
                commands.entity(entity).despawn();
                spawn_sandbox(
                    commands,
                    images.as_mut(),
                    settings.sandbox_size,
                    settings.sandbox_size,
                );
                gui.mode = GuiMode::MainGui;
            }
            ui.separator();
            let previous_language = settings.language;
            egui::ComboBox::from_label(get_text("language", settings.language))
                .selected_text(format!("{:?}", settings.language))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.language, Language::English, "English");
                    ui.selectable_value(&mut settings.language, Language::Nederlands, "Nederlands");
                });
            if settings.language != previous_language {
                gui.element_names = element_names(settings.language);
            }
            ui.separator();
            ui.label(format!(
                "{}: {} ms",
                get_text("simulation", settings.language),
                simulation.frame_time_ms
            ));
            ui.label(format!(
                "{}: {} ms",
                get_text("render", settings.language),
                sandbox.render_time_ms
            ));
            ui.separator();
            ui.hyperlink_to("Made by Bas", "https://www.basvs.dev");
        });
    } else if gui.mode == GuiMode::ElementSelect {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(egui_context.ctx_mut(), |ui| {
                ui.with_layout(
                    Layout::from_main_dir_and_cross_align(
                        egui::Direction::LeftToRight,
                        egui::Align::Min,
                    )
                    .with_main_wrap(true),
                    |ui| {
                        element_button_click(ui, &mut gui, Element::Sand, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Wood, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Iron, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Rock, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Water, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Acid, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Oil, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Lava, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Fire, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Life, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Seed, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::TNT, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Fuse, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::WaterSource, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::AcidSource, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::LavaSource, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::FireSource, &mut toolbox);
                        element_button_click(ui, &mut gui, Element::Drain, &mut toolbox);
                    },
                );
            });
    }
    if gui.mode == GuiMode::ToolSelect {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(egui_context.ctx_mut(), |ui| {
                ui.with_layout(
                    Layout::from_main_dir_and_cross_align(
                        egui::Direction::LeftToRight,
                        egui::Align::Min,
                    )
                    .with_main_wrap(true),
                    |ui| {
                        if ui
                            .add(
                                egui::widgets::ImageButton::new(
                                    &gui.icon_pencil_handle,
                                    [ICON_SIZE, ICON_SIZE],
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            toolbox.tool = Tool::Pixel;
                            gui.mode = GuiMode::MainGui;
                        };
                        if ui
                            .add(
                                egui::widgets::ImageButton::new(
                                    &gui.icon_circle_handle,
                                    [ICON_SIZE, ICON_SIZE],
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            toolbox.tool = Tool::Circle;
                            gui.mode = GuiMode::MainGui;
                        };
                        if ui
                            .add(
                                egui::widgets::ImageButton::new(
                                    &gui.icon_square_handle,
                                    [ICON_SIZE, ICON_SIZE],
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            toolbox.tool = Tool::Square;
                            gui.mode = GuiMode::MainGui;
                        };
                        if ui
                            .add(
                                egui::widgets::ImageButton::new(
                                    &gui.icon_spray_handle,
                                    [ICON_SIZE, ICON_SIZE],
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            toolbox.tool = Tool::Spray;
                            gui.mode = GuiMode::MainGui;
                        };
                        if ui
                            .add(
                                egui::widgets::ImageButton::new(
                                    &gui.icon_bucket_handle,
                                    [ICON_SIZE, ICON_SIZE],
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            toolbox.tool = Tool::Fill;
                            gui.mode = GuiMode::MainGui;
                        };
                        if toolbox.tool != Tool::Pixel && toolbox.tool != Tool::Fill {
                            ui.add(egui::Slider::new(&mut toolbox.tool_size, 1..=64));
                        }
                    },
                );
            });
    }
}

fn view_gui(ui: &mut Ui, gui: &mut SandboxGui, mut camera: Query<&mut Transform, With<Camera>>) {
    if ui
        .add(
            egui::widgets::ImageButton::new(&gui.icon_zoom_in_handle, [ICON_SIZE, ICON_SIZE])
                .frame(false),
        )
        .clicked()
    {
        let mut transform = camera.single_mut();
        transform.scale.x = (transform.scale.x * 0.9).clamp(0.1, 1.0);
        transform.scale.y = (transform.scale.y * 0.9).clamp(0.1, 1.0);
    };
    if ui
        .add(
            egui::widgets::ImageButton::new(&gui.icon_zoom_out_handle, [ICON_SIZE, ICON_SIZE])
                .frame(false),
        )
        .clicked()
    {
        let mut transform = camera.single_mut();
        transform.scale.x = (transform.scale.x * 1.1).clamp(0.1, 1.0);
        transform.scale.y = (transform.scale.y * 1.1).clamp(0.1, 1.0);
    };
    let move_button =
        egui::widgets::ImageButton::new(&gui.icon_move_handle, [ICON_SIZE, ICON_SIZE]).frame(false);
    let move_button = if gui.mode == GuiMode::MoveView {
        move_button.tint(Color32::LIGHT_GREEN)
    } else {
        move_button
    };
    if ui.add(move_button).clicked() {
        gui.mode = if gui.mode == GuiMode::MoveView {
            GuiMode::MainGui
        } else {
            GuiMode::MoveView
        }
    };
}

fn tool_gui(ui: &mut Ui, gui: &mut SandboxGui, toolbox: &mut ToolBox) {
    let eraser_button =
        egui::widgets::ImageButton::new(&gui.icon_eraser_handle, [ICON_SIZE, ICON_SIZE])
            .frame(false);
    let eraser_button = if toolbox.element == Element::Air {
        eraser_button.tint(Color32::LIGHT_GREEN)
    } else {
        eraser_button
    };
    if ui.add(eraser_button).clicked() {
        if toolbox.element == Element::Air {
            toolbox.element = gui.last_element;
        } else {
            toolbox.element = Element::Air;
        }
    };

    if element_button(ui, gui, toolbox.element).clicked() {
        if gui.mode == GuiMode::ElementSelect {
            gui.mode = GuiMode::MainGui;
        } else {
            gui.mode = GuiMode::ElementSelect;
        }
    };

    let tool_button = egui::widgets::ImageButton::new(
        match toolbox.tool {
            Tool::Pixel => &gui.icon_pencil_handle,
            Tool::Circle => &gui.icon_circle_handle,
            Tool::Square => &gui.icon_square_handle,
            Tool::Spray => &gui.icon_spray_handle,
            Tool::Fill => &gui.icon_bucket_handle,
        },
        [ICON_SIZE, ICON_SIZE],
    )
    .frame(false);
    let tool_button = if gui.mode == GuiMode::ToolSelect {
        tool_button.tint(Color32::LIGHT_GREEN)
    } else {
        tool_button
    };
    if ui.add(tool_button).clicked() {
        if gui.mode == GuiMode::ToolSelect {
            gui.mode = GuiMode::MainGui;
        } else {
            gui.mode = GuiMode::ToolSelect;
        }
    };
}

fn setup_gui(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    settings: Res<Settings>,
) {
    // General styling
    let mut style = egui::Style::default();
    style.spacing = Spacing::default();
    style.spacing.scroll_bar_width = 20.0;
    style.spacing.button_padding = bevy_egui::egui::Vec2::new(10.0, 10.0);
    egui_context.ctx_mut().set_style(style);

    // Generate element icons
    let background = image::load_from_memory(include_bytes!("../assets/icon_element.png")).unwrap();
    let element_icons = [
        generate_element_image(Element::Air, egui_context.as_mut(), &background),
        generate_element_image(Element::Sand, egui_context.as_mut(), &background),
        generate_element_image(Element::Rock, egui_context.as_mut(), &background),
        generate_element_image(Element::Water, egui_context.as_mut(), &background),
        generate_element_image(Element::Acid, egui_context.as_mut(), &background),
        generate_element_image(Element::Drain, egui_context.as_mut(), &background),
        generate_element_image(Element::Wood, egui_context.as_mut(), &background),
        generate_element_image(Element::Iron, egui_context.as_mut(), &background),
        generate_element_image(Element::Rust, egui_context.as_mut(), &background),
        generate_element_image(Element::Fire, egui_context.as_mut(), &background),
        generate_element_image(Element::Ash, egui_context.as_mut(), &background),
        generate_element_image(Element::Oil, egui_context.as_mut(), &background),
        generate_element_image(Element::Lava, egui_context.as_mut(), &background),
        generate_element_image(Element::Smoke, egui_context.as_mut(), &background),
        generate_element_image(Element::Life, egui_context.as_mut(), &background),
        generate_element_image(Element::Seed, egui_context.as_mut(), &background),
        generate_element_image(Element::Plant, egui_context.as_mut(), &background),
        generate_element_image(Element::TNT, egui_context.as_mut(), &background),
        generate_element_image(Element::Fuse, egui_context.as_mut(), &background),
        generate_element_image(Element::Explosion, egui_context.as_mut(), &background),
        generate_element_image(Element::WaterSource, egui_context.as_mut(), &background),
        generate_element_image(Element::AcidSource, egui_context.as_mut(), &background),
        generate_element_image(Element::OilSource, egui_context.as_mut(), &background),
        generate_element_image(Element::FireSource, egui_context.as_mut(), &background),
        generate_element_image(Element::LavaSource, egui_context.as_mut(), &background),
        generate_element_image(Element::Indestructible, egui_context.as_mut(), &background),
    ];

    let element_names = element_names(settings.language);

    commands.insert_resource(SandboxGui {
        mode: GuiMode::MainGui,
        last_element: Element::Sand,
        bucket_icon_handle: add_icon(
            &mut egui_context,
            "icon_bucket",
            include_bytes!("../assets/icon_bucket.png"),
        ),
        icon_circle_handle: add_icon(
            &mut egui_context,
            "icon_circle",
            include_bytes!("../assets/icon_circle.png"),
        ),
        icon_square_handle: add_icon(
            &mut egui_context,
            "icon_square",
            include_bytes!("../assets/icon_square.png"),
        ),
        icon_pencil_handle: add_icon(
            &mut egui_context,
            "icon_pencil",
            include_bytes!("../assets/icon_pencil.png"),
        ),
        icon_spray_handle: add_icon(
            &mut egui_context,
            "icon_spray",
            include_bytes!("../assets/icon_spray.png"),
        ),
        icon_bucket_handle: add_icon(
            &mut egui_context,
            "icon_bucket",
            include_bytes!("../assets/icon_bucket.png"),
        ),
        icon_play_handle: add_icon(
            &mut egui_context,
            "icon_play",
            include_bytes!("../assets/icon_play.png"),
        ),
        icon_pause_handle: add_icon(
            &mut egui_context,
            "icon_pause",
            include_bytes!("../assets/icon_pause.png"),
        ),
        icon_zoom_in_handle: add_icon(
            &mut egui_context,
            "icon_zoom_in",
            include_bytes!("../assets/icon_zoom_in.png"),
        ),
        icon_zoom_out_handle: add_icon(
            &mut egui_context,
            "icon_zoom_out",
            include_bytes!("../assets/icon_zoom_out.png"),
        ),
        icon_move_handle: add_icon(
            &mut egui_context,
            "icon_move",
            include_bytes!("../assets/icon_move.png"),
        ),
        icon_settings_handle: add_icon(
            &mut egui_context,
            "icon_settings",
            include_bytes!("../assets/icon_settings.png"),
        ),
        icon_eraser_handle: add_icon(
            &mut egui_context,
            "icon_eraser",
            include_bytes!("../assets/icon_eraser.png"),
        ),
        icon_step_handle: add_icon(
            &mut egui_context,
            "icon_step",
            include_bytes!("../assets/icon_step.png"),
        ),
        element_icons,
        element_names,
    });
}

fn add_icon(egui_context: &mut EguiContext, name: &str, image_data: &[u8]) -> TextureHandle {
    let image = image::load_from_memory(image_data).unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    let icon_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    let icon_texture_handle =
        egui_context
            .ctx_mut()
            .load_texture(name, icon_image, Default::default());
    icon_texture_handle
}

fn element_button_click(
    ui: &mut Ui,
    gui: &mut ResMut<SandboxGui>,
    element: Element,
    toolbox: &mut ResMut<ToolBox>,
) {
    if element_button(ui, gui, element).clicked() {
        toolbox.element = element;
        gui.mode = GuiMode::MainGui;
    }
}

fn element_button(ui: &mut Ui, gui: &mut SandboxGui, element: Element) -> Response {
    const SIZE: f32 = ICON_SIZE;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(SIZE, SIZE), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let mut mesh = Mesh::with_texture(gui.element_icons[element as usize].id());
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().add(Shape::mesh(mesh));
        // Element name (with simple shadow)
        ui.painter().text(
            rect.left_top() + Vec2::new(11.0, 16.0),
            Align2::LEFT_TOP,
            gui.element_names.get(&element).unwrap().replace(" ", "\n"),
            FontId::proportional(14.0),
            Color32::BLACK,
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(10.0, 15.0),
            Align2::LEFT_TOP,
            gui.element_names.get(&element).unwrap().replace(" ", "\n"),
            FontId::proportional(14.0),
            Color32::WHITE,
        );
    }
    response
}

// Create a button image for element selection
pub fn generate_element_image(
    element: Element,
    egui_context: &mut EguiContext,
    background: &DynamicImage,
) -> TextureHandle {
    // Generate a tiny sandbox containing our element
    let size = 64;
    let mut sandbox = SandBox::new(size, size);
    let mut toolbox = ToolBox::default();
    toolbox.element = element;
    toolbox.tool = Tool::Square;
    toolbox.tool_size = size;
    let center = (size / 2) as isize;
    toolbox.apply(&mut sandbox, size / 2, size / 2);

    let mut img = ColorImage::new([size, size], Color32::TRANSPARENT);
    let mut random = PseudoRandom::new();

    for y in 0..size {
        for x in 0..size {
            // Get the background image color
            let pixel = background.get_pixel(x as u32, y as u32);
            let (or, og, ob, oa) = (pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);

            // Get the element color
            let cell = sandbox.get_mut(x, y);
            let (cr, cg, cb) = cell_color(cell, &mut random);

            // Do a simplified alpha blend between the two to soften the edges
            let dx = (center - x as isize).abs() as f32;
            let dy = (center - y as isize).abs() as f32;
            let alpha = 1.0 - ((dx * dx + dy * dy) / (size as f32 / 2.0).powf(2.0)).powf(3.0);
            let r = (cr as f32 * alpha + or as f32 * (1.0 - alpha)) as u8;
            let g = (cg as f32 * alpha + og as f32 * (1.0 - alpha)) as u8;
            let b = (cb as f32 * alpha + ob as f32 * (1.0 - alpha)) as u8;
            img[(x, y)] = Color32::from_rgba_premultiplied(r, g, b, oa);
        }
    }

    egui_context.ctx_mut().load_texture(
        format!("element_{}", element as u8),
        img,
        Default::default(),
    )
}
