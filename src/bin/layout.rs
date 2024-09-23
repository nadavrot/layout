//! This is the command line tool that loads '.dot' files, renders the graph,
//!  and saves the output.

extern crate clap;
extern crate env_logger;
extern crate log;

use clap::{Arg, ArgAction, Command};
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
    let matches = Command::new("Layout")
        .version("1.x")
        .arg(
            Arg::new("d")
                .short('d')
                .long("debug")
                .help("Enables debug options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-layout")
                .long("no-layout")
                .help("Disable the node layout pass")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-optz")
                .long("no-optz")
                .help("Disable the graph optimizations")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("a")
                .short('a')
                .long("ast")
                .help("Dump the graph AST")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Path of the output file")
                .num_args(1),
        )
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    env_logger::builder().format_timestamp(None).init();

    let dump_ast = matches.get_flag("a");

    let mut cli = CLIOptions::new();
    cli.debug_mode = matches.get_flag("d");
    cli.disable_opt = matches.get_flag("no-optz");
    cli.disable_layout = matches.get_flag("no-layout");
    cli.output_path = matches
        .get_one::<String>("output")
        .cloned()
        .unwrap_or_else(|| String::from("/tmp/out.svg"));

    let input_path = matches.get_one::<String>("INPUT").unwrap();
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
