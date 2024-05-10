use notify::{RecursiveMode, Watcher, EventKind};
use notify_debouncer_full::new_debouncer;
use std::{path::{Path, PathBuf}, time::Duration, collections::HashMap};
use std::fs;
use std::{error::Error, io, process};

#[derive(Debug, serde::Deserialize)]
struct Record {
    sno : u16,
    source_path: String,
    dest_path: String,
}
/// Example for notify-debouncer-full
fn main() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    println!("Watching {path}");

    if let Err(error) = watch(path) {
        println!("Error: {error:?}");
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Create a new debounced file watcher with a timeout of 2 seconds.
    // The tickrate will be selected automatically, as well as the underlying watch implementation.
    let mut debouncer = new_debouncer(Duration::from_secs(5), Some(Duration::from_secs(5)), tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    // Initialize the file id cache for the same path. This will allow the debouncer to stitch together move events,
    // even if the underlying watch implementation doesn't support it.
    // Without the cache and with some watch implementations,
    // you may receive `move from` and `move to` events instead of one `move both` event.
    debouncer
        .cache()
        .add_root(path.as_ref(), RecursiveMode::Recursive);

    // print all events and errors
    // let somedata  = rx.recv();
    // println!("{:?}",somedata);
    // for result in somedata{
    //     println!("{},{:?}","\n\nLooping results.",result);
    // }
    for result in rx {
        println!("{}","\nLooping results.");
        match result {
            Ok(events) => {
                let mut unique_paths:HashMap<&str,EventKind> = HashMap::new();
                for result in events.iter(){
                    // println!("{:?}",result);
                    for i in 0..result.event.paths.len(){
                        let e_str =  result.event.paths[i].as_path().to_str().unwrap();
                        let e_kind = result.event.kind;
                        unique_paths.insert(e_str, e_kind);
                    }
                    
                }
                let _ = act_emitted_path(unique_paths);
            },
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        }

        println!("{}","waiting for the Next Event..");
    }

    Ok(())
}

fn act_emitted_path(dic:HashMap<&str,EventKind>)->Result<(), Box<dyn Error>>{
    for kv in dic{
        let file_exists = Path::new(kv.0).try_exists().unwrap();
        if !file_exists {
            continue;
        }
        if kv.1.is_remove(){
            println!("{}: This is a remove event not needed to act on..", kv.0);
            // records
        }else{
            println!("{}: use the path to check and act!!!",kv.0);
        }
    }
    
    Ok(())
}

// fn getPathMatch(e_path:&str)->Result<(), Box<dyn Error>>{

//     let c_path = std::path::Path::new(e_path);
//     let parent = c_path.parent().unwrap();
//     let filename = c_path.file_name().unwrap();
//     let ext = c_path.extension().unwrap();

//     let mut rdr = csv::Reader::from_path("config.csv")?;
//     for result in rdr.deserialize(){
//         let mut record: Record = result?;
//         if record.source_path.starts_with(parent.to_str().unwrap()){

//         }
//     }

//     Ok(())
// }