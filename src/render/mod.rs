mod camera;
mod mesh;
mod universe;

pub use camera::Camera;
pub use mesh::CubeSphere;
pub use universe::UniverseRenderer;

use log::info;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::Mutex;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference,
    Queue, RequestAdapterOptions,
};

/// An image id.
/// Images are a special subcategory of texture that
/// can be bound with image bind groups, and can only
/// be used in the fragment shader. This will eventually
/// be expanded to a full painter API.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ImageId(u64);

impl ImageId {
    pub fn from_raw_parts(id: u64) -> Self {
        Self(id)
    }

    pub fn to_egui(self) -> egui::TextureId {
        egui::TextureId::User(self.0)
    }
}

pub struct RenderContext {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    default_texture: wgpu::Texture,
    default_view: wgpu::TextureView,

    image_bind_group_layout: wgpu::BindGroupLayout,

    next_image_id: AtomicU64,
    images: Mutex<HashMap<u64, wgpu::BindGroup>>,
}

impl RenderContext {
    pub fn new() -> Self {
        futures::executor::block_on(async {
            info!("Initializing Renderer");

            info!("Initializing Instance");
            let instance = Instance::new(Backends::all());
            info!("Initializing Adapter");
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    compatible_surface: None,
                    force_fallback_adapter: false,
                    power_preference: PowerPreference::LowPower,
                })
                .await
                .unwrap();
            info!("Initializing Device");
            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        label: Some("Rendering GPU"),
                        features: Features::empty(),
                        limits: Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();

            let image_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            count: None,
                        },
                    ],
                });

            let default_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Default Texture"),
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::R8Uint,
                mip_level_count: 1,
                sample_count: 1,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
            });

            let default_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

            Self {
                instance,
                adapter,
                device,
                queue,
                default_texture,
                default_view,
                image_bind_group_layout,
                next_image_id: AtomicU64::new(0),
                images: Mutex::new(HashMap::default()),
            }
        })
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn default_texture(&self) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Texture"),
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::R8Uint,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        })
    }

    /// Creates a new texture ment to be used solely for being a depth attachment.
    /// It has 1 mip and only 1 layer.
    pub fn create_depth_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        return self.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Attachment"),
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            mip_level_count: 1,
            sample_count,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
    }

    pub fn create_hdr_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
        texture_binding: bool,
    ) -> wgpu::Texture {
        let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        if texture_binding {
            usage.insert(wgpu::TextureUsages::TEXTURE_BINDING);
        }

        return self.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Attachment"),
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            mip_level_count: 1,
            sample_count,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            usage,
        });
    }

    pub fn create_ldr_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
        texture_binding: bool,
    ) -> wgpu::Texture {
        let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        if texture_binding {
            usage.insert(wgpu::TextureUsages::TEXTURE_BINDING);
        }

        return self.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Attachment"),
            dimension: wgpu::TextureDimension::D2,
            format: LDR_FORMAT,
            mip_level_count: 1,
            sample_count,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            usage,
        });
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        DEPTH_FORMAT
    }

    pub fn hdr_format(&self) -> wgpu::TextureFormat {
        HDR_FORMAT
    }

    pub fn ldr_format(&self) -> wgpu::TextureFormat {
        LDR_FORMAT
    }

    /// Registers a `wgpu::Texture` with a `ImageId`.
    ///
    /// This enables the application to reference the texture inside an image ui element.
    /// This effectively enables off-screen rendering inside the egui UI. Texture must have
    /// the texture format `TextureFormat::Rgba8Unorm` and
    /// Texture usage `TextureUsage::SAMPLED`.
    pub fn register_image(&self, texture: &wgpu::Texture, filter: wgpu::FilterMode) -> ImageId {
        let id = self
            .next_image_id
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        let sampler = self.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{}_image_sampler", id).as_str()),
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        });

        // We've bound it here, so that we don't add it as a pending texture.
        let bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{}_image_bind_group", id).as_str()),
            layout: &self.image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let mut images = self.images.lock().unwrap();
        images.insert(id, bind_group);
        drop(images);

        ImageId(id)
    }

    pub fn image_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.image_bind_group_layout
    }

    pub fn get_image_bind_groups(&self) -> &Mutex<HashMap<u64, wgpu::BindGroup>> {
        &self.images
    }

    pub fn unregister_image(&self, image: ImageId) {
        let mut images = self.images.lock().unwrap();
        images.remove(&image.0);
        drop(images);
    }
}

#[derive(Clone)]
pub struct RenderCtxRef {
    inner: Arc<RenderContext>,
}

impl RenderCtxRef {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RenderContext::new()),
        }
    }

    pub fn adapter(&self) -> &Adapter {
        self.inner.adapter()
    }

    pub fn device(&self) -> &Device {
        self.inner.device()
    }

    pub fn instance(&self) -> &Instance {
        self.inner.instance()
    }

    pub fn queue(&self) -> &Queue {
        self.inner.queue()
    }

    pub fn default_texture(&self) -> wgpu::Texture {
        self.inner.default_texture()
    }

    pub fn create_depth_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        self.inner
            .create_depth_attachment(width, height, sample_count)
    }

    pub fn create_hdr_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
        texture_binding: bool,
    ) -> wgpu::Texture {
        self.inner
            .create_hdr_attachment(width, height, sample_count, texture_binding)
    }

    pub fn create_ldr_attachment(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
        texture_binding: bool,
    ) -> wgpu::Texture {
        self.inner
            .create_ldr_attachment(width, height, sample_count, texture_binding)
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        self.inner.depth_format()
    }

    pub fn hdr_format(&self) -> wgpu::TextureFormat {
        self.inner.hdr_format()
    }

    pub fn ldr_format(&self) -> wgpu::TextureFormat {
        self.inner.ldr_format()
    }

    pub fn register_image(&self, texture: &wgpu::Texture, filter: wgpu::FilterMode) -> ImageId {
        self.inner.register_image(texture, filter)
    }

    pub fn image_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.inner.image_bind_group_layout()
    }

    pub fn get_image_bind_groups(&self) -> &Mutex<HashMap<u64, wgpu::BindGroup>> {
        self.inner.get_image_bind_groups()
    }

    pub fn unregister_image(&self, image: ImageId) {
        self.inner.unregister_image(image)
    }
}

/// Represents
pub struct RenderTarget {
    pub format: wgpu::TextureFormat,
}

pub struct RenderTargetView<'a> {
    pub view: &'a wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

/// Format for all depth buffers.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
/// Format for low dynamic range color buffers.
/// This is a format renderable to (and viewable from) the moniter.
pub const LDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Format for high dynamic range color buffers.
/// This format is usually unrepresentable on a moniter, and must be tone-mapped to LDR.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
