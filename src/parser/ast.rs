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
 
use parser::structs::DBArrayDef;

#[derive(Debug, Clone)]
pub enum OpCode {
    Add, Sub, Times, Div
}

#[derive(Debug, Clone)]
pub enum Expr {
    O(Box<Expr>, OpCode, Box<Expr>),
    N(i32),
    V(String),
    A(DBArrayDef),
    E(Box<Expr>)
}

#[derive(Debug, Clone)]
pub enum Term {
   
}
