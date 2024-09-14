#[derive(Debug, PartialEq)]
pub enum SamplingMethod {
    AllSamples,
    AnySamples,
    Percentage(f64),
}
