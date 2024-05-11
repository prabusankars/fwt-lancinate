use config::Config;
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
use std::sync::Arc;
use std::thread;


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
    let settings = Config::builder()
        .add_source(config::File::with_name("appsettings"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    let csv = &settings.get_string("whitelist_csv").unwrap();
    let whitelisted = read_csv_whitelist(csv);
    let mut path:String = String::from(std::env::current_dir()?.to_str().unwrap());
    if std::env::args().len() >1 {
        path = std::env::args()
        .nth(1)
        .expect("Argument 1: Path to monitor must be provided.");
    }
    // let path = std::env::args()
    //     .nth(1)
    //     .expect("Argument 1: Path to monitor must be provided.");

    info!("Watching {path}");

    if let Err(error) = watch(path,whitelisted.unwrap()) {
        error!("Error: {error:?}");
    }
    Ok(())
}

fn watch<P: AsRef<Path>>(path: P, whitelist:Vec<Record>) -> notify::Result<()> {
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

    let shared_whitelist = Arc::new(whitelist);
    let mut unique_paths:HashMap<String,EventKind> = HashMap::new();
    for result in rx {
        // info!("{}","\nLooping results.");
        match result {
            Ok(events) => {
                for result in events.iter(){
                    // println!("{:?}",result);
                    for i in 0..result.event.paths.len(){
                        let e_str =  String::from(result.event.paths[i].to_str().unwrap()); //.as_path().to_str().unwrap();
                        let e_kind = result.event.kind.clone();
                        unique_paths.insert(e_str, e_kind);
                    }
                    
                }
            },
            Err(errors) => errors.iter().for_each(|error| error!("{error:?}")),
        }
        let child_whitelist = Arc::clone(&shared_whitelist);
        let up  = unique_paths.clone();
        let _ = thread::spawn(move || {
                let _ = act_emitted_path(up,child_whitelist);
        }); 
        // handler.join().unwrap();
        // let _ = act_emitted_path(unique_paths.clone());
        info!("{}","waiting for the Next Event..");
    }

    Ok(())
}

fn act_emitted_path(dic:HashMap<String,EventKind>,whitelist:Arc<Vec<Record>>)->Result<(), Box<dyn Error>>{

    // let get_dest_folder = |pstr:&str,wl:&Arc<Vec<Record>>|{
    //     println!("{},{:?}",pstr,wl);
    // };
    for kv in dic{
        let file_exists = Path::new(kv.0.as_str()).try_exists().unwrap();
        if !file_exists {
            if kv.1.is_remove(){
                info!("File Removed: {}", kv.0);
            }
            continue;
        }
        info!("File Created Or Modified: {}", kv.0);
        _ = get_path_match(kv.0.as_str(),&whitelist);
       
    }
    
    Ok(())
}

fn get_path_match(e_path:&str,wl:&Arc<Vec<Record>>)->Result<(), Box<dyn Error>>{

    let c_path = std::path::Path::new(e_path);
    if c_path.is_dir(){
        return Ok(());
    }
    let parent = c_path.parent().unwrap();
    let filename = c_path.file_name().unwrap();
    let ext = c_path.extension().unwrap();
    let fnwoext = c_path.file_stem().unwrap().to_str().unwrap();//.replace(".{}".format(ext.to_str()),"");

    info!("{:?}|{}|{}--> {:?}",parent,filename.to_str().unwrap(),ext.to_str().unwrap(), fnwoext);

    let matched = wl.iter().filter(|x|{
        e_path.starts_with(&x.source_folder) & fnwoext.starts_with(&x.base_name.split_once('.').iter().next().unwrap().0)
    }).next();

    if ! matched.is_none(){
        // println!("Copy to Location Matched:{:?}",matched);
        let rec = matched.unwrap();
        let  dest_fol =  Path::new(rec.dest_folder.as_str());
        if !dest_fol.exists(){
            info!("{} : folder not exists, created new.",dest_fol.to_str().unwrap());
            std::fs::DirBuilder::new()
            .recursive(true)
            .create(dest_fol).unwrap();
        }
        let status = std::fs::copy(e_path, Path::new(rec.dest_folder.as_str()).join(filename.to_str().unwrap()))?;
        info!("Status of Copy: {:?}",status);

    }

    Ok(())
}

fn read_csv_whitelist(csv_file:&String) -> Result<Vec<Record>, Box<dyn Error>> {
    println!("{}","into reading the csv");
    let mut whitelisted:Vec<Record> = vec![];
    let mut rdr = Reader::from_path(csv_file)?;
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