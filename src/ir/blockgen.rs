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
use parser::structs::{DBCommand, DBStmt};
use std::collections::HashSet;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SpecialOut {
    Next(usize),
    Return(usize),
    None,
}

#[derive(Debug, Clone)]
pub struct Block {
    root: bool,
    pub in_blocks: Vec<usize>,
    pub out_blocks: Vec<usize>,
    pub special_out: SpecialOut,
    pub cmds: Vec<DBCommand>,
    pub is_loop: bool,
    pub loop_exits: Vec<usize>,
    pub loop_nodes: HashSet<usize>,
}

impl Block {
    fn new() -> Block {
        return Block {
            root: false,
            in_blocks: Vec::new(),
            out_blocks: Vec::new(),
            special_out: SpecialOut::None,
            cmds: Vec::new(),
            is_loop: false,
            loop_exits: Vec::new(),
            loop_nodes: HashSet::new(),
        };
    }

    fn new_root() -> Block {
        return Block {
            root: true,
            in_blocks: Vec::new(),
            out_blocks: Vec::new(),
            special_out: SpecialOut::None,
            cmds: Vec::new(),
            is_loop: false,
            loop_exits: Vec::new(),
            loop_nodes: HashSet::new(),
        };
    }

    pub fn new_loop(exits: Vec<usize>, nodes: HashSet<usize>) -> Block {
        return Block {
            root: false,
            in_blocks: Vec::new(),
            out_blocks: Vec::new(),
            special_out: SpecialOut::None,
            cmds: Vec::new(),
            is_loop: true,
            loop_exits: exits,
            loop_nodes: nodes,
        };
    }

    fn add_out_block(&mut self, other: usize) {
        self.out_blocks.push(other);
    }

    fn add_in_block(&mut self, other: usize) {
        self.in_blocks.push(other);
    }

    fn clear_out_blocks(&mut self) {
        self.out_blocks.clear();
    }

    fn clear_in_blocks(&mut self) {
        self.in_blocks.clear();
    }
}

enum BlockSplitType {
    NoBreak,
    BreakAfter,
    BreakBefore,
    BreakBoth,
}

pub fn get_goto_targets(stmts: &[DBCommand]) -> HashSet<u32> {
    let mut goto_targets = HashSet::new();
    for cmd in stmts.iter() {
        match cmd.cmd {
            DBStmt::GOSUB { lineno } | DBStmt::IF { lineno, .. } | DBStmt::GOTO { lineno } => {
                goto_targets.insert(lineno);
            }

            _ => {}
        };
    }

    return goto_targets;
}

fn get_all_program_data(stmts: &[DBCommand]) -> Vec<u32> {
    let mut to_r = Vec::new();
    for cmd in stmts.iter() {
        if let DBStmt::DATA { seq: ref data } = cmd.cmd {
            for flt in data.iter() {
                to_r.push(*flt as u32);
            }
        }
    }

    return to_r;
}

pub fn to_blocks(stmts: Vec<DBCommand>) -> Vec<Block> {
    let mut all_stmts = stmts;
    // first, we need to resolve all the READ and DATA commands.
    // to do this, we first collect a vector of all the DATA items.
    // Then, we associate each READ command with the appropiate data
    let mut data = get_all_program_data(&all_stmts);
    data.reverse();

    for cmd in all_stmts.iter_mut() {
        let num_data_items = match cmd.cmd {
            DBStmt::READ { varnames: ref x } => x.len(),

            _ => 0,
        };
        let ln = cmd.ln;
        for _ in 0..num_data_items {
            cmd.add_data(
                data.pop()
                    .expect(format!("Not enough DATA for READ on line {}", ln).as_str()),
            );
        }
    }

    // iterate through each command, splitting them
    // into blocks. To do this, we will first build
    // a vector of every GOTO target. This will be used
    // to additionally add block splits before each
    // GOTO target.

    let goto_targets = get_goto_targets(&all_stmts);

    let mut blocks = Vec::new();
    let mut curr_block = Block::new_root();

    for cmd in all_stmts {
        let mut new_block = match cmd.cmd {
            DBStmt::END
            | DBStmt::RETURN
            | DBStmt::NEXT { .. }
            | DBStmt::GOSUB { .. }
            | DBStmt::GOTO { .. }
            | DBStmt::IF { .. } => BlockSplitType::BreakAfter,

            DBStmt::FOR { .. } => BlockSplitType::BreakBoth,

            _ => BlockSplitType::NoBreak,
        };

        if goto_targets.contains(&cmd.ln) {
            new_block = match new_block {
                BlockSplitType::NoBreak | BlockSplitType::BreakBefore => {
                    BlockSplitType::BreakBefore
                }
                BlockSplitType::BreakAfter | BlockSplitType::BreakBoth => BlockSplitType::BreakBoth,
            };
        }

        match new_block {
            BlockSplitType::BreakBefore => {
                if !curr_block.cmds.is_empty() {
                    blocks.push(curr_block);
                    curr_block = Block::new();
                }
                curr_block.cmds.push(cmd);
            }

            BlockSplitType::BreakAfter => {
                curr_block.cmds.push(cmd);
                blocks.push(curr_block);
                curr_block = Block::new();
            }

            BlockSplitType::BreakBoth => {
                if !curr_block.cmds.is_empty() {
                    blocks.push(curr_block);
                    curr_block = Block::new();
                }

                curr_block.cmds.push(cmd);
                blocks.push(curr_block);
                curr_block = Block::new();
            }

            BlockSplitType::NoBreak => {
                curr_block.cmds.push(cmd);
            }
        }
    }

    if !curr_block.cmds.is_empty() {
        panic!(
            "Last block was not empty -- does the \
             program end with an END statment? Last block was: {:?}",
            curr_block
        );
    }

    link_blocks(&mut blocks);

    ensure_no_dead_code(&blocks);
    return blocks;
}

enum FollowType {
    NextLineFollows,
    NextLineDoesNotFollow,
}

macro_rules! get_last_cmd {
    ($blk:expr) => {
        {
            $blk.cmds.iter().last()
                .expect("Block did not have any commands!")
                .clone()
        }

    }
}

macro_rules! get_first_cmd {
    ($blk:expr) => {
        {
            $blk.cmds.iter().take(1).last()
                .expect("Block did not have any commands!")
                .clone()
        }

    }
}

pub fn link_blocks(blocks: &mut Vec<Block>) {
    // first, build a map from line no to block.
    let mut lno_map = HashMap::new();

    for (idx, block) in blocks.iter().enumerate() {
        let block_cell = block;
        let cmds_iter = block_cell.cmds.iter();
        for stmt in cmds_iter {
            lno_map.insert(stmt.ln, idx);
        }
    }

    let mut i = 0;
    // need a while loop here because blocks.len will change
    while i < blocks.len() {
        // forward link each block
        let last = get_last_cmd!(blocks[i]);

        let nl = match last.cmd {
            DBStmt::FOR { ref varname, .. } => {
                let parent_varname = varname;
                // scan forward to find the matching call to NEXT
                let mut found_match = false;
                for j in i + 1..blocks.len() - 1 {
                    let cand_last = get_last_cmd!(blocks[j]);
                    if let DBStmt::NEXT { ref varname } = cand_last.cmd {
                        if parent_varname.trim() == varname.trim() {
                            found_match = true;
                            blocks[j].special_out = SpecialOut::Next(i);

                            blocks[i].add_in_block(j);

                            blocks[i].add_out_block(j + 1);
                            blocks[j + 1].add_in_block(i);
                            break;
                        }
                    }
                }

                if !found_match {
                    panic!("No matching NEXT statement for FOR statement!");
                }
                FollowType::NextLineFollows
            }

            DBStmt::NEXT { .. } => {
                match blocks[i].special_out {
                    SpecialOut::Next(..) => {}
                    _ => {
                        panic!("NEXT statement without preceeding FOR loop!");
                    }
                }

                FollowType::NextLineDoesNotFollow
            }

            DBStmt::GOSUB { ref lineno } => {
                // add the approp lineno to our outlist
                // then, add the line after us to our outlist

                let out_block = *lno_map
                    .get(lineno)
                    .expect("Could not find matching lineno for GOSUB");

                // check to make sure there is a return...
                let mut found_matching = false;
                let mut subroutine_start = 0;
                let mut subroutine_return = 0;

                for j in 0..blocks.len() {
                    if j == i {
                        continue;
                    }

                    let last = get_last_cmd!(blocks[j]);

                    if last.ln < *lineno {
                        continue;
                    }

                    if let DBStmt::RETURN = last.cmd {
                        if !blocks[j].out_blocks.is_empty() {
                            // TODO copy all the blocks from out_block
                            // to j. Add them to the end of the block
                            // list. Link to those.
                            let mut copy = Vec::new();

                            // Clippy suggests using an iterator,
                            // but this seems much cleaner than
                            // the alternative:
                            // blocks.iter().take(j+1).skip(out_block)
                            #[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
                            for i in out_block..j + 1 {
                                copy.push(blocks[i].clone());
                            }

                            let num_copied = copy.len();
                            let curr_blocks = blocks.len();
                            copy[0].clear_in_blocks();
                            copy[num_copied - 1].clear_out_blocks();

                            blocks.extend(copy);

                            subroutine_start = curr_blocks;
                            subroutine_return = blocks.len() - 1;
                            found_matching = true;
                            break;
                        } else {
                            subroutine_start = out_block;
                            subroutine_return = j;
                            found_matching = true;
                            break;
                        }
                    }
                }

                if !found_matching {
                    panic!("Could not find a RETURN for GOSUB");
                }

                blocks[subroutine_return].add_out_block(i + 1);
                blocks[i + 1].add_in_block(subroutine_return);

                blocks[subroutine_start].add_in_block(i);
                blocks[i].add_out_block(subroutine_start);

                blocks[i].special_out = SpecialOut::Return(i + 1);
                blocks[i + 1].add_in_block(i);

                FollowType::NextLineDoesNotFollow
            }

            DBStmt::GOTO { ref lineno } => {
                let out_block = *lno_map
                    .get(lineno)
                    .expect("Could not find line used by GOTO");

                blocks[out_block].add_in_block(i);
                blocks[i].add_out_block(out_block);

                FollowType::NextLineDoesNotFollow
            }

            DBStmt::IF { ref lineno, .. } => {
                // add the true branch as an output
                let out_block = *lno_map.get(lineno).expect("Could not find line used by IF");

                blocks[out_block].add_in_block(i);
                blocks[i].add_out_block(out_block);

                FollowType::NextLineFollows
            }

            DBStmt::END | DBStmt::RETURN => FollowType::NextLineDoesNotFollow,

            _ => FollowType::NextLineFollows,
        };

        if let FollowType::NextLineFollows = nl {
            if i + 1 > blocks.len() {
                panic!("Program does not end with an END statement!");
            }

            blocks[i].add_out_block(i + 1);
            blocks[i + 1].add_in_block(i);
        }

        i += 1;
    }
}

fn ensure_no_dead_code(blocks: &[Block]) {
    for i in blocks.iter() {
        if !i.root && i.in_blocks.is_empty() {
            let last = get_last_cmd!(i);
            panic!("Dead code in block ending at {}", last.ln);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use parser;

    #[test]
    #[should_panic]
    fn dead_code_test() {
        let test_program = "\
10 PRINT \"test\"
15 GOTO 35
20 LET X = 40 * 3
30 PRINT X
35 FOR i = 0 TO 40
40 PRINT i
50 NEXT i
70 END\n";

        let parse = parser::parse_bytes(test_program.as_bytes());
        to_blocks(parse);
    }

    #[test]
    fn all_types() {
        let test_program = "\
0  LET Y = 5
1  GOTO 10
2  LET X = 5*Y
3  RETURN
10 PRINT \"test\"
15 GOTO 35
35 FOR i = 0 TO 40
40 PRINT i
50 NEXT i
60 GOSUB 2
65 IF X > 20 THEN 70
67 PRINT X
70 END\n";

        let parse = parser::parse_bytes(test_program.as_bytes());
        to_blocks(parse);
    }

}
