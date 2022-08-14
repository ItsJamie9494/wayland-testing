// SPDX-License-Identifier: GPL-3.0-only

use smithay::{
    backend::renderer::{gles2::Gles2Renderer, Frame, ImportAll, Renderer},
    desktop::{
        draw_layer_popups, draw_layer_surface, draw_window, draw_window_popups,
        layer_map_for_output,
        space::{RenderElement, RenderError, SurfaceTree},
        utils::damage_from_surface_tree,
        Window,
    },
    utils::{Physical, Rectangle, Transform},
    wayland::{output::Output, shell::wlr_layer::Layer as WlrLayer},
};

use crate::state::CommonState;

smithay::custom_elements! {
    pub CustomElem<=Gles2Renderer>;
    SurfaceTree=SurfaceTree,
}

pub trait AsGles2Renderer {
    fn as_gles2(&mut self) -> &mut Gles2Renderer;
}
impl AsGles2Renderer for Gles2Renderer {
    fn as_gles2(&mut self) -> &mut Gles2Renderer {
        self
    }
}

static CLEAR_COLOR: [f32; 4] = [0.153, 1.0, 0.165, 1.0];

pub fn render_output<R>(
    renderer: &mut R,
    age: u8,
    state: &mut CommonState,
    output: &Output,
    hardware_cursor: bool,
) -> Result<Option<Vec<Rectangle<i32, Physical>>>, RenderError<R>>
where
    R: Renderer + ImportAll + AsGles2Renderer,
    <R as Renderer>::TextureId: Clone + 'static,
    CustomElem: RenderElement<R>,
{
    // TODO
    // Render either a desktop or window, based on presence
}

fn render_desktop<R>(
    renderer: &mut R,
    age: u8,
    state: &mut CommonState,
    output: &Output,
    hardware_cursor: bool,
) -> Result<Option<Vec<Rectangle<i32, Physical>>>, RenderError<R>>
where
    R: Renderer + ImportAll + AsGles2Renderer,
    <R as Renderer>::TextureId: Clone + 'static,
    CustomElem: RenderElement<R>,
{
    let mut custom_elements = Vec::<CustomElem>::new();

    // TODO
    // Get active space and render it
}

/// Renders a Wayland window
fn render_window<R>(
    renderer: &mut R,
    window: Window,
    state: &mut CommonState,
    output: &Output,
    hardware_cursor: bool,
) -> Result<Option<Vec<Rectangle<i32, Physical>>>, RenderError<R>>
where
    R: Renderer + ImportAll + AsGles2Renderer,
    <R as Renderer>::TextureId: Clone + 'static,
    CustomElem: RenderElement<R>,
{
    let transform = Transform::from(output.current_transform());
    let mode = output.current_mode().unwrap();
    let scale = output.current_scale().fractional_scale();

    let mut custom_elements = Vec::<CustomElem>::new();

    renderer
        .render(mode.size, transform, |renderer, frame| {
            let mut damage = window.accumulated_damage((0.0, 0.0), scale, None);
            frame.clear(
                CLEAR_COLOR,
                &[Rectangle::from_loc_and_size((0, 0), mode.size)],
            )?;
            draw_window(
                renderer,
                frame,
                &window,
                scale,
                (0.0, 0.0),
                &[Rectangle::from_loc_and_size((0, 0), mode.size)],
                &slog_scope::logger(),
            )?;
            draw_window_popups(
                renderer,
                frame,
                &window,
                scale,
                (0.0, 0.0),
                &[Rectangle::from_loc_and_size((0, 0), mode.size)],
                &slog_scope::logger(),
            )?;
            let layer_map = layer_map_for_output(output);
            for layer_surface in layer_map.layers_on(WlrLayer::Overlay) {
                let geo = layer_map.layer_geometry(&layer_surface).unwrap();
                draw_layer_surface(
                    renderer,
                    frame,
                    layer_surface,
                    scale,
                    geo.loc.to_f64().to_physical(scale),
                    &[Rectangle::from_loc_and_size(
                        (0, 0),
                        geo.size.to_physical_precise_round(scale),
                    )],
                    &slog_scope::logger(),
                )?;
                draw_layer_popups(
                    renderer,
                    frame,
                    layer_surface,
                    scale,
                    geo.loc.to_f64().to_physical(scale),
                    &[Rectangle::from_loc_and_size(
                        (0, 0),
                        geo.size.to_physical_precise_round(scale),
                    )],
                    &slog_scope::logger(),
                )?;
                damage.extend(damage_from_surface_tree(
                    layer_surface.wl_surface(),
                    geo.loc.to_f64().to_physical(scale),
                    scale,
                    None,
                ));
            }
            for elem in custom_elements {
                let loc = elem.location(scale);
                let geo = elem.geometry(scale);
                let elem_damage = elem.accumulated_damage(scale, None);
                elem.draw(
                    renderer,
                    frame,
                    scale,
                    loc,
                    &[Rectangle::from_loc_and_size((0, 0), geo.size)],
                    &slog_scope::logger(),
                )?;
                damage.extend(elem_damage.into_iter().map(|mut rect| {
                    rect.loc += geo.loc;
                    rect
                }))
            }
            Ok(Some(damage))
        })
        .and_then(std::convert::identity)
        .map_err(RenderError::<R>::Rendering)
}
