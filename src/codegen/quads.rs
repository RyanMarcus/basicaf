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

#[derive(Debug)]
pub enum BFQuad {
    To(u32),
    Left(u32),
    Right(u32),
    Zero(u32),
    Move(u32, u32),
    For(u32),
    Next(u32),
    Move2(u32, u32, u32),
    AddTo(u32, u32, u32),
    SubFrom(u32, u32),
    //MultiplyConst( u32, u32, u32), // unused. can put in if ever needed
    Constant(u32),
    SubConstant(u32),
    Times(u32, u32, u32, u32),
    Div(u32, u32, u32, u32, u32, u32, u32),
    If(u32),
    EndIf(u32),

    IfElse(u32, u32),
    Else(u32, u32),
    EndElse(u32),

    Or(u32, u32, u32),
    //And( u32, u32, u32), // unused. can put in if ever needed
    Not(u32, u32),

    SubtractMinimum(u32, u32, u32, u32, u32),
    NotEqual(u32, u32, u32, u32, u32),
    Equal(u32, u32, u32, u32, u32),
    Greater(u32, u32, u32, u32, u32),
    Less(u32, u32, u32, u32, u32),
    GreaterOrEqual(u32, u32, u32, u32, u32),
    LessOrEqual(u32, u32, u32, u32, u32),

    SetArray(u32, u32, u32),

    GetArray(u32, u32, u32),

    RawBF(&'static str),
    RawBFStr(String),
    Comment(String),
}

fn to(dest: u32) -> BFQuad {
    return BFQuad::To(dest);
}

// Clippy complains, but there's nothing we can do about this
// giant match statement.
#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
fn emit_step(quad: BFQuad, comment: bool) -> Vec<BFQuad> {
    let mut vec = Vec::new();

    match quad {
        BFQuad::Left(inc) => {
            for _ in 0..inc {
                vec.push(BFQuad::RawBF("<"));
            }

            if comment {
                vec.insert(0, BFQuad::RawBF("left: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Right(inc) => {
            for _ in 0..inc {
                vec.push(BFQuad::RawBF(">"));
            }

            if comment {
                vec.insert(0, BFQuad::RawBF("right: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Zero(loc) => {
            vec.push(to(loc));
            vec.push(BFQuad::RawBF("[-]"));

            if comment {
                vec.insert(0, BFQuad::RawBF("zero: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Move(from, dest) => {
            vec.push(to(from));
            vec.push(BFQuad::RawBF("["));
            vec.push(to(dest));
            vec.push(BFQuad::RawBF("+"));
            vec.push(to(from));
            vec.push(BFQuad::RawBF("-]"));

            if comment {
                vec.insert(0, BFQuad::RawBF("move: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::For(arg) => {
            vec.push(to(arg));
            vec.push(BFQuad::RawBF("["));

            if comment {
                vec.insert(0, BFQuad::RawBF("for: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }
        BFQuad::Next(arg) => {
            vec.push(to(arg));
            vec.push(BFQuad::RawBF("-]"));

            if comment {
                vec.insert(0, BFQuad::RawBF("next: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Move2(from, to1, to2) => {
            vec.push(BFQuad::For(from));
            vec.push(to(to1));
            vec.push(BFQuad::RawBF("+"));
            vec.push(to(to2));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::Next(from));
            // TODO need this? vec.push(BFQuad::RawBF("-]"));

            if comment {
                vec.insert(0, BFQuad::RawBF("move2: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::AddTo(from, dest, tmp) => {
            vec.push(BFQuad::Move2(from, dest, tmp));
            vec.push(BFQuad::Move(tmp, from));

            if comment {
                vec.insert(0, BFQuad::RawBF("addto: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::SubFrom(arg1, arg2) => {
            vec.push(BFQuad::For(arg2));
            vec.push(BFQuad::To(arg1));
            vec.push(BFQuad::RawBF("-"));
            vec.push(BFQuad::Next(arg2));

            if comment {
                vec.insert(0, BFQuad::RawBF("subfrom: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        /*BFQuad::MultiplyConst ( from, dest, by ) => {
            vec.push(BFQuad::For(from));
            vec.push(to(dest));
            vec.push(BFQuad::Constant(by));
            vec.push(BFQuad::Next(from));

            if comment {
                vec.insert(0, BFQuad::RawBF("multconst: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        },*/
        BFQuad::Constant(val) => {
            for _ in 0..val {
                vec.push(BFQuad::RawBF("+"));
            }

            if comment {
                vec.insert(0, BFQuad::RawBF("const: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::SubConstant(val) => {
            for _ in 0..val {
                vec.push(BFQuad::RawBF("-"));
            }

            if comment {
                vec.insert(0, BFQuad::RawBF("subconst: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Times(v1, v2, dest, tmp) => {
            vec.push(BFQuad::For(v1));
            vec.push(BFQuad::AddTo(v2, dest, tmp));
            vec.push(BFQuad::Next(v1));
            vec.push(BFQuad::Zero(v2)); // TODO need this?

            if comment {
                vec.insert(0, BFQuad::RawBF("times: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::If(v1) => {
            vec.push(to(v1));
            vec.push(BFQuad::RawBF("["));

            if comment {
                vec.insert(0, BFQuad::RawBF("if: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::EndIf(v1) => {
            vec.push(BFQuad::Zero(v1));
            vec.push(BFQuad::RawBF("]"));

            if comment {
                vec.insert(0, BFQuad::RawBF("endif: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::IfElse(v1, t) => {
            vec.push(to(t));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::If(v1));
            vec.push(to(t));
            vec.push(BFQuad::RawBF("-"));

            if comment {
                vec.insert(0, BFQuad::RawBF("ifelse: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Else(v1, t) => {
            vec.push(BFQuad::EndIf(v1));
            vec.push(BFQuad::If(t));

            if comment {
                vec.insert(0, BFQuad::RawBF("else: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::EndElse(t) => {
            vec.push(BFQuad::EndIf(t));

            if comment {
                vec.insert(0, BFQuad::RawBF("endelse: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Or(s1, s2, d) => {
            vec.push(BFQuad::Move(s1, d));
            vec.push(BFQuad::Move(s2, d));

            if comment {
                vec.insert(0, BFQuad::RawBF("or: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        /*BFQuad::And(s1, s2, d) => {
            vec.push(BFQuad::If(s1));
            vec.push(BFQuad::Move(s2, d));
            vec.push(BFQuad::EndIf(s1));
            vec.push(BFQuad::Zero(s2));

            if comment {
                vec.insert(0, BFQuad::RawBF("and: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        },*/
        BFQuad::Not(s, d) => {
            vec.push(to(d));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::If(s));
            vec.push(to(d));
            vec.push(BFQuad::RawBF("-"));
            vec.push(BFQuad::EndIf(s));

            if comment {
                vec.insert(0, BFQuad::RawBF("not: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Div(quo, t1, div, rem, res, t3, t4) => {
            assert!(t1 - quo == 1);
            assert!(div - t1 == 1);
            assert!(rem - div == 1);
            assert!(res - rem == 1);
            assert!(t3 - res == 1);
            assert!(t4 - t3 == 1); // TODO how many temps are needed?

            vec.push(BFQuad::To(quo));
            vec.push(BFQuad::RawBF("[->+>-[>+>>]>[+[-<+>]>+>>]<<<<<<]"));

            /*
            vec.push(BFQuad::Move(div, t4));
            vec.push(BFQuad::To(quo));
            vec.push(BFQuad::RawBF("["));
            vec.push(BFQuad::AddTo(t4, div, t1));
            vec.push(BFQuad::Zero(t1));
            vec.push(BFQuad::SubtractMinimum(quo, div,
                                             t1, t2, t3));
            
            // if quo and div are both zero, it was divisible
            //    and we are done.
            // if only quo is zero, then div is the remainder
            // if only div is zero, then we've taken another
            //    div out of quo

            vec.push(BFQuad::AddTo(quo, t1, t2));
            vec.push(BFQuad::Zero(t2));
            vec.push(BFQuad::AddTo(div, t2, t3));
            vec.push(BFQuad::Zero(t3));
            // t1 = quo
            // t2 = div

            // to see if we are done and it was divisible,
            // we want !quo && !div
            // we want to know if we aren't done, so we
            // want: !(!quo && !div)
            // which is !(!(quo || div)), quo || div
            vec.push(BFQuad::Or(t1, t2, t5));
            vec.push(BFQuad::IfElse(t5, t6));

            // here, either:
            // 1) div is zero, but quo is not, which means
            //    we have further to go.
            // 2) quo is zero, but div is not, which means
            //    that div is the remainder
            vec.push(BFQuad::AddTo(quo, t1, t2));
            vec.push(BFQuad::Zero(t2));
            vec.push(BFQuad::AddTo(div, t2, t3));
            vec.push(BFQuad::Zero(t3));
            // t1 = quo
            // t2 = div

            vec.push(BFQuad::IfElse(t1, t3));
            // if t1 is true, then t2 is false, which means
            // that the div is zero. we still have farther to go
            // increment res, because we took another step
            // without going below div
            vec.push(BFQuad::To(res));
            vec.push(BFQuad::Constant(1));

            vec.push(BFQuad::Else(t1, t3));

            // if t1 is false, then t2 is true, which means
            // that the quo is zero, but the div is not.
            // the value in div is the remainder.
            vec.push(BFQuad::Move(div, rem));
            
            vec.push(BFQuad::EndElse(t3));
            vec.push(BFQuad::Else(t5, t6));
            // div and rem both ended on zero -- add one, and we are done!
            vec.push(BFQuad::To(res));
            vec.push(BFQuad::Constant(1));
            vec.push(BFQuad::EndElse(t6));
            
            vec.push(BFQuad::To(quo));
            vec.push(BFQuad::RawBF("]"));
             */

            if comment {
                vec.insert(0, BFQuad::RawBF("div: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::SubtractMinimum(x1, x2, t1, t2, t3) => {
            /*vec.push(BFQuad::AddTo(x1, t1, t3));
            vec.push(BFQuad::AddTo(x2, t2, t3));
            vec.push(BFQuad::And(t1, t2, t3));
            vec.push(to(t3));
            vec.push(BFQuad::RawBF("["));
            vec.push(BFQuad::Zero(t3));
            vec.push(to(x1));
            vec.push(BFQuad::RawBF("-"));
            vec.push(to(x2));
            vec.push(BFQuad::RawBF("-"));
            vec.push(BFQuad::AddTo(x1, t1, t3));
            vec.push(BFQuad::AddTo(x2, t2, t3));
            vec.push(BFQuad::And(t1, t2, t3));
            vec.push(to(t3));
            vec.push(BFQuad::RawBF("]"));*/
            vec.push(BFQuad::For(x1));
            vec.push(BFQuad::AddTo(x2, t1, t2));
            vec.push(BFQuad::IfElse(t1, t2));
            vec.push(to(x2));
            vec.push(BFQuad::RawBF("-"));
            vec.push(BFQuad::Else(t1, t2));
            vec.push(to(t3));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::EndElse(t2));
            vec.push(BFQuad::Next(x1));
            vec.push(BFQuad::Move(t3, x1));

            if comment {
                vec.insert(0, BFQuad::RawBF("submin: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::NotEqual(x1, x2, d, t1, t2) => {
            vec.push(BFQuad::SubtractMinimum(x1, x2, d, t1, t2));
            vec.push(BFQuad::Or(x1, x2, d));

            if comment {
                vec.insert(0, BFQuad::RawBF("neq: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Equal(x1, x2, d, t1, t2) => {
            vec.push(BFQuad::NotEqual(x1, x2, t1, d, t2));
            vec.push(BFQuad::Not(t1, d));

            if comment {
                vec.insert(0, BFQuad::RawBF("eq: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Greater(x1, x2, d, t1, t2) => {
            vec.push(BFQuad::SubtractMinimum(x1, x2, d, t1, t2));
            vec.push(BFQuad::Zero(x2));
            vec.push(BFQuad::Move(x1, d));

            if comment {
                vec.insert(0, BFQuad::RawBF("gt: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Less(x1, x2, d, t1, t2) => {
            vec.push(BFQuad::SubtractMinimum(x1, x2, d, t1, t2));
            vec.push(BFQuad::Zero(x1));
            vec.push(BFQuad::Move(x2, d));

            if comment {
                vec.insert(0, BFQuad::RawBF("lt: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::GreaterOrEqual(x1, x2, d, t1, t2) => {
            vec.push(to(x1));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::Greater(x1, x2, d, t1, t2));

            if comment {
                vec.insert(0, BFQuad::RawBF("geq: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::LessOrEqual(x1, x2, d, t1, t2) => {
            vec.push(to(x2));
            vec.push(BFQuad::RawBF("+"));
            vec.push(BFQuad::Less(x1, x2, d, t1, t2));

            if comment {
                vec.insert(0, BFQuad::RawBF("leq: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::SetArray(b, i, v) => {
            vec.push(BFQuad::Move2(i, b + 1, b + 2));
            vec.push(BFQuad::Move(v, b + 3));
            vec.push(BFQuad::Zero(b));
            vec.push(BFQuad::To(b));
            vec.push(BFQuad::RawBF(">[>>>[-<<<<+>>>>]<[->+<]<[->+<]<[->+<]>-]>>>[-]<[->+<]<[[-<+>]<<<[->>>>+<<<<]>>-]<<"));

            if comment {
                vec.insert(0, BFQuad::RawBF("setarr: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::GetArray(b, i, v) => {
            vec.push(BFQuad::Move2(i, b + 1, b + 2));
            vec.push(BFQuad::Zero(b + 3));
            vec.push(BFQuad::Zero(b));
            vec.push(BFQuad::To(b));
            vec.push(BFQuad::RawBF(">[>>>[-<<<<+>>>>]<<[->+<]<[->+<]>-]>>>[-<+<<+>>>]<<<[->>>+<<<]>[[-<+>]>[-<+>]<<<<[->>>>+<<<<]>>-]<<"));

            // now the result is at b+3
            vec.push(BFQuad::Move(b + 3, v));
            if comment {
                vec.insert(0, BFQuad::RawBF("getarr: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::To(_) => {
            vec.push(quad);

            if comment {
                vec.insert(0, BFQuad::RawBF("to: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }
        BFQuad::RawBF(_) => {
            vec.push(quad);

            if comment {
                vec.insert(0, BFQuad::RawBF("raw: "));
                vec.push(BFQuad::RawBF("\n"));
            }
        }

        BFQuad::Comment(_) | BFQuad::RawBFStr(_) => {
            vec.push(quad);
        }
    }

    return vec;
}

fn resolve_tos(quads: Vec<BFQuad>) -> Vec<BFQuad> {
    let mut vec = Vec::new();
    let mut curr = 0;

    for quad in quads {
        match quad {
            BFQuad::To(dest) => {
                if curr > dest {
                    vec.push(BFQuad::Left(curr - dest));
                } else if dest > curr {
                    vec.push(BFQuad::Right(dest - curr));
                }
                curr = dest;
                //vec.push(BFQuad::Comment(String::from(format!("(now at {})", curr))));
            }

            BFQuad::RawBF(_) | BFQuad::RawBFStr(_) | BFQuad::Comment(_) => {
                vec.push(quad);
            }

            _ => {
                panic!(
                    "resolve_tos called with some quads \
                     that must first be emitted!"
                );
            }
        }
    }

    return vec;
}

fn emit(quad: BFQuad, quad_comments: bool, sem_comments: bool) -> Vec<BFQuad> {
    let mut vec = emit_step(quad, quad_comments);

    loop {
        let mut new_vec = Vec::new();
        let mut did_mod = false;
        for x in vec {
            let to_add = match x {
                BFQuad::RawBF(_) | BFQuad::To(_) | BFQuad::RawBFStr(_) => vec![x],

                BFQuad::Comment(_) => {
                    if sem_comments {
                        vec![x]
                    } else {
                        vec![]
                    }
                }
                _ => {
                    did_mod = true;
                    emit_step(x, false)
                }
            };

            new_vec.extend(to_add);
        }

        vec = new_vec;

        if !did_mod {
            break;
        }
    }

    return vec;
}

pub fn resolve(quads: Vec<BFQuad>, quad_comments: bool, sem_comments: bool) -> Vec<BFQuad> {
    let mut vec = Vec::new();

    for quad in quads {
        vec.extend(emit(quad, quad_comments, sem_comments));
    }

    let vec2 = resolve_tos(vec);
    let mut vec3 = Vec::new();

    for quad in vec2 {
        let to_add = emit(quad, false, sem_comments);
        vec3.extend(to_add);
    }

    return vec3;
}

pub fn create_string(quads: Vec<BFQuad>) -> String {
    let mut s = String::new();

    for quad in quads {
        match quad {
            BFQuad::RawBF(x) => {
                s.push_str(x);
            }

            BFQuad::RawBFStr(x) => {
                s.push_str(x.as_str());
            }

            BFQuad::Comment(comment) => {
                let tmp = comment.as_str();
                if tmp.contains('+') || tmp.contains('-') || tmp.contains('[') || tmp.contains(']')
                    || tmp.contains('<') || tmp.contains('>')
                {
                    panic!("Comment contained a BF instruction: {}", comment);
                }
                s.push_str(&comment);
            }

            _ => {
                panic!("non-RawBF quad in create_string");
            }
        }
    }

    return s;
}
