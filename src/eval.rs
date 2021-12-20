use lazy_static::lazy_static;
use std::{
    env,
    os::unix::prelude::CommandExt,
    process::{Child, Command},
    string::ToString,
    sync::mpsc::Sender,
};

/* pub struct Internalcommand {
    keyword: String,
    args: Vec<String>,
} */

pub enum CommandError {
    Error(Option<&'static str>),
    Exit(Option<i32>),
    Terminated(i32), // If the program was terminated by the user
}

// TODO: Stop relying so much on regexes when they're not really needed
lazy_static! {
    static ref SEMICOLON: fancy_regex::Regex = fancy_regex::Regex::new("(?<!\\\\)\\;$").unwrap();
    static ref QUOTE_START: fancy_regex::Regex = fancy_regex::Regex::new("^\"").unwrap();
    static ref QUOTE_END: fancy_regex::Regex = fancy_regex::Regex::new("(?<!\\\\)\\\"$").unwrap();
}

#[derive(Debug)]
pub struct InternalCommand {
    orig: String,
    not_sync: bool,
    commands: CommandStructure,
}

#[derive(Debug)]
pub enum CommandStructure {
    And {
        lhs: Box<CommandStructure>,
        rhs: Box<CommandStructure>,
    },
    Or {
        lhs: Box<CommandStructure>,
        rhs: Box<CommandStructure>,
    },
    Both {
        lhs: Box<CommandStructure>,
        rhs: Box<CommandStructure>,
    },
    Uncalled(Instruction),
    Finished {
        return_code: i32,
        stdout: String,
        stderr: String,
    },
}

#[derive(Debug)]
pub enum Instruction {
    Exit(Option<i32>),
    Cd(Option<String>),
    Normal { command: String, args: Vec<String> },
    Exec { command: String, args: Vec<String> },
    Empty,
    Incorrect(String),
}


impl Instruction {
    fn new(com: String, mut args: Vec<String>) -> Self {
        match (com.as_ref(), &args) {
            ("cd", _) => {
                let t = if args.len() > 0 {
                    Some(args.remove(0))
                } else {
                    None
                };
                Self::Cd(t)
            }
            ("exec", _) => {
                let com;
                if args.len() > 0 {
                    com = args.remove(0);
                } else {
                    com = String::new();
                }
                Self::Exec { command: com, args }
            }
            ("exit", _) => {
                if let Some(i) = args.get(0) {
                    match i.parse::<i32>() {
                        Ok(e) => Self::Exit(Some(e)),
                        Err(i) => Self::Incorrect(format!(
                            "exit: argument '{}' is not a valid integer.",
                            i
                        )),
                    }
                } else {
                    Self::Exit(None)
                }
            }
            ("", _) => Self::Empty,
            (_, _) => Self::Normal { command: com, args },
        }
    }
}

// Expand values. For now this is used only to expand ~ into $HOME,
// but it could easily be modified to be used for variables
fn expand(raw: String) -> String {
    lazy_static! {
        static ref RE: fancy_regex::Regex = fancy_regex::Regex::new("(?<!\\\\)\\~").unwrap();
    }
    RE.replace_all(&raw, env::var("HOME").unwrap()).to_string()
}

impl InternalCommand {
    pub fn new(i: String) -> Result<InternalCommand, CommandError> {
        let i = i.trim().to_owned();
        let i = expand(i);
        let words = i.split_whitespace().map(str::to_string).collect();
        let words = format_quotes(words)?;
        let not_sync = words.iter().last() == Some(&"&".to_owned());
        Ok(InternalCommand {
            not_sync,
            orig: i,
            commands: CommandStructure::construct(words)?,
        })
    }
    pub fn call(&mut self) -> Result<i32, CommandError> {
        self.commands.call()
    }
}

impl CommandStructure {

    fn construct(mut i: Vec<String>) -> Result<CommandStructure, CommandError> {
        if i.is_empty() {
            return Ok(Self::Uncalled(Instruction::Empty));
        }
        if i[0] == "&&" || &i[0] == "||" {
            return Err(CommandError::Error(Some("Incorrect syntax!")));
        }

        let command = i.remove(0);
        for idx in 0..i.len() {
            if SEMICOLON.is_match(&i[idx]).unwrap() {
                if idx == i.len() - 1 {
                    return Ok(Self::Uncalled(Instruction::new(command, i)));
                }
                let rest = i.split_off(idx + 1);
                return Ok(Self::Both {
                    lhs: Box::new(Self::Uncalled(Instruction::new(command, i))),
                    rhs: Box::new(Self::construct(rest)?),
                });
            }
            if &i[idx] == "&&" {
                if idx == i.len() - 1 {
                    return Err(CommandError::Error(Some("Incorrect syntax!")));
                }
                let rest = i.split_off(idx + 1);
                i.pop(); // get rid of the trailing "&&"
                return Ok(Self::And {
                    lhs: Box::new(Self::Uncalled(Instruction::new(command, i))),
                    rhs: Box::new(Self::construct(rest)?),
                });
            }
            if &i[idx] == "||" {
                if idx == i.len() - 1 {
                    return Err(CommandError::Error(Some("Incorrect syntax!")));
                }
                let rest = i.split_off(idx + 1);
                i.pop(); // get rid of the trailing "||"
                return Ok(Self::Or {
                    lhs: Box::new(Self::Uncalled(Instruction::new(command, i))),
                    rhs: Box::new(Self::construct(rest)?),
                });
            }
        }
        Ok(Self::Uncalled(Instruction::new(command, i)))
    }

    fn call(&mut self) -> Result<i32, CommandError> {
        match self {
            CommandStructure::And { lhs, rhs } => {
                let left = lhs.call()?;
                if left == 0 {
                    rhs.call()
                } else {
                    Ok(left)
                }
            }
            CommandStructure::Or { lhs, rhs } => {
                let left = lhs.call()?;
                if left != 0 {
                    rhs.call()
                } else {
                    Ok(left)
                }
            }
            CommandStructure::Both { lhs, rhs } => {
                lhs.call()?;
                rhs.call()
            }
            CommandStructure::Uncalled(inst) => {
                match inst {
                    Instruction::Exit(code) => Err(CommandError::Exit(*code)),
                    Instruction::Cd(dir) => {
                        if let Some(d) = dir {
                            match env::set_current_dir(d) {
                                Ok(()) => Ok(0),
                                Err(_) => Err(CommandError::Error(Some("No such directory!"))),
                            }
                        } else {
                            env::set_current_dir(env::var("HOME").unwrap()).unwrap();
                            Ok(0)
                        }
                    }
                    Instruction::Normal { command, args } => {
                        match Command::new(command).args(args).spawn() {
                            Err(_) => Err(CommandError::Error(Some("vsh: No such command."))),
                            Ok(child) => match child.wait_with_output() {
                                Err(e) => Err(CommandError::Error(None)),
                                Ok(o) => {
                                    *self = CommandStructure::Finished {
                                        return_code: o.status.code().unwrap_or(127),
                                        stdout: String::from_utf8(o.stdout).unwrap_or_default(),
                                        stderr: String::from_utf8(o.stderr).unwrap_or_default(),
                                    };
                                    Ok(o.status.code().unwrap_or(127))
                                }
                            },
                        }
                    }
                    Instruction::Exec { command, args } => {
                        Command::new(command).args(args).exec();
                        Err(CommandError::Error(Some("vsh: command not found")))
                    }
                    Instruction::Empty => {
                        println!();
                        Ok(0)
                    }
                    Instruction::Incorrect(msg) => {
                        eprintln!("{}", msg);
                        Err(CommandError::Error(None))
                    }
                }
            }
            CommandStructure::Finished {
                return_code,
                stdout,
                stderr,
            } => Ok(*return_code),
        }
    }
}

fn format_quotes(i: Vec<String>) -> Result<Vec<String>, CommandError> {
    let mut curr = None;
    let mut to_return = Vec::with_capacity(i.len());
    for mut word in i {
        if QUOTE_START.is_match(&word).unwrap() {
            if curr.is_none() {
                word.remove(0);
                curr = Some(word);
                continue;
            } else {
                return Err(CommandError::Error(Some("Incorrect syntax!")));
            }
        }

        if curr.is_some() {
            if !QUOTE_END.is_match(&word).unwrap() {
                curr = curr.map(|mut x| {
                    x.extend(word.chars());
                    x
                });
            } else {
                let mut to_append = curr.take().unwrap();
                to_append.extend(word.chars());
                to_return.push(to_append);
            }
            continue;
        }
        to_return.push(word);
    }
    Ok(to_return)
}
