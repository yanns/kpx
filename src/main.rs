use std::io::prelude::*;
use std::path::Path;
use std::{env, io};

use inquire::{Confirm, Password, Select, Text};
use kdbx_rs::{binary::Unlocked, database::Group, CompositeKey, Kdbx};

type Database = Kdbx<Unlocked>;

fn main() {
    let args: Vec<String> = env::args().collect();
    let prg_name = &args[0];
    if args.len() != 2 {
        usage(prg_name);
        return;
    }
    let database_path = &args[1];
    let secret = Password::new("Encryption Key:")
        .without_confirmation()
        .prompt();
    match secret {
        Ok(secret) => {
            let database = open_database(database_path, secret).unwrap();
            handle_database(database);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn usage(prg_name: &str) {
    println!("Usage: {prg_name} <database file>");
}

fn open_database<P: AsRef<Path>>(file_path: P, secret: String) -> Result<Database, kdbx_rs::Error> {
    let kdbx = kdbx_rs::open(file_path)?;
    let key = CompositeKey::from_password(&secret);
    let unlocked = kdbx.unlock(&key)?;
    Ok(unlocked)
}

fn handle_database(database: Database) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Database '{}'", &database.name());
    while handle_group(database.root()) != GroupExitAction::Quit {}
}

const SELECT_A_GROUP: &str = "Select a group";
const SELECT_AN_ENTRY: &str = "Select an entry";
const SEARCH_AN_ENTRY: &str = "Search an entry";
const BACK_TO_PREVIOUS: &str = "Back to previous";
const QUIT: &str = "Quit";

#[derive(PartialEq)]
enum GroupExitAction {
    Quit,
    BackToParent,
}

fn handle_group(group: &Group) -> GroupExitAction {
    loop {
        let entity_count = group.entries().size_hint().0;
        let group_count = group.groups().size_hint().0;
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("Group '{}'", group.name());
        println!("- {} entries", entity_count);
        println!("- {} groups\n", group_count);
        let mut options: Vec<&str> = vec![];
        if group_count > 0 {
            options.push(SELECT_A_GROUP);
        }
        if entity_count > 0 {
            options.push(SELECT_AN_ENTRY);
        }
        options.push(SEARCH_AN_ENTRY);
        options.push(BACK_TO_PREVIOUS);
        options.push(QUIT);

        let ans = Select::new("Next action?", options).prompt().unwrap();
        match ans {
            SELECT_A_GROUP => {
                let groups = group.groups().map(|g| g.name()).collect::<Vec<_>>();
                if let Ok(selected_group_name) = Select::new("\nSelect a group", groups).prompt() {
                    let selected_group = group
                        .find_group(|g| g.name() == selected_group_name)
                        .unwrap();
                    if handle_group(selected_group) == GroupExitAction::Quit {
                        return GroupExitAction::Quit;
                    }
                }
            }
            SELECT_AN_ENTRY => {
                let entries = group
                    .entries()
                    .map(|e| e.title().unwrap_or_default())
                    .collect::<Vec<_>>();
                if let Ok(selected_entry_name) = Select::new("\nSelect an entry", entries).prompt()
                {
                    if let Some(selected_entry) =
                        group.find_entry(|e| e.title().unwrap_or_default() == selected_entry_name)
                    {
                        println!("- title: {}", selected_entry.title().unwrap_or_default());
                        println!(
                            "- username: {}",
                            selected_entry.username().unwrap_or_default()
                        );
                        println!(
                            "- password: {}",
                            selected_entry.password().unwrap_or_default()
                        );

                        press_key();
                    }
                }
            }
            SEARCH_AN_ENTRY => {
                if let Ok(search_term) = Text::new("Search term: ").prompt() {
                    let search_term = search_term.to_ascii_lowercase();
                    group.recursive_entries().for_each(|entry| {
                        if entry
                            .title()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains(&search_term)
                        {
                            println!("\ntitle     : {}", entry.title().unwrap_or_default());
                            println!("- username: {}", entry.username().unwrap_or_default());
                            println!("- password: {}", entry.password().unwrap_or_default());
                            println!("- url     : {}", entry.url().unwrap_or_default());
                        }
                    });
                    press_key();
                }
            }
            QUIT => {
                if let Ok(confirmation) = Confirm::new("Do you want to quit?")
                    .with_default(false)
                    .prompt()
                {
                    if confirmation {
                        return GroupExitAction::Quit;
                    }
                }
            }
            _ => return GroupExitAction::BackToParent,
        }
    }
}

fn press_key() {
    println!("\nPress any key to continue...");
    let mut stdin = io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
}
