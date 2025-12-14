use crate::dap::dap_interface::DapInterface;
use crate::ui::MemVisorUi;
use crate::ui_renderer::EguiRenderer;
use egui_wgpu::wgpu;
use std::sync::Arc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod dap;
pub mod data;
mod ui;
mod ui_renderer;
pub mod widget;

pub struct MemVisorState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub dap_interface: Arc<DapInterface>,
}

pub struct MemVisorApp {
    instance: wgpu::Instance,
    ui: MemVisorUi,
    state: Option<MemVisorState>,
    window: Option<Arc<Window>>,
}

fn main() {
    env_logger::init();
    log::info!("Log enabled");

    let _ = tracy_client::Client::start();

    let event_loop = EventLoop::new().expect("should create event loop");

    event_loop.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(500)));

    let mut app = MemVisorApp::new();
    event_loop
        .run_app(&mut app)
        .expect("app runs fine and dandy");
}

impl MemVisorState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("should create adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                    ..Default::default()
                },
            )
            .await
            .expect("should create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);

        let scale_factor = 1.0;

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            dap_interface: Arc::new(DapInterface::new()),
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

impl MemVisorApp {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        Self {
            instance,
            ui: MemVisorUi::new(),
            state: None,
            window: None,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);

        let size = window.inner_size();
        let initial_width = size.width;
        let initial_height = size.height;

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = MemVisorState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_height,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) {
        let _span = tracy_client::span!("handle_redraw");
        // Attempt to handle minimizing window
        if let Some(true) = self.window.as_ref().and_then(|window| window.is_minimized()) {
            log::info!("Window is minimized");
            return;
        }

        let state = self.state.as_mut().unwrap();

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.scale_factor,
        };

        let surface_texture = state.surface.get_current_texture();

        match surface_texture {
            Err(wgpu::SurfaceError::Outdated) => {
                // Ignoring outdated to allow resizing and minimization
                log::error!("wgpu surface outdated");
                return;
            }
            Err(_) => {
                surface_texture.expect("should be able to acquire next swapchain image");
                return;
            }
            Ok(_) => {}
        };

        let surface_texture = surface_texture.unwrap();

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();

        {
            let _egui_frame = tracy_client::span!("egui_redraw");
            state.egui_renderer.begin_frame(window);

            self.ui.update(
                state.egui_renderer.context(),
                Arc::clone(&state.dap_interface),
            );

            state.egui_renderer.draw_frame(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        {
            let _graphics_submit_span = tracy_client::span!("frame_submit");
            state.queue.submit(Some(encoder.finish()));
            surface_texture.present();

            tracy_client::frame_mark();
        }
    }
}
impl Default for MemVisorApp {
    fn default() -> Self {
        Self::new()
    }
}
impl ApplicationHandler for MemVisorApp {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            tracy_client::Client::start().message("Resume time reached: redraw", 0);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("MemVisor")
                    .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
            )
            .expect("should create window");
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let _span = tracy_client::span!("window_event");

        // let egui render to process the event first
        let w_state = self.state.as_mut().unwrap();

        if let Err(e) = w_state.dap_interface.process_dap_events() {
            log::error!("DAP Interface Error: {e}");
        }

        w_state
            .egui_renderer
            .handle_input(self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                tracy_client::Client::start().message("Redraw Requested", 0);
                self.handle_redraw();
                
                let window = self.window.as_ref().expect("must have window here");
                let state = self.state.as_ref().expect("should have memvisor state");
                if state.egui_renderer.context().has_requested_repaint() {
                    tracy_client::Client::start().message("Repaint requested", 0);
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }

        if let Some(window) = self.window.as_ref() {
            // We request redraw on every window event, except redraw requested.
            // If we requested a redraw on a RedrawRequested event it would mean
            // the app would always keep redrawing, nonstop.
            if event != WindowEvent::RedrawRequested {
                window.request_redraw();
            }
        }

        // By default try to run at 2 FPS even if there are no events
        event_loop.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(500)));
    }
}
