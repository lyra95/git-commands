use anyhow::Result;
use clap::Parser;
use git2::Commit;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

#[derive(Parser)]
#[command(author, version, about = "prints commit meta info", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value("."), help = "path to git repo")]
    pub repo: String,
    #[arg(default_value("HEAD"), help = "(sha/short sha/refs to print meta info")]
    pub commit: String,
    #[arg(
        short,
        long,
        default_value(r#""${author}","${email}","${jira}","${summary}","${message}""#),
        help = "print format"
    )]
    pub format: String,
}

impl Args {
    pub fn get_info(&self) -> Result<HashMap<String, String>> {
        let repo = git2::Repository::open(&self.repo)?;
        let id = repo.revparse_single(&self.commit)?.id();
        let commit = repo.find_commit(id)?;
        Ok(get_info(commit))
    }

    pub fn execute(&self) -> Result<String> {
        let info = self.get_info()?;
        Ok(format(&self.format, info))
    }
}

// hack: "은 없애고, 공백은 1space로 압축하고...
// csv 컬럼 인식이 잘 안되서 이런짓함
fn replace(message: String) -> String {
    lazy_static! {
        static ref BLANKS: Regex = Regex::new(r"([\s\t\n\r]+)").unwrap();
    }

    BLANKS
        .replace_all(&message.replace('"', ""), " ")
        .to_string()
}

fn get_info(commit: Commit) -> HashMap<String, String> {
    let mut result = HashMap::new();

    result.insert("email".to_string(), get_email(&commit));
    result.insert("author".to_string(), get_author(&commit));
    result.insert("message".to_string(), replace(get_message(&commit)));
    result.insert("summary".to_string(), replace(get_summary(&commit)));
    result.insert("jira".to_string(), get_jira_link(&commit));

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

fn get_jira_link(commit: &Commit) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(NIK-\d+)").unwrap();
    }

    let text = get_summary(commit);
    let result = RE
        .captures(&text)
        .map(|caps| format!("http://dev.shiftup.co.kr/jira/browse/{}", &caps[1].trim()));
    if let Some(link) = result {
        return link;
    }

    let text = get_message(commit);
    RE.captures(&text).map_or_else(
        || "null".to_string(),
        |caps| format!("http://dev.shiftup.co.kr/jira/browse/{}", &caps[1].trim()),
    )
}

fn format(format: &str, info: HashMap<String, String>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\$\{(?P<key>\w+)\}").unwrap();
    }

    let result = RE.replace_all(format, |caps: &regex::Captures| {
        info.get(&caps[1])
            .map_or_else(|| "null".to_string(), |x| x.to_owned())
    });
    result.into_owned()
}
