pub struct View {
    pub blocks: bool,
    pub boundary_2d: bool,
    pub interior_2d: bool,
    pub complement: bool,
    pub intersect_area: bool,
    pub boundary_3d: bool,
    pub interior_3d: bool,
    pub convex_hull: bool,
    pub outer_corners: bool,
    pub center_blocks: bool,
    pub bounds: bool,
    pub mirrors: bool,
    pub boundary_2d_colorful: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            blocks: true,
            boundary_2d: false,
            interior_2d: false,
            intersect_area: false,
            complement: false,
            boundary_3d: false,
            interior_3d: false,
            convex_hull: false,
            outer_corners: false,
            center_blocks: false,
            bounds: false,
            mirrors: false,
            boundary_2d_colorful: true,
        }
    }
}
