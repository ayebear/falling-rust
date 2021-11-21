#![windows_subsystem = "windows"]

mod camera;
mod cell;
mod element;
mod mouse_input;
mod render;
mod sandbox;
mod simulation;
mod toolbox;

use bevy::{
    prelude::*,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use camera::camera_controller;
use element::Element;
use mouse_input::{mouse_input_handler, MouseInputState};
use render::level_texture_updater;
use sandbox::*;
use simulation::{level_updater, Simulation};
use toolbox::{Tool, ToolBox};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Falling-Rust".to_string(),
            width: 1024.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .init_resource::<MouseInputState>()
        .init_resource::<SandBox>()
        .init_resource::<Simulation>()
        .init_resource::<ToolBox>()
        .add_startup_system(setup.system())
        .add_system(gui_system.system().label("gui"))
        .add_system(level_updater.system())
        .add_system(level_texture_updater.system())
        .add_system(mouse_input_handler.system())
        .add_system(camera_controller.system())
        .add_system(level_editor.system().after("gui"))
        .run();
}

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sandbox: Res<SandBox>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Create texture for displaying
    let texture = Texture::new_fill(
        Extent3d::new(sandbox.width() as u32, sandbox.height() as u32, 1),
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    );
    let th = textures.add(texture);

    // Now spawn the sprite for the level
    commands.spawn().insert_bundle(SpriteBundle {
        material: materials.add(th.into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn gui_system(
    egui_context: ResMut<EguiContext>,
    mut toolbox: ResMut<ToolBox>,
    mut simulation: ResMut<Simulation>,
    mut level: ResMut<SandBox>,
) {
    egui::SidePanel::left("left_tools")
        .min_width(180.0)
        .show(egui_context.ctx(), |ui| {
            ui.label("Tool element:");
            ui.radio_value(&mut toolbox.element, Element::Air, "Air");
            ui.radio_value(&mut toolbox.element, Element::Sand, "Sand");
            ui.radio_value(&mut toolbox.element, Element::Wood, "Wood");
            ui.radio_value(&mut toolbox.element, Element::Rock, "Rock");
            ui.radio_value(&mut toolbox.element, Element::Water, "Water");
            ui.radio_value(&mut toolbox.element, Element::Acid, "Acid");
            ui.radio_value(&mut toolbox.element, Element::Oil, "Oil");
            ui.radio_value(&mut toolbox.element, Element::Lava, "Lava");
            ui.radio_value(&mut toolbox.element, Element::Fire, "Fire");
            ui.radio_value(&mut toolbox.element, Element::Life, "Life");
            ui.radio_value(&mut toolbox.element, Element::WaterSource, "Water source");
            ui.radio_value(&mut toolbox.element, Element::AcidSource, "Acid Source");
            ui.radio_value(&mut toolbox.element, Element::OilSource, "Oil source");
            ui.radio_value(&mut toolbox.element, Element::LavaSource, "Lava source");
            ui.radio_value(&mut toolbox.element, Element::FireSource, "Fire source");
            ui.radio_value(&mut toolbox.element, Element::Drain, "Liquid drain");
            ui.separator();
            ui.label("Tool size:");
            ui.add(egui::Slider::new(&mut toolbox.tool_size, 1..=64));
            ui.separator();
            ui.label("Tool shape:");
            ui.radio_value(&mut toolbox.tool, Tool::FillCircle, "Fill Circle");
            ui.radio_value(&mut toolbox.tool, Tool::FillSquare, "Fill Square");
            ui.radio_value(&mut toolbox.tool, Tool::SprayCircle, "Spray Circle");
        });

    egui::SidePanel::right("right_tools").show(egui_context.ctx(), |ui| {
        ui.label("Simulation:");
        ui.checkbox(&mut simulation.running, "Running");
        if ui.button("Step").clicked() {
            simulation.step = true;
        }
        ui.label(format!("Step time: {} ms", simulation.frame_time_ms));
        ui.separator();
        ui.label("Sandbox:");
        if ui.button("Clear").clicked() {
            level.clear();
        }
    });
}

fn level_editor(
    mouse: Res<MouseInputState>,
    egui_context: Res<EguiContext>,
    mut toolbox: ResMut<ToolBox>,
    mut level: ResMut<SandBox>,
    mut query: Query<&Transform>,
) {
    if egui_context.ctx().wants_pointer_input() || egui_context.ctx().wants_keyboard_input() {
        // GUI gets priority for input events
        return;
    }
    for transform in query.iter_mut() {
        let x = mouse.world_position.x - transform.translation.x + level.width() as f32 / 2.0;
        let y = level.height() as f32
            - (mouse.world_position.y - transform.translation.y + level.height() as f32 / 2.0);
        if x > 0.0 && x < level.width() as f32 && y > 0.0 && y < level.height() as f32 {
            if mouse.left_button_down {
                toolbox.apply(&mut level, x.floor() as usize, y.floor() as usize);
            }
            if mouse.right_button_down {
                let element = toolbox.element;
                toolbox.element = Element::Air;
                toolbox.apply(&mut level, x.floor() as usize, y.floor() as usize);
                toolbox.element = element;
            }
        }
    }
}
