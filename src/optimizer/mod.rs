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

mod big_nums;

use self::big_nums::{NearestPerfectSquare, NumStrategy, Product, SimpleConstant};

pub fn optimized_constant(num: u32) -> (String, usize) {
    if num == 0 {
        return (String::from(""), 1);
    }

    let optimizers: Vec<&NumStrategy> =
        vec![&SimpleConstant {}, &Product {}, &NearestPerfectSquare {}];

    let best = optimizers
        .iter()
        .map(|opt| opt.for_num(num))
        .min_by(|x, y| x.0.len().cmp(&y.0.len()));

    return best.unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nums() {
        for i in 1..1000 {
            let (code, _) = optimized_constant(i);
            assert!(code.len() <= i as usize);
        }
    }
}
