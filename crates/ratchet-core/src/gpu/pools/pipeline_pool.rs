use std::borrow::Cow;

use crate::{gpu::WgpuDevice, kernels, KernelKey, KernelSourceHandle};

use super::{
    PipelineLayoutHandle, StaticResourcePool, StaticResourcePoolAccessor,
    StaticResourcePoolReadLockAccessor,
};

slotmap::new_key_type! { pub struct ComputePipelineHandle; }

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComputePipelineDescriptor {
    pub pipeline_layout: PipelineLayoutHandle,
    pub kernel_key: KernelKey,
    pub compute_module: Option<KernelSourceHandle>,
}

pub struct ComputePipelinePool {
    inner:
        StaticResourcePool<ComputePipelineHandle, ComputePipelineDescriptor, wgpu::ComputePipeline>,
}

impl Default for ComputePipelinePool {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputePipelinePool {
    pub fn new() -> Self {
        Self {
            inner: StaticResourcePool::default(),
        }
    }

    pub fn get_or_create(
        &self,
        desc: &ComputePipelineDescriptor,
        device: &WgpuDevice,
    ) -> ComputePipelineHandle {
        self.inner.get_or_create(desc, |desc| {
            let label = Some(desc.kernel_key.as_str());
            let kernel_resources = device.kernel_source_resources();

            let shader_source = if let Some(source) = desc.compute_module {
                let kernel_source = kernel_resources.get(source).unwrap();
                wgpu::ShaderSource::Wgsl(kernel_source.0.clone())
            } else {
                let shader = kernels()
                    .get(desc.kernel_key.as_str())
                    .unwrap_or_else(|| panic!("Kernel {} not found", desc.kernel_key));
                wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader))
            };

            let shader_module_desc = wgpu::ShaderModuleDescriptor {
                label,
                source: shader_source,
            };

            let module = if std::env::var("RATCHET_CHECKED").is_ok() {
                log::warn!("Using checked shader compilation");
                device.create_shader_module(shader_module_desc)
            } else {
                unsafe { device.create_shader_module_unchecked(shader_module_desc) }
            };

            let pipeline_layouts = device.pipeline_layout_resources();
            let pipeline_layout = pipeline_layouts.get(desc.pipeline_layout).unwrap();
            println!("MODULE: {:?}", module);

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label,
                layout: Some(pipeline_layout),
                module: &module,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions {
                    zero_initialize_workgroup_memory: false,
                    ..Default::default()
                },
                cache: None,
            })
        })
    }

    /// Locks the resource pool for resolving handles.
    ///
    /// While it is locked, no new resources can be added.
    pub fn resources(
        &self,
    ) -> StaticResourcePoolReadLockAccessor<'_, ComputePipelineHandle, wgpu::ComputePipeline> {
        self.inner.resources()
    }
}
