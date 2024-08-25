use crate::{widgets::Widget, WidgetTree};
use sdl2::{
  event::Event,
  image::LoadSurface,
  surface::Surface,
  sys,
  video::{GLProfile, SwapInterval},
};
use skia_safe::{
  gpu::{
    backend_render_targets,
    context_options::Enable,
    gl::{Format, FramebufferInfo},
    ContextOptions, SurfaceOrigin,
  },
  Color, ColorType, Rect,
};
use std::{ffi::CStr, time::Instant};

#[derive(Debug)]
pub struct App<'a> {
  pub title: &'a str,
  pub size: (u32, u32),
  pub child: Option<Widget<'a>>,
}

impl Default for App<'_> {
  fn default() -> Self {
    Self {
      title: "",
      size: (800, 600),
      child: None,
    }
  }
}

pub fn run(app: App<'_>) {
  sdl2::hint::set(
    CStr::from_bytes_with_nul(sys::SDL_HINT_WINDOWS_DPI_SCALING)
      .unwrap()
      .to_str()
      .unwrap(),
    "1",
  );

  let sdl = sdl2::init().unwrap();
  let vid_subsys = sdl.video().unwrap();
  let gl_attr = vid_subsys.gl_attr();
  let mut ctx_flags_builder = gl_attr.set_context_flags();
  let mut ctx_flags_builder = ctx_flags_builder.forward_compatible();

  #[cfg(debug_assertions)]
  {
    ctx_flags_builder = ctx_flags_builder.debug();
  }

  ctx_flags_builder.set();

  #[cfg(not(debug_assertions))]
  {
    gl_attr.set_context_no_error(true);
  }

  gl_attr.set_context_profile(GLProfile::Core);
  gl_attr.set_context_version(4, 6);

  let mut window = vid_subsys
    .window(app.title, app.size.0, app.size.1)
    .allow_highdpi()
    .opengl()
    .position_centered()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file("assets/images/favicon.png") {
    window.set_icon(favicon);
  }

  let _gl_ctx = window.gl_create_context().unwrap();
  gl::load_with(|name| vid_subsys.gl_get_proc_address(name) as *const _);

  // Try to use Adaptive VSync
  vid_subsys
    .gl_set_swap_interval(SwapInterval::LateSwapTearing)
    .unwrap_or_else(|_| {
      // If not supported, fallback to VSync
      vid_subsys
        .gl_set_swap_interval(SwapInterval::VSync)
        .unwrap();
    });

  let skia_interface = skia_safe::gpu::gl::Interface::new_native().unwrap();

  let mut skia_ctx_opts = ContextOptions::new();
  skia_ctx_opts.allow_msaa_on_new_intel = false;
  skia_ctx_opts.allow_multiple_glyph_cache_textures = Enable::Yes;
  skia_ctx_opts.allow_path_mask_caching = false;
  skia_ctx_opts.always_use_text_storage_when_available = false;
  skia_ctx_opts.avoid_stencil_buffers = false;
  skia_ctx_opts.buffer_map_threshold = -1;
  skia_ctx_opts.disable_coverage_counting_paths = false;
  skia_ctx_opts.disable_distance_field_paths = true;
  skia_ctx_opts.disable_driver_correctness_workarounds = false;
  skia_ctx_opts.disable_gpu_yuv_conversion = false;
  skia_ctx_opts.do_manual_mipmapping = false;
  skia_ctx_opts.internal_multisample_count = 0;
  skia_ctx_opts.max_cached_vulkan_secondary_command_buffers = -1;
  skia_ctx_opts.prefer_external_images_over_es3 = false;
  skia_ctx_opts.reduced_shader_variations = false;

  #[cfg(debug_assertions)]
  {
    skia_ctx_opts.skip_gl_error_checks = Enable::No;
    skia_ctx_opts.suppress_prints = false;
  }

  #[cfg(not(debug_assertions))]
  {
    skia_ctx_opts.skip_gl_error_checks = Enable::Yes;
    skia_ctx_opts.suppress_prints = true;
  }

  skia_ctx_opts.suppress_mipmap_support = false;
  skia_ctx_opts.use_draw_instead_of_clear = Enable::No;

  let mut gr_ctx =
    skia_safe::gpu::direct_contexts::make_gl(skia_interface, &skia_ctx_opts).unwrap();

  let drawable_size = window.drawable_size();

  let backend_render_target = backend_render_targets::make_gl(
    (drawable_size.0 as _, drawable_size.1 as _),
    gl_attr.multisample_samples() as usize,
    gl_attr.stencil_size() as _,
    FramebufferInfo {
      fboid: 0,
      format: Format::RGBA8.into(),
      ..Default::default()
    },
  );

  let mut skia_surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
    &mut gr_ctx,
    &backend_render_target,
    SurfaceOrigin::BottomLeft,
    ColorType::RGBA8888,
    None,
    None,
  )
  .unwrap();

  let canvas = skia_surface.canvas();
  const TPS: f32 = 240.0;
  const MAX_FRAME_TICK_COUNT: u32 = 8;
  let mut event_pump = sdl.event_pump().unwrap();

  let mut widget_tree = WidgetTree::new(
    app.child,
    Rect::from_wh(drawable_size.0 as _, drawable_size.1 as _),
  );

  let mut now = Instant::now();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }

      widget_tree.process_event(&event);
    }

    let mut tick_count = 0;
    let mut frame_time = now.elapsed().as_secs_f32();
    now = Instant::now();

    while frame_time > 0.0 && tick_count < MAX_FRAME_TICK_COUNT {
      let dt = frame_time.min(1.0 / TPS);
      widget_tree.update(dt);
      frame_time -= dt;
      tick_count += 1;
    }

    canvas.clear(Color::BLACK);
    widget_tree.draw(canvas);
    gr_ctx.flush_and_submit();
    window.gl_swap_window();
  }

  window.hide();

  // todo:
  // for event in sdl.event_pump().unwrap().wait_iter() {
  //   if let Event::Quit { .. } = event {
  //     break;
  //   }

  //   canvas.clear(Color::BLACK);
  //   gr_ctx.flush_and_submit();
  //   window.gl_swap_window();
  // }
}
