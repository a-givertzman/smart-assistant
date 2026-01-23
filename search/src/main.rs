mod hnsw;
#[cfg(test)]
mod tests;
use std::{fs::OpenOptions, io::Read, path::{Path, PathBuf}, time::{Duration, Instant}};

use debugging::session::debug_session::{DebugSession, LogLevel};
///
/// Search application entry point
use hnswlib_rs::{Hnsw, HnswConfig, InMemoryVectorStore, L2};
use lopdf::Document;
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

    // Load a model from the Hugging Face Hub or a local path.
    // Arguments: (repo_or_path, hf_token, normalize_embeddings, subfolder_in_repo)
    let model = StaticModel::from_pretrained(
        // "assets/potion-multilingual-128M",
        "minishlab/potion-base-32M",
        // "potion-multilingual-128M",  // Model ID from Hugging Face or local path to model directory
        None,                               // Optional: Hugging Face API token for private models
        None,                           // Optional: bool to override model's default normalization. `None` uses model's config.
        None                            // Optional: subfolder if model files are not at the root of the repo/path
    ).map_err(|err| error.pass_with("Can't load StaticModel", err.to_string()))?;

    //
    // Loading hnsw
    let dim = 512;  // Because of `StaticModel` default DIM
    let max_nodes = 10_000_000;
    let cfg = HnswConfig::new(dim, max_nodes)
        .m(48)
        .ef_construction(400)
        // Don't bake search accuracy into the index. Keep \(M\) moderate and adjust efSearch at query time to balance speed and accuracy.
        .ef_search(50);
    let hnsw = Hnsw::new(L2::new(), cfg);

    //
    // Loading Index
    let path = "./assets/index.bin";
    let f = OpenOptions::new()
        .read(true)
        .open(&path);
    let index = match f {
        Ok(mut f) => {
            let (index, _) = InMemoryVectorStore::<f32>::load_from(&mut f)
                .map_err(|err| error.pass_with(format!("Can't load index from '{}'", path), err.to_string()))?;
            index
        }
        Err(err) => {
            log::warn!("{dbg} | Can't read index from '{}', error: \n\t{:?}", path, err);
            InMemoryVectorStore::<f32>::new(dim, max_nodes)
        }
    };

    //
    // Checking args, embedding if "--update path" found then embedd new files
    if let (Some(arg), Some(param)) = (args.get(1), args.get(2)) {
        if arg == "--update" && !param.is_empty() {
            embedding(param, &path, &model, &hnsw, &index)?;
        }
    }

    let t = Instant::now();
    let mut query = String::new();
    loop {
        println!("Type your search query: ");
        match std::io::stdin().read_line(&mut query) {
            Ok(_) => {
                log::debug!("Query     {:?}", query);
                let query = model.encode_single(&query);
                log::debug!("Embedding {:?}", query);
                // let v = vec![val; dim];
                let t = Instant::now();
                let hits = hnsw.search(&index, &query, 10, None)
                    .map_err(|err| error.pass_with("Search error", err.to_string()))?;
                let elapsed = t.elapsed();
                log::debug!("Elapsed {:?}", elapsed);
                log::debug!("Search hits [{}]:", hits.len());
                for hit in hits {
                    log::debug!("\t {:?}", hit);
                }
            }
            Err(err) => log::warn!("{dbg} | Can't read query, error: \n\t{:?}", err),
        }
    }
}

fn embedding(src_path: &str, index_path: &str, model: &StaticModel, hnsw: &Hnsw<String, L2>, index: &InMemoryVectorStore<f32>) -> Result<(), Error> {
    let dbg = Dbg::own("embedding");
    let error = Error::new("", &dbg);
    let path = PathBuf::from(src_path);
    if !path.is_dir() {
        log::warn!("{dbg} | Can't update from a single file '{}', specify the folder", path.display());
    }
    match std::fs::read_dir(&path) {
        Ok(dir) => {
            let t = Instant::now();
            let mut transformed = 0;
            for path in dir {
                if let Ok(path) = path {
                    let path = path.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            let text = match ext.to_str() {
                                Some(ext) => match ext {
                                    "txt" => read_txt(path),
                                    "pdf" => read_pdf(path),
                                    _ => Err(error.err(format!("File format '{}' - isn't supported", ext))),
                                }
                                None => Err(error.err(format!("Wrong file extension '{:?}' in {}", ext, path.display()))),
                            };
                            match text {
                                Ok(text) => {
                                    let key = path.to_str().unwrap();
                                    // Generate embeddings with the default batch size, 256
                                    let embedding = model.encode_single(&doc);
                                    log::debug!("{dbg} | Embedding length: {}", embedding.len()); // -> Embeddings length: 4
                                    hnsw.insert(index, key.to_owned(), &embedding)
                                        .map_err(|err| error.pass(err.to_string()))?;
                                    transformed += 1;
                                }
                                Err(err) => {
                                    log::warn!("Can't read pdf {}, error: {:?}", path.display(), err);
                                }
                            }
                        }
                    }
                }
            }
            if transformed > 0 {
                log::debug!("{dbg} | Embedded {} chunks in: {:?}", transformed, t.elapsed());
                let f = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(&path);
                match f {
                    Ok(mut f) => {
                        index
                            .save_to(&mut f, index.max_nodes())
                            .map_err(|err| error.pass_with("Can't store Index", err.to_string()))
                    }
                    Err(err) => Err(error.pass_with(format!("Can't store index into '{}'", index_path), err.to_string())),
                }
            } else {
                log::warn!("{dbg} | Nothing embedded");
                Ok(())
            }
        }
        Err(err) => {
            Err(error.pass_with(format!("Can't read path '{}'", path.display()), err.to_string()))
        }
    }
}
///
/// Reads TXT into String
fn read_txt<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let path = path.as_ref();
    let dbg = Dbg::own("read_txt");
    let error = Error::new("", &dbg);
    let mut doc = String::new();
    let mut f = OpenOptions::new().read(true).open(&path)
        .map_err(|err| error.pass_with(format!("Can't open '{}'", path.display()), err.to_string()))?;
    f.read_to_string(&mut doc)
        .map_err(|err| error.pass_with(format!("Can't read from '{}'", path.display()), err.to_string()))?;
    Ok(doc)
}
    ///
/// Reads PDF into String
fn read_pdf<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let path = path.as_ref();
    let dbg = Dbg::own("read_pdf");
    let error = Error::new("", &dbg);
    let doc = Document::load(path)
        .map_err(|err| error.pass_with(format!("Can't read '{}'", path.display()), err.to_string()))?;
    match doc.is_encrypted() {
        true => Err(error.err(format!("Can't read encripted pdf '{}'", path.display()))),
        false => {
            let pages = doc.get_pages();
            let page_numbers: Vec<u32> = pages.keys().cloned().collect();
            doc.extract_text(&page_numbers)
                .map_err(|err| error.pass_with(format!("Can't extract text from '{}'", path.display()), err.to_string()))
        }
    }
}