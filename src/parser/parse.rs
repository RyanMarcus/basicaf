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
use nom::{digit,not_line_ending,line_ending,alphanumeric, IResult};
use std::str;
use std::str::FromStr;

use parser::structs::{DBCommand, DBArrayDef, DBStmt, DBExpr, DBLetTarget};
use parser::ast::{Expr, OpCode};
use unescape::unescape;


// parses a >= 0 integer value, like a line number
named!(line_number<&[u8], u32>,
       map_res!(
           map_res!(digit, str::from_utf8),
           FromStr::from_str)
       );

// parses a float value
named!(db_float<&[u8], f32>,
       map_res!(
           map_res!(is_a_s!("0123456789.-"), str::from_utf8),
           FromStr::from_str)
       );


// parses a function or variable name
named!(db_name<&[u8], String>,
       map!(map_res!(alphanumeric, str::from_utf8), String::from)
       );

// parses valid ops for if statements
named!(db_rela_op<&[u8], &str>,
       map_res!(alt!(tag!("=")  |
                     tag!("!=") |
                     tag!("<=") |
                     tag!(">=") |
                     tag!("<")  |
                     tag!(">")),
                str::from_utf8)
       );

// parses valid ops for arthimatic
named!(db_op<&[u8], OpCode>,
       alt!(tag!("+") => {|_| OpCode::Add } |
            tag!("-") => {|_| OpCode::Sub } |
            tag!("*") => {|_| OpCode::Times } |
            tag!("/") => {|_| OpCode::Div }
            )
       );

named!(ast_term<&[u8], Expr>,
       alt!(
           complete!(delimited!(tag!("("), ast_expr, tag!(")")))
               => {|x| Expr::E(Box::new(x))} |
           
           complete!(db_array_dim) => { |x| Expr::A(x) } |
           complete!(line_number) => { |x| Expr::N(x as i32) } |
           complete!(db_name) => { |x| Expr::V(String::from(x)) }
           )
       );

named!(ast_expr<&[u8], Expr>,
       alt!(
           complete!(do_parse!(
               expr1: ws!(ast_term)
                   >> op: ws!(db_op)
                   >> expr2: ast_expr
                   >> ( Expr::O(Box::new(expr1), op, Box::new(expr2)) )
                   )) |
           do_parse!(
               many0!(tag!(" "))
                   >> term: ast_term
                   >> (term)
                   )
               )
       );

// parses an expression, which can either be a quoted string
// or an arithmetic sequence with functions, but no rel ops.
named!(db_expr<&[u8], DBExpr>,
       alt!(
           do_parse!(
               many0!(tag!(" "))
                   >> tag!("\"")
                   >> txt: is_not_s!("\"\n\r")
                   >> tag!("\"")
                   >>( DBExpr::S(unescape(str::from_utf8(txt).unwrap()).unwrap()) )
                   )
               |
           do_parse!(
               expr: ast_expr
                   >> ( DBExpr::E(expr)  )
                   )
               )
       );
             

// parses an array dimension specifier, like D(2, 5)
named!(db_array_dim<&[u8], DBArrayDef>,
       do_parse!(
           name: ws!(db_name)
               >> tag!("(")
               >> dims: separated_nonempty_list!(ws!(tag!(",")),
                                                 ast_expr)
                                                 
               >> tag!(")")
               >> (DBArrayDef { varname: String::from(name),
                                dims: dims } )
                                          
           )
       );


// parses a def statement, like DEF f(X) = 5 * X
named!(db_def_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("DEF")
               >> fname: ws!(db_name)
               >> ws!(tag!("("))
               >> vname: db_name
               >> ws!(tag!(")"))
               >> ws!(tag!("="))
               >> expr: db_expr
               >> line_ending
               >> ( DBStmt::DEF{funcname: String::from(fname),
                                varname: String::from(vname),
                                expr: expr} )
               )
       );


// parses a dim statement, like DIM F(5, 2), D(4)
named!(db_dim_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("DIM")
               >> dims: many1!(db_array_dim)
               >> line_ending
               >> ( DBStmt::DIM { arrays: dims } )
               )
       );

// parses an end statement, like END
named!(db_end_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("END")
               >> line_ending
               >> (DBStmt::END)
               )
       );

// parses a stop statement, like STOP
named!(db_stop_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("STOP")
               >> line_ending
               >> (DBStmt::END)
               )
       );

// parses a for statement, like FOR x = 5 TO y STEP 2
named!(db_for_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("FOR"))
               >> var: db_name
               >> ws!(tag!("="))
               >> from: db_expr
               >> ws!(tag!("TO"))
               >> to: db_expr
               >> step: opt!(do_parse!(
                   tag!("STEP")
                             >> to_r: ws!(db_expr)
                             >> ( to_r )
                             ))
               >> line_ending
               >> (DBStmt::FOR{ varname: String::from(var),
                                from_expr: from,
                                to_expr: to,
                                step_expr: step })
               )
       );

// parses a next statement, like NEXT
named!(db_next_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("NEXT"))
               >> var: db_name
               >> line_ending
               >> (DBStmt::NEXT{ varname: String::from(var) })
               )
       );


// parses a GOSUB statement, like GOSUB 15
named!(db_gosub_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("GOSUB"))
               >> lno: line_number
               >> line_ending
               >> (DBStmt::GOSUB{ lineno: lno })
               )
       );

// parses a return statement, like RETURN
named!(db_return_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("RETURN")
               >> line_ending
               >> (DBStmt::RETURN)
               )
       );

// parses a GOTO statement, like GOTO 15
named!(db_goto_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("GOTO"))
               >> lno: line_number
               >> line_ending
               >> (DBStmt::GOTO{ lineno: lno })
               )
       );

// parses an if statement, like IF x = 5 THEN 27
named!(db_if_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("IF"))
               >> expr1: db_expr
               >> op: ws!(db_rela_op)
               >> expr2: db_expr
               >> ws!(tag!("THEN"))
               >> lno: line_number
               >> line_ending
               >> (DBStmt::IF{ expr1: expr1,
                               op: String::from(op),
                               expr2: expr2,
                               lineno: lno })
               )
       );

// parses a let statement, like LET x = 5 * 5 or LET X(5) = 4
named!(db_let_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("LET"))
               >> varname: let_target
               >> ws!(tag!("="))
               >> expr: db_expr
               >> line_ending
               >> (DBStmt::LET{ target: varname,
                                expr: expr })
               )
       );

// parses a print statement, like PRINT x, "is the value of x"
named!(db_print_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("PRINT")
               >> exprs: separated_list_complete!(ws!(tag!(",")),
                                                  db_expr)
               >> dbg_dmp!(line_ending)
               >> (DBStmt::PRINT { seq: exprs })
               )
       );

// parses a data statement, like DATA 5, 6, 2.0
named!(db_data_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("DATA"))
               >> data: separated_list_complete!(ws!(tag!(",")),
                                                 db_float)
               >> line_ending
               >> (DBStmt::DATA { seq: data })
               )
       );

named!(let_target<&[u8], DBLetTarget>,
       alt!(
           complete!(db_array_dim) => { |x| DBLetTarget::ARR(x) } |
           complete!(db_name) =>  { |x| DBLetTarget::VAR(String::from(x)) }
           )
       );

// parses a read statement, like READ x, y, z
// or READ x(1, 2), y(3, 4), z
named!(db_read_stmt<&[u8], DBStmt>,
       do_parse!(
           ws!(tag!("READ"))
               >> vars: separated_nonempty_list!(ws!(tag!(",")),
                                                 let_target)
               >> line_ending
               >> (DBStmt::READ {
                   varnames: vars
                   })
               )
       );

// parses a rem statement (commment), like REM this is a comment
named!(db_rem_stmt<&[u8], DBStmt>,
       do_parse!(
           tag!("REM")
               >> not_line_ending
               >> line_ending
               >> ( DBStmt::REM )
               )
       );
                                        

// parses any statement
named!(db_stmt<&[u8], DBStmt>,
       complete!(
           ws!(alt!(db_def_stmt    |
                    db_dim_stmt    |
                    db_end_stmt    |
                    db_stop_stmt   |
                    db_for_stmt    |
                    db_next_stmt   |
                    db_gosub_stmt  |
                    db_return_stmt |
                    db_goto_stmt   |
                    db_if_stmt     |
                    db_let_stmt    |
                    db_print_stmt  |
                    db_data_stmt   |
                    db_read_stmt   |
                    db_rem_stmt))
               )
       );


// parses any command (a line number and a statement)
named!(db_command<&[u8], DBCommand>,
       dbg_dmp!(do_parse!(
           lnp: line_number
               >> cmdp: db_stmt
               >> (DBCommand{ ln : lnp,
                              cmd: cmdp,
                              data: Vec::new() } )

               
               ))
       );

// parses a whole program (list of commands)
named!(db_prgm<&[u8], Vec<DBCommand> >,
       ws!(do_parse!(
           res: many1!(db_command)
               >> eof!()
               >> (res)
               )
           )
       );


pub fn parse_bytes(to_parse: &[u8]) -> Vec<DBCommand> {
    
    let res = match db_prgm(to_parse) {
        IResult::Done(_, value) => value,
        IResult::Error(err) => {
            println!("Err {:?}",err);
            panic!();
        },
        IResult::Incomplete(needed) => {
            println!("Needed {:?}",needed);
            panic!();
        }
    };

    return res;
    
}



