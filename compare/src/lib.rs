use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;
use git2::Oid;
use git2::Repository;

#[derive(Parser)]
#[command(author, version, about = "Compare two branches", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = ".", help = "Path to git repository")]
    pub repo: String,

    #[arg(
        default_value = "HEAD",
        help = "First target. sha, short-sha, refs are all valid input"
    )]
    pub target1: String,

    #[arg(
        default_value = "HEAD",
        help = "Second target. sha, short-sha, refs are all valid input"
    )]
    pub target2: String,
}

impl Args {
    pub fn execute(self) -> Result<Vec<(WhichBranch, Oid)>> {
        let c = Configuration::from_args(&self)?;
        c.execute()
    }
}

#[derive(Debug)]
pub enum WhichBranch {
    Branch1Only,
    Both,
    Branch2Only,
}

struct Configuration {
    repo: Repository,
    branch1: Oid,
    branch2: Oid,
}

impl Configuration {
    fn from_args(args: &Args) -> Result<Self> {
        let repo = Repository::open(&args.repo)?;
        let branch1 = repo.revparse_single(&args.target1)?.id();
        let branch2 = repo.revparse_single(&args.target2)?.id();
        Ok(Configuration {
            repo,
            branch1,
            branch2,
        })
    }

    fn execute(self) -> Result<Vec<(WhichBranch, Oid)>> {
        let base = self.repo.merge_base(self.branch1, self.branch2)?;
        let set1 = list_commits(&self.repo, base, self.branch1)?;
        let set2 = list_commits(&self.repo, base, self.branch2)?;
        Ok(set_compare(set1, set2))
    }
}

type PatchId = Oid;
type CommitId = Oid;

fn set_compare(
    patch_ids_from1: HashMap<PatchId, CommitId>,
    patch_ids_from2: HashMap<PatchId, CommitId>,
) -> Vec<(WhichBranch, Oid)> {
    let mut result = Vec::new();

    for (patch_id, commit_id) in patch_ids_from1.iter() {
        if patch_ids_from2.contains_key(patch_id) {
            result.push((WhichBranch::Both, *commit_id))
        } else {
            result.push((WhichBranch::Branch1Only, *commit_id))
        }
    }

    for (patch_id, commit_id) in patch_ids_from2.iter() {
        if !patch_ids_from1.contains_key(patch_id) {
            result.push((WhichBranch::Branch2Only, *commit_id))
        }
    }

    result
}

// commit tree <-> parent tree 간의 diff 로부터 patch id를 뽑아냄
// 원본 커밋과 체리픽 커밋이 sha는 다르더라도 diff에서 뽑은 patch id는 동일 (충돌 해결만 없다면)
fn get_patchid_from_commit(repo: &Repository, commit: Oid) -> Result<PatchId> {
    let commit = repo.find_commit(commit)?;
    let commit_tree = commit.tree()?;
    let parent_tree = {
        let parent = commit.parent_id(0)?;
        repo.find_commit(parent)?.tree()?
    };

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;
    let oid = diff.patchid(None)?;
    Ok(oid)
}

fn list_commits(repo: &Repository, base: Oid, head: Oid) -> Result<HashMap<PatchId, CommitId>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.hide(base)?;
    revwalk.push(head)?;
    revwalk.simplify_first_parent()?;

    let mut res = HashMap::new();
    for rev in revwalk {
        let commit_id = rev?;
        res.insert(get_patchid_from_commit(repo, commit_id)?, commit_id);
    }

    Ok(res)
}
