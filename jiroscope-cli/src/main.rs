use std::io::Read;

use clap::Parser;
use jiroscope_core::jira::{AtlassianDoc, IssueEdit};

#[derive(Debug, Clone, Parser)]
#[clap(version = "1.0", author = "Stanisław Zagórowski")]
struct Args {
    #[clap(short, long, help = "jiroscope auth_config.toml file")]
    identity: Option<String>,
    #[clap(short, long, help = "Jira server url")]
    server: Option<String>,
    #[clap(short, long, help = "Jira user login")]
    user: Option<String>,
    #[clap(short, long, help = "Jira API token")]
    api_token: Option<String>,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
enum Subcommand {
    Issue {
        board_id: String,
        issue_id: String,
    },
    Edit {
        board_id: String,
        issue_id: String,
        #[clap(long)]
        summary: Option<String>,
        #[clap(long)]
        description: Option<String>,
        #[clap(long)]
        priority: Option<String>,
        #[clap(long)]
        status: Option<String>,
        #[clap(long)]
        assignee: Option<String>,
    },
    All,
    CreateMeta,
    EditMeta {
        board_id: String,
        issue_id: String,
    },
    Delete {
        board_id: String,
        issue_id: String,
    },
    Events,
}

fn main() {
    let args = Args::parse();

    let subcommand = args.subcommand;

    let mut server = None;
    let mut user = None;
    let mut api_token = None;

    if let Some(s) = args.identity {
        let mut file = match std::fs::File::open(s) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };
        let mut content: String = String::new();
        file.read_to_string(&mut content)
            .expect("Error reading file");

        let table = toml::from_str::<toml::Table>(content.as_str()).unwrap();

        if let Some(s) = table.get("server") {
            server = Some(s.as_str().unwrap().to_string());
        }

        if let Some(s) = table.get("user") {
            user = Some(s.as_str().unwrap().to_string());
        }

        if let Some(s) = table.get("api_token") {
            api_token = Some(s.as_str().unwrap().to_string());
        }
    }

    if let Some(s) = args.server {
        server = Some(s);
    }

    if let Some(s) = args.user {
        user = Some(s);
    }

    if let Some(s) = args.api_token {
        api_token = Some(s);
    }

    if server.is_none() || user.is_none() || api_token.is_none() {
        eprintln!("Error: missing auth config");
        std::process::exit(1);
    }

    let config = jiroscope_core::Config::new(server.unwrap());
    let auth = jiroscope_core::Auth::new(user.unwrap(), api_token.unwrap());

    let mut jiroscope = jiroscope_core::Jiroscope::new(config, auth);
    jiroscope.init().unwrap();

    match subcommand {
        Subcommand::All => {
            let issues = handle_error(jiroscope.get_all_issues());
            println!("{:#?}", issues);
        }
        Subcommand::Issue { board_id, issue_id } => {
            let issue =
                handle_error(jiroscope.get_issue(format!("{}-{}", board_id, issue_id).as_str()));
            println!("{:#?}", issue);
        }
        Subcommand::Edit {
            board_id,
            issue_id,
            summary,
            description,
            priority,
            status,
            assignee,
        } => {
            let issue =
                handle_error(jiroscope.get_issue(format!("{}-{}", board_id, issue_id).as_str()));

            let mut issue_edit = IssueEdit::default();

            issue_edit.fields.summary = summary;
            issue_edit.fields.description = description.map(|d| AtlassianDoc::text(&d));
            // TODO: rest of the fields

            handle_error(
                jiroscope.edit_issue(format!("{}-{}", board_id, issue_id).as_str(), issue_edit),
            );
        }
        Subcommand::CreateMeta => {
            let meta = handle_error(jiroscope.get_issue_creation_meta());
            println!("{:#?}", meta);
        }
        Subcommand::EditMeta { board_id, issue_id } => {
            let meta = handle_error(
                jiroscope.get_issue_edit_meta(format!("{}-{}", board_id, issue_id).as_str()),
            );
            println!("{:#?}", meta);
        }
        Subcommand::Events => {
            let events = jiroscope.get_issue_events().unwrap();
            println!("{:#?}", events);
        }
        Subcommand::Delete { board_id, issue_id } => {
            handle_error(jiroscope.delete_issue(format!("{}-{}", board_id, issue_id).as_str()));
        }
    }
}

fn handle_error<T>(result: Result<T, jiroscope_core::Error>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            match e {
                jiroscope_core::Error::Jiroscope { message } => {
                    eprintln!("Error: {}", message);
                }
                jiroscope_core::Error::Auth { message } => {
                    eprintln!("Error: {}", message);
                }
                jiroscope_core::Error::Io(e) => {
                    eprintln!("Error: {}", e);
                }
                jiroscope_core::Error::Ureq(e) => match *e {
                    jiroscope_core::ureq::Error::Status(code, response) => {
                        eprintln!("Error: {} {}", code, response.status_text());
                        eprintln!("{}", response.into_string().unwrap());
                    }
                    _ => {
                        eprintln!("Error: {}", e);
                    }
                },
                jiroscope_core::Error::Jira(code, e) => {
                    eprintln!("Error {}: {}", code, e);
                }
            }
            std::process::exit(1);
        }
    }
}
