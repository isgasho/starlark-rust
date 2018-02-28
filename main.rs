// Copyright 2018 The Starlark in Rust Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate getopts;
extern crate starlark;
extern crate codemap;
extern crate codemap_diagnostic;

use getopts::Options;
use std::env;
use std::sync::{Arc, Mutex};
use starlark::syntax::errors::SyntaxError;
use starlark::syntax::lexer::Lexer;
use starlark::syntax::parser::parse_file;
use std::fs::File;
use std::io::Read;
use codemap_diagnostic::{ColorConfig, Emitter};

fn lex(filename: &str) {
    let mut map = codemap::CodeMap::new();
    let mut content = String::new();
    let mut file = File::open(filename).unwrap();
    file.read_to_string(&mut content).unwrap();
    let file_map = map.add_file(filename.to_string(), content.clone());
    let mut emitter = Emitter::stderr(ColorConfig::Always, Some(&map));
    for r in Lexer::new(&content) {
        match r {
            Ok((_i, t, _j)) => println!("{:?}", t),
            Err(x) => emitter.emit(&[x.to_diagnostic(file_map.span)]),
        }
    }
}

fn print_ast(filename: &str, build_file: bool) {
    let map = Arc::new(Mutex::new(codemap::CodeMap::new()));
    match parse_file(&map, filename, build_file) {
        Ok(s) => println!("{}", s.node),
        Err(x) => Emitter::stderr(ColorConfig::Always, Some(&map.lock().unwrap())).emit(&[x]),
    }
}

macro_rules! print_usage {
    ($program: expr, $opts: expr) => (
        {
            let brief = format!("[Rust Starlark evaluator]

Usage: {} command [options] [arg1..argn]

Available commands:
  lex: parse files given in arguments and return the list of lexical tokens
  ast: generate the ASTs from argument given in parameter
", $program);
            eprint!("{}", $opts.usage(&brief));
        }
    );
    ($program: expr, $opts: expr, $($fmt:expr),+) => (
        {
            print_usage!($program, $opts);
            eprintln!($($fmt),+);
        }
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag(
        "b",
        "build_file",
        "Parse the build file format instead of full Skylark",
    );
    let matches = opts.parse(&args[1..]).unwrap();
    let build_file = matches.opt_present("b");
    let command = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage!(&program, opts);
        return;
    };
    match &command[..] {
        "lex" => {
            for i in matches.free.into_iter().skip(1) {
                lex(&i);
            }
        }
        "ast" => {
            for i in matches.free.into_iter().skip(1) {
                print_ast(&i, build_file);
            }
        }
        cmd => print_usage!(program, opts, "Invalid command: {}", cmd),
    }
}