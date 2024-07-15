use crate::app::helpers::lin_alg::{Mat2, Vec2};

// Captures a bit matrix. The length of the vector should always be edge_length**2
// Order is x first left to right, then y down to up (to match coord space)
#[derive(Default, Clone)]
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

impl Square {
    pub(crate) fn new(index: usize, edge_length: usize, origin: Vec2, center_offset: Vec2, sqrt_quad_form: Mat2) -> Self {
        let lb = Vec2::from([(index % edge_length) as f64,
            (index / edge_length) as f64 ]) - (origin + center_offset);
        let rb = lb + Vec2::from([1.0, 0.0]);
        let lt = lb + Vec2::from([0.0, 1.0]);
        let rt = lb + Vec2::from([1.0, 1.0]);

        // Apply the rotate/scale sqrt_quad_form to all corner points LB, RB, LT, RT
        let m_lb = sqrt_quad_form * lb;
        let m_rb = sqrt_quad_form * rb;
        let m_lt = sqrt_quad_form * lt;
        let m_rt = sqrt_quad_form * rt;

        Square {
            lb, rb, lt, rt, m_lb, m_rb, m_lt, m_rt
        }
    }

    pub fn for_all_corners<F>(&self, check: F) -> bool where F: Fn(Vec2) -> bool {
        check(self.lb) && check(self.rb) && check(self.lt) && check(self.rt)
    }
    
    pub fn for_any_corner<F>(&self, check: F) -> bool where F: Fn(Vec2) -> bool {
        check(self.lb) || check(self.rb) || check(self.lt) || check(self.rt)
    }

    pub fn for_all_edges<F>(&self, check: F) -> bool where F: Fn([Vec2; 2]) -> bool {
        check([self.lb, self.rb]) && check([self.rb, self.rt]) && check([self.lt, self.rt]) && check([self.lt, self.lb])
    }
    
    pub fn for_any_edge<F>(&self, check: F) -> bool where F: Fn([Vec2; 2]) -> bool {
        check([self.lb, self.rb]) || check([self.rb, self.rt]) || check([self.lt, self.rt]) || check([self.lt, self.lb])
    }
    
    pub fn for_all_m_corners<F>(&self, check: F) -> bool where F: Fn(Vec2) -> bool {
        check(self.m_lb) && check(self.m_rb) && check(self.m_lt) && check(self.m_rt)
    }
    
    pub fn for_any_m_corner<F>(&self, check: F) -> bool where F: Fn(Vec2) -> bool {
        check(self.m_lb) || check(self.m_rb) || check(self.m_lt) || check(self.m_rt)
    }

    pub fn for_all_m_edges<F>(&self, check: F) -> bool where F: Fn([Vec2; 2]) -> bool {
        check([self.m_lb, self.m_rb]) && check([self.m_rb, self.m_rt]) && check([self.m_lt, self.m_rt]) && check([self.m_lt, self.m_lb])
    }
    
    pub fn for_any_m_edge<F>(&self, check: F) -> bool where F: Fn([Vec2; 2]) -> bool {
        check([self.m_lb, self.m_rb]) || check([self.m_rb, self.m_rt]) || check([self.m_lt, self.m_rt]) || check([self.m_lt, self.m_lb])
    }
}