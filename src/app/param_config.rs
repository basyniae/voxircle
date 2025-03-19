// describe
/// Additional global options for how parameters are read from the options
pub struct ParamConfig {
    pub single_radius: bool,
}

impl Default for ParamConfig {
    fn default() -> Self {
        ParamConfig {
            single_radius: true,
        }
    }
}
