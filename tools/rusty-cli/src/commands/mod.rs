mod doctor;
mod make_command;
mod make_controller;
mod make_page;
mod make_view;
mod make_view_model;
mod migration;
mod test;

mod audit;

// RUSTY_COMMAND_MODULES

use std::error::Error;

use crate::cli::CliCommand;

pub(crate) type CommandResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) fn run(command: CliCommand) -> CommandResult {
    match command {
        CliCommand::Migration(args) => migration::run(args),
        CliCommand::MakeTest(args) => test::run(args),
        CliCommand::MakeCommand(args) => make_command::run(args),
        CliCommand::MakeController(args) => make_controller::run(args),
        CliCommand::MakePage(args) => make_page::run(args),
        CliCommand::MakeView(args) => make_view::run(args),
        CliCommand::MakeViewModel(args) => make_view_model::run(args),
        CliCommand::Doctor => doctor::run(),
        CliCommand::Audit => audit::run(),
        // RUSTY_COMMAND_DISPATCH
    }
}
