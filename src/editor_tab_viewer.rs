use bevy::{camera::Viewport, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{bevy_egui::{EguiContext, PrimaryEguiContext}, bevy_inspector::{self, hierarchy::{SelectedEntities, hierarchy_ui}, ui_for_entities_shared_components, ui_for_entity_with_children}};
use egui_dock::{DockArea, DockState, NodeIndex, Style, egui};
use std::any::TypeId;

#[derive(Clone, Debug)]
pub enum EditorTab {
    GameView,
    Hierarchy,
    Resources,
    Inspector,
}

#[derive(Resource)]
pub struct EditorState {
    state: DockState<EditorTab>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selected_resource: Option<(TypeId, String)>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut state = DockState::new(vec![EditorTab::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EditorTab::Hierarchy]);
        let [_game, _bottom] = tree.split_below(game, 0.8, vec![EditorTab::Resources]);

        Self {
            state,
            viewport_rect: egui::Rect::NOTHING,
            selected_entities: SelectedEntities::default(),
            selected_resource: None,
        }
    }
}

struct EditorTabViewer<'w> {
    world: &'w mut World,
    viewport_rect: &'w mut egui::Rect,
    selected_entities: &'w mut SelectedEntities,
    selected_resource: &'w mut Option<(TypeId, String)>,
}

impl egui_dock::TabViewer for EditorTabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::GameView => "Game".into(),
            EditorTab::Hierarchy => "Hierarchy".into(),
            EditorTab::Resources => "Resources".into(),
            EditorTab::Inspector => "Inspector".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::GameView => *self.viewport_rect = ui.clip_rect(),
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

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::GameView)
    }
}

impl EditorState {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut viewer = EditorTabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selected_resource: &mut self.selected_resource,
        };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(&ui.global_style()))
            .show_inside(ui, &mut viewer);
    }
}

pub fn show_editor(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<
            &mut EguiContext,
            With<PrimaryEguiContext>,
        >()
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

/// Makes the game camera only render to the part of the window not obstructed by the UI.
pub fn set_camera_viewport(
    editor_state: Res<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<&mut Camera, Without<PrimaryEguiContext>>,
) {
    let scale_factor = window.scale_factor();
    let position = editor_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let size = editor_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(position.x as u32, position.y as u32);
    let physical_size = UVec2::new(size.x as u32, size.y as u32);

    let window_size = window.physical_size();
    let end = physical_position + physical_size;
    if physical_size.x > 0 && physical_size.y > 0 && end.x <= window_size.x && end.y <= window_size.y {
        camera.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}
