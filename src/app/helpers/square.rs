use crate::app::helpers::linear_algebra::{Mat2, Vec2};

/// Abstraction of a square on the grid and its image under the sqrt_quad_form.
/// Contains easy ::new method and some checks that run over all points or edges (to prevent
///  excessive repetition in the algorithms themselves and make them easier to read)
/// The points on the grid (shifted to get the superellipse center as the point (0,0)) are called
///  lb, rb, lt, rt, (left/right bottom/top), the corresponding points in the coords where the
///  superellipse is aligned to the grid and normalized are prefixed by m_ (modified) (i.e., these
///  points are multiplied by the sqrt_quad_form).
#[derive(Default, Clone, Debug)]
pub struct Square {
    pub lb: Vec2,
    pub rb: Vec2,
    pub lt: Vec2,
    pub rt: Vec2,
    pub m_lb: Vec2,
    pub m_rb: Vec2,
    pub m_lt: Vec2,
    pub m_rt: Vec2,
}

#[allow(dead_code)] // the unused methods make sense for completeness
impl Square {
    /// Create new square from the usual parameters
    pub(crate) fn new(
        index: usize,
        edge_length: usize,
        origin: Vec2,
        center_offset: Vec2,
        sqrt_quad_form: Mat2,
    ) -> Self {
        let lb = Vec2::from([(index % edge_length) as f64, (index / edge_length) as f64])
            - (origin + center_offset);
        let rb = lb + Vec2::from([1.0, 0.0]);
        let lt = lb + Vec2::from([0.0, 1.0]);
        let rt = lb + Vec2::from([1.0, 1.0]);

        // Apply the rotate/scale sqrt_quad_form to all corner points LB, RB, LT, RT
        let m_lb = sqrt_quad_form * lb;
        let m_rb = sqrt_quad_form * rb;
        let m_lt = sqrt_quad_form * lt;
        let m_rt = sqrt_quad_form * rt;

        Square {
            lb,
            rb,
            lt,
            rt,
            m_lb,
            m_rb,
            m_lt,
            m_rt,
        }
    }

    /// Does the supplied check hold for all corners?
    pub fn for_all_corners<F>(&self, check: F) -> bool
    where
        F: Fn(Vec2) -> bool,
    {
        check(self.lb) && check(self.rb) && check(self.lt) && check(self.rt)
    }

    /// Does the supplied check hold for any corner?
    pub fn for_any_corner<F>(&self, check: F) -> bool
    where
        F: Fn(Vec2) -> bool,
    {
        check(self.lb) || check(self.rb) || check(self.lt) || check(self.rt)
    }

    /// Does the supplied check hold for all edges?
    pub fn for_all_edges<F>(&self, check: F) -> bool
    where
        F: Fn([Vec2; 2]) -> bool,
    {
        check([self.lb, self.rb])
            && check([self.rb, self.rt])
            && check([self.lt, self.rt])
            && check([self.lt, self.lb])
    }

    /// Does the supplied check hold for any edge?
    pub fn for_any_edge<F>(&self, check: F) -> bool
    where
        F: Fn([Vec2; 2]) -> bool,
    {
        check([self.lb, self.rb])
            || check([self.rb, self.rt])
            || check([self.lt, self.rt])
            || check([self.lt, self.lb])
    }

    /// Does the supplied check hold for all modified corners?
    pub fn for_all_m_corners<F>(&self, check: F) -> bool
    where
        F: Fn(Vec2) -> bool,
    {
        check(self.m_lb) && check(self.m_rb) && check(self.m_lt) && check(self.m_rt)
    }

    /// Does the supplied check hold for any modified corner?
    pub fn for_any_m_corner<F>(&self, check: F) -> bool
    where
        F: Fn(Vec2) -> bool,
    {
        check(self.m_lb) || check(self.m_rb) || check(self.m_lt) || check(self.m_rt)
    }

    /// Does the supplied check hold for all modified edges?
    pub fn for_all_m_edges<F>(&self, check: F) -> bool
    where
        F: Fn([Vec2; 2]) -> bool,
    {
        check([self.m_lb, self.m_rb])
            && check([self.m_rb, self.m_rt])
            && check([self.m_lt, self.m_rt])
            && check([self.m_lt, self.m_lb])
    }

    /// Does the supplied check hold for any modified edge?
    pub fn for_any_m_edge<F>(&self, check: F) -> bool
    where
        F: Fn([Vec2; 2]) -> bool,
    {
        check([self.m_lb, self.m_rb])
            || check([self.m_rb, self.m_rt])
            || check([self.m_lt, self.m_rt])
            || check([self.m_lt, self.m_lb])
    }
}
