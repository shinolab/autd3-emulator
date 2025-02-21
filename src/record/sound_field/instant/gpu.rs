use std::{borrow::Cow, collections::VecDeque, time::Duration};

use crate::{EmulatorError, record::transducer::output_ultrasound::OutputUltrasound};

use autd3::{driver::defined::ULTRASOUND_PERIOD_COUNT, prelude::Point3};

use bytemuck::NoUninit;
use indicatif::ProgressBar;
use rayon::prelude::*;
use wgpu::{Buffer, BufferAddress, util::DeviceExt};

// GRCOV_EXCL_START
#[derive(NoUninit, Clone, Copy)]
#[repr(C)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
    _pad: f32,
}

impl From<Point3> for Vec3 {
    fn from(v: Point3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            _pad: 0.,
        }
    }
}

#[derive(NoUninit, Clone, Copy)]
#[repr(C)]
struct Pc {
    t: f32,
    sound_speed: f32,
    num_trans: u32,
    offset: i32,
    output_ultrasound_stride: u32,
    _pad: [u32; 3],
}
// GRCOV_EXCL_STOP

#[derive(Debug)]
pub(crate) struct Gpu<'a> {
    output_ultrasound: Vec<OutputUltrasound<'a>>,
    output_ultrasound_cache: Vec<VecDeque<f32>>,
    frame_window_size: usize,
    num_transducers: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    buf_staging_output_ultrasound: Buffer,
    buf_storage_output_ultrasound: Buffer,
    buf_output_ultrasound_size: BufferAddress,
    buf_storage_dst: Buffer,
    buf_staging_dst: Buffer,
    update_buf_output_ultrasound: bool,
    cache: Vec<Vec<f32>>,
}

impl<'a> Gpu<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        x: &[f32],
        y: &[f32],
        z: &[f32],
        transducer_positions: impl Iterator<Item = Point3>,
        output_ultrasound: Vec<OutputUltrasound<'a>>,
        frame_window_size: usize,
        num_points_in_frame: usize,
        cache_size: isize,
    ) -> Result<Self, EmulatorError> {
        let target_pos = itertools::izip!(x.iter(), y.iter(), z.iter())
            .map(|(&x, &y, &z)| Vec3 { x, y, z, _pad: 0. })
            .collect::<Vec<_>>();
        let transducer_pos = transducer_positions.map(Vec3::from).collect::<Vec<_>>();

        let buf_output_ultrasound_size = (output_ultrasound.len()
            * cache_size as usize
            * ULTRASOUND_PERIOD_COUNT
            * size_of::<f32>()) as BufferAddress;
        let buf_dst_size = (target_pos.len() * size_of::<f32>()) as BufferAddress;
        let buf_target_pos_size = (target_pos.len() * size_of::<Vec3>()) as BufferAddress;
        let buf_tr_pos_size = (transducer_pos.len() * size_of::<Vec3>()) as BufferAddress;

        let instance = wgpu::Instance::default();

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok_or(EmulatorError::NoSuitableAdapterFound)?;

        let (device, queue) = pollster::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: std::mem::size_of::<Pc>() as u32,
                        max_storage_buffers_per_shader_stage: 5,
                        max_storage_buffer_binding_size: buf_output_ultrasound_size
                            .max(buf_target_pos_size)
                            .max(buf_tr_pos_size)
                            as _,
                        ..wgpu::Limits::downlevel_defaults()
                    },
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            ),
        )?;

        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let buf_storage_target_pos = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&target_pos),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let buf_storage_trans_pos = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&transducer_pos),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let buf_staging_output_ultrasound = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buf_output_ultrasound_size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let buf_storage_output_ultrasound = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            size: buf_output_ultrasound_size,
            mapped_at_creation: false,
        });

        let buf_staging_dst = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buf_dst_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buf_storage_dst = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buf_dst_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_storage_output_ultrasound.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_storage_trans_pos.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_storage_target_pos.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_storage_dst.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<Pc>() as u32,
            }],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &cs_module,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            output_ultrasound,
            output_ultrasound_cache: Vec::new(),
            frame_window_size,
            num_transducers: transducer_pos.len() as _,
            device,
            queue,
            pipeline,
            bind_group,
            buf_staging_output_ultrasound,
            buf_storage_output_ultrasound,
            buf_output_ultrasound_size,
            update_buf_output_ultrasound: false,
            buf_storage_dst,
            buf_staging_dst,
            cache: vec![vec![0.0f32; target_pos.len()]; num_points_in_frame],
        })
    }

    pub(crate) fn init(&mut self, cache_size: isize, cursor: &mut isize, rem_frame: &mut usize) {
        if self.output_ultrasound_cache.is_empty() {
            self.output_ultrasound_cache = self
                .output_ultrasound
                .par_iter_mut()
                .map(|ut| {
                    (0..cache_size)
                        .flat_map(|i| {
                            if *cursor + i >= 0 {
                                ut._next(1)
                                    .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT])
                            } else {
                                vec![0.; ULTRASOUND_PERIOD_COUNT]
                            }
                        })
                        .collect()
                })
                .collect();
            *cursor += cache_size;
            *rem_frame = self.frame_window_size;
            self.update_buf_output_ultrasound = true;
            self.buf_output_ultrasound_size = (self
                .output_ultrasound_cache
                .iter()
                .map(|c| c.len())
                .sum::<usize>()
                * size_of::<f32>()) as _;
        }
    }

    pub(crate) fn progress(&mut self, cursor: &mut isize) {
        let n = match *cursor {
            c if (c + self.frame_window_size as isize) < 0 => 0,
            c if c >= 0 => {
                self.update_buf_output_ultrasound = true;
                self.frame_window_size
            }
            c => {
                self.update_buf_output_ultrasound = true;
                (c + self.frame_window_size as isize) as usize
            }
        };
        self.output_ultrasound_cache
            .iter_mut()
            .zip(self.output_ultrasound.iter_mut())
            .par_bridge()
            .for_each(|(cache, output_ultrasound)| {
                drop(cache.drain(0..ULTRASOUND_PERIOD_COUNT * n));
                (0..n).for_each(|_| {
                    cache.extend(
                        output_ultrasound
                            ._next(1)
                            .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT]),
                    );
                })
            });
        *cursor += self.frame_window_size as isize;
    }

    fn copy_output_ultrasound(&self) -> Result<(), EmulatorError> {
        let buffer_slice = self.buf_staging_output_ultrasound.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Write, move |r| sender.send(r).unwrap());
        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        receiver.recv()??;
        let src = self
            .output_ultrasound_cache
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        buffer_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&src));
        self.buf_staging_output_ultrasound.unmap();
        Ok(())
    }

    pub(crate) fn compute(
        &mut self,
        start_time: Duration,
        time_step: Duration,
        num_points_in_frame: usize,
        sound_speed: f32,
        offset: isize,
        pb: &ProgressBar,
    ) -> Result<&Vec<Vec<f32>>, EmulatorError> {
        for i in 0..num_points_in_frame {
            let t = (start_time + i as u32 * time_step).as_secs_f32();
            let pc = Pc {
                t,
                sound_speed,
                num_trans: self.num_transducers,
                offset: offset as _,
                output_ultrasound_stride: self.output_ultrasound_cache[0].len() as _,
                _pad: [0; 3],
            };

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            if self.update_buf_output_ultrasound {
                self.copy_output_ultrasound()?;
                encoder.copy_buffer_to_buffer(
                    &self.buf_staging_output_ultrasound,
                    0,
                    &self.buf_storage_output_ultrasound,
                    0,
                    self.buf_output_ultrasound_size,
                );
                self.update_buf_output_ultrasound = false;
            }

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &self.bind_group, &[]);
                cpass.set_push_constants(0, bytemuck::bytes_of(&pc));
                cpass.dispatch_workgroups(((self.cache[0].len() - 1) / 64 + 1) as _, 1, 1);
            }
            encoder.copy_buffer_to_buffer(
                &self.buf_storage_dst,
                0,
                &self.buf_staging_dst,
                0,
                (self.cache[0].len() * size_of::<f32>()) as _,
            );

            self.queue.submit(Some(encoder.finish()));

            let buffer_slice = self.buf_staging_dst.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
            receiver.recv()??;
            {
                let data = buffer_slice.get_mapped_range();
                self.cache[i].copy_from_slice(bytemuck::cast_slice(&data));
            }
            self.buf_staging_dst.unmap();
            pb.inc(1);
        }

        Ok(&self.cache)
    }
}
