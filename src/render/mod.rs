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

pub struct Renderer {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    image_bind_group_layout: wgpu::BindGroupLayout,

    next_image_id: AtomicU64,
    images: Mutex<HashMap<u64, wgpu::BindGroup>>,
}

impl Renderer {
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

            Self {
                instance,
                adapter,
                device,
                queue,
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
pub struct RenderHandle {
    inner: Arc<Renderer>,
}

impl RenderHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Renderer::new()),
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

    pub fn register_image(&self, texture: &wgpu::Texture, filter: wgpu::FilterMode) -> ImageId {
        self.inner.register_image(texture, filter)
    }

    pub fn image_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.image_bind_group_layout()
    }

    pub fn get_image_bind_groups(&self) -> &Mutex<HashMap<u64, wgpu::BindGroup>> {
        self.inner.get_image_bind_groups()
    }

    pub fn unregister_image(&self, image: ImageId) {
        self.inner.unregister_image(image)
    }
}
