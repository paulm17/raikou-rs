use std::collections::HashMap;

use raikou_core::Size;
use raikou_text::{Ellipsize, EllipsizeHeightLimit, Wrap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TextMeasureKey {
    text: String,
    font_family: String,
    font_size_bits: u32,
    line_height_bits: u32,
    wrap_mode: u8,
    ellipsize_mode: u8,
    ellipsize_limit_kind: u8,
    ellipsize_limit_value_bits: u64,
    available_width_bits: u32,
    available_height_bits: u32,
}

impl TextMeasureKey {
    fn new(
        text: &str,
        font_family: &str,
        font_size: f32,
        line_height: f32,
        wrap: Wrap,
        ellipsize: Ellipsize,
        available: Size,
    ) -> Self {
        Self {
            text: text.to_string(),
            font_family: font_family.to_string(),
            font_size_bits: font_size.to_bits(),
            line_height_bits: line_height.to_bits(),
            wrap_mode: wrap_as_u8(wrap),
            ellipsize_mode: ellipsize_as_u8(ellipsize),
            ellipsize_limit_kind: ellipsize_limit_kind(ellipsize),
            ellipsize_limit_value_bits: ellipsize_limit_value(ellipsize),
            available_width_bits: available.width.to_bits(),
            available_height_bits: available.height.to_bits(),
        }
    }
}

const fn wrap_as_u8(wrap: Wrap) -> u8 {
    match wrap {
        Wrap::None => 0,
        Wrap::Glyph => 1,
        Wrap::Word => 2,
        Wrap::WordOrGlyph => 3,
    }
}

const fn ellipsize_as_u8(ellipsize: Ellipsize) -> u8 {
    match ellipsize {
        Ellipsize::None => 0,
        Ellipsize::Start(_) => 1,
        Ellipsize::Middle(_) => 2,
        Ellipsize::End(_) => 3,
    }
}

const fn ellipsize_limit_kind(ellipsize: Ellipsize) -> u8 {
    match ellipsize {
        Ellipsize::None => 0,
        Ellipsize::Start(l) | Ellipsize::Middle(l) | Ellipsize::End(l) => match l {
            EllipsizeHeightLimit::Lines(_) => 1,
            EllipsizeHeightLimit::Height(_) => 2,
        },
    }
}

fn ellipsize_limit_value(ellipsize: Ellipsize) -> u64 {
    match ellipsize {
        Ellipsize::None => 0,
        Ellipsize::Start(l) | Ellipsize::Middle(l) | Ellipsize::End(l) => match l {
            EllipsizeHeightLimit::Lines(v) => v as u64,
            EllipsizeHeightLimit::Height(v) => v.to_bits() as u64,
        },
    }
}

/// Cache for text measurement results to avoid repeated `set_size` + `shape` calls.
///
/// Keyed by `(text, font_family, font_size, line_height, wrap_mode, ellipsize, available_width, available_height)`.
/// The cache stores the resulting `Size` from measuring text layout.
#[derive(Clone, Debug, Default)]
pub struct TextMeasureCache {
    cache: HashMap<TextMeasureKey, Size>,
}

impl TextMeasureCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a cached measurement.
    pub fn get(
        &self,
        text: &str,
        font_family: &str,
        font_size: f32,
        line_height: f32,
        wrap: Wrap,
        ellipsize: Ellipsize,
        available: Size,
    ) -> Option<Size> {
        let key = TextMeasureKey::new(text, font_family, font_size, line_height, wrap, ellipsize, available);
        self.cache.get(&key).copied()
    }

    /// Store a measurement result.
    pub fn insert(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f32,
        line_height: f32,
        wrap: Wrap,
        ellipsize: Ellipsize,
        available: Size,
        size: Size,
    ) {
        let key = TextMeasureKey::new(text, font_family, font_size, line_height, wrap, ellipsize, available);
        self.cache.insert(key, size);
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_and_miss() {
        let mut cache = TextMeasureCache::new();
        let available = Size::new(100.0, f32::INFINITY);

        assert!(cache.get("hello", "", 12.0, 16.0, Wrap::Word, Ellipsize::None, available).is_none());

        cache.insert("hello", "", 12.0, 16.0, Wrap::Word, Ellipsize::None, available, Size::new(30.0, 16.0));

        assert_eq!(
            cache.get("hello", "", 12.0, 16.0, Wrap::Word, Ellipsize::None, available),
            Some(Size::new(30.0, 16.0))
        );

        // Different text -> miss
        assert!(cache.get("world", "", 12.0, 16.0, Wrap::Word, Ellipsize::None, available).is_none());
        // Different font family -> miss
        assert!(cache.get("hello", "Mono", 12.0, 16.0, Wrap::Word, Ellipsize::None, available).is_none());
        // Different font size -> miss
        assert!(cache.get("hello", "", 14.0, 16.0, Wrap::Word, Ellipsize::None, available).is_none());
        // Different line height -> miss
        assert!(cache.get("hello", "", 12.0, 20.0, Wrap::Word, Ellipsize::None, available).is_none());
        // Different wrap -> miss
        assert!(cache.get("hello", "", 12.0, 16.0, Wrap::None, Ellipsize::None, available).is_none());
        // Different ellipsize -> miss
        assert!(cache.get("hello", "", 12.0, 16.0, Wrap::Word, Ellipsize::End(EllipsizeHeightLimit::Lines(1)), available).is_none());
        // Different available width -> miss
        assert!(
            cache.get("hello", "", 12.0, 16.0, Wrap::Word, Ellipsize::None, Size::new(200.0, f32::INFINITY))
                .is_none()
        );
    }
}
