use clap::Parser;

use celebtir::{Cli, Exit, run};

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let usage_error = e.use_stderr();
            let _ = e.print();
            // clap exits 2 on usage errors, but 2 means "partial failure" here (CLI-2).
            std::process::exit(if usage_error {
                Exit::Validation
            } else {
                Exit::Success
            } as i32);
        }
    };
    std::process::exit(run(cli) as i32);
}
