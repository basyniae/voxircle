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
    pub boundary_2d_graph: bool,
    pub build_sequence_viewport: bool,
    pub build_sequence_panel: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            blocks: true,
            boundary_2d: true, // todo: change default
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
            boundary_2d_graph: false,
            build_sequence_viewport: false,
            build_sequence_panel: true, // todo: change default
        }
    }
}
