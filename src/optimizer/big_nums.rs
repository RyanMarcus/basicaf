// < begin copyright >
// Copyright Ryan Marcus 2017
//
// This file is part of basicaf.
//
// basicaf is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// basicaf is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with basicaf.  If not, see <http://www.gnu.org/licenses/>.
//
// < end copyright >

pub trait NumStrategy {
    fn for_num(&self, num: u32) -> (String, usize);
}

pub struct SimpleConstant {}

impl NumStrategy for SimpleConstant {
    fn for_num(&self, num: u32) -> (String, usize) {
        let mut to_r = String::new();

        for _ in 0..num {
            to_r.push('+');
        }

        return (to_r, 1);
    }
}

pub struct Product {}

impl NumStrategy for Product {
    fn for_num(&self, num: u32) -> (String, usize) {
        // find the sqrt...
        let mut sqrt = (f64::from(num)).sqrt().floor() as u32;

        while num % sqrt != 0 && sqrt <= num {
            sqrt += 1;
        }

        let other_factor = num / sqrt;

        let mut to_r = String::new();

        to_r.push('>');
        for _ in 0..other_factor {
            to_r.push('+');
        }

        to_r.push_str("[<");

        for _ in 0..sqrt {
            to_r.push('+');
        }

        to_r.push_str(">-]<");

        return (to_r, 2);
    }
}

pub struct NearestPerfectSquare {}

impl NumStrategy for NearestPerfectSquare {
    fn for_num(&self, num: u32) -> (String, usize) {
        // find the sqrt...
        let sqrt = (f64::from(num)).sqrt().floor() as u32;

        // find the nearest perfect square
        let ps = sqrt * sqrt;

        // find the difference
        let diff = num - ps;

        let p = Product {};
        let sc = SimpleConstant {};

        let (mut code, _) = p.for_num(ps);
        let (const_code, _) = sc.for_num(diff);

        code.push_str(const_code.as_str());

        return (code, 2);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use interp::BFEnv;

    #[test]
    fn test_simple() {
        let sc = SimpleConstant {};

        for i in 1..1000 {
            let mut env = BFEnv::new();
            let (code, size) = sc.for_num(i);
            assert_eq!(size, 1);

            env.execute(code);
            assert_eq!(env.data_at(0), i);
            assert_eq!(env.ptr_value(), 0);
        }
    }

    #[test]
    fn test_product() {
        let p = Product {};

        for i in 1..1000 {
            let mut env = BFEnv::new();
            let (code, size) = p.for_num(i);
            assert_eq!(size, 2);

            env.execute(code);
            assert_eq!(env.data_at(0), i);
            assert_eq!(env.ptr_value(), 0);
        }
    }

    #[test]
    fn nps() {
        let nps = NearestPerfectSquare {};

        for i in 1..1000 {
            let mut env = BFEnv::new();
            let (code, size) = nps.for_num(i);
            assert_eq!(size, 2);

            println!("{} {}", i, code);

            env.execute(code);
            assert_eq!(env.data_at(0), i);
            assert_eq!(env.ptr_value(), 0);
        }
    }
}
