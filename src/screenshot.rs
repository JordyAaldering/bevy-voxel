use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use better_default::Default;
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};

#[derive(Default)]
pub struct ScreenshotPlugin(
    #[default("docs/img".into())]
    String,
);

#[derive(Resource)]
pub struct ScreenshotConfig(PathBuf);

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenshotConfig(PathBuf::from(&self.0)))
            .add_systems(Update, take_screenshot);
    }
}

/// Captures the whole window, including egui and the dock, when F12 is pressed.
pub fn take_screenshot(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<ScreenshotConfig>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if !keys.just_pressed(KeyCode::F12) {
        return Ok(());
    }

    std::fs::create_dir_all(&config.0)
        .context(format!("Failed to create {}", config.0.display()))?;

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let path = config.0.join(format!("screenshot_{secs}.png"));
    info!("Saving screenshot to {}", path.display());

    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));

    Ok(())
}
