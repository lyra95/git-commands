use anyhow::Result;
use clap::Parser;
use git2::Commit;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = git2::Repository::open(&args.repo)?;
    let id = repo.revparse_single(&args.commit)?.id();
    let commit = repo.find_commit(id)?;

    let info = get_info(&commit);
    print_info(&args.format, info);
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about = "Compare two branches", long_about = None)]
struct Args {
    #[arg(short, long, default_value("."), help = "path to git repo")]
    repo: String,
    #[arg(default_value("HEAD"), help = "(sha/short sha/refs to print meta info")]
    commit: String,
    #[arg(
        short,
        long,
        default_value("${author},${email},${pr},${jira},${summary},${message}"),
        help = "print format"
    )]
    format: String,
}

fn get_info<'a>(commit: &'a Commit) -> HashMap<&'a str, String> {
    let mut result = HashMap::new();

    result.insert("email", get_email(commit));
    result.insert("author", get_author(commit));
    result.insert("message", get_message(commit));
    result.insert("summary", get_summary(commit));
    result.insert("pr", get_pr_link(commit));
    result.insert("jira", get_jira_link(commit));

    result
}

fn get_email(commit: &Commit) -> String {
    commit
        .author()
        .email()
        .map_or_else(|| "null".to_string(), |x| x.to_owned())
}

fn get_author(commit: &Commit) -> String {
    commit
        .author()
        .name()
        .map_or_else(|| "null".to_string(), |x| x.to_owned())
}

fn get_message(commit: &Commit) -> String {
    commit
        .message()
        .map_or_else(|| "null".to_string(), |x| x.trim().replace('\n', "\t"))
}

fn get_summary(commit: &Commit) -> String {
    commit
        .summary()
        .map_or_else(|| "null".to_string(), |x| x.trim().to_owned())
}

fn get_pr_link(commit: &Commit) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"#(\d+)").unwrap();
    }

    let text = get_summary(commit);
    RE.captures(&text).map_or_else(
        || "null".to_string(),
        |caps| {
            format!(
                "https://github.com/shiftupcorp/nk-backend/pull/{}",
                &caps[1].trim()
            )
        },
    )
}

fn get_jira_link(commit: &Commit) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(NIK-\d+)").unwrap();
    }

    let text = get_summary(commit);
    let result = RE.captures(&text).map(|caps| format!("http://dev.shiftup.co.kr/jira/browse/{}", &caps[1].trim()));
    if result.is_some() {
        return result.unwrap()
    }

    let text = get_message(commit);
    RE.captures(&text).map_or_else(|| "null".to_string(), |caps| format!("http://dev.shiftup.co.kr/jira/browse/{}", &caps[1].trim()))
}

fn print_info(format: &str, info: HashMap<&str, String>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\$\{(?P<key>\w+)\}").unwrap();
    }

    let result = RE.replace_all(format, |caps: &regex::Captures| {
        info.get(&caps[1])
            .map_or_else(|| "null".to_string(), |x| x.to_owned())
    });
    println!("{}", result);
}
