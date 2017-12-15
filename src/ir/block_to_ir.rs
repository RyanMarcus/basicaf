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
use ir::blockgen::{Block, SpecialOut};
use ir::allocator::{Allocator};
use std::collections::{HashMap};
use codegen::{BFQuad};
use parser::structs::{DBStmt, DBExpr, DBArrayDef, DBLetTarget};
use parser::ast::{Expr, OpCode};
use optimizer;

pub struct BlockToIR {
    ir: Vec<BFQuad>,
    alloc: Allocator,
    def_map: HashMap<String, (String, DBExpr)>,
    array_t: HashMap<String, (Vec<usize>, u32)>,
    symbol_t: HashMap<String, u32>,
    blocks: Vec<Block>,
    loop_stack: Vec<(usize, u32, u32)>,
    const_opt: bool,
    already_used_array: bool
}


macro_rules! get_and_zero {
    ($self:ident) => {
        {
            let v = $self.alloc.reserve();
            $self.ir.push(BFQuad::Zero(v));
            v
        }
    }
}

macro_rules! mark_loop_done {
    ($self:ident, $loop_var:expr, $cond_var:expr, $idx:expr) => {
        comment!($self, format!("marking loop {} as complete with exit {}",
                         $loop_var, $idx + 1));
        
        $self.ir.push(BFQuad::To($loop_var));
        $self.ir.push(BFQuad::Zero($loop_var));
        $self.ir.push(BFQuad::To($cond_var));
        $self.ir.push(BFQuad::Constant(($idx as u32) + 1));

        comment!($self, "done marking loop complete");
    }
}

macro_rules! comment {
    ($self: expr, $txt:expr) => {
        let mut s = String::from($txt);
        s.insert(0, '\n');
        s.push('\n');
        $self.ir.push(BFQuad::Comment(s));
    }
}

impl BlockToIR {
    pub fn new(blocks: Vec<Block>, const_opt: bool) -> BlockToIR {
        return BlockToIR {
            ir: Vec::new(),
            alloc: Allocator::new(),
            def_map: HashMap::new(),
            array_t: HashMap::new(),
            symbol_t: HashMap::new(),
            blocks: blocks,
            loop_stack: Vec::new(),
            const_opt: const_opt,
            already_used_array: false
        };
    }

    pub fn done(&mut self) {
        for v in self.symbol_t.values() {
            self.alloc.free(*v);
        }

        for &(ref dims, ref pos) in self.array_t.values() {
            let mut accum: u32 = 1;
            for dim in dims {
                accum *= *dim as u32;
            }
            
            self.alloc.free_array(*pos, accum);
        }
        
        self.alloc.assert_empty();
    }

    pub fn generate_ir(&mut self) {
        self.block_to_ir(0);
    }

    pub fn get_ir(mut self) -> Vec<BFQuad> {
        self.done();
        let x = self.ir;
        return x;
    }

    
    fn block_to_ir(&mut self, block: usize) {
        let is_loop = self.blocks[block].is_loop;
        
        let should_be_end = match is_loop {
            true => {
                // we are starting a loop!
                let loop_var = get_and_zero!(self);
                let cond_var = get_and_zero!(self);
                comment!(self, format!("Starting loop: cond_var is {} and loop_var is {}",
                                       cond_var, loop_var));

                self.loop_stack.push((block, loop_var, cond_var));

                self.ir.push(BFQuad::To(loop_var));
                self.ir.push(BFQuad::Constant(1));
                self.ir.push(BFQuad::RawBF("["));
                comment!(self, "start of loop body");
                let oblock = self.blocks[block].out_blocks[0];
                self.block_to_ir(oblock);
                comment!(self, "end of loop body");
                self.ir.push(BFQuad::To(loop_var));
                self.ir.push(BFQuad::RawBF("]"));
                             
                self.loop_stack.pop();
                comment!(self, "Finished loop code: now checking exit conditions");
                
                // depending on the value of the loop_var, we need to branch to a
                // specific place.
                let loop_exits = self.blocks[block].loop_exits.clone();
                for (idx, out_blk) in loop_exits.iter().enumerate() {
                    comment!(self, format!("Checking loop condition {}", idx + 1));
                    let cond = get_and_zero!(self);
                    let t1 = get_and_zero!(self);
                    let t2 = get_and_zero!(self);
                    let v = get_and_zero!(self);
                    
                    self.ir.push(BFQuad::To(v));
                    self.ir.push(BFQuad::Constant(idx as u32 + 1));
                    self.ir.push(BFQuad::Equal(cond_var, v, cond, t1, t2));
                    
                    comment!(self, "if loop condition is true: taking this exit");
                    self.ir.push(BFQuad::If(cond));
                    self.block_to_ir(*out_blk);
                    self.ir.push(BFQuad::EndIf(cond));
                    comment!(self, format!("end if for loop condition {}", idx + 1));

                    self.alloc.free(cond);
                    self.alloc.free(t1);
                    self.alloc.free(t2);
                    self.alloc.free(v);
                }
                self.alloc.free(loop_var);
                self.alloc.free(cond_var);

                comment!(self, format!("Loop complete with loop_var={}", loop_var));
                
                true
            },
            
            false => self.emit_non_loop(block)
        };
        
        
        if !should_be_end {
            // progress to the next block, if there is one.
            if self.blocks[block].out_blocks.len() == 1 {
                let out_idx = self.blocks[block].out_blocks[0];
                self.block_to_ir(out_idx);
            } else if self.blocks[block].out_blocks.len() > 1 {
                panic!("Block has multiple outputs but does not end with a
                            branching instruction");
            }
        }

    }

    fn emit_non_loop(&mut self, block: usize) -> bool {
        let mut should_be_end = false;
        for dbcmd in self.blocks[block].cmds.clone().iter() {

            if should_be_end {
                panic!("Command type should have ended a block but didn't!");
            }
            
            match dbcmd.cmd {
                DBStmt::DEF { ref funcname, ref varname, ref expr } => {
                    self.def_map.insert(funcname.clone(),
                                        (varname.clone(), expr.clone()));
                },
                
                DBStmt::DIM { ref arrays } => {
                    
                    for array in arrays {
                        if self.already_used_array {
                            panic!("Defintion of array {} must come before the
                                    first use of any array",
                                   array.varname);
                        }
                        
                        // reserve space for the array
                        let mut dimensions = Vec::new();
                        let mut total_size: u32 = 1;
                        
                        for expr in array.dims.iter() {
                            match *expr {
                                Expr::O (_,_,_) |
                                    Expr::A(_) |
                                    Expr::V(_) |
                                    Expr::E(_)
                                    => {
                                        panic!("DIM statement for array {} must give a fixed size",
                                               array.varname);
                                    }
                                Expr::N(num) => {
                                    if num < 1 {
                                        panic!("Array {} must have all > 0 dimensions",
                                               array.varname);
                                    }
                                    dimensions.push(num as usize);
                                    total_size *= num as u32;
                                }
                            }
                        }

                        let arr_pos = self.alloc.reserve_array(total_size);
                        self.array_t.insert(array.varname.clone(),
                                            (dimensions, arr_pos));
                    }
                },
                
                DBStmt::FOR { ref varname, ref from_expr, ref to_expr, .. } => {
                    if !self.symbol_t.contains_key(varname) {
                        self.symbol_t.insert(varname.clone(),
                                             self.alloc.reserve());
                    }
                    
                    let var_pos = *(self.symbol_t.get(varname).unwrap());
                    
                    // initialize the variable
                    {
                        let (expr_result_pos, expr_code) = self.ir_for_expression(from_expr);
                        self.ir.extend(expr_code);
                        self.ir.push(BFQuad::Zero(var_pos));
                        self.ir.push(BFQuad::Move(expr_result_pos, var_pos));
                        self.alloc.free(expr_result_pos);
                    }
                    
                    let cond_pos = self.alloc.reserve();
                    
                    // check the condition
                    {
                        let (expr_result_pos, expr_code) = self.ir_for_expression(to_expr);
                        self.ir.extend(expr_code);
                        
                        let var_copy = self.alloc.reserve();
                        let t2 = self.alloc.reserve();
                        let t3 = self.alloc.reserve();
                        
                        self.ir.push(BFQuad::Zero(var_copy));
                        self.ir.push(BFQuad::Zero(cond_pos));
                        self.ir.push(BFQuad::Zero(t2));
                        self.ir.push(BFQuad::Zero(t3));
                        
                        self.ir.push(BFQuad::AddTo(var_pos, var_copy, cond_pos));
                        self.ir.push(BFQuad::Zero(cond_pos));
                        self.ir.push(BFQuad::To(cond_pos));
                        self.ir.push(BFQuad::NotEqual(var_copy, expr_result_pos,
                                                      cond_pos, t2, t3));
                        self.alloc.free(t2);
                        self.alloc.free(t3);
                        self.alloc.free(expr_result_pos);
                        self.alloc.free(var_copy);
                        self.ir.push(BFQuad::To(cond_pos));       
                    }
                    
                    self.ir.push(BFQuad::RawBF("["));
                    let out_idx = self.blocks[block].out_blocks[1];
                    self.block_to_ir(out_idx);
                    
                    // increment the variable
                    self.ir.push(BFQuad::To(var_pos));
                    self.ir.push(BFQuad::RawBF("+"));
                    
                    // check the condition
                    {
                        let (expr_result_pos, expr_code) = self.ir_for_expression(to_expr);
                        self.ir.extend(expr_code);
                        
                        let var_copy = self.alloc.reserve();
                        let t2 = self.alloc.reserve();
                        let t3 = self.alloc.reserve();
                        
                        self.ir.push(BFQuad::Zero(var_copy));
                        self.ir.push(BFQuad::Zero(cond_pos));
                        self.ir.push(BFQuad::Zero(t2));
                        self.ir.push(BFQuad::Zero(t3));
                        
                        self.ir.push(BFQuad::AddTo(var_pos, var_copy, cond_pos));
                        self.ir.push(BFQuad::Zero(cond_pos));
                        self.ir.push(BFQuad::To(cond_pos));
                        self.ir.push(BFQuad::NotEqual(var_copy, expr_result_pos,
                                                      cond_pos, t2, t3));
                        self.alloc.free(t2);
                        self.alloc.free(t3);
                        self.alloc.free(expr_result_pos);
                        self.alloc.free(var_copy);
                        self.ir.push(BFQuad::To(cond_pos));       
                        self.alloc.free(cond_pos);
                    }
                    
                    self.ir.push(BFQuad::RawBF("]"));
                    let out_idx = self.blocks[block].out_blocks[0];
                    self.block_to_ir(out_idx);
                    should_be_end = true;
                },
                
                DBStmt::NEXT { .. } => {
                    should_be_end = true;
                }
                
                DBStmt::GOSUB { .. } => {
                    let out_idx0 = self.blocks[block].out_blocks[0];
                    let out_idx1 = match self.blocks[block].special_out {
                        SpecialOut::Return (pos) => pos,
                        _ => panic!("GOSUB did not have special out set!")
                    };
                    
                    self.block_to_ir(out_idx0);
                    self.block_to_ir(out_idx1);
                    should_be_end = true;
                },

                DBStmt::RETURN => {
                    if self.blocks[block].out_blocks.len() != 1 {
                        panic!("Return statment should have exactly one
                                out block!");
                    }
                    
                    let return_loc = self.blocks[block].out_blocks[0];
                    
                    // we only need to do something if this return is
                    // breaking out a loop.
                    if !self.loop_stack.is_empty() {
                        let &(block_idx, loop_var, cond_var) =
                            self.loop_stack.last().unwrap();

                        let index = {
                            let loop_exits = &self.blocks[block_idx].loop_exits;
                            loop_exits.iter()
                                .position(|&e| e == return_loc as usize)
                        };

                        match index {
                            Some (pos) => {
                                mark_loop_done!(self, loop_var, cond_var, pos);
                            },

                            None => { }
                        };
                    }
                    should_be_end = true;
                },
                
                DBStmt::GOTO { .. } => {
                    let out_idx = self.blocks[block].out_blocks[0];
                    should_be_end = true;
                    // it is possible that this GOTO is taking us out of a loop
                    // check to see if this GOTO jumps back to the current loop header
                    if !self.loop_stack.is_empty()
                        && self.loop_stack.last().unwrap().0 == out_idx
                    {
                        // this GOTO is jumping back to the loop header!
                        // we shouldn't do anything but stop.
                    } else {
                        // this is a forward GOTO
                        self.block_to_ir(out_idx);
                    }
                },
                
                DBStmt::IF { ref expr1, ref op, ref expr2, .. } => {
                    comment!(self, "Start of if statement");
                    
                    let (loc1, expr1_code) = self.ir_for_expression(expr1);
                    let (loc2, expr2_code) = self.ir_for_expression(expr2);
                    
                    self.ir.extend(expr1_code);
                    self.ir.extend(expr2_code);
                    
                    let cond = get_and_zero!(self);
                    let t1 = get_and_zero!(self);
                    let t2 = get_and_zero!(self);
                    
                    let action = match op.as_ref() {
                        "="  => BFQuad::Equal(loc1, loc2, cond, t1, t2),
                        ">"  => BFQuad::Greater(loc1, loc2, cond, t1, t2),
                        "<"  => BFQuad::Less(loc1, loc2, cond, t1, t2),
                        "!=" => BFQuad::NotEqual(loc1, loc2, cond, t1, t2),
                        ">=" => BFQuad::GreaterOrEqual(loc1, loc2, cond, t1, t2),
                        "<=" => BFQuad::LessOrEqual(loc1, loc2, cond, t1, t2),
                        _ => panic!("unsupported relop")
                    };
                    
                    self.ir.push(action);
                    self.alloc.free(t1);
                    self.alloc.free(t2);
                    self.alloc.free(loc1);
                    self.alloc.free(loc2);
                    
                    let else_tmp = get_and_zero!(self);
                    
                    let out_idx0 = self.blocks[block].out_blocks[0];
                    let out_idx1 = self.blocks[block].out_blocks[1];
                    self.ir.push(BFQuad::IfElse(cond, else_tmp));
                    
                    let does_loop_exit: bool;
                    if !self.loop_stack.is_empty() {
                        let &(block_idx, loop_var, cond_var) =
                            self.loop_stack.last().unwrap();

                        let index0;
                        let index1;
                        {
                            let loop_exits = &self.blocks[block_idx].loop_exits;
                        
                            index0 = loop_exits.iter()
                                .position(|&e| e == out_idx0);
                            index1 = loop_exits.iter()
                                .position(|&e| e == out_idx1);
                        }
                        
                        if index0.is_some() || index1.is_some() {
                            does_loop_exit = true;
                            // one of these if branches leaves the loop!
                            match index0 {
                                None => {},
                                Some (_) => {
                                    assert!(!index1.is_some());
                                    mark_loop_done!(self, loop_var,
                                                    cond_var, index0.unwrap());
                                    comment!(self, "else");
                                    self.ir.push(BFQuad::Else(cond, else_tmp));
                                    self.block_to_ir(out_idx1);

                                    comment!(self, "end if");
                                    self.ir.push(BFQuad::EndElse(else_tmp));
                                }
                            };
                            
                            match index1 {
                                None => {},
                                Some (_) => {
                                    assert!(!index0.is_some());
                                    self.block_to_ir(out_idx1);
                                    
                                    comment!(self, "else");
                                    self.ir.push(BFQuad::Else(cond, else_tmp));
                                    mark_loop_done!(self, loop_var,
                                                    cond_var, index1.unwrap());

                                    comment!(self, "end if");
                                    self.ir.push(BFQuad::EndElse(else_tmp));
                                }
                            };
                        } else {
                            does_loop_exit = false;
                        }
                    } else {
                        does_loop_exit = false;
                    }
                    
                    if !does_loop_exit {
                        self.block_to_ir(out_idx0);

                        comment!(self, "else");
                        self.ir.push(BFQuad::Else(cond, else_tmp));
                        self.block_to_ir(out_idx1);

                        comment!(self, "end if");
                        self.ir.push(BFQuad::EndElse(else_tmp));
                    }
                    
                    self.alloc.free(cond);
                    self.alloc.free(else_tmp);
                    should_be_end = true;

                },
                
                DBStmt::LET { ref target, ref expr } => {
                    self.emit_let(target, expr);
                },

                DBStmt::READ { ref varnames } => {
                    for (idx, vname) in varnames.iter().enumerate() {
                        let expr = DBExpr::E(Expr::N(dbcmd.data[idx] as i32));
                        self.emit_let(vname, &expr);
                    }
                },
                
                DBStmt::PRINT { ref seq } => {
                    comment!(self, format!("Printing"));
                    for expr in seq {
                        let code = self.ir_for_print(expr);
                        self.ir.extend(code);
                    }
                    comment!(self, "End of print");
                },
                
                _ => {}
            };
        }
        
        return should_be_end;
    }

    fn emit_let(&mut self, target: &DBLetTarget, expr: &DBExpr) {
        match *target {
            DBLetTarget::VAR (ref varname) => {
                comment!(self, format!("LET for variable {}", varname));
                if !self.symbol_t.contains_key(varname) {
                    self.symbol_t.insert(varname.clone(),
                                         self.alloc.reserve());
                }
                
                let var_pos = *(self.symbol_t.get(varname).unwrap());
                let (loc, code) = self.ir_for_expression(expr);
                self.ir.extend(code);
                
                self.ir.push(BFQuad::Zero(var_pos));
                self.ir.push(BFQuad::Move(loc, var_pos));
                self.alloc.free(loc);
            },
            
            DBLetTarget::ARR (ref indexing_expressions) => {
                comment!(self, format!("LET for array {}",
                                       indexing_expressions.varname));

                self.already_used_array = true;
                
                let (arr_pos, arr_idx, idx_code)
                    = self.compute_array_index(indexing_expressions);
                
                self.ir.extend(idx_code);
                

                let (loc, expr_code) = self.ir_for_expression(expr);
                self.ir.extend(expr_code);

                self.ir.push(BFQuad::SetArray(arr_pos,
                                              arr_idx,
                                              loc));

                self.alloc.free(loc);
                self.alloc.free(arr_idx);
            }
        };
        
        
        comment!(self, format!("End of LET"));
        
    }

    fn compute_array_index(&mut self, indexing_expressions: &DBArrayDef)
                           -> (u32, u32, Vec<BFQuad>)
    {
        if !self.array_t.contains_key(&indexing_expressions.varname) {
            panic!("indexing to array {} before it is declared!",
                   indexing_expressions.varname);
        }
        
        let (adef, arr_pos) = self.array_t.get(&indexing_expressions.varname)
            .unwrap().clone();

        let mut to_r = Vec::new();
        let mut dim_indexes = Vec::new();

        for dim in indexing_expressions.dims.iter() {
            let (pos, code) = self.ir_for_expr(dim);
            dim_indexes.push(pos);
            to_r.extend(code);
        }

        // if I have a 5x6 array and I want to access element
        // 2, 3, then I need to compute 2 * 6 + 3.
        // if I have an 5x6x7 array and I want to access element
        // 2, 3, 4, then I need to compute 2*6 + 3*7 + 4
        let accum = self.alloc.reserve();
        to_r.push(BFQuad::Zero(accum));
        
        let last_index = dim_indexes.len() - 1;
        for (idx, dim_idx) in dim_indexes.iter().enumerate() {
            if idx == last_index {
                // this is the last index position, just add.
                to_r.push(BFQuad::Move(*dim_idx, accum));
                self.alloc.free(*dim_idx);
                continue;
            }

            // this is not the last index position.
            // first, multiply it by the dimension of the next position
            // then add it to accum
            let next_dim_size = adef[idx+1];
            let (const_pos, const_code) = self.ir_for_const(
                next_dim_size as i32
                    );
            to_r.extend(const_code);

            to_r.push(BFQuad::For(*dim_idx));
            to_r.push(BFQuad::To(accum));
            to_r.push(BFQuad::Constant(next_dim_size as u32));
            to_r.push(BFQuad::Next(*dim_idx));
            self.alloc.free(const_pos);
            self.alloc.free(*dim_idx);
        }

        return (arr_pos, accum, to_r);
    }
    
    fn ir_for_expression(&mut self, expr: &DBExpr) -> (u32, Vec<BFQuad>) {

        match *expr {
            DBExpr::S (_) => panic!("Found string in mathematical expression!"),
            DBExpr::E ( ref expr ) => {
                return self.ir_for_expr(expr);
            }
        }
    }

    fn ir_for_expr(&mut self, expr: &Expr) -> (u32, Vec<BFQuad>) {
        let mut to_r = Vec::new();
        
        match *expr {
            Expr::O(ref e1, ref op, ref e2) => {
                let (e1l, e1c) = self.ir_for_expr(&*e1);
                let (e2l, e2c) = self.ir_for_expr(&*e2);

                to_r.extend(e1c);
                to_r.extend(e2c);
                
                match *op {
                    OpCode::Add => {
                        let tmp = self.alloc.reserve();
                        to_r.push(BFQuad::Zero(tmp));
                        to_r.push(BFQuad::AddTo(e2l, e1l, tmp));
                        self.alloc.free(tmp);
                        self.alloc.free(e2l);
                        return (e1l, to_r);
                    },

                    OpCode::Sub => {
                        let tmp = self.alloc.reserve();
                        to_r.push(BFQuad::SubFrom(e1l, e2l));
                        self.alloc.free(e2l);
                        self.alloc.free(tmp);
                        return(e1l, to_r);
                    },

                    OpCode::Times => {
                        let tmp = self.alloc.reserve();
                        let loc = self.alloc.reserve();
                        to_r.push(BFQuad::Zero(tmp));
                        to_r.push(BFQuad::Zero(loc));
                        to_r.push(BFQuad::Times(e1l, e2l, loc, tmp));
                        self.alloc.free(e1l);
                        self.alloc.free(e2l);
                        self.alloc.free(tmp);
                        return (loc, to_r);
                    },

                    OpCode::Div => {
                        let tmp_start = self.alloc.reserve_range(7);
                        //quo, t1, div, rem, res, t3, t4

                        let t1 = self.alloc.reserve();

                        to_r.push(BFQuad::Zero(tmp_start + 0));
                        to_r.push(BFQuad::Zero(tmp_start + 1));
                        to_r.push(BFQuad::Zero(tmp_start + 2));
                        to_r.push(BFQuad::Zero(tmp_start + 3));
                        to_r.push(BFQuad::Zero(tmp_start + 4));
                        to_r.push(BFQuad::Zero(tmp_start + 5));
                        to_r.push(BFQuad::Zero(tmp_start + 6));

                        to_r.push(BFQuad::AddTo(e1l, tmp_start, t1));
                        to_r.push(BFQuad::Zero(t1));
                        to_r.push(BFQuad::AddTo(e2l, tmp_start + 2, t1));

                        to_r.push(BFQuad::Div(tmp_start + 0,
                                              tmp_start + 1,
                                              tmp_start + 2,
                                              tmp_start + 3,
                                              tmp_start + 4,
                                              tmp_start + 5,
                                              tmp_start + 6));

                        let loc = self.alloc.reserve();
                        to_r.push(BFQuad::Zero(t1));
                        to_r.push(BFQuad::AddTo(tmp_start + 4,
                                                loc, t1));

                        self.alloc.free(t1);
                        self.alloc.free(tmp_start + 0);
                        self.alloc.free(tmp_start + 1);
                        self.alloc.free(tmp_start + 2);
                        self.alloc.free(tmp_start + 3);
                        self.alloc.free(tmp_start + 4);
                        self.alloc.free(tmp_start + 5);
                        self.alloc.free(tmp_start + 6);
                        self.alloc.free(e1l);
                        self.alloc.free(e2l);
                        return (loc, to_r);          
                    }

                }
            },

            // N V A E
            Expr::N(ref t) => {
                return self.ir_for_const(*t);
            },

            Expr::V(ref vname) => {
                return self.ir_for_var(vname);
            },

            Expr::A(ref array_dim) => {
                let mut to_r = Vec::new();
                let indexing_expressions = array_dim;
                let (pos, idx, code) = self.compute_array_index(indexing_expressions);
                to_r.extend(code);
                
                let tmp = self.alloc.reserve();
                to_r.push(BFQuad::Zero(tmp));
                to_r.push(BFQuad::GetArray(pos, idx, tmp));

                self.alloc.free(idx);

                return (tmp, to_r);
            },

            Expr::E(ref e) => {
                return self.ir_for_expr(e);
            }

            
            
        }
    }

    fn ir_for_const(&mut self, val: i32) -> (u32, Vec<BFQuad>) {
        let mut to_r = Vec::new();
        if val < 0 {
            panic!("BF does not support negative values!");
        }
        
        return match self.const_opt {
            false => {
                let dest = self.alloc.reserve();
                to_r.push(BFQuad::Zero(dest));
                to_r.push(BFQuad::To(dest));
                to_r.push(BFQuad::Constant(val as u32));
                (dest, to_r)
            },
            
            true => {
                let (code, size) = optimizer::optimized_constant(val as u32);
                let dest = self.alloc.reserve_range(size as u32);
                for i in dest..dest+(size as u32) {
                    to_r.push(BFQuad::Zero(i));
                }
                to_r.push(BFQuad::To(dest));
                to_r.push(BFQuad::RawBFStr(code));
                for i in dest+1..dest+(size as u32) {
                    self.alloc.free(i);
                }
                
                (dest, to_r)
            }
        };
    }

    fn ir_for_var(&mut self, varname: &String) -> (u32, Vec<BFQuad>) {
        let mut to_r = Vec::new();
        let varloc = *(self.symbol_t.get(varname)
                       .expect(format!("Variable {} is not defined!", varname).as_str()));
        
        let pos = self.alloc.reserve();
        let tmp = self.alloc.reserve();
        to_r.push(BFQuad::Zero(pos));
        to_r.push(BFQuad::Zero(tmp));
        to_r.push(BFQuad::AddTo(varloc, pos, tmp));
        self.alloc.free(tmp);
        return (pos, to_r);
    }
    
    fn ir_for_print(&mut self, expr: &DBExpr) -> Vec<BFQuad> {
        let mut to_r = Vec::new();
        match *expr {
            DBExpr::S(ref s) => {
                let ascii = self.alloc.reserve();
                to_r.push(BFQuad::Zero(ascii));
                to_r.push(BFQuad::To(ascii));

                let mut curr_val = 0;
                
                for chr in s.chars() {
                    let ichr = chr as u32;
                    if ichr > curr_val {
                        to_r.push(BFQuad::Constant(ichr - curr_val))
                    } else if ichr < curr_val {
                        to_r.push(BFQuad::SubConstant(curr_val - ichr));
                    }
                    curr_val = ichr;

                    to_r.push(BFQuad::RawBF("."));
                }

                self.alloc.free(ascii);
            },

            DBExpr::E (ref expr) => {
                let (el, ec) = self.ir_for_expr(expr);
                to_r.extend(ec);

                let tmp = self.alloc.reserve_range(15);

                for i in tmp..tmp+15 {
                    to_r.push(BFQuad::Zero(i));
                }
                
                to_r.push(BFQuad::Move(el, tmp));
                self.alloc.free(el);

                to_r.push(BFQuad::To(tmp));

                to_r.push(BFQuad::RawBF("[>>+>+<<<-]>>>[<<<+>>>-]<<+>[<->[>++++++++++<[->-[>+>>]>[+[-<+>]>+>>]<<<<<]>[-]++++++++[<++++++>-]>[<<+>>-]>[<<+>>-]<<]>]<[->>++++++++[<++++++>-]]<[.[-]<]<"));
                
                for i in tmp..tmp+15 {
                    self.alloc.free(i);
                }

            }

        }

        return to_r;
    }

}

#[cfg(test)]
mod test {
    use super::*;

    use parser;
    use ir::blockgen;
    
    #[test]
    fn test_simple() {
        let test_program = "\
0  LET Y = 5
1  PRINT \"hello world\"
2  LET Y = 7 * 8
3  FOR X = 1 TO Y
4  PRINT \"!\"
5  NEXT X
70 END\n";

        let parse = parser::parse_bytes(test_program.as_bytes());
        let res = blockgen::to_blocks(parse);

        let mut ir_gen = BlockToIR::new(res, false);
        let ir = ir_gen.generate_ir();
        println!("{:?}", ir);
    }

}
    
