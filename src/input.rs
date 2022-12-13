use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ButtonState,
    },
    prelude::*,
    render::camera::Camera,
};
use bevy_egui::EguiContext;

use crate::{element::Element, sandbox::SandBox, toolbox::ToolBox};

#[derive(Default, Resource)]
pub struct MouseInputState {
    pub left_button_down: bool,
    pub middle_button_down: bool,
    pub right_button_down: bool,
    pub position: Vec2,
    pub world_position: Vec2,
}

pub fn mouse_editor_input(
    mut state: ResMut<MouseInputState>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera: Query<(&Camera, &mut Transform, &GlobalTransform)>,
    mut egui_context: ResMut<EguiContext>,
    mut toolbox: ResMut<ToolBox>,
    mut sandbox: Query<&mut SandBox>,
) {
    let sandbox = sandbox.get_single_mut();
    if sandbox.is_err() {
        // Sandbox not active, so skip this
        return;
    }
    let mut sandbox = sandbox.unwrap();

    if egui_context.ctx_mut().wants_pointer_input() || egui_context.ctx_mut().wants_keyboard_input()
    {
        // GUI gets priority for input events
        return;
    }

    // Record latest position
    for event in cursor_moved_events.iter() {
        state.position = event.position;
    }
    let (camera, mut transform, global_transform) = camera.single_mut();
    let world_pos = camera
        .viewport_to_world(global_transform, state.position)
        .unwrap()
        .origin;
    state.world_position = Vec2::new(
        world_pos.x + (sandbox.width() / 2) as f32,
        (sandbox.height() / 2) as f32 - world_pos.y,
    );

    // Determine button state
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left {
            state.left_button_down = event.state == ButtonState::Pressed;
        }
        if event.button == MouseButton::Middle {
            state.middle_button_down = event.state == ButtonState::Pressed;
        }
        if event.button == MouseButton::Right {
            state.right_button_down = event.state == ButtonState::Pressed;
        }
    }

    // Zoom camera
    for event in mouse_wheel_events.iter() {
        if event.y > 0.0 {
            transform.scale.x = (transform.scale.x * 0.9).clamp(0.1, 1.0);
            transform.scale.y = (transform.scale.y * 0.9).clamp(0.1, 1.0);
        } else if event.y < 0.0 {
            transform.scale.x = (transform.scale.x * 1.1).clamp(0.1, 1.0);
            transform.scale.y = (transform.scale.y * 1.1).clamp(0.1, 1.0);
        }
    }

    // Pan camera
    for event in mouse_motion_events.iter() {
        if state.middle_button_down {
            transform.translation.x = transform.translation.x - event.delta.x * transform.scale.x;
            transform.translation.y = transform.translation.y + event.delta.y * transform.scale.y;
        }
    }

    // Edit the world
    let (x, y) = (state.world_position.x, state.world_position.y);
    if x > 0.0 && x < sandbox.width() as f32 && y > 0.0 && y < sandbox.height() as f32 {
        if state.left_button_down {
            toolbox.apply(&mut sandbox, x.floor() as usize, y.floor() as usize);
        } else if state.right_button_down {
            let element = toolbox.element;
            toolbox.element = Element::Air;
            toolbox.apply(&mut sandbox, x.floor() as usize, y.floor() as usize);
            toolbox.element = element;
        }
    }
}
