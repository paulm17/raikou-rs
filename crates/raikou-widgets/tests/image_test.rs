//! Functional tests for the Image component.

mod common;

use common::Harness;
use fyrox::asset::untyped::ResourceKind;
use fyrox::core::uuid::Uuid;
use fyrox::graph::SceneGraph;
use fyrox::gui::texture::{
    TextureKind, TexturePixelKind, TextureResource, TextureResourceExtension,
};
use raikou_core::ImageFit;
use raikou_widgets::{set_image_fit, set_image_texture, Image};

#[test]
fn image_from_rgba_builds_texture() {
    let mut h = Harness::new();
    let img = h.build(|cx| {
        Image::from_rgba(2, 2, vec![255; 16])
            .with_fit(ImageFit::Contain)
            .with_width(64.0)
            .with_height(64.0)
            .build(cx)
    });

    assert!(
        h.ui.try_get_node(img.handle).is_ok(),
        "image widget must exist in the graph"
    );
}

#[test]
fn image_helpers_update_state() {
    let mut h = Harness::new();
    let mut img = h.build(|cx| Image::from_rgba(2, 2, vec![128; 16]).build(cx));

    // set_image_fit must swap the stored fit mode without panicking.
    set_image_fit(&mut img, &mut h.ui, ImageFit::Cover);

    // set_image_texture accepts an optional texture (from_bytes yields one).
    let texture = TextureResource::from_bytes(
        Uuid::new_v4(),
        TextureKind::Rectangle {
            width: 2,
            height: 2,
        },
        TexturePixelKind::RGBA8,
        vec![64; 16],
        ResourceKind::Embedded,
    );
    set_image_texture(&mut img, &mut h.ui, texture);
    set_image_texture(&mut img, &mut h.ui, None);
}
