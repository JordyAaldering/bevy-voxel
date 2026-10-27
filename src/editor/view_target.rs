use bevy::{
    prelude::*,
    camera::RenderTarget,
};
use bevy_inspector_egui::egui;

/// An offscreen image a camera renders to, which is displayed inside a dock tab.
#[derive(Default)]
pub struct ViewTarget {
    image: Handle<Image>,
    texture: egui::TextureId,
    /// The area the tab occupied this frame, or `None` if the tab is not visible.
    rect: Option<egui::Rect>,
}

impl ViewTarget {
    pub fn new(image: Handle<Image>, texture: egui::TextureId) -> Self {
        Self { image, texture, rect: None }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        self.rect = Some(ui.available_rect_before_wrap());
        ui.add(egui::Image::new(egui::load::SizedTexture::new(self.texture, size)));
    }

    pub fn clear(&mut self) {
        self.rect = None;
    }

    pub fn is_visible(&self) -> bool {
        self.rect.is_some()
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        self.rect.is_some_and(|rect| rect.contains(egui::pos2(pos.x, pos.y)))
    }

    pub fn try_resize(&self, scale_factor: f32) -> Option<UVec2> {
        self.rect.map(|rect| {
            let size = rect.size() * scale_factor;
            UVec2::new(size.x as u32, size.y as u32)
        }).filter(|size| size.x > 0 && size.y > 0)
    }

    pub fn target(&self) -> RenderTarget {
        self.image.clone().into()
    }

    pub fn id(&self) -> AssetId<Image> {
        self.image.id()
    }
}
