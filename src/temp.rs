use std::{fmt::Display, ops::AddAssign, simd::Simd};

pub struct Temp {
    min: i32,
    sum: i32,
    cnt: i32,
    max: i32,
}

impl Default for Temp {
    fn default() -> Self {
        Self {
            min: i32::MAX,
            sum: 0,
            cnt: 0,
            max: i32::MIN,
        }
    }
}

impl Display for Temp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = Simd::from([self.min as f32, self.sum as f32, self.max as f32]);
        let w = Simd::from([10.0, self.cnt as f32, 10.0]);
        let [min, avg, max] = (v / w).to_array();

        write!(f, "{min:.1}/{avg:.1}/{max:.1}")
    }
}

impl AddAssign for Temp {
    fn add_assign(&mut self, rhs: Self) {
        self.min = rhs.min.min(self.min);
        self.sum += rhs.sum;
        self.cnt += rhs.cnt;
        self.max = rhs.max.max(self.max);
    }
}

impl AddAssign<i32> for Temp {
    fn add_assign(&mut self, rhs: i32) {
        self.min = rhs.min(self.min);
        self.sum += rhs;
        self.cnt += 10;
        self.max = rhs.max(self.max);
    }
}

#[rustfmt::skip]
pub fn parse(temperature: &[u8]) -> i32 {
    let f = |x| (x - b'0') as i32;
    match temperature {
        [b'-', h, d, b'.', u] => -f(h) * 100 - f(d) * 10 - f(u),
        [b'-',    d, b'.', u] =>             - f(d) * 10 - f(u),
        [      h, d, b'.', u] =>  f(h) * 100 + f(d) * 10 + f(u),
        [         d, b'.', u] =>               f(d) * 10 + f(u),
        _ => unreachable!(),
    }
}
