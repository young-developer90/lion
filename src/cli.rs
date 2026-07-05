pub enum Command {
    Run {
        file: String,
        disassemble: bool,
    },
    Repl,
    Version,
    Fmt {
        file: String,
    },
    Test {
        path: Option<String>,
    },
    Help,
}

pub fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Command::Help;
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: lion run <file> [--disassemble]");
                std::process::exit(1);
            }
            let file = args[2].clone();
            let disassemble = args.iter().any(|a| a == "--disassemble");
            Command::Run { file, disassemble }
        }
        "repl" => Command::Repl,
        "version" | "--version" | "-v" => Command::Version,
        "fmt" => {
            if args.len() < 3 {
                eprintln!("Usage: lion fmt <file>");
                std::process::exit(1);
            }
            Command::Fmt {
                file: args[2].clone(),
            }
        }
        "test" => {
            let path = if args.len() > 2 {
                Some(args[2].clone())
            } else {
                None
            };
            Command::Test { path }
        }
        _ => Command::Help,
    }
}
