use std::any::TypeId;

use bevy::{
    prelude::*,
    camera::RenderTarget,
    image::Image,
    render::render_resource::{Extent3d, TextureFormat},
    window::PrimaryWindow,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiTextureHandle, EguiUserTextures, PrimaryEguiContext},
    bevy_inspector::{
        self,
        hierarchy::{SelectedEntities, hierarchy_ui},
        ui_for_entities_shared_components,
        ui_for_entity_with_children,
}};
use egui_dock::{DockArea, DockState, NodeIndex, Style, egui};

#[derive(Clone, Copy, Debug)]
pub enum EditorTab {
    GameView,
    EditorView,
    Hierarchy,
    Resources,
    Inspector,
}

/// An offscreen image a camera renders to, which is displayed inside a dock tab.
#[derive(Default)]
struct ViewTarget {
    image: Handle<Image>,
    texture: egui::TextureId,
    /// The area the tab occupied this frame, or `None` if the tab is not visible.
    rect: Option<egui::Rect>,
}

impl ViewTarget {
    fn show(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        self.rect = Some(ui.available_rect_before_wrap());
        ui.add(egui::Image::new(egui::load::SizedTexture::new(self.texture, size)));
    }
}

#[derive(Resource)]
pub struct EditorState {
    state: DockState<EditorTab>,
    active_tab: Option<EditorTab>,
    game_view: ViewTarget,
    editor_view: ViewTarget,
    selected_entities: SelectedEntities,
    selected_resource: Option<(TypeId, String)>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut state = DockState::new(vec![EditorTab::GameView, EditorTab::EditorView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] = tree.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EditorTab::Hierarchy]);
        let [_game, _bottom] = tree.split_below(game, 0.8, vec![EditorTab::Resources]);
        Self {
            state,
            active_tab: None,
            game_view: ViewTarget::default(),
            editor_view: ViewTarget::default(),
            selected_entities: SelectedEntities::default(),
            selected_resource: None,
        }
    }
}

struct EditorTabViewer<'w> {
    world: &'w mut World,
    game_view: &'w mut ViewTarget,
    editor_view: &'w mut ViewTarget,
    selected_entities: &'w mut SelectedEntities,
    selected_resource: &'w mut Option<(TypeId, String)>,
}

impl egui_dock::TabViewer for EditorTabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::GameView => "Game".into(),
            EditorTab::EditorView => "Editor".into(),
            EditorTab::Hierarchy => "Hierarchy".into(),
            EditorTab::Resources => "Resources".into(),
            EditorTab::Inspector => "Inspector".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::GameView => self.game_view.show(ui),
            EditorTab::EditorView => self.editor_view.show(ui),
            EditorTab::Hierarchy => {
                if hierarchy_ui(self.world, ui, self.selected_entities) {
                    *self.selected_resource = None;
                }
            }
            EditorTab::Resources => {
                let registry = self
                    .world
                    .resource::<AppTypeRegistry>()
                    .0
                    .clone();
                let registry = registry.read();

                let mut resources: Vec<_> = registry
                    .iter()
                    .filter(|registration| {
                        registration.data::<ReflectResource>().is_some()
                    })
                    .map(|registration| {
                        (
                            registration
                                .type_info()
                                .type_path_table()
                                .short_path()
                                .to_owned(),
                            registration.type_id(),
                        )
                    })
                    .collect();

                resources.sort_by(|a, b| a.0.cmp(&b.0));

                for (name, type_id) in resources {
                    let selected = self
                        .selected_resource
                        .as_ref()
                        .is_some_and(|(selected_id, _)| {
                            *selected_id == type_id
                        });

                    if ui.selectable_label(selected, &name).clicked() {
                        *self.selected_resource =
                            Some((type_id, name));
                        self.selected_entities.clear();
                    }
                }
            }

            EditorTab::Inspector => {
                let Some((type_id, name)) =
                    self.selected_resource.as_ref()
                else {
                    match self.selected_entities.as_slice() {
                        &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                        &[] => { ui.label("Select an entity or resource to inspect."); }
                        entities => ui_for_entities_shared_components(self.world, entities, ui),
                    }
                    return;
                };

                ui.heading(name);

                let registry = self
                    .world
                    .resource::<AppTypeRegistry>()
                    .0
                    .clone();
                let registry = registry.read();

                bevy_inspector::by_type_id::ui_for_resource(
                    self.world,
                    *type_id,
                    ui,
                    name,
                    &registry,
                );
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(format!("{tab:?}"))
    }
}

impl EditorState {
    pub fn is_game_view_visible(&self) -> bool {
        self.game_view.rect.is_some()
    }

    /// Whether the game view is the tab that was last clicked in.
    pub fn is_game_view_active(&self) -> bool {
        matches!(self.active_tab, Some(EditorTab::GameView))
    }

    pub fn is_editor_view_active(&self) -> bool {
        matches!(self.active_tab, Some(EditorTab::EditorView))
    }

    /// `position` is in logical window coordinates, which match egui's points.
    pub fn game_view_contains(&self, position: Vec2) -> bool {
        self.game_view.rect.is_some_and(|rect| rect.contains(egui::pos2(position.x, position.y)))
    }

    pub fn editor_view_contains(&self, position: Vec2) -> bool {
        self.editor_view.rect.is_some_and(|rect| rect.contains(egui::pos2(position.x, position.y)))
    }

    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        // Only tabs that are visible this frame report a rect
        self.game_view.rect = None;
        self.editor_view.rect = None;

        let mut viewer = EditorTabViewer {
            world,
            game_view: &mut self.game_view,
            editor_view: &mut self.editor_view,
            selected_entities: &mut self.selected_entities,
            selected_resource: &mut self.selected_resource,
        };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(&ui.global_style()))
            .show_inside(ui, &mut viewer);

        // Keep the last focused tab, so clicking empty space doesn't deactivate it
        if let Some((_, tab)) = self.state.find_active_focused() {
            self.active_tab = Some(tab.clone());
        }
    }
}

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
    editor_state.editor_view.rect.is_some()
}

fn create_view_target(images: &mut Assets<Image>, egui_textures: &mut EguiUserTextures) -> ViewTarget {
    let image = images.add(Image::new_target_texture(
        1280,
        720,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let texture = egui_textures.add_image(EguiTextureHandle::Strong(image.clone()));
    ViewTarget { image, texture, rect: None }
}

pub fn setup_view_targets(
    mut editor_state: ResMut<EditorState>,
    mut images: ResMut<Assets<Image>>,
    mut egui_textures: ResMut<EguiUserTextures>,
) {
    editor_state.game_view = create_view_target(&mut images, &mut egui_textures);
    editor_state.editor_view = create_view_target(&mut images, &mut egui_textures);
}

pub fn game_camera_target(editor_state: &EditorState) -> RenderTarget {
    editor_state.game_view.image.clone().into()
}

pub fn editor_camera_target(editor_state: &EditorState) -> RenderTarget {
    editor_state.editor_view.image.clone().into()
}

/// Resizes each camera's image to match its tab, and disables cameras whose tab is hidden
pub fn update_view_targets(
    editor_state: Res<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut game_camera: Single<&mut Camera, (With<crate::PlayerCamera>, Without<crate::EditorCamera>)>,
    mut editor_camera: Single<&mut Camera, (With<crate::EditorCamera>, Without<crate::PlayerCamera>)>,
) {
    let scale_factor = window.scale_factor();
    update_view_target(&editor_state.game_view, scale_factor, &mut images, &mut game_camera);
    update_view_target(&editor_state.editor_view, scale_factor, &mut images, &mut editor_camera);
}

fn update_view_target(
    view: &ViewTarget,
    scale_factor: f32,
    images: &mut Assets<Image>,
    camera: &mut Camera,
) {
    let size = view.rect.map(|rect| {
        let size = rect.size() * scale_factor;
        UVec2::new(size.x as u32, size.y as u32)
    });

    let Some(size) = size.filter(|size| size.x > 0 && size.y > 0) else {
        camera.is_active = false;
        return;
    };

    camera.is_active = true;

    if let Some(mut image) = images.get_mut(&view.image) {
        if image.size() != size {
            image.resize(Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            });
        }
    }
}
