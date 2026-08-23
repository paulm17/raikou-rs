//! image panel — playground demo for the raikou `Image` component.
//!
//! Three procedural textures (wide gradient, tall gradient, checkerboard) are
//! rendered at fixed bounds inside the standard playground shell; buttons
//! live-toggle `ImageFit` (Fill / Contain / Cover) on every texture at once.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

/// Display size each texture is rendered at.
fn display_size(index: usize) -> (f32, f32) {
    match index {
        0 => (150.0, 75.0),
        1 => (75.0, 115.0),
        _ => (115.0, 115.0),
    }
}

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
            let on = (x / cell + y / cell).is_multiple_of(2);
            let v = if on { 235 } else { 32 };
            pixels.push(v);
            pixels.push(v);
            pixels.push(v);
            pixels.push(255);
        }
    }
    pixels
}

const CODE: &str = "let image = Image::from_rgba(256, 256, checkerboard())\n\
                        .with_fit(ImageFit::Contain)\n\
                        .with_width(140.0)\n\
                        .with_height(140.0);\n\n\
                        // Switch the fit mode at runtime:\n\
                        set_image_fit(component, ui, ImageFit::Cover);\n";

pub fn image_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    // --- code panel ---------------------------------------------------------
    let code_handle = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Image.rs", code_handle)
        .height(260.0)
        .build(&mut cx);

    // --- preview content ----------------------------------------------------
    // One image per texture; all share a single live `ImageFit` state that the
    // toggle buttons drive.
    let textures: Vec<Image> = vec![
        Image::from_rgba(400, 100, wide_gradient()),
        Image::from_rgba(100, 400, tall_gradient()),
        Image::from_rgba(256, 256, checkerboard()),
    ];

    let mut components = Vec::new();
    let mut row = Group::new().spacing(12.0);
    for (i, base) in textures.into_iter().enumerate() {
        let (w, h) = display_size(i);
        let image = base
            .clone()
            .with_fit(ImageFit::Contain)
            .with_width(w)
            .with_height(h)
            .build(&mut cx);
        components.push(image);
    }

    let images_rc = Rc::new(RefCell::new(components));
    let handles: Vec<Handle<UiNode>> = images_rc.borrow().iter().map(|c| c.handle).collect();
    for handle in &handles {
        row = row.child(*handle);
    }

    // Fit toggles in a vertical column so the row stays inside the card.
    let mut fit_buttons = Stack::new().spacing(8.0);
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
                .build(&mut cx),
        );
    }

    let primary = theme
        .color("text.primary")
        .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));
    let preview_content = Stack::new()
        .spacing(10.0)
        .child(
            Label::new("Wide 400x100 · Tall 100x400 · Checkerboard 256x256")
                .font_size(13.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(row.child(fit_buttons.build(&mut cx)).build(&mut cx))
        .build(&mut cx);

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(520.0, 300.0)
        .build(&mut cx);

    // --- info ----------------------------------------------------------------
    let notes = playground_notes(
        &mut cx,
        "Image",
        &[
            "Image draws an RGBA pixel buffer at fixed bounds; ImageFit decides",
            "how the source maps onto those bounds.",
            "Fill stretches to fill, Contain scales to fit inside, Cover fills",
            "and crops — toggle each mode live with the buttons above.",
        ],
    )
    .build(&mut cx);

    // --- shell ---------------------------------------------------------------
    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(260.0)
        .code_height(260.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(920.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
