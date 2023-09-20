use anyhow::{anyhow, Result};
use git2::{Diff, DiffFindOptions, Oid, Patch, Repository, Signature, Tree};

pub struct DeltaInfo<'a> {
    pub patch: Patch<'a>,
    pub add_count: usize,
    pub del_count: usize,
}

pub struct CommitInfo<'a> {
    pub oid: String,
    pub parentoid: Option<String>,
    pub author: Signature<'a>,
    pub summary: Option<String>,
    pub msg: Option<String>,
    pub commit_tree: Tree<'a>,
    pub parent_tree: Option<Tree<'a>>,
    pub diff: Diff<'a>,
    pub deltas: Vec<DeltaInfo<'a>>,
    pub add_count: usize,
    pub del_count: usize,
    pub file_count: usize,
}

pub fn get_commitinfo(repo: &Repository, oid: String) -> Result<CommitInfo> {
    let commit = repo.find_commit(Oid::from_str(&oid)?)?;
    let parent = commit.parent(0).ok();
    let parentoid = parent.as_ref().map(|c| c.id().to_string());
    let author = commit.author().to_owned();
    let summary = commit.summary().map(|s| s.into());
    let msg = commit.message().map(|s| s.into());
    let commit_tree = commit.tree()?;
    let parent_tree = parent.map(|c| c.tree().ok()).flatten();
    let mut diff = Repository::diff_tree_to_tree(
        repo,
        parent_tree.as_ref(),
        Some(&commit_tree),
        None,
    )?;

    // Diff stats
    let mut add_count = 0;
    let mut del_count = 0;
    let file_count = diff.deltas().len();
    let mut opts = &mut DiffFindOptions::new();
    // Find exact match renames and copies
    opts = opts.renames(true).copies(true).exact_match_only(true);
    diff.find_similar(Some(&mut opts))?;
    let mut deltas = vec![];
    for (idx, _) in diff.deltas().enumerate() {
        let patch = Patch::from_diff(&diff, idx)?
            .ok_or(anyhow!("Error getting patch"))?;
        let (_, add, del) = patch.line_stats()?;
        let di = DeltaInfo {
            patch,
            add_count: add,
            del_count: del,
        };
        add_count += add;
        del_count += del;
        deltas.push(di);
    }

    Ok(CommitInfo {
        oid,
        parentoid,
        author,
        summary,
        msg,
        commit_tree,
        parent_tree,
        diff,
        deltas,
        add_count,
        del_count,
        file_count,
    })
}
