use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WireframePlugin)
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup);
    }
}

fn setup(mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = false;
}
