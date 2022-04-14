use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
            BindingType, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
            TextureViewDimension, VertexFormat,
        },
        renderer::RenderDevice,
    },
};

pub struct VoxelMaterialPlugin;

impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<VoxelMaterial>::default());
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "54bc975d-65a0-43ff-9dc4-883ffeb21ba6"]
pub struct VoxelMaterial {
    pub base_color_texture: Option<Handle<Image>>,
}

#[derive(Clone)]
pub struct GpuVoxelMaterial {
    bind_group: BindGroup,
}

impl VoxelMaterial {
    pub const ATTRIBUTE_LAYER: MeshVertexAttribute = MeshVertexAttribute::new(
        "Vertex_Layer",
        0x00A30D49AEC8477D,
        VertexFormat::Sint32,
    );
}

impl RenderAsset for VoxelMaterial {
    type ExtractedAsset = VoxelMaterial;
    type PreparedAsset = GpuVoxelMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<MaterialPipeline<Self>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline, gpu_images): &mut SystemParamItem<
            Self::Param,
        >,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let (base_color_texture_view, base_color_sampler) =
            if let Some(result) =
                material_pipeline.mesh_pipeline.get_image_texture(
                    gpu_images,
                    &extracted_asset.base_color_texture,
                )
            {
                result
            } else {
                return Err(PrepareAssetError::RetryNextUpdate(
                    extracted_asset,
                ));
            };

        let bind_group =
            render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            base_color_texture_view,
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(base_color_sampler),
                    },
                ],
                label: Some("voxel_material_bind_group"),
                layout: &material_pipeline.material_layout,
            });

        Ok(GpuVoxelMaterial { bind_group })
    }
}

impl Material for VoxelMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/voxels.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/voxels.wgsl"))
    }

    fn bind_group(
        render_asset: &<Self as RenderAsset>::PreparedAsset,
    ) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("voxel_material_layout"),
        })
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Self::ATTRIBUTE_LAYER.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
