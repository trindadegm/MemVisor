use egui_wgpu::wgpu::{TextureFormat, TextureView};
use egui_wgpu::{ScreenDescriptor, wgpu};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct EguiRenderer {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    frame_started: bool,
}

pub struct RendererResources<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub window_surface_view: &'a TextureView,
    pub screen_descriptor: ScreenDescriptor,
}

impl EguiRenderer {
    pub fn context(&self) -> &egui::Context {
        self.state.egui_ctx()
    }

    pub fn new(
        device: &wgpu::Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        let egui_ctx = egui::Context::default();

        let egui_state = egui_winit::State::new(
            egui_ctx,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            egui_wgpu::RendererOptions {
                depth_stencil_format: output_depth_format,
                msaa_samples,
                ..Default::default()
            },
        );

        Self {
            state: egui_state,
            renderer: egui_renderer,
            frame_started: false,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn set_pixels_per_point(&self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    pub fn begin_ui_frame(
        &mut self,
        window: &Window,
        mut ui_fn: impl FnMut(&egui::Context, &mut egui::Ui),
    ) -> egui::FullOutput {
        let input = self.state.take_egui_input(window);
        let ctx = self.state.egui_ctx();
        let full_output = ctx.run_ui(input, |ui| ui_fn(ctx, ui));

        self.frame_started = true;

        full_output
    }

    // pub fn begin_frame(&mut self, window: &Window) {
    //     let input = self.state.take_egui_input(window);
    //     self.state.egui_ctx().begin_pass(input);
    //     self.frame_started = true;
    // }

    pub fn draw_frame(
        &mut self,
        RendererResources {
            device,
            queue,
            encoder,
            window,
            window_surface_view,
            screen_descriptor,
        }: RendererResources,
        ui_output: egui::FullOutput,
    ) {
        let _span = tracy_client::span!("egui_draw_frame");
        assert!(
            self.frame_started,
            "frame must be started before being drawn"
        );

        self.set_pixels_per_point(screen_descriptor.pixels_per_point);

        self.state
            .handle_platform_output(window, ui_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(ui_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, img_delta) in &ui_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, img_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui_draw_frame_render_pass"),
            occlusion_query_set: None,
            multiview_mask: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &ui_output.textures_delta.free {
            self.renderer.free_texture(x);
        }

        self.frame_started = false;
    }
}
