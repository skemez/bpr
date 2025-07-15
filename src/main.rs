use std::{
    env,
    fs::{self, OpenOptions, create_dir_all},
    path::PathBuf,
};

use clap::Parser;
use sqlite::{Connection, State};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    num_page: u16,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_path = PathBuf::from(&args.path);
    let mut name = match file_path.file_stem() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return Err(anyhow::anyhow!("Error: getting file name")),
    };
    let parent_dir = file_path
        .parent()
        .expect("Error: getting parent directory")
        .to_string_lossy();

    // get file name without numbering
    let reg = regex::Regex::new(r"^(?P<name>.*?)\s*\(\d+\)$").unwrap();
    name = if let Some(cap) = reg.captures(&name) {
        cap["name"].to_owned()
    } else {
        name
    };
    let mut new_entry = false;

    let conn = init_data()?;
    let mut stmt = conn.prepare(format!(
        "select 1 from books where name = '{}' and path = '{}'",
        name, parent_dir
    ))?;
    match stmt.next()? {
        State::Done => {
            new_entry = true;
        }
        State::Row => {}
    };
    let new_name = format!(
        "{}({}).{}",
        name,
        args.num_page,
        file_path.extension().unwrap().to_string_lossy()
    );

    if new_entry {
        let query = format!(
            "insert into books values('{}', '{}', {});",
            name, parent_dir, args.num_page
        );
        conn.execute(query)?;
    } else {
        let query = format!(
            "update books set page = {} where name = '{}' and path = '{}'",
            args.num_page, name, parent_dir
        );
        conn.execute(query)?;
    };
    let new_path = PathBuf::from(parent_dir.into_owned()).join(&new_name);
    if let Err(_) = fs::rename(&args.path, new_path) {
        println!("Error renaming file");
    }
    Ok(())
}
fn init_data() -> anyhow::Result<Connection> {
    let data_dir = if let Ok(dir) = env::var("XDG_DATA_HOME") {
        PathBuf::from(dir)
    } else if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".local/share")
    } else {
        PathBuf::from("/tmp")
    };
    let data_dir = data_dir.join("bpr");
    match create_dir_all(&data_dir) {
        _ => {}
    }
    let data_file = data_dir.join("data.sqlite");
    match OpenOptions::new()
        .create(true)
        .write(false)
        .open(&data_file)
    {
        _ => {}
    }
    // data directory + file guaranteed to exist
    let conn = sqlite::open(&data_file)?;
    let init_query = "
        create table if not exists books (
            name TEXT,
            path TEXT,
            page INTEGER,
            primary key (name, path)
        ); 
    ";
    conn.execute(init_query)?;
    Ok(conn)
}
