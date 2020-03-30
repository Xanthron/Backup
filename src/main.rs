use chrono::Utc;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{prelude::*, stdin, stdout, BufRead, BufReader},
};
use users::{get_current_uid, get_user_by_uid};

static COPY_PATH_LIST_FILE: &str = "copy_file_list.txt";
static BACKUP_NAME: &str = "Sicherung";

fn main() {
    assert_eq!(String::from("s: &str"), String::from("s: &str"));
    let user_name = get_user_name();
    // let user_name = get_user_name();
    // let paths = get_copy_paths();
    let mut dirs = Vec::<PathBuf>::new();
    let num: usize;
    clear_terminal();
    loop {
        dirs.clear();
        get_all_drives(format!("/run/media/{}", user_name), &mut dirs);
        if dirs.len() == 0 {
            println!("Es konnten keine eingehängten Dartenträger gefunden werden.");
            println!("Bitte hängen sie einen Datenträger ein und drücken sie 'Enter'.");
        } else {
            if dirs.len() == 1 {
                println!("Es wurde 1 eingehängter Datenträger gefunden:")
            } else {
                println!(
                    "Es konnten {} eingehängte Datenträger gefunden werden:",
                    dirs.len()
                )
            }
            println!("Bitte wählen Sie den zu beschreibenden Datenträger mit der Nummer aus, oder geben sie 'beenden' ein um das Programm zu beenden.");
            for i in 0..dirs.len() {
                if let Some(path) = dirs[i].as_path().file_name() {
                    println!("   {}. {}", i + 1, path.to_string_lossy());
                }
            }
        }
        print!(" > ");
        let input = get_input();
        let text = input.trim_end();
        if text == "beenden" {
            println!("Sicherung wird abgebrochen.");
            return;
        } else if let Ok(value) = text.parse::<usize>() {
            if value > 0 && value <= dirs.len() {
                num = value - 1;
                if let Some(path) = dirs[num].as_path().file_name() {
                    clear_terminal();
                    println!(
                        "Es wird ein Backup auf '{}' erstellt. ",
                        path.to_string_lossy()
                    );
                }
                break;
            }
        }
        clear_terminal();
        print!("'{}' ist keine valide Option. ", text);
    }

    let drive = &dirs[num];
    {
        let mut backups = Vec::<PathBuf>::new();
        get_all_previous_backups(drive, &mut backups);
        if backups.len() > 0 {
            loop {
                if backups.len() == 1 {
                    println!("Es wurde 1 ältere Sicherung gefunden.");
                } else {
                    println!("Es wurden {} ältere Sicherungen gefunden.", backups.len());
                }
                println!("Bitte geben sie eine Zahl zwischen 0 - {} ein um bis zu dieser Zahl ältere Sicherungen zu löschen, oder geben sie 'beenden' ein um das Programm zu beenden.", backups.len());
                for i in 0..backups.len() {
                    println!(
                        "   {}. {}",
                        i + 1,
                        backups[i].as_path().file_name().unwrap().to_string_lossy()
                    );
                }
                print!(" > ");
                let input = get_input();
                let text = input.trim_end();
                if text == "beenden" {
                    println!("Sicherung wird abgebrochen.");
                    return;
                } else if let Ok(value) = text.parse::<usize>() {
                    if value <= backups.len() {
                        if value == 0 {
                            println!("Es werden keine Sicherungen gelöscht");
                        } else {
                            if value == 1 {
                                println!("Es wird 1 Sicherung gelöscht");
                            } else {
                                println!("Es werden {} Sicherungen gelöscht", value);
                            }
                            for i in 0..value {
                                let backup = &backups[i];
                                match std::fs::remove_dir_all(backup) {
                                    Ok(()) => println!(
                                        "'{}' wurde erfolgreich gelöscht.",
                                        backup.as_path().file_name().unwrap().to_string_lossy()
                                    ),
                                    Err(message) => println!(
                                        "Das Löschen von '{}' ist fehlgeschlagen. '{}'",
                                        backup.as_path().file_name().unwrap().to_string_lossy(),
                                        message
                                    ),
                                }
                            }
                        }
                        break;
                    }
                }
                clear_terminal();
                print!("'{}' ist keine valide Option. ", text);
            }
        }
    }

    let mut paths = Vec::<PathBuf>::new();
    get_copy_paths(&mut paths);
    let date_time_now = Utc::now();

    let save_path = format!(
        "{}/{}-{}",
        drive.as_path().to_string_lossy(),
        BACKUP_NAME,
        date_time_now.to_rfc3339().replace(":", "-")
    );
    println!("{}", save_path);
    match std::fs::create_dir(&save_path) {
        Ok(()) => {
            println!("Erstellen des Ordners '{}' war erfolgreich.", &save_path);
        }
        Err(err) => {
            println!(
                "Erstellen des Ordners '{}' ist fehlgeschlagen, '{}'",
                &save_path, err
            );
        }
    }
    for path in paths {
        copy(path, &save_path);
    }
}

fn copy<P: AsRef<Path>, Q: AsRef<Path>>(copy_path: P, save_path: Q) {
    let copy_path = copy_path.as_ref();
    let save_path = save_path.as_ref();

    let copy_path_str = copy_path.to_string_lossy();
    let save_path_str = save_path.to_string_lossy();

    let total_save_path_str = format!("{}{}", save_path_str, copy_path_str);
    if copy_path.is_dir() {
        match std::fs::create_dir_all(&total_save_path_str) {
            Ok(()) => {
                println!(
                    "Erstellen des Ordners '{}' war erfolgreich.",
                    &total_save_path_str
                );
            }
            Err(err) => {
                println!(
                    "Erstellen des Ordners '{}' ist fehlgeschlagen, '{}'",
                    &total_save_path_str, err
                );
            }
        }
        for entry in std::fs::read_dir(copy_path).unwrap() {
            copy(entry.unwrap().path(), save_path);
        }
    } else {
        match std::fs::copy(copy_path, &total_save_path_str) {
            Ok(_) => {
                println!("Kopieren von '{}' war erfolgreich.", copy_path_str);
            }
            Err(err) => {
                println!(
                    "Kopieren von '{}' ist fehlgeschlagen. '{}'",
                    copy_path_str, err
                );
                print!("");
            }
        }
    }
}

fn clear_terminal() {
    print!("\x1B[2J");
    print!("\x1B[1;1H");
}

fn get_user_name() -> String {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    return String::from(user.name().to_string_lossy().as_ref());
}

fn get_copy_paths(paths: &mut Vec<PathBuf>) {
    let file = File::open(COPY_PATH_LIST_FILE);

    let mut path_strings = Vec::<String>::new();
    match file {
        Ok(result) => {
            println!("Folgende Ordner und Dateien werden gesichert");
            let buffer = BufReader::new(result);
            for (_, line) in buffer.lines().enumerate() {
                path_strings.push(line.unwrap());
            }
        }
        Err(_) => {
            println!("Es konnte keine '{}' datei im Instalationsordner gefunden werden. Geben Sie bitte manuel die zu speichernden Orte ein.",COPY_PATH_LIST_FILE);
            print!("  > ");
            for (_, path) in get_input().split_whitespace().enumerate() {
                path_strings.push(String::from(path));
            }
        }
    };
    for path_string in path_strings {
        let path = Path::new(&path_string);
        if path.exists() {
            paths.push(PathBuf::from(path));
        }
    }
}

fn get_input() -> String {
    let mut s = String::new();
    loop {
        let _ = stdout().flush();
        match stdin().read_line(&mut s) {
            Ok(_) => {
                break;
            }
            Err(_) => println!("Keine korrekte Eingabe."),
        };
    }
    return s;
}

fn get_all_drives<P: AsRef<Path>>(path: P, drives: &mut Vec<PathBuf>) {
    if path.as_ref().is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            drives.push(entry.unwrap().path());
        }
    }
}

fn get_all_previous_backups<P: AsRef<Path>>(path: P, backups: &mut Vec<PathBuf>) {
    if path.as_ref().is_dir() {
        let mut dir_entries = Vec::<DirEntry>::new();
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path_buf = entry.path();
            let path = path_buf.as_path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(BACKUP_NAME)
            {
                dir_entries.push(entry);
            }
        }
        dir_entries.sort_by(|v1, v2| {
            v1.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&v2.metadata().unwrap().modified().unwrap())
        });
        for dir_entry in dir_entries {
            backups.push(dir_entry.path());
        }
    }
}
