//! This is the command line tool that loads '.dot' files, renders the graph,
//!  and saves the output.

extern crate clap;
extern crate env_logger;
extern crate log;

use clap::{App, Arg};
use gv::parser::DotParser;
use gv::GraphBuilder;
use layout::backends::svg::SVGWriter;
use layout::core::utils::save_to_file;
use layout::gv;
use layout::topo::layout::VisualGraph;
use std::fs;

struct CLIOptions {
    disable_opt: bool,
    disable_layout: bool,
    output_path: String,
    debug_mode: bool,
}

impl CLIOptions {
    pub fn new() -> Self {
        Self {
            disable_opt: false,
            disable_layout: false,
            output_path: String::new(),
            debug_mode: false,
        }
    }
}

fn generate_svg(graph: &mut VisualGraph, options: CLIOptions) {
    let mut svg = SVGWriter::new();
    graph.do_it(
        options.debug_mode,
        options.disable_opt,
        options.disable_layout,
        &mut svg,
    );
    let content = svg.finalize();

    let res = save_to_file(&options.output_path, &content);
    if let Result::Err(err) = res {
        log::error!("Could not write the file {}", options.output_path);
        log::error!("Error {}", err);
        return;
    }
    log::info!("Wrote {}", options.output_path);
}

fn main() {
    let matches = App::new("Layout")
        .version("1.x")
        .arg(
            Arg::with_name("d")
                .short("d")
                .long("debug")
                .help("Enables debug options"),
        )
        .arg(
            Arg::with_name("no-layout")
                .long("no-layout")
                .help("Disable the node layout pass"),
        )
        .arg(
            Arg::with_name("no-optz")
                .long("no-optz")
                .help("Disable the graph optimizations"),
        )
        .arg(
            Arg::with_name("a")
                .short("a")
                .long("ast")
                .help("Dump the graph AST"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Path of the output file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    env_logger::builder().format_timestamp(None).init();

    let dump_ast = matches.occurrences_of("a") != 0;

    let mut cli = CLIOptions::new();
    cli.debug_mode = matches.occurrences_of("d") != 0;
    cli.disable_opt = matches.occurrences_of("no-optz") != 0;
    cli.disable_layout = matches.occurrences_of("no-layout") != 0;
    cli.output_path = matches
        .value_of("output")
        .unwrap_or("/tmp/out.svg")
        .to_string();

    let input_path = matches.value_of("INPUT").unwrap();
    let contents = fs::read_to_string(input_path).expect("Can't open the file");
    let mut parser = DotParser::new(&contents);

    let tree = parser.process();

    match tree {
        Result::Err(err) => {
            parser.print_error();
            log::error!("Error: {}", err);
        }

        Result::Ok(g) => {
            if dump_ast {
                gv::dump_ast(&g);
            }
            let mut gb = GraphBuilder::new();
            gb.visit_graph(&g);
            let mut vg = gb.get();
            generate_svg(&mut vg, cli);
        }
    }
}
