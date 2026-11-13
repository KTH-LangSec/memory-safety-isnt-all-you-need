// SPDX-License-Identifier: Apache-2.0

use std::{
    env, fs,
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, Result, bail};

use ism::compare;
use ism::condition;
use ism::interpret::{self, Configuration};
use ism::lex;
use ism::parse::{self, Program};
use ism::state::{self, State};
use ism::trace::Trace;
use ism::visualize::{self, Diagram};

static DEBUG: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
    let program = env::args().next().unwrap();
    let usage = format!("usage: {program} <compare | run | debug> <file>");

    let mode = env::args().nth(1).context(usage.to_owned())?;

    let filename = env::args().nth(2).context(usage)?;
    let program = read_file(&filename)?
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("//") {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + " ";

    match mode.as_str() {
        "compare" => compare(&program),
        "run" => run(&program),
        "debug" => {
            DEBUG.store(true, Ordering::Relaxed);
            run(&program)
        }
        s => bail!("invalid mode '{s}'"),
    }
}

fn compare(program: &str) -> Result<()> {
    let program = parse(program)?;

    println!("RUN #1");
    println!("======");
    let (state1, trace1, diagram1) = execute(program.clone())?;
    write_file("compare-run-1.mermaid", &diagram1)?;

    println!("RUN #2");
    println!("======");
    let (state2, trace2, diagram2) = execute(program)?;
    write_file("compare-run-2.mermaid", &diagram2)?;

    println!("DIFFERENCE");
    println!("==========");
    compare::compare(&state1, &trace1, &state2, &trace2);

    println!();
    println!("CONDITION");
    println!("=========");
    println!(
        "Weakness:      {}",
        match condition::weakness(&state1, &trace1, &state2, &trace2) {
            condition::Weakness::None => "No",
            condition::Weakness::Stateless => "Yes (stateless)",
            condition::Weakness::Stateful => "Yes (stateful)",
        }
    );

    Ok(())
}

fn run(program: &str) -> Result<()> {
    let program = parse(program)?;
    let (_, _, diagram) = execute(program)?;
    write_file("run.mermaid", &diagram)?;
    Ok(())
}

fn execute(program: Program) -> Result<(State, Trace, Diagram)> {
    let mut state = State::new();
    for input in parse::detect_inputs(&program) {
        print!("Value for '{input}'?: ");
        io::stdout().flush()?;

        let mut raw_value = String::new();
        io::stdin().read_line(&mut raw_value)?;

        let value: isize = raw_value.trim().parse()?;
        state.insert_input(&input, value);
    }

    let mut trace = Trace::new();
    let mut configuration = Configuration::new(program, state);
    while !configuration.is_done() {
        if debug() {
            println!("{configuration}");
            println!();
        }

        configuration = interpret::step(configuration, &mut trace)?;
    }

    let state = configuration.get_state();
    let diagram = visualize::memory_diagram(&state, &trace);

    println!();
    println!("State:");
    println!("{state}");
    println!();
    println!("Trace:");
    println!("{trace}");
    println!();

    println!("Result:");
    match state.get_output() {
        Ok(state::Value::Number(n)) => println!("{n}"),
        Ok(state::Value::Nil) | Err(_) => println!("nil"),
        _ => println!("INVALID"),
    }
    println!();

    Ok((state, trace, diagram))
}

fn parse(program: &str) -> Result<Program> {
    let tokens = lex::lex(program)?;
    if debug() {
        println!("TOKENS:");
        println!("{tokens:?}");
        println!();
    }

    let program = parse::parse(tokens)?;
    if debug() {
        println!("PROGRAM:");
        println!("{program:?}");
        println!();
    }

    Ok(program)
}

fn read_file(filename: &str) -> Result<String> {
    let x = fs::read_to_string(filename)?;
    Ok(x)
}

fn write_file(filename: &str, content: &str) -> Result<()> {
    let mut file = fs::File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}
