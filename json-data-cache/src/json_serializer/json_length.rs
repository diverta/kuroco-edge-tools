/// Helps to distinguish between specifically the length of sting value in a JSON
/// outer part is accounting for the surrounding quotes, but the inner part does not
pub struct JsonLength {
    pub inner: usize,
    pub outer: usize,
}
impl From<usize> for JsonLength {
    fn from(value: usize) -> Self {
        Self {
            inner: value,
            outer: value,
        }
    }
}
impl From<(usize, usize)> for JsonLength {
    fn from((inner, outer): (usize, usize)) -> Self {
        Self {
            inner,
            outer,
        }
    }
}