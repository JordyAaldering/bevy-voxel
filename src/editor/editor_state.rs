use super::{EditorTab, EditorTabViewer, ViewTarget};

use std::any::TypeId;

use bevy::{
    prelude::*,
    camera::RenderTarget,
};
use bevy_inspector_egui::{
    bevy_inspector::hierarchy::SelectedEntities,
    egui,

};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

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

impl EditorState {
    pub fn set_game_view(&mut self, view: ViewTarget) {
        self.game_view = view;
    }

    pub fn set_editor_view(&mut self, view: ViewTarget) {
        self.editor_view = view;
    }

    pub fn get_game_view(&self) -> &ViewTarget {
        &self.game_view
    }

    pub fn get_editor_view(&self) -> &ViewTarget {
        &self.editor_view
    }

    pub fn is_game_view_visible(&self) -> bool {
        self.game_view.is_visible()
    }

    pub fn is_editor_view_visible(&self) -> bool {
        self.editor_view.is_visible()
    }

    /// Whether the game view is the tab that was last clicked in.
    pub fn is_game_view_active(&self) -> bool {
        self.active_tab == Some(EditorTab::GameView)
    }

    pub fn is_editor_view_active(&self) -> bool {
        self.active_tab == Some(EditorTab::EditorView)
    }

    pub fn game_view_contains(&self, position: Vec2) -> bool {
        self.game_view.contains(position)
    }

    pub fn editor_view_contains(&self, position: Vec2) -> bool {
        self.editor_view.contains(position)
    }

    pub fn game_camera_target(&self) -> RenderTarget {
        self.game_view.target()
    }

    pub fn editor_camera_target(&self) -> RenderTarget {
        self.editor_view.target()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        // Only tabs that are visible this frame report a rect
        self.game_view.clear();
        self.editor_view.clear();

        let mut viewer = EditorTabViewer::new(
            world,
            &mut self.game_view,
            &mut self.editor_view,
            &mut self.selected_entities,
            &mut self.selected_resource,
        );

        DockArea::new(&mut self.state)
            .style(Style::from_egui(&ui.global_style()))
            .show_inside(ui, &mut viewer);

        // Keep the last focused tab, so clicking empty space doesn't deactivate it
        if let Some((_, tab)) = self.state.find_active_focused() {
            self.active_tab = Some(tab.clone());
        }
    }
}
