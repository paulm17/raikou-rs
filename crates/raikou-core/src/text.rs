#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

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

    pub const fn normalized(self) -> Self {
        Self::new(self.min(), self.max())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CaretAffinity {
    Upstream,
    #[default]
    Downstream,
}
