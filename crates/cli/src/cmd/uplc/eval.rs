use miette::IntoDiagnostic;
use std::path::PathBuf;
use uplc::{
    ast::{FakeNamedDeBruijn, Name, NamedDeBruijn, Program, Term},
    machine::cost_model::ExBudget,
    parser,
};

#[derive(clap::Args)]
/// Evaluate an Untyped Plutus Core program
pub struct Args {
    script: PathBuf,

    #[clap(short, long)]
    flat: bool,

    /// Arguments to pass to the uplc program
    args: Vec<String>,
}

pub fn exec(Args { script, flat, args }: Args) -> miette::Result<()> {
    let mut program = if flat {
        let bytes = std::fs::read(&script).into_diagnostic()?;

        let prog = Program::<FakeNamedDeBruijn>::from_flat(&bytes).into_diagnostic()?;

        prog.into()
    } else {
        let code = std::fs::read_to_string(&script).into_diagnostic()?;

        let prog = parser::program(&code).into_diagnostic()?;

        Program::<NamedDeBruijn>::try_from(prog).into_diagnostic()?
    };

    for arg in args {
        let term: Term<NamedDeBruijn> = parser::term(&arg)
            .into_diagnostic()?
            .try_into()
            .into_diagnostic()?;

        program = program.apply_term(&term);
    }

    let (term, cost, logs) = program.eval();

    match term {
        Ok(term) => {
            let term: Term<Name> = term.try_into().into_diagnostic()?;

            println!("\nResult\n------\n\n{}\n", term.to_pretty());
        }
        Err(err) => {
            eprintln!("\nError\n-----\n\n{}\n", err);
        }
    }

    let budget = ExBudget::default();

    println!(
        "\nCosts\n-----\ncpu: {}\nmemory: {}",
        budget.cpu - cost.cpu,
        budget.mem - cost.mem
    );
    println!(
        "\nBudget\n------\ncpu: {}\nmemory: {}\n",
        cost.cpu, cost.mem
    );

    if !logs.is_empty() {
        println!("\nLogs\n----\n{}", logs.join("\n"))
    }

    Ok(())
}
