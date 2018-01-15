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

#![allow(unknown_lints)] // for clippy
#![allow(explicit_iter_loop, needless_return)]
#![feature(match_default_bindings)]

extern crate clap;
#[macro_use]
extern crate nom;
extern crate unescape;

mod parser;
mod codegen;
mod ir;
mod interp;
mod optimizer;
mod compile;

use std::fs::File;
use std::io::prelude::*;

use clap::{App, Arg};

fn main() {
    let matches = App::new("BASICAF")
        .version("0.1.2")
        .author("Ryan Marcus <ryan@ryanmarc.us>")
        .about("A BASIC to Brainf**k compiler, https://github.com/RyanMarcus/basicaf")
        .arg(Arg::with_name("semcom")
             .long("semantic-comments")
             .short("s")
             .help("Include comments in the output related to the input BASIC program"))
        .arg(Arg::with_name("ircom")
             .long("ir-comments")
             .short("i")
             .help("Include comments in the output related to the IR of the compiler"))
        .arg(Arg::with_name("graphviz")
             .conflicts_with("semcom")
             .conflicts_with("ircom")
             .conflicts_with("disable-opt")
             .long("graphviz")
             .short("g")
             .help("Output a graphviz representation of the flow control of the input program, instead of compiling it"))
        .arg(Arg::with_name("disable-opt")
             .long("disable-opt")
             .short("d")
             .help("Disables the optimizer"))
        .arg(Arg::with_name("input file")
             .help("The input BASIC file")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("execute")
             .conflicts_with("graphviz")
             .conflicts_with("semcom")
             .conflicts_with("ircom")
             .conflicts_with("disable-opt")
             .short("e")
             .long("execute")
             .help("Executes the input file as a Brainf**k program"))
        .get_matches();

    let sem_comments = matches.is_present("semcom");
    let ir_comments = matches.is_present("ircom");
    let no_opt = matches.is_present("disable-opt");
    let gv = matches.is_present("graphviz");
    let execute = matches.is_present("execute");

    let inp_file = matches.value_of("input file").unwrap();

    let mut f = File::open(inp_file).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file!");

    if execute {
        let mut interp = interp::BFEnv::new();
        let result = interp.execute(contents);
        println!("{}", result);
        return;
    }

    let s = if gv {
        compile::to_graphviz(contents)
    } else {
        compile::compile(contents, sem_comments, ir_comments, !no_opt)
    };

    println!("{}", s);
}
