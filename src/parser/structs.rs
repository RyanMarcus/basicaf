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
use parser::ast;

#[derive(Debug, Clone)]
pub enum DBExpr {
    S(String),
    E(ast::Expr),
}

#[derive(Debug, Clone)]
pub enum DBLetTarget {
    VAR(String),
    ARR(DBArrayDef),
}

#[derive(Debug, Clone)]
pub struct DBArrayDef {
    pub varname: String,
    pub dims: Vec<ast::Expr>,
}

#[derive(Debug, Clone)]
pub enum DBStmt {
    DEF {
        funcname: String,
        varname: String,
        expr: DBExpr,
    },
    DIM {
        arrays: Vec<DBArrayDef>,
    },
    END,
    FOR {
        varname: String,
        from_expr: DBExpr,
        to_expr: DBExpr,
        step_expr: Box<Option<DBExpr>>,
    }, // boxing the step for memory layout
    NEXT {
        varname: String,
    },
    GOSUB {
        lineno: u32,
    },
    RETURN,
    GOTO {
        lineno: u32,
    },
    IF {
        expr1: DBExpr,
        op: String,
        expr2: DBExpr,
        lineno: u32,
    },
    LET {
        target: DBLetTarget,
        expr: DBExpr,
    },
    PRINT {
        seq: Vec<DBExpr>,
    },
    DATA {
        seq: Vec<f32>,
    },
    READ {
        varnames: Vec<DBLetTarget>,
    },
    REM,
}

impl DBStmt {
    pub fn get_string_type(&self) -> &'static str {
        return match *self {
            DBStmt::DEF { .. } => "DEF",
            DBStmt::DIM { .. } => "DIM",
            DBStmt::END { .. } => "END",
            DBStmt::FOR { .. } => "FOR",
            DBStmt::NEXT { .. } => "NEXT",
            DBStmt::GOSUB { .. } => "GOSUB",
            DBStmt::RETURN { .. } => "RETURN",
            DBStmt::GOTO { .. } => "GOTO",
            DBStmt::IF { .. } => "IF",
            DBStmt::LET { .. } => "LET",
            DBStmt::PRINT { .. } => "PRINT",
            DBStmt::DATA { .. } => "DATA",
            DBStmt::READ { .. } => "READ",
            DBStmt::REM { .. } => "REM",
        };
    }
}

#[derive(Debug, Clone)]
pub struct DBCommand {
    pub ln: u32,
    pub cmd: DBStmt,
    pub data: Vec<u32>,
}

impl DBCommand {
    pub fn add_data(&mut self, d: u32) {
        self.data.push(d);
    }
}
