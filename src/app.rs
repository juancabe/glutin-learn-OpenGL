use glam::Vec3;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::{config::GetGlConfig, context::NotCurrentContext};
use glutin_winit::GlWindow;
use raw_window_handle::HasWindowHandle;
use std::ffi::CString;
use std::rc::Rc;
use std::time::Instant;
use std::vec;
use std::{error::Error, num::NonZeroU32};
use winit::event::ElementState;
use winit::keyboard::PhysicalKey;
use winit::window::Window;

use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::PossiblyCurrentContext,
    prelude::*,
};
use glutin_winit::DisplayBuilder;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
};

use crate::camera::{Camera, CameraMovement};
use crate::entities::Entity;
use crate::entities::hello_triangle::HelloTriangle;
use crate::entities::sun::Sun;
use crate::entities::tex_cube::TexCube;
use crate::entities::utah_teapot::UtahTeapot;
// use crate::entities::utah_teapot::UtahTeapot;
use crate::gl::{self};
use crate::helpers::{FpsCounter, GlPosition, Mat3DUpdate, RendererControl};
use crate::renderer::shader::GlslPass;
use crate::renderer::shader::uniform::{EnabledLighting, EyePos, Fog, LightPos, Lighting, Uniform};
use crate::terrain_builder;
use crate::{GlDisplayCreationState, renderer::Renderer, window_attributes};
use glutin::surface::{Surface, SwapInterval, WindowSurface};

const DEFAULT_WINDOW_WIDTH: usize = 800;
const DEFAULT_WINDOW_HEIGHT: usize = 600;

const CLEAR_COLOR: glam::Vec3 = glam::Vec3 {
    x: 0.1,
    y: 0.1,
    z: 0.1,
};

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
    entities: Vec<Box<dyn Entity>>,
    next_frame_entities_uniforms: Vec<Box<dyn Uniform>>,
    sun: Sun,
    camera: Camera,
    last_frame: Instant,
}

pub struct App {
    template: ConfigTemplateBuilder,
    renderer: Option<Renderer>,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    fps_counter: FpsCounter,
    gl_context: Option<PossiblyCurrentContext>,
    gl_display: GlDisplayCreationState,
    pub exit_state: Result<(), Box<dyn Error>>,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            template,
            gl_display: GlDisplayCreationState::Builder(display_builder),
            exit_state: Ok(()),
            fps_counter: FpsCounter::default(),
            gl_context: None,
            state: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, gl_config) = match &self.gl_display {
            // We just created the event loop, so initialize the display, pick the config, and
            // create the context.
            GlDisplayCreationState::Builder(display_builder) => {
                let (window, gl_config) = match display_builder.clone().build(
                    event_loop,
                    self.template.clone(),
                    gl_config_picker,
                ) {
                    Ok((window, gl_config)) => (window.unwrap(), gl_config),
                    Err(err) => {
                        self.exit_state = Err(err);
                        event_loop.exit();
                        return;
                    }
                };

                log::info!("Picked a config with {} samples", gl_config.num_samples());

                // Mark the display as initialized to not recreate it on resume, since the
                // display is valid until we explicitly destroy it.

                self.gl_display = GlDisplayCreationState::Init;

                // Create gl context.
                self.gl_context =
                    Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());

                (window, gl_config)
            }
            GlDisplayCreationState::Init => {
                println!("Recreating window in `resumed`");
                // Pick the config which we already use for the context.
                let gl_config = self.gl_context.as_ref().unwrap().config();
                match glutin_winit::finalize_window(event_loop, window_attributes(), &gl_config) {
                    Ok(window) => (window, gl_config),
                    Err(err) => {
                        self.exit_state = Err(err.into());
                        event_loop.exit();
                        return;
                    }
                }
            }
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
            .expect("Cursor should be grabbable");

        window.set_cursor_visible(false);

        // The context needs to be current for the Renderer to set up shaders and
        // buffers.
        let gl_context = self.gl_context.as_ref().unwrap();
        gl_context.make_current(&gl_surface).unwrap();

        let gl_fns = gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_config
                .display()
                .get_proc_address(symbol.as_c_str())
                .cast()
        });
        let gl_fns = Rc::new(gl_fns);

        self.renderer.get_or_insert_with(|| {
            Renderer::new(
                gl_fns.clone(),
                glam::USizeVec2::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
                CLEAR_COLOR,
            )
        });

        // Try setting vsync.
        if let Err(res) = gl_surface
            .set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            log::error!("Error setting vsync: {res:?}");
        }

        const FLOOR_SIDE: usize = 50;
        const HEIGHT: usize = 4;
        const CS: f32 = 1.0;

        let tb = terrain_builder::terrain_builder(123, HEIGHT);

        let mut cubes_floor = vec![];
        for x in 0..FLOOR_SIDE {
            for z in 0..FLOOR_SIDE {
                for y in 0..=tb(x, z) {
                    cubes_floor.push(Vec3::new(x as f32, y as f32, z as f32))
                }
            }
        }

        const MIDDLE: f32 = FLOOR_SIDE as f32 * CS / 2.0;

        let utahs: Vec<Box<UtahTeapot>> = [
            glam::Vec2::new(3.0 + MIDDLE, 5.0 + MIDDLE),
            glam::Vec2::new(MIDDLE - 5.0, MIDDLE + 2.0),
            glam::Vec2::new(MIDDLE, MIDDLE),
        ]
        .iter()
        .map(|utah| {
            Box::new(UtahTeapot::new(
                GlPosition::new(
                    utah.x,
                    tb(utah.x as usize, utah.y as usize) as f32 + 0.5,
                    utah.y,
                ),
                Vec3::new(1.0, 0.0, 0.0),
            ))
        })
        .collect();

        let mut entities: Vec<Box<dyn Entity>> = vec![
            Box::new(HelloTriangle::new((
                GlPosition::new(MIDDLE + 3.0, HEIGHT as f32 + 1.0, MIDDLE + 3.0),
                CS,
            ))),
            Box::new(TexCube::new(
                cubes_floor,
                CS,
                // Dirt cubes floor
                Some("./assets/dirt.webp".into()),
            )),
        ];

        for utah in utahs {
            entities.push(utah);
        }

        let mut sun = Sun::new(GlPosition::new(MIDDLE, HEIGHT as f32 + 10.0, MIDDLE));

        let dimensions = self
            .renderer
            .as_ref()
            .expect("Set before")
            .get_window_dimensions();

        let dimensions = glam::Vec2::new(dimensions.x as f32, dimensions.y as f32);

        let entities_transformations_3d = Mat3DUpdate::default_from_dimensions(&dimensions);

        let init_uniforms: Vec<Box<dyn Uniform>> = vec![
            Box::new(Lighting::new()),
            Box::new(Fog::new(CLEAR_COLOR)),
            Box::new(EnabledLighting::enabled()),
        ];

        // Init Glsl for drawables
        for entity in entities.iter_mut() {
            entity.init(gl_fns.clone(), entities_transformations_3d, &init_uniforms);
        }
        sun.init(gl_fns, entities_transformations_3d, &[]);

        assert!(
            self.state
                .replace(AppState {
                    last_frame: Instant::now(),
                    entities,
                    gl_surface,
                    window,
                    camera: Camera::from_pos(GlPosition::new(MIDDLE, HEIGHT as f32 + 1.0, MIDDLE)),
                    sun,
                    next_frame_entities_uniforms: vec![]
                })
                .is_none()
        );
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");

        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        self.state = None;

        // Make context not current.
        self.gl_context = Some(
            self.gl_context
                .take()
                .unwrap()
                .make_not_current()
                .unwrap()
                .treat_as_possibly_current(),
        );
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event else {
            return;
        };
        if let Some(state) = self.state.as_mut() {
            state.camera.mouse_moved(dx as f32, -dy as f32)
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                {
                    let gl_surface = &self.state.as_mut().unwrap().gl_surface;
                    let gl_context = self.gl_context.as_ref().unwrap();
                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );

                    let renderer = self.renderer.as_mut().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match self.state.as_mut() {
                Some(state) => {
                    if let Some(movement) = CameraMovement::from_keycode(code) {
                        match key_state {
                            ElementState::Pressed => state.camera.want_move(movement),
                            ElementState::Released => state.camera.stop_move(movement),
                        }
                    }
                    if let Some(control) = RendererControl::from_keycode(code) {
                        let uniform = Box::new(match control {
                            RendererControl::EnableLight => EnabledLighting::enabled(),
                            RendererControl::DisableLight => EnabledLighting::default(),
                            RendererControl::EnableFog => todo!(),
                            RendererControl::DisableFog => todo!(),
                        });
                        state.next_frame_entities_uniforms.push(uniform);
                    }
                }
                None => log::warn!("Key pressed before state init"),
            },

            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // NOTE: The handling below is only needed due to nvidia on Wayland to not crash
        // on exit due to nvidia driver touching the Wayland display from on
        // `exit` hook.
        let _gl_display = self.gl_context.take().unwrap().display();

        // Clear the window.
        self.state = None;
        #[cfg(egl_backend)]
        #[allow(irrefutable_let_patterns)]
        if let glutin::display::Display::Egl(display) = _gl_display {
            unsafe {
                display.terminate();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(AppState {
            last_frame,
            gl_surface,
            window,
            entities,
            camera,
            sun,
            next_frame_entities_uniforms,
        }) = self.state.as_mut()
        {
            let renderer = self.renderer.as_mut().unwrap();

            let dt = last_frame.elapsed();
            *last_frame = Instant::now();

            if let Some(fps) = self.fps_counter.tick() {
                log::info!("FPS: {fps}");
                log::info!("Sun position: {:?}", sun.get_pos());
            }

            let gl_context = self.gl_context.as_ref().unwrap();

            let renderer_refs = entities.iter_mut().map(|e| e.as_mut() as &mut dyn GlslPass);

            camera.update(&dt);

            let mat3d = Mat3DUpdate {
                view: Some(camera.as_view()),
                ..Default::default()
            };

            let base_frame_update_uniforms = [
                Box::new(LightPos::new(sun.get_pos())) as Box<dyn Uniform>,
                Box::new(EyePos::new(camera.pos)),
            ];

            renderer.clear();
            renderer.draw(
                [sun as &mut dyn GlslPass].into_iter(),
                mat3d,
                &base_frame_update_uniforms,
            );
            next_frame_entities_uniforms.extend(base_frame_update_uniforms);
            renderer.draw(renderer_refs, mat3d, next_frame_entities_uniforms);

            next_frame_entities_uniforms.clear();

            window.request_redraw();

            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    }
}
