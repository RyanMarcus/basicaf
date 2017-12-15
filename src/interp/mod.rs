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
 
use std::char;

mod full_tests;

pub struct BFEnv {
    data: Vec<u32>,
    pt: usize
}

impl BFEnv {
    pub fn new() -> BFEnv {
        let mut to_r = BFEnv {
            data: Vec::new(),
            pt: 0
        };

        to_r.data.push(0);
        return to_r;
    }

    fn execute_single(&mut self, instruction: char) -> Option<String> {
        match instruction {
            '+' => {
                self.data[self.pt] += 1
            },

            '-' => {
                self.data[self.pt] -= 1
            },

            '.' => {
                return Some(char::from_u32(self.data[self.pt]).unwrap().to_string());
            },

            '>' => {
                self.pt += 1;
                
                if self.pt >= self.data.len() {
                    self.data.push(0);
                }                
            },

            '<' => {
                if self.pt == 0 {
                    panic!("Trying to move past the zero element!");
                }
                
                self.pt -= 1;
            },

            _ => { }
        };

        return None;
    }

    fn find_matching_close(open: usize, prgm: &Vec<char>) -> usize {
        let mut count = 0;

        for pos in open..prgm.len() {
            count += match prgm[pos] {
                '[' => 1,
                ']' => -1,
                _ => 0
            };

            if count == 0 {
                return pos;
            }
        }

        panic!("Could not find matching close bracket for opening bracket at position {}",
               open);
    }

    fn find_matching_open(close: usize, prgm: &Vec<char>) -> usize {
        let mut count = 1;

        for pos in (0..close).rev() {
            count += match prgm[pos] {
                ']' => 1,
                '[' => -1,
                _ => 0
            };

            if count == 0 {
                return pos;
            }
        }

        panic!("Could not find matching open bracket for closing bracket at position {}",
               close);
    }

    pub fn execute(&mut self, source: String) -> String {
        let mut result = String::new();
        let program = {
            let mut p = Vec::new();
            p.extend(source.chars());
            p
        };
        
        let mut pc = 0;

        while pc < program.len() {
            match program[pc] {
                '[' => {
                    if self.data[self.pt] == 0 {
                        pc = BFEnv::find_matching_close(pc, &program);
                    } else {
                        pc += 1;
                    }
                },

                ']' => {
                    if self.data[self.pt] != 0 {
                        pc = BFEnv::find_matching_open(pc, &program);
                    } else {
                        pc += 1;
                    }
                },

                _ => {
                    let res = self.execute_single(program[pc]);
                    match res {
                        Some(s) => result.push_str(s.as_str()),
                        None => {}
                    };
                    
                    pc += 1;
                }
            }
        }

        return result;
    }

    #[cfg(test)]
    pub fn data_at(&self, pos: usize) -> u32 {
        return self.data[pos];
    }

    #[cfg(test)]
    pub fn ptr_value(&self) -> usize {
        return self.pt;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn simple_test() {
        let mut interp = BFEnv::new();
        interp.execute(String::from("+++>++>+"));
        assert_eq!(interp.data[0], 3);
        assert_eq!(interp.data[1], 2);
        assert_eq!(interp.data[2], 1);
    }

    #[test]
    fn loop_test() {
        let mut interp = BFEnv::new();
        interp.execute(String::from("+++[>++<-]"));
        assert_eq!(interp.data[0], 0);
        assert_eq!(interp.data[1], 6);
    }

    #[test]
    fn power_loop() {
        let mut interp = BFEnv::new();
        // 2 * (3 * 4)^5
        interp.execute(String::from("++>>+++++[<<[>+++<-]>[<++++>-]>-]<<"));
        assert_eq!(interp.data[0], 497664);
        assert_eq!(interp.data[1], 0);
        assert_eq!(interp.data[2], 0);

    }
}
