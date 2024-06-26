//! # rsminder
//!
//! `rsminder` is a command-line interface (CLI) application that
//! allows users to manage todos, reminders, and create
//! custom tables to store useful information.
//!
//! ## Usage
//!
//! The CLI app provides various subcommands to perform different operations. Here are the available
//! subcommands:
//!
//! - `new-key`: Resets the account key.
//! - `logout`: Logs out from the account.
//! - `list`: Lists tables with specifications or table contents. It supports options like filtering
//!   by table name, specifying a group, and sorting the output.
//! - `create`: Creates a new table.
//! - `drop`: Deletes a table.
//! - `add`: Adds a task into a table. It supports adding tasks from text input or file input with
//!   options like specifying due date, group, etc.
//! - `remove`: Removes a task from a table.
//! - `update`: Updates a task from a table. It supports updating task description, due date, group,
//!   etc.
//! - `clear`: Clears completely a table.
//!
//! ## Subcommands and Arguments
//!
//! Each subcommand has its own set of arguments and options. Below are the details of each subcommand
//! and their corresponding arguments:
//!
//! - `new-key`: No arguments.
//!
//! - `logout`: No arguments.
//!
//! - `list`:
//!     - `tablename`: Name of the table to show (optional).
//!     - `group`: Specify the group to show (requires `tablename`).
//!     - `sort-by`: The key to sort the output by (requires `tablename`).
//!
//! - `create`:
//!     - `tablename`: Name of the table to create (required).
//!     - `due`: Set if the table has a due time, defaults to false.
//!
//! - `drop`:
//!     - `tablename`: Name of the table to remove (required).
//!
//! - `add`:
//!     - `tablename`: Name of the table where to add the task (required).
//!     - `task`: The task to add as text (conflicts with `file`).
//!     - `file`: File from where to find the description of the task to add (conflicts with `task`).
//!     - `line`: Add task from a specific line (requires `file`).
//!     - `range`: Add task from a range (requires `file`).
//!     - `due`: The due of the task in one of the formats: 'hh:mm' or 'YYYY-MM-dd hh:mm'.
//!     - `group`: The group of the task.
//!
//! - `remove`:
//!     - `tablename`: Name of the table where to remove the task (required).
//!     - `desc`: The description of the task to remove (required).
//!
//! - `update`:
//!     - `tablename`: Name of the table where to update the task (required).
//!     - `desc`: The description of the task to update (required).
//!     - `task`: The new description of the task as text (conflicts with `file`).
//!     - `file`: The new description of the task from a file (conflicts with `task`).
//!     - `line`: Add task from a specific line (requires `file`).
//!     - `range`: Add task from a range (requires `file`).
//!     - `due`: The due of the task in one of the formats: 'hh:mm' or 'YYYY-MM-dd hh:mm'.
//!     - `group`: The group of the task.
//!
//! - `clear`:
//!     - `tablename`: Name of the table where to clear (required).
//!
//! ## Main Function
//!
//! The `main` function initializes the CLI app, sets up logging, parses command-line arguments,
//! handles subcommands, interacts with the API, and performs corresponding actions based on user input.
//!
//! ## Modules
//!
//! The `main.rs` file includes several modules:
//!
//! - `api`: Contains API-related functionalities.
//! - `error`: Defines custom error types and handling.
//! - `parsers`: Provides parsers for parsing input data.
//! - `utils`: Includes utility functions for configuration management, user interaction, etc.
//!
//! ## Dependencies
//!
//! The CLI app uses several external crates:
//!
//! - `clap`: For parsing command-line arguments.
//! - `dotenv`: For loading environment variables from a .env file.
//! - `log4rs`: For logging configuration and management.
//!
//! ## Additional Notes
//!
//! - The CLI app interacts with an API for user authentication and data management.
//! - It handles user authentication (login/signup) and supports basic CRUD operations on tables
//!   and tasks.
//! - Error handling is implemented to provide informative error messages to the user.
//! - Configuration management is handled through a `Config` struct.
//! - The app provides a user-friendly interface with prompts for user input.
//! - Logging is configured to log events and errors for debugging and monitoring purposes.
//!
//! For further details on specific functions and implementations, refer to the comments and code
//! in the `main.rs` file.
use std::io::Write;
use std::{collections::HashMap, path::PathBuf};
use std::{env, io};

use clap::{command, value_parser, Arg, ArgAction, ArgGroup, Command};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use parsers::LineRange;
use utils::config_helper::{Config, Token};
use utils::find_log_path;

use crate::api::{ErrorResponse, SuccessfulResponse};
use crate::error::Result;
use crate::parsers::Due;
use crate::utils::{get_user_choice, resolve_file_input, Choice};
use crate::{api::Api, error::Error};

pub mod api;
pub mod error;
pub mod parsers;
pub mod utils;

/// Return the clalp arg matcher for the cli input
fn app_args() -> clap::ArgMatches {
    command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("new-key").about("Resets the account key"))
        .subcommand(Command::new("logout").about("Logout from the account"))
        .subcommand(
            Command::new("list")
                .about("List tables with specs or table contents")
                .arg(
                    Arg::new("tablename")
                        .required(false)
                        .help("Name of the table to show"),
                )
                .arg(
                    Arg::new("group")
                        .short('g')
                        .long("group")
                        .requires("tablename")
                        .help("Specify the group to show"),
                )
                .arg(
                    Arg::new("sort-by")
                        .short('s')
                        .long("sort-by")
                        .requires("tablename")
                        .help("The key to sort the output by"), // .value_parser(["due", "group"]),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Creates a new table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table to create"),
                )
                .arg(
                    Arg::new("due")
                        .long("due")
                        .short('d')
                        .action(ArgAction::SetTrue)
                        .help("Set if the table has due time, defaults to false"),
                ),
        )
        .subcommand(
            Command::new("drop").about("Deletes a table").arg(
                Arg::new("tablename")
                    .required(true)
                    .help("Name of the table to remove"),
            ),
        )
        .subcommand(
            Command::new("add")
                .about("Adds a task into a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to add the task"),
                )
                .group(
                    ArgGroup::new("source")
                        .required(true)
                        .args(&["task", "file"]),
                )
                .arg(
                    Arg::new("task")
                        .long("task")
                        .short('t')
                        .conflicts_with("file")
                        .help("The task to add as text")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .conflicts_with("task")
                        .help("File from where to find the description of the task to add")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("line")
                        .long("line")
                        .short('l')
                        .requires("file")
                        .help("Add task from a specific line")
                        .value_parser(value_parser!(u16)), // non negative number
                )
                .arg(
                    Arg::new("range")
                        .long("range")
                        .short('r')
                        .value_name("START..END")
                        .requires("file")
                        .help("Add task from a range")
                        .value_parser(value_parser!(LineRange)),
                )
                .arg(
                    Arg::new("due")
                        .long("due")
                        .short('d')
                        .help("The due of the task in one of the formats: 'hh:mm' or 'YYYY-MM-dd hh:mm'")
                        .value_parser(value_parser!(Due)),
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .short('g')
                        .help("The group of the task"),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Removes a task from a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to remove the task"),
                )
                .arg(
                    Arg::new("desc")
                        .required(true)
                        .help("The description of the task to remove")
                        .value_parser(value_parser!(String)),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Updates a task from a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to update the task"),
                )
                .arg(
                    Arg::new("desc")
                        .required(true)
                        .help("The description of the task to update")
                        .value_parser(value_parser!(String)),
                )
                .group(
                    ArgGroup::new("source")
                        .required(true)
                        .args(&["task", "file"]),
                )
                .arg(
                    Arg::new("task")
                        .long("task")
                        .short('t')
                        .conflicts_with("file")
                        .help("The new description of the task as text")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .conflicts_with("task")
                        .help("The new description of the task from a file")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("line")
                        .long("line")
                        .short('l')
                        .requires("file")
                        .help("Add task from a specific line")
                        .value_parser(value_parser!(u16)), // non negative number
                )
                .arg(
                    Arg::new("range")
                        .long("range")
                        .short('r')
                        .value_name("START..END")
                        .requires("file")
                        .help("Add task from a range")
                        .value_parser(value_parser!(LineRange)),
                )
                .arg(
                    Arg::new("due")
                        .long("due")
                        .short('d')
                        .help("The due of the task in one of the formats: 'hh:mm' or 'YYYY-MM-dd hh:mm'")
                        .value_parser(value_parser!(Due)),
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .short('g')
                        .help("The group of the task"),
                ),
        )
        .subcommand(
            Command::new("clear")
                .about("Clears completely a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to clear"),
                ),
        )
        .get_matches()
}

const ENV_FILE: &str = include_str!("env_path.txt");

/// Handles all the matching of the cli areguments
fn main() -> Result<()> {
    dotenv::from_path(ENV_FILE.trim()).unwrap();

    let log_path = find_log_path();
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}",
        )))
        .build(log_path)
        .unwrap();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("file_logger", Box::new(file_appender)))
        .logger(
            Logger::builder()
                .appender("file_logger")
                .build("app::backend", log::LevelFilter::Info),
        )
        .build(
            Root::builder()
                .appender("file_logger")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    //init config and if it is the first time running show the default prompt
    let mut config = Config::get_config()?;
    let args = app_args();

    let mut api = if !args.subcommand_matches("new-key").is_some() {
        if config.first_run {
            let api = Api::new_without_token();
            show_first_run_prompt(&api, &mut config)?;
            config.first_run = false;
            config.update_config()?;
        }
        Api::new()?
    } else {
        Api::new_without_token()
    };

    match args.subcommand() {
        Some(("new-key", _)) => {
            println!("Please input your credentials: ");
            print!("username: ");
            io::stdout().flush().map_err(|_| Error::RsmFailed)?;

            let mut username = String::new();
            io::stdin()
                .read_line(&mut username)
                .map_err(|_| Error::RsmFailed)?;

            let password =
                rpassword::prompt_password("password: ").map_err(|_| Error::RsmFailed)?;

            // prettier output
            println!("");
            let handle = terminal_spinners::SpinnerBuilder::new()
                .spinner(&terminal_spinners::DOTS)
                .text("Making a new key...")
                .start();
            let res = api.post_lostkey(&username, &password)?;
            handle.done();
            log::info!("Successfully sent POST lostkey request and received response");

            let res_type = &res.as_any();
            if res_type.is::<ErrorResponse>() {
                res.print();
                return Err(Error::FailedToUpdateKey);
            } else if res_type.is::<SuccessfulResponse>() {
                res.print();
                println!("\x1b[34mNow login again\x1b[0m\n");
                config.first_run = true;
                config.update_config()?;

                let (key, token) = login(&api).map_err(|e| {
                    log::error!("{e:?}");
                    e
                })?;
                config.key = Some(key.0.replace("\n", ""));
                let token: String = token.into();
                config.token = Some(token.replace("\n", ""));
                config.first_run = false;
                config.update_config()?;

                log::info!("successful key change process");
            }
        }
        Some(("logout", _)) => {
            print!("Do you really want to log out(yes, [no]): ");
            std::io::stdout().flush().map_err(|_| Error::RsmFailed)?;
            let choice = get_user_choice().map_err(|_| Error::RsmFailed)?;

            let logout: bool = match choice {
                Choice::Yes => true,
                Choice::No => false,
            };

            match api.post_logout(logout) {
                Ok(res) => {
                    log::info!("Successfully sent POST logout request and received response");
                    if logout {
                        // reset config
                        config.token = None;
                        config.first_run = true;
                        config.key = None;
                    }
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while logging out: {:?}", err);
                    return Err(err);
                }
            }
            config.update_config()?;
            api.update_token()?;
        }
        Some(("list", sub_matches)) => {
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.as_str());
            let group = sub_matches.get_one::<String>("group").map(|s| s.as_str());
            let sort_key = sub_matches.get_one::<String>("sort-by").map(|s| s.as_str());

            let mut opts_map: HashMap<&str, &str> = HashMap::new();
            if let Some(group_value) = group {
                opts_map.insert("group", group_value);
            }
            if let Some(sort_by_value) = sort_key {
                opts_map.insert("sort_by", sort_by_value);
            }

            match api.get_tasks(tablename, opts_map) {
                Ok(res) => {
                    log::info!("Successfully sent GET list request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while fetching tasks: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("create", sub_matches)) => {
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.to_owned())
                .unwrap();

            let has_due = sub_matches
                .get_one::<bool>("due")
                .map(|b| b.clone())
                .unwrap();

            match api.create_table(tablename, has_due) {
                Ok(res) => {
                    log::info!("Successfully sent POST create table request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while fetching tasks: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("drop", sub_matches)) => {
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.to_owned())
                .unwrap();

            match api.remove_table(tablename) {
                Ok(res) => {
                    log::info!(
                        "Successfully sent DELETE remove table request and received response"
                    );
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while fetching tasks: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("add", sub_matches)) => {
            // if tablename isnt present something really wrong happened
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.clone())
                .unwrap();
            let task = sub_matches.get_one::<String>("task");
            let file = sub_matches.get_one::<PathBuf>("file");
            let line = sub_matches.get_one::<u16>("line");
            let range = sub_matches.get_one::<LineRange>("range");
            let due = sub_matches.get_one::<Due>("due");
            let group = sub_matches.get_one::<String>("group");

            // get the task
            let task = if let Some(file) = file {
                // file input
                let task = resolve_file_input(file, line, range).map_err(|e| {
                    Error::FailedToResolveFile {
                        detail: e.to_string(),
                    }
                })?;
                task
            } else {
                // text input
                task.map_or("".to_owned(), |task| task.clone())
            };

            let mut opts_map: HashMap<&str, &str> = HashMap::new();
            if let Some(due) = due {
                opts_map.insert("due", &due.0);
            }

            if let Some(group) = group {
                opts_map.insert("group", group);
            }

            opts_map.insert("description", &task);

            match api.add_task(tablename, opts_map) {
                Ok(res) => {
                    log::info!("Successfully sent POST add request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while adding task: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("remove", sub_matches)) => {
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.to_owned())
                .unwrap();
            let desc = sub_matches
                .get_one::<String>("desc")
                .map(|s| s.to_owned())
                .unwrap();

            match api.remove_task(tablename, desc) {
                Ok(res) => {
                    log::info!("Successfully sent DELETE task request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while removing task: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("update", sub_matches)) => {
            // if tablename or the old desc isnt present something really wrong happened
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.clone())
                .unwrap();
            let old_desc = sub_matches
                .get_one::<String>("desc")
                .map(|s| s.clone())
                .unwrap();
            let task = sub_matches.get_one::<String>("task");
            let file = sub_matches.get_one::<PathBuf>("file");
            let line = sub_matches.get_one::<u16>("line");
            let range = sub_matches.get_one::<LineRange>("range");
            let due = sub_matches.get_one::<Due>("due");
            let group = sub_matches.get_one::<String>("group");

            let task = if let Some(file) = file {
                // file input
                let task = resolve_file_input(file, line, range).map_err(|e| {
                    Error::FailedToResolveFile {
                        detail: e.to_string(),
                    }
                })?;
                task
            } else {
                // text input
                task.map_or("".to_owned(), |task| task.clone())
            };

            let mut opts_map: HashMap<&str, &str> = HashMap::new();
            if let Some(due) = due {
                opts_map.insert("due", &due.0);
            }

            if let Some(group) = group {
                opts_map.insert("group", group);
            }

            opts_map.insert("description", &task);

            match api.update_task(tablename, old_desc, opts_map) {
                Ok(res) => {
                    log::info!("Successfully sent PUT update request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while adding task: {:?}", err);
                    return Err(err);
                }
            }
        }
        Some(("clear", sub_matches)) => {
            let tablename = sub_matches
                .get_one::<String>("tablename")
                .map(|s| s.to_owned())
                .unwrap();

            match api.clear_table(tablename) {
                Ok(res) => {
                    log::info!("Successfully sent DELETE clear request and received response");
                    res.print();
                }
                Err(err) => {
                    log::error!("Error occurred while adding task: {:?}", err);
                    return Err(err);
                }
            }
        }
        _ => unreachable!("If you are reading this something really bad happened"),
    }

    Ok(())
}

/// If it is the first time running the app for the user this function handles his login or signup
///
/// # Args
/// - api: struct `Api` that represents the interface to the api
/// - config: struct `Config` that represents the config management
fn show_first_run_prompt(api: &Api, config: &mut Config) -> Result<()> {
    println!("\x1b[34mWelcome to RsMember!\x1b[0m\n");

    print!("do you already have a key([yes]/no): ");
    std::io::stdout().flush().map_err(|_| Error::RsmFailed)?;
    let choice = get_user_choice().map_err(|_| Error::RsmFailed)?;

    match choice {
        // send login req
        Choice::Yes => {
            let (key, token) = login(api).map_err(|e| {
                log::error!("{e:?}");
                e
            })?;
            config.key = Some(key.0.replace("\n", ""));
            let token: String = token.into();
            config.token = Some(token.replace("\n", ""));

            log::info!("successful login");
            Ok(())
        }
        // send signup req
        Choice::No => {
            signup(api).map_err(|e| {
                log::error!("{e:?}");
                e
            })?;
            println!("Log in:");

            let (key, token) = login(api).map_err(|e| {
                log::error!("{e:?}");
                e
            })?;
            config.key = Some(key.0.replace("\n", ""));
            let token: String = token.into();
            config.token = Some(token.replace("\n", ""));

            log::info!("successful signup and login");
            Ok(())
        }
    }
}

/// Wrapper struct that represents an api key
struct Key(String);

impl From<String> for Key {
    fn from(value: String) -> Key {
        Key(value)
    }
}

/// Handles the login logic
///
/// # Args
/// - api: struct `Api` that represents the interface to the api
fn login(api: &Api) -> Result<(Key, Token)> {
    println!("Please input your key");

    let mut key = String::new();
    io::stdin()
        .read_line(&mut key)
        .map_err(|_| Error::RsmFailed)?;

    // prettier output
    println!("");
    let handle = terminal_spinners::SpinnerBuilder::new()
        .spinner(&terminal_spinners::DOTS)
        .text("Signing up...")
        .start();
    let res = api.post_login(&key)?;
    handle.done();

    let res_type = &res.0.as_any();
    if res_type.is::<ErrorResponse>() {
        res.0.print();
        return Err(Error::LoginFail);
    } else if res_type.is::<SuccessfulResponse>() {
        res.0.print();
        println!("\x1b[34mWelcome to this machine!\x1b[0m\n");
    }
    Ok((key.into(), res.1.into()))
}

/// Handles the signup logic
///
/// # Args
/// - api: struct `Api` that represents the interface to the api
fn signup(api: &Api) -> Result<()> {
    println!("Create Account:");
    print!("username: ");
    io::stdout().flush().map_err(|_| Error::RsmFailed)?;

    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .map_err(|_| Error::RsmFailed)?;

    let password = rpassword::prompt_password("password: ").map_err(|_| Error::RsmFailed)?;

    // prettier output
    println!("");
    let handle = terminal_spinners::SpinnerBuilder::new()
        .spinner(&terminal_spinners::DOTS)
        .text("Signing up...")
        .start();
    let res = api.post_signup(&username, &password)?;
    handle.done();

    let res_type = &res.as_any();
    if res_type.is::<ErrorResponse>() {
        res.print();
        return Err(Error::FirstRunFailed);
    } else if res_type.is::<SuccessfulResponse>() {
        println!("Account creation successful, you can now log in!");
        res.print();
    }
    Ok(())
}
