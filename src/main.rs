#[allow(dead_code)]
#[allow(unused_imports)]
use notify::{RecursiveMode, Watcher, EventKind};
use notify_debouncer_full::new_debouncer;
use std::{collections::HashMap, os::unix::fs::MetadataExt, path::Path, time::Duration};
// use std::fs;
use std::error::Error;
// use std::time::SystemTime;
extern crate chrono;
// use chrono::Local;
use log::error;
use log::info;
// use log::warn;
// use log::{debug, LevelFilter};
use csv::Reader;


#[derive(Debug, serde::Deserialize)]
struct Record {
    sno : u16,
    source_folder: String,
    dest_folder: String,
    file_name:String,
    base_name:String,
}

const DATE_FORMAT_STR: &'static str = "[%Y-%m-%d][%H:%M:%S]";

fn main() -> Result<(),Box<dyn Error>> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let whitelisted = read_csv_whitelist();
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    info!("Watching {path}");

    if let Err(error) = watch(path) {
        error!("Error: {error:?}");
    }
    Ok(())
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
        // info!("{}","\nLooping results.");
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
            Err(errors) => errors.iter().for_each(|error| error!("{error:?}")),
        }
        info!("{}","waiting for the Next Event..");
    }

    Ok(())
}

fn act_emitted_path(dic:HashMap<&str,EventKind>)->Result<(), Box<dyn Error>>{
    for kv in dic{
        let file_exists = Path::new(kv.0).try_exists().unwrap();
        if !file_exists {
            continue;
        }
        // let date = Local::now();
        if kv.1.is_remove(){
            info!("File Removed: {}", kv.0);
            // records
        }else{
            // info!("{}| emitted",kv.0);
            _ = get_path_match(kv.0);
        }
    }
    
    Ok(())
}

fn get_path_match(e_path:&str)->Result<(), Box<dyn Error>>{

    let c_path = std::path::Path::new(e_path);
    let parent = c_path.parent().unwrap();
    let filename = c_path.file_name().unwrap();
    let ext = c_path.extension().unwrap();
    let fnwoext = c_path.metadata().unwrap();

    info!("{:?}|{}|{}--> {}",parent,filename.to_str().unwrap(),ext.to_str().unwrap(), fnwoext.size());

    // let mut rdr = csv::Reader::from_path("config.csv")?;
    // for result in rdr.deserialize(){
    //     let mut record: Record = result?;
    //     if record.source_path.starts_with(parent.to_str().unwrap()){

    //     }
    // }

    Ok(())
}

fn read_csv_whitelist() -> Result<Vec<Record>, Box<dyn Error>> {
    println!("{}","into reading the csv");
    let mut whitelisted:Vec<Record> = vec![];
    let mut rdr = Reader::from_path("foo.csv")?;
    let mut iter = rdr.records();
    loop{
        if let Some(result) = iter.next(){
            let record = result?;
            let row: Record = record.deserialize(None)?;
            // println!("{:?}----> {:?}", record,row);
            // println!("{:?}",row);
            whitelisted.push(row);
        }
        else {
            break;
        }
    }
    Ok(whitelisted)
}