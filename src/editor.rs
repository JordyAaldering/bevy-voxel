mod editor_state;
mod editor_tab;
mod view_target;

pub use editor_state::EditorState;
pub use editor_tab::{EditorTab, EditorTabViewer};
pub use view_target::ViewTarget;

use crate::{EditorCamera, PlayerCamera};

use bevy::{
    prelude::*,
    image::Image,
    render::render_resource::{Extent3d, TextureFormat},
    window::PrimaryWindow,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiTextureHandle, EguiUserTextures, PrimaryEguiContext},
    egui,
};

pub fn show_editor(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
    else {
        return;
    };

    let ctx = egui_context.get_mut();

    let mut ui = egui::Ui::new(
        ctx.clone(),
        "editor_viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    world.resource_scope::<EditorState, _>(
        |world, mut editor_state| {
            editor_state.ui(&mut ui, world);
        },
    );
}

pub fn is_editor_view_visible(editor_state: Res<EditorState>) -> bool {
    editor_state.is_editor_view_visible()
}

pub fn setup_view_targets(
    mut editor_state: ResMut<EditorState>,
    mut images: ResMut<Assets<Image>>,
    mut egui_textures: ResMut<EguiUserTextures>,
) {
    editor_state.set_game_view(create_view_target(&mut images, &mut egui_textures));
    editor_state.set_editor_view(create_view_target(&mut images, &mut egui_textures));
}

fn create_view_target(images: &mut Assets<Image>, egui_textures: &mut EguiUserTextures) -> ViewTarget {
    let image = images.add(Image::new_target_texture(
        1280,
        720,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let texture = egui_textures.add_image(EguiTextureHandle::Strong(image.clone()));
    ViewTarget::new(image, texture)
}

/// Resizes each camera's image to match its tab, and disables cameras whose tab is hidden
pub fn update_view_targets(
    editor_state: Res<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut game_camera: Single<&mut Camera, (With<PlayerCamera>, Without<EditorCamera>)>,
    mut editor_camera: Single<&mut Camera, (With<EditorCamera>, Without<PlayerCamera>)>,
) {
    let scale_factor = window.scale_factor();
    update_view_target(editor_state.get_game_view(), scale_factor, &mut images, &mut game_camera);
    update_view_target(editor_state.get_editor_view(), scale_factor, &mut images, &mut editor_camera);
}

fn update_view_target(
    view: &ViewTarget,
    scale_factor: f32,
    images: &mut Assets<Image>,
    camera: &mut Camera,
) {
    let Some(size) = view.try_resize(scale_factor) else {
        camera.is_active = false;
        return;
    };

    camera.is_active = true;

    if let Some(mut image) = images.get_mut(view.id()) {
        if image.size() != size {
            image.resize(Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            });
        }
    }
}
