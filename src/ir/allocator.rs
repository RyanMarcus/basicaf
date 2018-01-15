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
use std::collections::BTreeSet;
use std::cmp;

#[derive(Debug)]
pub struct Allocator {
    used_set: BTreeSet<u32>,
    free_set: BTreeSet<u32>,
}

impl Allocator {
    pub fn new() -> Allocator {
        let mut a = Allocator {
            used_set: BTreeSet::new(),
            free_set: BTreeSet::new(),
        };

        for i in 0..30 {
            a.free_set.insert(i);
        }

        return a;
    }

    pub fn reserve(&mut self) -> u32 {
        if self.free_set.is_empty() {
            // nothing is available.
            let val = self.used_set.len() + self.free_set.len();
            let cval = val as u32;
            self.used_set.insert(cval);

            return cval;
        }

        // pull something out of the free set,
        // add it to the used set, and return it.
        let el = *self.free_set.iter().take(1).last().unwrap();
        self.free_set.remove(&el);
        self.used_set.insert(el);

        return el;
    }

    pub fn free(&mut self, var: u32) {
        if !self.used_set.contains(&var) {
            panic!("trying to free a variable ({}) that was not in use!", var);
        }

        self.used_set.remove(&var);
        self.free_set.insert(var);
    }

    pub fn reserve_range(&mut self, size: u32) -> u32 {
        // first, see if our free set has any runs of size in it
        let tot_size = (self.used_set.len() + self.free_set.len()) as u32;

        // if we don't have that many at all, we def
        // don't have a run.
        if tot_size > size {
            for i in 0..tot_size - size {
                let mut found = true;
                for j in i..size + i {
                    if !self.free_set.contains(&j) {
                        found = false;
                        break;
                    }
                }

                if found {
                    for j in i..size + i {
                        self.free_set.remove(&j);
                        self.used_set.insert(j);
                    }

                    return i;
                }
            }
        }

        // we weren't able to find a run in the free list.
        // make a new one.
        for i in tot_size..tot_size + size {
            self.used_set.insert(i);
        }

        return tot_size;
    }

    pub fn reserve_array(&mut self, size: u32) -> u32 {
        // arrays live for the length of the whole program,
        // so we should always stick them after the last possible thing...
        let used_max = *self.used_set.iter().last().unwrap_or(&0);
        let free_max = *self.free_set.iter().last().unwrap_or(&0);
        let max = cmp::max(used_max, free_max) + 1;
        for i in max..max + size + 4 {
            self.used_set.insert(i);
        }
        return max;
    }

    pub fn free_array(&mut self, loc: u32, size: u32) {
        for i in loc..(loc + size + 4) {
            self.free(i);
        }
    }

    pub fn assert_empty(&mut self) {
        if !self.used_set.is_empty() {
            panic!("some variables were not free: {:?}", self.used_set);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alloc_when_empty() {
        let mut a = Allocator::new();

        let v1 = a.reserve();
        let v2 = a.reserve();
        let v3 = a.reserve();

        assert!(v1 != v2);
        assert!(v1 != v3);
        assert!(v2 != v3);
    }

    #[test]
    fn alloc_and_free() {
        let mut a = Allocator::new();

        let v1 = a.reserve();
        let v2 = a.reserve();

        assert!(v1 != v2);

        a.free(v1);

        let v3 = a.reserve();

        assert!(v1 == v3);
    }

    #[test]
    fn range() {
        let mut a = Allocator::new();

        a.reserve_range(5);
        assert_eq!(a.used_set.len(), 5);
    }

    #[test]
    #[should_panic]
    fn panic_on_invalid_free() {
        let mut a = Allocator::new();

        a.free(5);
    }

    #[test]
    fn reserve_array() {
        let mut a = Allocator::new();

        let t = a.reserve_array(100);
        a.free_array(t, 100);

        a.assert_empty();
    }
}
