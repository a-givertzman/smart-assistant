mod hnsw;
#[cfg(test)]
mod tests;
use std::{fs::OpenOptions, io::Read, path::{Path, PathBuf}, time::{Duration, Instant}};

use debugging::session::debug_session::{DebugSession, LogLevel};
use hnsw_rs::{hnsw::Hnsw, hnswio::HnswIo, prelude::DistCosine};
use lopdf::Document;
use model2vec_rs::model::StaticModel;
use sal_core::{dbg::Dbg, error::Error};

use crate::hnsw::{Index, Meta};

///
/// Search application entry point
fn main() -> Result<(), Error> {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("lopdf", LogLevel::Info)
        .init();
    let dbg = Dbg::own("main");
    let error = Error::new("", &dbg);
    let args: Vec<String> = std::env::args().collect();

    // Load a model from the Hugging Face Hub or a local path.
    // Arguments: (repo_or_path, hf_token, normalize_embeddings, subfolder_in_repo)
    let model = StaticModel::from_pretrained(
        // "minishlab/potion-base-32M",                     // Model ID from Hugging Face or local path to model directory
        "minishlab/potion-multilingual-128M",
        None,                               // Optional: Hugging Face API token for private models
        None,                           // Optional: bool to override model's default normalization. `None` uses model's config.
        None                            // Optional: subfolder if model files are not at the root of the repo/path
    ).map_err(|err| error.pass_with("Can't load StaticModel", err.to_string()))?;

    //
    // Loading hnsw
    let dim = 256;  // Because of `StaticModel` default DIM
    let nb_elem = 250;      // Temporary buffer size that determines how many candidate neighbors are kept while building or updating the HNSW graph.
    let max_nb_connection = 24;
    let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
    let ef_construction = 400;
    let ef_search = 256;     // controls the width of the search in the lowest level, it must be greater than number of neighbours asked, more then 300 is not efficent
    let search_knbn = 10;
    println!(
        " number of elements to insert {:?} , setting max nb layer to {:?} ef_construction {:?}",
        nb_elem, nb_layer, ef_construction
    );

    //
    // Loading Index
    let path = "./assets/index.json";
    let dump_dir = Path::new("./assets/");
    let dump_name = "dump";
    let mut hnswio = HnswIo::new(dump_dir, dump_name);
    let (mut index, mut hnsw) = match dump_dir.join(dump_name).is_file() {
        true => {
            let hnsw = hnswio.load_hnsw()
                .map_err(|err| error.pass_with(format!("Can't load HNSW dump '{}'", path), err.to_string()))?;
            let index = Index::load(path)
                .map_err(|err| error.pass_with(format!("Can't load index from '{}'", path), err.to_string()))?;
            (index, hnsw)
        }
        false => {
            log::warn!("{dbg} | Can't find hnsw dump '{}'", dump_dir.join(dump_name).display());
            (
                Index::new(path),
                Hnsw::<f32, DistCosine>::new(max_nb_connection, nb_elem, nb_layer, ef_construction, DistCosine {})
            )
        }
    };
    hnsw.set_extend_candidates(false);
    //
    hnsw.modify_level_scale(0.25);

    //
    // Checking args, embedding if "--update path" found then embedd new files
    if let (Some(arg), Some(param)) = (args.get(1), args.get(2)) {
        if arg == "--update" && !param.is_empty() {
            embedding(param, dump_dir, dump_name, &model, &hnsw, &mut index)?;
        }
    }

    let t = Instant::now();
    loop {
        let mut query = String::new();
        println!("Type your search query: ");
        match std::io::stdin().read_line(&mut query) {
            Ok(_) => {
                let query = query.trim();
                log::debug!("Query     {:?}", query);
                let query = model.encode_single(&query);
                log::debug!("Query embedding {:?}", query);
                // let v = vec![val; dim];
                let t = Instant::now();
                let hits: Vec<(usize, f32, Option<Meta>)> = hnsw.search(&query, search_knbn, ef_search)
                    .iter().map(|h| {
                        (h.d_id, h.distance, index.get(h.d_id).cloned())
                    }).collect();
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
///
/// Transforms documents from `src_path` into vectors and storing the Index
fn embedding(src_path: &str, dump_dir: &Path, dump_name: &str, model: &StaticModel, hnsw: &Hnsw<f32, DistCosine>, index: &mut Index) -> Result<(), Error> {
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
                                    "txt" => read_txt(&path),
                                    "pdf" => read_pdf(&path),
                                    _ => Err(error.err(format!("File format '{}' - isn't supported", ext))),
                                }
                                None => Err(error.err(format!("Wrong file extension '{:?}' in {}", ext, path.display()))),
                            };
                            match text {
                                Ok(text) => {
                                    let meta = Meta::new(path.to_str().unwrap(), path.to_str().unwrap());
                                    let key = index.insert(meta);
                                    // Generate embeddings with the default batch size, 256
                                    let embedding = model.encode_single(&text);
                                    log::debug!("{dbg} | Embedding length: {}", embedding.len());
                                    hnsw.insert_slice((&embedding, key));
                                    transformed += 1;
                                }
                                Err(err) => {
                                    log::warn!("Can't read file {}, error: {:?}", path.display(), err);
                                }
                            }
                        }
                    }
                }
            }
            if transformed > 0 {
                log::debug!("{dbg} | Embedded {} documents in: {:?}", transformed, t.elapsed());
                index.store()?;
                // hnsw.file_dump(dump_dir, dump_name)
                //     .map(|dump| log::debug!("HNSW Graph stored to {dump}"))
                //     .map_err(|err| error.pass_with("Can't store HNSW Graph", err.to_string()))
                Ok(())
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
            match doc.extract_text(&page_numbers) {
                Ok(text) => {
                    std::fs::write(path.with_extension("txt"), text.as_bytes()).unwrap();
                    Ok(text)
                }
                Err(err) => Err(error.pass_with(format!("Can't extract text from '{}'", path.display()), err.to_string()))
            }
        }
    }
}