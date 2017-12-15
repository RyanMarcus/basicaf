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
 
use parser;
use ir;
use codegen;


pub fn compile(prgm: String, semcom: bool, ircom: bool, opts: bool) 
               -> String
{
    let parsed = parser::parse_bytes(prgm.as_bytes());

    let mut blocks = ir::to_blocks(parsed);

    ir::elim_gotos(&mut blocks);
        
    let quads = ir::to_ir(blocks, opts);
    
    let v = codegen::resolve(quads, ircom, semcom);
    let s = codegen::create_string(v);
    return s;
     
}



pub fn to_graphviz(prgm: String) -> String {
    let parsed = parser::parse_bytes(prgm.as_bytes());

    let mut blocks = ir::to_blocks(parsed);

    ir::elim_gotos(&mut blocks);
    
    return ir::to_graphviz(blocks);
}
