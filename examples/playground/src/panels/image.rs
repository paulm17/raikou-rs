//! image panel — playground demo for the raikou `Image` component.
//!
//! Shows three procedural textures (wide gradient, tall gradient, checkerboard)
//! rendered at fixed bounds with each of the three `ImageFit` modes (Fill /
//! Contain / Cover), plus buttons to live-toggle the fit via `set_image_fit`.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;

use std::cell::RefCell;
use std::rc::Rc;

/// Fixed display bounds for every image in the playground.
const BOUNDS_W: f32 = 200.0;
const BOUNDS_H: f32 = 150.0;

/// A horizontal gradient texture.
fn wide_gradient() -> Vec<u8> {
    let (w, h) = (400usize, 100usize);
    let mut pixels = Vec::with_capacity(w * h * 4);
    for _ in 0..h {
        for x in 0..w {
            let t = x as f32 / (w - 1) as f32;
            pixels.push(((0.15 + 0.75 * t) * 255.0) as u8);
            pixels.push(((0.20 + 0.30 * (1.0 - t)) * 255.0) as u8);
            pixels.push(((0.85 - 0.65 * t) * 255.0) as u8);
            pixels.push(255);
        }
    }
    pixels
}

/// A vertical gradient texture.
fn tall_gradient() -> Vec<u8> {
    let (w, h) = (100usize, 400usize);
    let mut pixels = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let t = y as f32 / (h - 1) as f32;
        for _ in 0..w {
            pixels.push(((0.10 + 0.60 * t) * 255.0) as u8);
            pixels.push(((0.70 - 0.50 * t) * 255.0) as u8);
            pixels.push(((0.20 + 0.60 * t) * 255.0) as u8);
            pixels.push(255);
        }
    }
    pixels
}

/// A 32px checkerboard texture.
fn checkerboard() -> Vec<u8> {
    let size = 256usize;
    let cell = 32usize;
    let mut pixels = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let on = (x / cell + y / cell) % 2 == 0;
            let v = if on { 235 } else { 32 };
            pixels.push(v);
            pixels.push(v);
            pixels.push(v);
            pixels.push(255);
        }
    }
    pixels
}

/// Builds the three images (Fill / Contain / Cover) for one texture and returns
/// their components for live re-fitting.
fn build_texture_row(
    cx: &mut BuildCx,
    name: &str,
    base: &Image,
) -> (Component, Rc<RefCell<Vec<Component>>>) {
    let primary = cx
        .theme()
        .color("text.primary")
        .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));

    let mut images = Vec::new();
    for fit in [ImageFit::Fill, ImageFit::Contain, ImageFit::Cover] {
        let image = base
            .clone()
            .with_fit(fit)
            .with_width(BOUNDS_W)
            .with_height(BOUNDS_H)
            .build(cx);
        images.push(image);
    }

    let images_rc = Rc::new(RefCell::new(images));
    let images_handles: Vec<Handle<UiNode>> =
        images_rc.borrow().iter().map(|c| c.handle).collect();

    let mut fit_buttons = Group::new().spacing(8.0);
    for (label, fit) in [
        ("Fill", ImageFit::Fill),
        ("Contain", ImageFit::Contain),
        ("Cover", ImageFit::Cover),
    ] {
        let images = Rc::clone(&images_rc);
        fit_buttons = fit_buttons.child(
            Button::new()
                .text(label)
                .on_click(move |ui, _| {
                    for component in images.borrow_mut().iter_mut() {
                        set_image_fit(component, ui, fit);
                    }
                })
                .build(cx),
        );
    }

    let row = Group::new()
        .spacing(12.0)
        .child(images_handles[0])
        .child(images_handles[1])
        .child(images_handles[2])
        .child(fit_buttons.build(cx))
        .build(cx);

    let section = Stack::new()
        .spacing(6.0)
        .child(Label::new(name).font_size(16.0).color(primary).build(cx))
        .child(row)
        .build(cx);

    (section, images_rc)
}

pub fn image_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let wide = Image::from_rgba(400, 100, wide_gradient());
    let tall = Image::from_rgba(100, 400, tall_gradient());
    let check = Image::from_rgba(256, 256, checkerboard());

    let (section_wide, _) = build_texture_row(&mut cx, "Wide gradient (400x100)", &wide);
    let (section_tall, _) = build_texture_row(&mut cx, "Tall gradient (100x400)", &tall);
    let (section_check, _) = build_texture_row(&mut cx, "Checkerboard (256x256)", &check);

    let shell = Stack::new()
        .spacing(24.0)
        .child(Label::new("Image playground").font_size(22.0).build(&mut cx))
        .child(section_wide)
        .child(section_tall)
        .child(section_check)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(920.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
