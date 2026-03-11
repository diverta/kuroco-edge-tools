#[derive(Debug)]
pub struct Range {
    pub start: usize, // Including
    pub end: usize // Excluding
}

impl From<(usize, usize)> for Range {
    fn from(value: (usize, usize)) -> Self {
        Self {
            start: value.0,
            end: value.1
        }
    }
}