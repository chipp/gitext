use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("gitext")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(auth())
        .subcommand(browse())
        .subcommand(create())
        .subcommand(pr())
        .subcommand(prs())
        .subcommand(switch())
        .subcommand(ticket())
}

fn auth() -> Command {
    Command::new("auth")
}

fn browse() -> Command {
    Command::new("browse")
        .subcommand(Command::new("pr").arg(id(true)))
        .subcommand(Command::new("repo"))
}

fn create() -> Command {
    Command::new("create").arg(
        Arg::new("project")
            .required(false)
            .value_name("PROJECT CODE"),
    )
}

fn pr() -> Command {
    Command::new("pr")
        .subcommand(Command::new("browse").alias("b").arg(id(true)))
        .subcommand(Command::new("checkout").alias("co").arg(id(true)))
        .subcommand(Command::new("info").alias("i").arg(id(false)))
        .subcommand(
            Command::new("new")
                .alias("n")
                .arg(Arg::new("target").required(true).value_name("BRANCH")),
        )
        .subcommand(Command::new("new-or-browse").hide(true))
}

fn prs() -> Command {
    // TODO: specify `my` and `assigned`
    Command::new("prs").arg(Arg::new("filter").required(false).value_name("USERNAME"))
}

fn switch() -> Command {
    Command::new("switch").arg(Arg::new("id").required(true).value_name("PR id"))
}

fn ticket() -> Command {
    Command::new("ticket")
}

fn id(required: bool) -> Arg {
    Arg::new("id")
        .required(required)
        .value_name("PR id")
        .value_parser(clap::value_parser!(u16))
}
