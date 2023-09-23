use egui::{ClippedPrimitive, Context, TexturesDelta, RichText, FontFamily, FontId, Align, Stroke};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use egui_winit::EventResponse;
use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::{fractals::{Fractals, COLOUR_GRADIENTS}, Flags};

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

/// Application state.
pub struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    window_position: (f32, f32),
    // Track the position and size of the egui window.
    window_open_size: (f32, f32),
    window_closed_size: (f32, f32),
    font: FontId,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        window_position: (f32, f32),
        window_open_size: (f32, f32),
        window_closed_size: (f32, f32),
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        // egui handle
        let egui_ctx = Context::default();
        // Change font colour to white
        let mut style = egui::Style::default();
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255)); // set text to white
        style.spacing.item_spacing = egui::Vec2::new(6.0, 6.0); // increase spacing between items
        egui_ctx.set_style(style);

        let visual = egui::Visuals::dark();
        visual.gray_out(egui::Color32::from_rgb(255, 255,255));
        egui_ctx.set_visuals(visual);

        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        
        let gui = Gui::new(window_position, window_open_size, window_closed_size);

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) -> EventResponse {
        self.egui_state.on_event(&self.egui_ctx, event)
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window, current_fractal: &mut Fractals, flags: &mut Flags) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx, current_fractal, flags);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

/// Adds a label and a method to adjust the variable e.g. a slider
/// If a setting is changed then fractal_change is set to true to redraw the fractal
macro_rules! create_fractal_setting {
    ($ui:ident, $fractal_change:ident, $font:ident, $(($label:expr, $setting_type:ident)),+) => {
        $(
            $ui.horizontal(|ui| {
                ui.label(RichText::new($label).font($font.clone()));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(10.0);
                    let change = ui.add($setting_type).changed();
                    *$fractal_change |= change;
                });
            });
        )*
    };
}

/// Adds a selectable value to a combo box for a colour gradient
macro_rules! create_colour_gradient_option {
    ($ui:ident, $current_colour_gradient:ident, $font:ident, $colour_gradient:ident) => {
        $ui.selectable_value($current_colour_gradient, String::from($colour_gradient), RichText::new($colour_gradient).font($font.clone()));
    };
}

impl Gui {
    /// Create a `Gui`.
    fn new(window_position: (f32, f32), window_open_size: (f32,f32), window_closed_size: (f32,f32)) -> Self {
        Self { 
            window_open: true,
            window_position,
            window_open_size,
            window_closed_size,
            font: FontId {
                size: 15.0,
                family: FontFamily::default(),
            },
        }
    }

    pub fn get_window_size(&self) -> (f32, f32) {
        if self.window_open {
            self.window_open_size
        } else {
            self.window_closed_size
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context, current_fractal: &mut Fractals, flags: &mut Flags) {
        let size = self.get_window_size();
        egui::Area::new("Settings")
        .fixed_pos(self.window_position)
        .movable(false)
        .show(ctx, |ui| {
            // Change size depending on if the window is open or closed
            ui.set_width(size.0);
            ui.set_height(size.1);

            // Move title slightly down when opening window
            if self.window_open {
                ui.add_space(10.0);
            } else {
                ui.add_space(5.0);
            }
            
            // Add background to area
            ui.painter().rect_stroke(ui.max_rect(), 2.0, Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(150, 150, 150, 200)));
            ui.painter().rect_filled(ui.max_rect(), 2.0, egui::Color32::from_rgba_premultiplied(0, 0, 0, 255));

            let drop_down_title = RichText::new("Settings").color(egui::Color32::WHITE).font(self.font.clone());
            let collapse_button = ui.collapsing(drop_down_title, |ui| {
                ui.separator();

                let display_name = match current_fractal {
                    Fractals::Mandelbrot {..} => "Mandelbrot",
                    Fractals::Julia {..} => "Julia",
                    Fractals::Newton {..} => "Newton"
                };
                
                // Fractal selection
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Fractal:").font(self.font.clone()));
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(10.0); // add space to right side of combo box
                        egui::ComboBox::from_label("")
                        .selected_text(display_name)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(current_fractal, Fractals::Mandelbrot {max_iterations: 100, escape_radius: 2.0, colour_gradient: "Magma".into()},
                                RichText::new("Mandelbrot").font(self.font.clone()));
                            ui.selectable_value(current_fractal, Fractals::Julia {max_iterations: 100, escape_radius: 2.0, c: (-0.7,0.27015), colour_gradient: "Magma".into()}, 
                                RichText::new("Julia").font(self.font.clone()));
                            ui.selectable_value(current_fractal, Fractals::Newton {max_iterations: 100, colour_gradient: "Magma".into()},
                                RichText::new("Newton").font(self.font.clone()));
                        })
                    });
                });

                // Colour gradient selection
                let current_colour_gradient = match current_fractal {
                    Fractals::Mandelbrot {ref mut colour_gradient, ..} => colour_gradient,
                    Fractals::Julia {ref mut colour_gradient, ..} => colour_gradient,
                    Fractals::Newton {ref mut colour_gradient, ..} => colour_gradient
                };
                let old_colour = current_colour_gradient.clone();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Colour:").font(self.font.clone()));
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(10.0); // add space to right side of combo box
                        egui::ComboBox::from_label(" ")
                        .selected_text(current_colour_gradient.clone())
                        .show_ui(ui, |ui| {
                            let font = self.font.clone();
                            for colour_gradient in COLOUR_GRADIENTS.iter() {
                                let colour_gradient = *colour_gradient;
                                create_colour_gradient_option!(ui, current_colour_gradient, font, colour_gradient);
                            }
                        });
                    });
                });

                ui.separator();
                
                let font = &self.font;
                // create a mutable reference to fractal_change as "." cant be used inside a macro call
                let generate_fractal = &mut flags.generate_fractal;
                // Display the correct settings for the selected fractal
                match current_fractal {
                    Fractals::Mandelbrot { ref mut max_iterations, ref mut escape_radius, ref colour_gradient,.. } => {
                        let slider1 = egui::Slider::new(max_iterations, 1..=10000).text("").clamp_to_range(true);
                        let slider2 = egui::Slider::new(escape_radius, 1.0..=10.0).text("").clamp_to_range(true);
                        create_fractal_setting!(ui, generate_fractal, font, ("Max Iterations", slider1), ("Escape Radius", slider2));
                        
                        flags.reset |= display_name != "Mandelbrot";
                        flags.generate_fractal |= flags.reset || old_colour != *colour_gradient
                    },
                    Fractals::Julia { ref mut max_iterations, ref mut escape_radius, ref mut c, ref mut colour_gradient, ..} => {
                        let slider1 = egui::Slider::new(max_iterations, 1..=10000).text("").clamp_to_range(true);
                        let slider2 = egui::Slider::new(escape_radius, 1.0..=10.0).text("").clamp_to_range(true);
                        let slider3 = egui::Slider::new(&mut c.0, -1.5..=1.5).clamp_to_range(true);
                        let slider4 = egui::Slider::new(&mut c.1, -1.5..=1.5).clamp_to_range(true);
                        create_fractal_setting!(ui, generate_fractal, font, ("Max Iterations", slider1), ("Escape Radius", slider2), ("Real", slider3), ("Imaginary", slider4));
                        
                        flags.reset |= display_name != "Julia";
                        flags.generate_fractal |= flags.reset || old_colour != *colour_gradient
                    },
                    Fractals::Newton { ref mut max_iterations, ref mut colour_gradient,.. } => {
                        let slider1 = egui::Slider::new(max_iterations, 1..=10000).text("").clamp_to_range(true);
                        create_fractal_setting!(ui, generate_fractal, font, ("Max Iterations", slider1));

                        flags.reset |= display_name != "Newton";
                        flags.generate_fractal |= flags.reset || old_colour != *colour_gradient
                    }
                };

                // Reset button in bottom right
                if self.window_open {
                    ui.with_layout(egui::Layout::right_to_left(Align::BOTTOM), |ui| {
                        ui.add_space(10.0); // add space to the right of the button
                        ui.with_layout(egui::Layout::bottom_up(Align::RIGHT), |ui| {
                            ui.add_space(10.0); // add space below the button
                            if ui.button("Reset").clicked() {
                                *current_fractal = match current_fractal{
                                    Fractals::Mandelbrot {colour_gradient, ..} => Fractals::Mandelbrot {max_iterations: 100, escape_radius: 2.0, colour_gradient: colour_gradient.to_string()},
                                    Fractals::Julia {colour_gradient,..} => Fractals::Julia {max_iterations: 100, escape_radius: 2.0, c: (-0.7,0.27015), colour_gradient: colour_gradient.to_string()},
                                    Fractals::Newton {colour_gradient,..} => Fractals::Newton {max_iterations: 100, colour_gradient: colour_gradient.to_string()}
                                };
                                flags.reset = true;
                                flags.generate_fractal = true;
                            }
                        });
                    });
                }
            });
            self.window_open = collapse_button.fully_open();
        });   
    }
}