//! Backend-agnostic text primitives: ranges and caret affinity.

/// A half-open range of UTF-8 byte offsets into a text buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// A collapsed (empty) range anchored at `offset`.
    pub const fn collapsed(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    pub const fn is_collapsed(self) -> bool {
        self.start == self.end
    }

    pub const fn min(self) -> usize {
        if self.start <= self.end {
            self.start
        } else {
            self.end
        }
    }

    pub const fn max(self) -> usize {
        if self.start >= self.end {
            self.start
        } else {
            self.end
        }
    }

    /// Returns the range with `start` and `end` in ascending order.
    pub const fn normalized(self) -> Self {
        Self::new(self.min(), self.max())
    }
}

/// Which side of a character boundary a collapsed caret sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CaretAffinity {
    Upstream,
    #[default]
    Downstream,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_range() {
        let range = TextRange::collapsed(4);
        assert!(range.is_collapsed());
        assert_eq!(range, TextRange::new(4, 4));
    }

    #[test]
    fn non_collapsed_range() {
        let range = TextRange::new(2, 5);
        assert!(!range.is_collapsed());
    }

    #[test]
    fn normalization_reorders_reversed_range() {
        let range = TextRange::new(9, 3);
        assert_eq!(range.min(), 3);
        assert_eq!(range.max(), 9);
        assert_eq!(range.normalized(), TextRange::new(3, 9));
    }

    #[test]
    fn normalized_keeps_ordered_range() {
        let range = TextRange::new(1, 4);
        assert_eq!(range.normalized(), range);
    }

    #[test]
    fn caret_affinity_defaults_to_downstream() {
        assert_eq!(CaretAffinity::default(), CaretAffinity::Downstream);
    }
}
