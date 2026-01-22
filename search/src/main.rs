mod hnsw;
#[cfg(test)]
mod tests;
use std::{fs::OpenOptions, io::Read, path::PathBuf, time::{Duration, Instant}};

use debugging::session::debug_session::{DebugSession, LogLevel};
///
/// Search application entry point
use hnswlib_rs::{Hnsw, HnswConfig, InMemoryVectorStore, L2};
use model2vec_rs::model::StaticModel;
use sal_core::{dbg::Dbg, error::Error};

fn main() -> Result<(), Error> {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("module-name::sub::path::Class", LogLevel::Info)
        .init();
    let dbg = Dbg::own("main");
    let error = Error::new("", &dbg);
    let args: Vec<String> = std::env::args().collect();
    let dim = 1024;
    let max_nodes = 10_000_000;

    let cfg = HnswConfig::new(dim, max_nodes)
        .m(48)
        .ef_construction(400)
        // Don't bake search accuracy into the index. Keep \(M\) moderate and adjust efSearch at query time to balance speed and accuracy.
        .ef_search(50);

    let hnsw = Hnsw::new(L2::new(), cfg);
    let path = "./index.bin";
    let mut f = OpenOptions::new()
        .read(true)
        .open(&path);
    let index = match f {
        Ok(mut f) => {
            let (index, _) = InMemoryVectorStore::<f32>::load_from(&mut f)
                .map_err(|err| error.pass(err.to_string()))?;
            index
        }
        Err(err) => {
            log::warn!("{dbg} | Can't read index from '{}', error: \n\t{:?}", path, err);
            InMemoryVectorStore::<f32>::new(dim, max_nodes)
        }
    };
    if let Some(update_from_path) = args.first() {
        embedding(update_from_path, &hnsw, &index)?;
    }

    let values = 100_000;
    let mut target = vec![];
    let t = Instant::now();
    let total_ins = t.elapsed();

    let mut total_query = Duration::ZERO;
    for (key, val) in target {
        let v = vec![val; dim];
        let t = Instant::now();
        let hits = hnsw.search(&index, &v, 10, None)
            .map_err(|err| error.pass(err.to_string()))?;
        let elapsed = t.elapsed();
        total_query += elapsed;
        log::debug!("Elapsed {:?}", elapsed);
        assert_eq!(hits[0].key, key);
    }
    log::debug!("Insertion Elapsed {:?}", total_ins);
    log::debug!("Insertion Elapsed per query {:?}", total_ins / values);
    log::debug!("Insertion Elapsed {:?}", total_query);
    log::debug!("Insertion Elapsed per query {:?}", total_query / values);
    Ok(())
}

fn embedding(path: &str, hnsw: &Hnsw<String, L2>, index: &InMemoryVectorStore<f32>) -> Result<(), Error> {
    let dbg = Dbg::own("embedding");
    let error = Error::new("", &dbg);
    let path = PathBuf::from(path);
    if !path.is_dir() {
        log::warn!("{dbg} | Can't update from a single file '{}', specify the folder", path.display());
    }
    // Load a model from the Hugging Face Hub or a local path.
    // Arguments: (repo_or_path, hf_token, normalize_embeddings, subfolder_in_repo)
    let model = StaticModel::from_pretrained(
        "potion-multilingual-128M",  // Model ID from Hugging Face or local path to model directory
        None,                               // Optional: Hugging Face API token for private models
        None,                           // Optional: bool to override model's default normalization. `None` uses model's config.
        None                            // Optional: subfolder if model files are not at the root of the repo/path
    ).map_err(|err| error.pass(err.to_string()))?;
    match std::fs::read_dir(&path) {
        Ok(dir) => {
            for path in dir {
                if let Ok(path) = path {
                    let path = path.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "txt" {
                                let mut f = OpenOptions::new()
                                    .read(true)
                                    .open(&path);
                                match f {
                                    Ok(mut f) => {
                                        let mut doc = String::new();
                                        match f.read_to_string(&mut doc) {
                                            Ok(_) => {
                                                let key = path.to_str().unwrap();
                                                // Generate embeddings with the default batch size, 256
                                                let embedding = model.encode_single(&doc);
                                                log::debug!("{dbg} | Embedding length: {}", embedding.len()); // -> Embeddings length: 4
                                                hnsw.insert(index, key.to_owned(), &embedding)
                                                    .map_err(|err| error.pass(err.to_string()))?;
                                            }
                                            Err(err) => log::warn!("Can't read from {}, error: {:?}", path.display(), err),
                                        }

                                    }
                                    Err(err) => log::warn!("Can't open {}, error: {:?}", path.display(), err),
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        Err(err) => {
            Err(error.pass_with(format!("Can't read path '{}'", path.display()), err.to_string()))
        }
    }
}
