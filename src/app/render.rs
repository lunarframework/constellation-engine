use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference,
    Queue, RequestAdapterOptions,
};

use log::info;

pub struct Renderer {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
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

            Self {
                instance,
                adapter,
                device,
                queue,
            }
        })

        // futures::executor::block_on(async {
        //     info!("Initializing Renderer");

        //     info!("Initializing Instance");
        //     let instance = Instance::new(Backends::all());
        //     info!("Initializing Adapter");
        //     let adapter = instance
        //         .enumerate_adapters(wgpu::Backends::DX12)
        //         .next()
        //         .unwrap();
        //     info!("Initializing Device");
        //     let (device, queue) = adapter
        //         .request_device(
        //             &DeviceDescriptor {
        //                 label: Some("Rendering GPU"),
        //                 features: Features::empty(),
        //                 limits: Limits::default(),
        //             },
        //             None,
        //         )
        //         .await
        //         .unwrap();

        //     Self {
        //         instance,
        //         adapter,
        //         device,
        //         queue,
        //     }
        // })
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
}
