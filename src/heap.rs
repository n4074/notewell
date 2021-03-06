use anyhow::{Result, bail, Context};
use std::process::Command; 
use std::path::{PathBuf, Path};
use log::{debug, info};

use toml;

use crate::repo;
use crate::index;
use crate::card::Card;

#[derive(Debug)]
struct HeapState {
    commit: Option<String>, // Last index commit
    path: PathBuf 
}

pub struct Heap {
    path: PathBuf,
    db: sled::Db,
    index: index::Index,
    repo: repo::Repo, 
}

impl std::fmt::Debug for Heap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Heap")
         .field("path", &self.path)
         .field("db", &self.db)
         .finish()
    }
}

const NB_SUBDIR: &str = ".nb";

impl Heap {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Heap> {
        let path = path.as_ref().to_owned();
        log::debug!("input_path:{:?}", path);

        if path.exists() {
            bail!("Directory exists: {}", path.display());
        }

        let repo = crate::repo::Repo::init(&path)?;

        let mut nb_path = path.clone();

        nb_path.push(NB_SUBDIR);
        std::fs::create_dir(&nb_path)?;

        let mut index_path = nb_path.clone();
        index_path.push("index");

        std::fs::create_dir(&index_path)?;

        let index = crate::Index::create(index_path)?;

        let mut db_path = nb_path.clone();
        db_path.push("db");

        let db = sled::Config::default()
            .path(db_path)
            .create_new(true)
            .open()?;

        Ok(Heap {
            path,
            db,
            index,
            repo
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Heap> {
        let path: PathBuf = path.as_ref().to_owned().canonicalize()?;

        let repo = crate::repo::Repo::open(&path)?;

        let mut nb_path = path.clone();
        nb_path.push(NB_SUBDIR);

        let mut index_path = nb_path.clone();
        index_path.push("index");

        let mut db_path = nb_path.clone();
        db_path.push("db");

        let index = crate::Index::open(index_path)?;
        let db = sled::open(db_path)?;

        Ok(Heap {
            path,
            db,
            index,
            repo
        })
    }

    pub fn sync(&mut self) -> Result<()> {

        let latest_commit = match self.db.get(b"commit")? {
            Some(ivec) => {
                Some(std::str::from_utf8(ivec.as_ref())?.to_owned())
            }
            _ => None
        };

        let head = self.repo.head()?;
        let diffs = self.repo.diff(latest_commit.as_ref(), None)?;
    
        for diff in diffs {
            match diff {
                (git2::Delta::Added, path) | (git2::Delta::Modified, path) => { 
                    self.index.delete(&path);
                    println!("{:?}", path);

                    let mut note = self.index.notebuilder(&path);

                    let content = std::fs::read_to_string(&self.path.join(&note.path))?;
                    note.body(&content);

                    //let doc = doc_builder.document();

                    self.index.add(&path, note);
                }
                (git2::Delta::Deleted, path) => { self.index.delete(&path) } 
                (git2::Delta::Renamed, _path) => { todo!("Handling Renaming. Need both old and new path") } 
                _ => todo!()
            }
        } 

        self.index.commit()?;
        self.index.reload()?;

        self.db.insert(b"commit", head.id().to_string().into_bytes())?;

        Ok(())
    }

    /// TODO: Fix this 
    pub fn find(&self, query: &str) -> anyhow::Result<Vec<index::QueryResult>> {
        let result = self.index.query(query)?;
        debug!("query_result: {:?}", result);
        //for doc in result {
        //    let res = QueryResult("wat");
        //}
        return Ok(result)
    }

    pub fn add_card<P: AsRef<Path>>(&mut self, path: Option<P>) -> Result<()> {

        if let Some(_path) = path {

        }
        unimplemented!() 
    }
    
    pub fn edit_card<P: AsRef<Path> + Copy>(&mut self, path: P) -> Result<()> {
        let mut child = Command::new("vim")
            .args(&[self.path.join(path)])
            .spawn()
            .expect("failed to launch editor");

        let _exit = child.wait().context("failed to wait on editor subprocess")?;
        
        self.repo.commit_paths(&[path])
    }
}

pub struct SearchResult(PathBuf, String);

//impl HeapState {
//    fn open<P: AsRef<Path>>(path: P) -> Result<State> {
//         
//    }
//
//    fn create<P: AsRef<Path>>(path: P) -> Result<State> {
//        if !path.as_ref().to_owned().exists() {
//            let mut state: State = Default::default();
//            state.path = path.as_ref().to_owned();
//            state.save()?;
//            Ok(state)
//        } else {
//            let file = File::open(&path)?;
//            let mut state: State = serde_json::from_reader(file)
//                .context("Failed to deserialize state")?;
//            state.path = path.as_ref().to_owned();
//            Ok(state)
//        }
//    }
//
//    fn save(&self) -> Result<()> {
//        debug!("Saving state to {}", self.path.to_string_lossy());
//        let file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&self.path)?;
//        serde_json::to_writer(file, &self).context("Failed to serialise state")
//    }
//}

//struct App {
//    pub appdir: PathBuf,
//    pub config: config::Config,
//    pub index: index::Index,
//    pub repo: repo::Repo, 
//    pub state: State,
//}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_heap() {

        let path = "/tmp/wat";

        let heap = Heap::init(path);

        //println!("Heap: {:?}", heap);
        //match heap {
        //    Ok(_) => { println!("Wat") }
        //    Err(_) => { println!("Watwat") }
        //}

        drop(heap);

        let heap_opened = Heap::open(path);
        println!("Heap: {:?}", heap_opened);
        //assert!(false);
    }
}