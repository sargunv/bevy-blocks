#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{PrimitiveTopology, WgpuFeatures},
        settings::WgpuSettings,
    },
};
use block_mesh::ilattice::glam::Vec3A;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility,
    RIGHT_HANDED_Y_UP_CONFIG,
};

use crate::voxel_material::VoxelMaterial;

mod assets;
mod diagnostics;
mod pan_orbit_camera;
mod voxel_material;

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(diagnostics::DiagnosticsPlugin)
        .add_plugin(voxel_material::VoxelMaterialPlugin)
        .add_plugin(pan_orbit_camera::PanOrbitCameraPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_system_set(
            SystemSet::on_enter(assets::GameState::Running).with_system(setup),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut voxel_materials: ResMut<Assets<voxel_material::VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_handles: Res<assets::AssetHandles>,
) {
    let greedy_sphere_mesh = generate_greedy_mesh(&mut meshes, |p| {
        VoxelState(if p.length() < 0.9 { 1 } else { 0 })
    });

    commands.spawn().insert_bundle(MaterialMeshBundle {
        mesh: greedy_sphere_mesh,
        transform: Transform::from_translation(Vec3::new(-17., -17., -17.)),
        material: voxel_materials.add(voxel_material::VoxelMaterial {
            base_color_texture: Some(asset_handles.blocks_png.clone()),
        }),
        ..Default::default()
    });

    pan_orbit_camera::spawn_camera(commands);
}

fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    generator_fn: impl Fn(Vec3A) -> VoxelState,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let sample_pos = SampleShape::delinearize(i);
        let [sx, sy, sz] = sample_pos;
        let unit_pos = (2.0 / 32 as f32)
            * Vec3A::new(sx as f32, sy as f32, sz as f32)
            - 1.0;
        samples[i as usize] = generator_fn(unit_pos);
    }

    let coord_config = RIGHT_HANDED_Y_UP_CONFIG;

    let mut quad_buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &SampleShape {},
        [0; 3],
        [33; 3],
        &coord_config.faces,
        &mut quad_buffer,
    );

    let num_quads = quad_buffer.quads.num_quads();
    let num_indices = 6 * num_quads;
    let num_vertices = 4 * num_quads;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut colors = Vec::with_capacity(num_vertices);

    for (group, face) in quad_buffer
        .quads
        .groups
        .into_iter()
        .zip(coord_config.faces.into_iter())
    {
        for quad in group.into_iter() {
            indices.extend_from_slice(
                &face.quad_mesh_indices(positions.len() as u32),
            );
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                coord_config.u_flip_face,
                true,
                &quad,
            ));
            colors.extend([1; 4]);
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(tex_coords),
    );
    render_mesh.insert_attribute(
        VoxelMaterial::ATTRIBUTE_LAYER,
        VertexAttributeValues::Sint32(colors),
    );
    render_mesh.set_indices(Some(Indices::U32(indices)));

    meshes.add(render_mesh)
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct VoxelState(u16);
const EMPTY: VoxelState = VoxelState(0);

impl Voxel for VoxelState {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for VoxelState {
    type MergeValue = bool;

    fn merge_value(&self) -> Self::MergeValue {
        *self == EMPTY
    }
}
