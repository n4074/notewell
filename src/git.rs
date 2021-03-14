use std::process::Command;
use anyhow::Result;
use std::path::{Path,PathBuf};

use git2::{Repository, Delta, Oid, Commit, Tree, RepositoryInitOptions};
use git2;

pub struct NoteWell {
    repo: Repository,
    synced: Option<Oid>, // Last index commit
}

impl NoteWell {

    pub fn init(path: &Path, synced: Option<Oid>) -> Result<NoteWell> {
        let repo = Repository::init(path)?;
        Ok(NoteWell { repo, synced })
    }

    fn last_tree(&self) -> Result<Option<Tree<'_>>> {
        if let Some(oid) = self.synced {
            let tree = self.repo.find_commit(oid)?.tree()?;
            Ok(Some(tree))
        } else {
            Ok(None)
        }
    }

    pub fn synced(&mut self, commit: Oid) -> Result<()> {
        self.synced = Some(commit);
        Ok(())
    }

    pub fn diff(&self) -> Result<(Oid, Vec<(Delta, PathBuf)>)> {
        let head = self.repo.head()?.peel_to_commit()?;
        let tree = head.tree()?; // If no head, we have nothing to index
        let diff = self.repo.diff_tree_to_tree(self.last_tree()?.as_ref(), Some(&tree), None)?;

        Ok((head.id(), diff.deltas().map(|delta| {
            let path = delta.new_file().path().unwrap().to_owned();
            (delta.status(), path)
        }).collect()))
    }

    fn get_changes(&self) -> Result<Vec<(Delta,PathBuf)>> {
        let head = self.repo.head();
        let mut diffs = vec!();
        if self.synced.is_some() && head.is_ok() {
            let head = head.unwrap().peel_to_tree()?;
            let synced = self.repo.find_commit(self.synced.unwrap())?.tree()?;
            let tree_diff = self.repo.diff_tree_to_tree(Some(&synced), Some(&head), None)?;
            let workdir_dir = self.repo.diff_tree_to_workdir_with_index(Some(&head), None)?; 

            for delta in tree_diff.deltas().chain(workdir_dir.deltas()) {
                let path = delta.new_file().path().unwrap().to_owned();
                diffs.push((delta.status(), path));
            }

        } else if self.synced.is_none() || (head.is_err() && head.err().unwrap().code() == git2::ErrorCode::UnbornBranch) {
            for item in self.repo.index()?.iter() {
                let path = Path::new(std::str::from_utf8(&item.path)?).to_owned(); 
                diffs.push((Delta::Added, path))
            }
        } else {
            todo!("Handle error here")
        }

        return Ok(diffs);
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use anyhow::Context;
    use std::io::Write;
    use git2::{Repository,Signature};

    #[test]
    fn test_list_changes() -> anyhow::Result<()> {
        let testnotes_dir = tempdir().context("failed to create tempdir")?;
        let a = testnotes_dir.path().join("a");
        let b = testnotes_dir.path().join("b");
        let c = testnotes_dir.path().join("c");
        let d = testnotes_dir.path().join("d");
        let mut a = std::fs::File::create(&a)?;
        let mut b = std::fs::File::create(b)?;
        let mut c = std::fs::File::create(c)?;
        let mut d = std::fs::File::create(d)?;

        let repo = Repository::init(testnotes_dir.path())?;


        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("a"))?;
        let tree_id = index.write_tree()?;
        index.write()?;

        let tree = repo.find_tree(tree_id)?;

        let author = Signature::now("you", "you@us")?;
        let committer = Signature::now("me", "me@us")?;
        let message = "Initial commit";

        let mut parents: Vec<&git2::Commit<'_>> = vec!();
        let mut head: git2::Commit<'_>;


        if let Ok(commit) = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid)) {
            head = commit;
            parents.push(&head);
        };

        let repo2 = Repository::open(testnotes_dir.path())?;
        let nb_none = NoteWell { repo: repo2, synced: None };

        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &parents)?;

        let oid = repo.head()?.peel_to_commit()?.id();

        let repo2 = Repository::open(testnotes_dir.path())?;
        let nb = NoteWell { repo: repo2, synced: Some(oid) };


        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("b"))?;
        index.add_path(std::path::Path::new("c"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        index.write()?;

        head = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid))?;
        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &[&head])?;

        let mut index = repo.index()?;
        index.remove_path(std::path::Path::new("b"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        index.write()?;

        head = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid))?;
        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &[&head])?;


        let mut index = repo.index()?;
        c.write("wat".as_bytes())?;
        index.add_path(std::path::Path::new("c"))?;
        index.add_path(std::path::Path::new("d"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        index.write()?;


        let repo3 = Repository::open(testnotes_dir.path())?;
        let nb = NoteWell { repo: repo3, synced: Some(oid) };
        println!("{:?}", nb.get_changes()?);
        println!("{:?}", nb.diffs()?);
        println!("{:?}", nb_none.get_changes()?);
        println!("{:?}", nb_none.diffs()?);

        println!("{:?}\n", testnotes_dir.path());


        for reference in repo.references()?.names() {
            print!("ref: {:?}\n", reference);
        }

        print!("head: {:?}\n", repo.revparse_single("HEAD")?.peel_to_tree());
        print!("head~3: {:?}\n", repo.revparse_single("HEAD~2")?.peel_to_tree());

        let tree_a = repo.revparse_single("HEAD")?.peel_to_tree()?;
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree_a), None)?;

        print!("Deltas: {:?}\n", diff.deltas().len());

        for delta in diff.deltas() {
            println!("{:?}\n", delta);
        }

        assert!(false);
        Ok(())
    }

}