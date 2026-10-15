use bevy::ecs::{query::With, reflect::{AppTypeRegistry, ReflectResource}, resource::Resource, world::World};
use bevy_inspector_egui::{bevy_egui::{EguiContext, PrimaryEguiContext}, bevy_inspector};
use egui_dock::{DockArea, DockState, NodeIndex, Style, egui};
use std::any::TypeId;

#[derive(Clone, Debug)]
pub enum EditorTab {
    Resources,
    Inspector,
}

#[derive(Resource)]
pub struct EditorState {
    state: DockState<EditorTab>,
    selected_resource: Option<(TypeId, String)>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut dock_state =
            DockState::new(vec![EditorTab::Resources]);

        dock_state
            .main_surface_mut()
            .split_right(
                NodeIndex::root(),
                0.7,
                vec![EditorTab::Inspector],
            );

        Self {
            state: dock_state,
            selected_resource: None,
        }
    }
}

struct EditorTabViewer<'w> {
    world: &'w mut World,
    selected_resource: &'w mut Option<(TypeId, String)>,
}

impl egui_dock::TabViewer for EditorTabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Resources => "Resources".into(),
            EditorTab::Inspector => "Inspector".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
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
                    }
                }
            }

            EditorTab::Inspector => {
                let Some((type_id, name)) =
                    self.selected_resource.as_ref()
                else {
                    ui.label("Select a resource to inspect.");
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
        todo!()
    }
}

impl EditorState {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut viewer = EditorTabViewer {
            world,
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
