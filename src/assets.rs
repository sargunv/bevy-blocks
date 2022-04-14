use bevy::{
    asset::LoadState,
    prelude::*,
    render::render_resource::{AddressMode, SamplerDescriptor},
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::LoadingAssets)
            .insert_resource(AssetHandles::default())
            .insert_resource(AssetsLoading::default())
            .add_startup_system(setup)
            .add_system_set(
                SystemSet::on_update(GameState::LoadingAssets)
                    .with_system(check_assets_ready),
            );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    LoadingAssets,
    Running,
}

#[derive(Default)]
pub struct AssetHandles {
    pub blocks_png: Handle<Image>,
}

#[derive(Default)]
struct AssetsLoading(Vec<HandleUntyped>);

fn setup(
    server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut handles: ResMut<AssetHandles>,
) {
    handles.blocks_png = server.load("images/blocks.png");
    loading.0.push(handles.blocks_png.clone_untyped());
}

fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut game_state: ResMut<State<GameState>>,
    mut imags: ResMut<Assets<Image>>,
    handles: Res<AssetHandles>,
) {
    match server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => {
            println!("failed loading assets");
        }
        LoadState::Loaded => {
            println!("loaded assets");

            let mut blocks_image = imags.get_mut(&handles.blocks_png).unwrap();
            blocks_image.reinterpret_stacked_2d_as_array(2);
            blocks_image.sampler_descriptor = SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                ..Default::default()
            };

            commands.remove_resource::<AssetsLoading>();
            game_state.set(GameState::Running).unwrap();
        }
        _ => {
            println!("loading assets");
        }
    }
}
