use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferSize, BufferUsages, ShaderStages,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline, Material2dPlugin},
};

pub struct SoSmoothPlugin;

impl Plugin for SoSmoothPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<CustomMaterial>::default());
    }
}

#[derive(Debug, Clone, TypeUuid, AsStd140)]
#[uuid = "200dfa11-9b87-40e1-8774-ffcd3fcb17df"]
pub struct CustomMaterial {
    pub color: Vec4,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl Material2d for CustomMaterial {
    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &bevy::render::renderer::RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(CustomMaterial::std140_size_static() as u64),
                },
                count: None, // TODO: what?
            }],
            label: None, // TODO: WHAT?
        })
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("custom_material.wgsl"))
    }
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let custom_material_std140 = extracted_asset.as_std140();
        let custom_material_bytes = custom_material_std140.as_bytes();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: custom_material_bytes,
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}
