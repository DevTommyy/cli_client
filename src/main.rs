use clap::{command, value_parser, Arg, ArgAction, Command};

// TODO: make this validated due (not actually fully validated, just the thing that if you write
// only the time it detects the day)
#[derive(Clone, Debug, Default)]
struct Due(String);
impl std::str::FromStr for Due {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() > 2 || parts.is_empty() {
            return Err("Invalid date and time format");
        }

        if parts.len() == 1 {
            // only time is provided
            let time_raw = parts.get(0).map_or("", |s| s).trim();
            if time_raw.split(":").collect::<Vec<&str>>().len() != 2 || time_raw.is_empty() {
                return Err("Invalid time");
            }

            let time = chrono::NaiveTime::parse_from_str(time_raw, "%H:%M")
                .map_err(|_| Self::Err::from("Invalid time format"))?;

            let now = chrono::Local::now().time();
            let today = chrono::Local::now().naive_local();

            // if the time is in the past then the date has to be tomorrow
            let date = if time < now {
                (today + chrono::Duration::days(1)).date()
            } else {
                today.date()
            }
            .to_string();

            Ok(Due(format!("{date}T{time_raw}:00")))
        } else {
            // date and time are provided
            let date_raw = parts.get(0).map_or("", |s| s).trim();
            if date_raw.split("-").collect::<Vec<&str>>().len() != 3 || date_raw.is_empty() {
                return Err("Invalid date");
            }

            let time_raw = parts.get(1).map_or("", |s| s).trim();
            if time_raw.split(":").collect::<Vec<&str>>().len() != 2 || time_raw.is_empty() {
                return Err("Invalid time");
            }
            Ok(Due(format!("{date_raw}T{time_raw}:00")))
        }
    }
}

fn app_args() -> clap::ArgMatches {
    command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        // auth routes (login and signup are done automatically on the first login)
        .subcommand(Command::new("logout").about("Logout from the account"))
        // item route commands
        .subcommand(
            Command::new("list")
                .about("List tables or specified table contents")
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
                        .help("The key to sort the output by"),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Adds an item into a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to add the task"),
                )
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the item to add, as text")
                        .value_parser(value_parser!(String)),
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
            Command::new("update")
                .about("Updates an item from a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to update the task"),
                )
                .arg(
                    Arg::new("id")
                        .required(true)
                        .help("The id of the item to remove")
                        .value_parser(value_parser!(i32)),
                )
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the item to update, as text")
                        .value_parser(value_parser!(String)),
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
                .about("Removes an item from a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table where to remove the item"),
                )
                .arg(
                    Arg::new("id")
                        .required(true)
                        .help("The id of the item to remove")
                        .value_parser(value_parser!(i32)),
                ),
        )
        .subcommand(
            Command::new("clear")
                .about("Removes all items in a table")
                .arg(
                    Arg::new("tablename")
                        .required(true)
                        .help("Name of the table to clear"),
                ),
        )
        // table routes
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
                        .help("Set if the table supports due time, defaults to false"),
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .short('g')
                        .action(ArgAction::SetFalse)
                        .help("Set if the table supports groups, defaults to true"),
                ),
        )
        .subcommand(
            Command::new("drop").about("Deletes a table").arg(
                Arg::new("tablename")
                    .required(true)
                    .help("Name of the table to remove"),
            ),
        )
        .get_matches()
}

// TODO: make 2 apps in one, you can chose the frontend, if its just cli or tui (with ratatui)
// put that thing in the config file
fn main() {
    println!("Hello, world!");
}
