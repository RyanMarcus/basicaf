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


mod allocator;

#[macro_use]
mod blockgen;
mod block_to_ir;
mod goto_elim;

use std::io::Write;
use self::blockgen::{Block, SpecialOut};
use parser::structs::{DBCommand};
use codegen::{BFQuad};
use self::goto_elim::{eliminate_gotos};

pub fn elim_gotos(stmts: &mut Vec<Block>) {
    return eliminate_gotos(stmts);
}

pub fn to_blocks(stmts: Vec<DBCommand>) -> Vec<Block> {
    let res = blockgen::to_blocks(stmts);
    return res;
}

pub fn to_ir(blocks: Vec<Block>, const_opt: bool) -> Vec<BFQuad> {
    let mut ir_gen = block_to_ir::BlockToIR::new(blocks, const_opt);
    ir_gen.generate_ir();
    return ir_gen.get_ir();
}

pub fn to_graphviz(blocks: Vec<Block>) -> String{
    let mut w = Vec::new();

    write!(&mut w, "digraph G {{\n").unwrap();

    for (idx, b) in blocks.iter().enumerate() {
        match b.is_loop {
            false => {
                let lno_last;
                let cmd_last;
                {
                    let last = get_last_cmd!(b);
                    lno_last = last.ln;
                    cmd_last = last.cmd;
                }
                
                let lno_first;
                {
                    let first = get_first_cmd!(b);
                    lno_first = first.ln;
                }
                
                write!(&mut w, "{} [label=\"{}: {} - {} ({})\"];\n",
                       idx, idx, lno_first, lno_last,
                       cmd_last.get_string_type()).unwrap();
            },

            true => {
                write!(&mut w, "{} [label=\"Loop\"];\n",
                       idx).unwrap();
            }
        };
    }

    for (idx, b) in blocks.iter().enumerate() {
        match b.is_loop {
            true => {
                for out_blk in b.out_blocks.iter() {
                    write!(&mut w, "{} -> {};\n", idx, out_blk).unwrap();
                }
            },

            false => {
                match b.special_out {
                    SpecialOut::Next (pos) => {
                        write!(&mut w, "{} -> {} [style=dotted];\n",
                               idx, pos).unwrap();
                    },

                    SpecialOut::Return (pos) => {
                        write!(&mut w, "{} -> {} [style=dotted];\n",
                               idx, pos).unwrap();
                    }

                    SpecialOut::None => {}
                };

                for out_blk in b.out_blocks.iter() {
                    write!(&mut w, "{} -> {};\n", idx, out_blk).unwrap();
                }
            }
        };
    }

    write!(&mut w, "}}\n").unwrap();
    
    return String::from_utf8(w).unwrap();
}
