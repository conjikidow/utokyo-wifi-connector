use std::io;

use clap::CommandFactory;

use crate::cli::{Cli, CompletionsArgs};

pub fn run(args: &CompletionsArgs) {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    clap_complete::generate(args.shell, &mut command, name, &mut io::stdout());
}
