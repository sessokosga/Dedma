mod data_access;

use anyhow::Ok;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    env, fmt,
    fs::{self, File},
    io::Write,
    process::Command,
    vec,
};

enum CommitSource {
    File(String),
    Git,
}

#[derive(PartialEq)]
enum ExecutionMode {
    Help,
    Execute,
}

#[derive(Debug, PartialEq)]
struct ParsedLine {
    kind: String,
    title: String,
    content: String,
    hash: String,
}

pub struct Config {
    output: String,
    source: CommitSource,
    tag: String,
    mode: ExecutionMode,
}

impl Config {
    pub fn build(args: &[String]) -> Config {
        let mut output = String::from("whats_new.md");
        let mut source = CommitSource::Git;
        let tag = String::from("tag");
        if args.contains(&String::from("--help")) {
            return Config {
                output,
                source,
                tag,
                mode: ExecutionMode::Help,
            };
        }
        if args.len() >= 3 {
            source = CommitSource::File(args[1].clone());
            output = args[2].clone();
        } else if args.len() >= 2 {
            output = args[1].clone();
        }
        // println!("{output}");
        Config {
            output,
            source,
            tag,
            mode: ExecutionMode::Execute,
        }
    }
}

fn get_tag() -> anyhow::Result<(String, String)> {
    let tags = Command::new("git")
        .arg("tag")
        .arg("--sort=-v:refname")
        .output()?;

    let tags = String::from_utf8_lossy(&tags.stdout).trim().to_string();
    if tags.len() <= 0 {
        return Ok(("no_tag".to_string(), "no_tag".to_string()));
    }

    let mut vers: Vec<String> = vec![];

    let max = if tags.lines().count() >= 2 { 2 } else { 1 };

    for tag in tags.lines() {
        if vers.len() >= max {
            break;
        } else {
            vers.push(tag.to_string());
        }
    }
    if max == 2 {
        Ok((vers[1].clone(), vers[0].clone()))
    } else {
        Ok((vers[0].clone(), vers[0].clone()))
    }
}

fn read_from_git() -> anyhow::Result<String> {
    let tags = get_tag()?;

    let commits;
    // print!("git log ");
    if tags.0 != tags.1 {
        let args = format!("{}..{}", tags.0, tags.1);
        commits = Command::new("git")
            .arg("log")
            .arg(&args)
            .arg("--format=%s :%H")
            .output()?;

        // print!("{} ",&args);
    } else {
        commits = Command::new("git")
            .arg("log")
            .arg("--format=%s :%H")
            .output()?;
    }
    // println!("--format=\"%s :%H\"");

    Ok(String::from_utf8_lossy(&commits.stdout).to_string())
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    if config.mode == ExecutionMode::Help {
        if env::var("LANG_FR").is_ok() {
            println!("Dedma v0.1.2
Générateur de notes de versions (Release notes)
Un outil qui converti vos derniers commit en notes de version.

La note est générée en anglais par défaut.

Pour générer une note en français, ajouter `LANG_FR` dans les variables d'environnement.
Pour les systèmes basés sur Unix
    LANG_FR=1 

Pour Windows PowerShell
    $Env:LANG_FR=1

Générer des notes à partir des commit dans un fichier
    dedma fichier_d_entree fichier_de_sortie

Générer les notes à partir des commit de Git
    dedma fichier_de_sortie
    
ou 

    dedma

pour générer les notes dans le fichier `whats_new.md`

Structure de commit idéale
    type (titre): contenu

Le `titre` et le `contenu` peuvent être ce que vous voulez.

Voici une liste des types supportés actuellement, classés par ordre d'apparence dans la note générées.

 ______________________________________
    
|   type   | nom complet               |
| -------- | ------------------------- |
|   feat   | Nouvelles fonctionnalités |
|   fix    | Correction d'erreur       |
|  chore   | Chore                     |
| refactor | Refactoring               |
|   docs   | Documentation             |
|  style   | Style de Code             |
|   test   | Test                      |
|   perf   | Performances              |
|    ci    | Déploiements Continue     |
|  build   | Système de Build          |
|  revert  | Annulations               |
|  update  | Mise à jour               |
 --------------------------------------");
        }else{
            println!("Dedma v0.1.2
Release notes generator
A Command Line Interface (CLI) that generates release notes from your latest commits. 

The release notes are generated in english by default.

To generate the release notes in french add `LANG_FR` to the environment variables
For systems based on Unix
    LANG_FR=1 

For Windows PowerShell
    $Env:LANG_FR=1

Get the commits from a file
    dedma input_file output file

Get them directly from git
    dedma output_file
    
        or 

    dedma

to generate the notes in the file `whats_new.md`

Ideal commit structure
    kind (title): content
For `title` and `content` you can put whatever you want.  
Here are the supported `kind` right now in the order of appearance in the generated notes

  ______________________________________

|   kind   | full name                   |
| :------: | --------------------------- |
|   feat   | New features                |
|   fix    | Bug fix                     |
|  chore   | Chore                       |
| refactor | Refactoring                 |
|   docs   | Documentation               |
|  style   | Code Style                  |
|   test   | Test                        |
|   perf   | Performances                |
|    ci    | Continuous Integration (CI) |
|  build   | Build System                |
|  revert  | Reverts                     |
|  update  | Updates                     |
  ______________________________________
            ");
        }
        return Ok(());
    }

    let contents;
    let tag;
    if let CommitSource::File(file) = config.source {
        // println!("Gathering commits from '{file}...'");
        contents = fs::read_to_string(file)?;
        tag = config.tag;
    } else {
        // println!("Gathering commits from Git repository...");
        contents = read_from_git()?;
        let tagi = get_tag()?;
        tag = tagi.1;
    }
    let size: u64 = contents.lines().count().try_into().unwrap();
    if env::var("LANG_FR").is_ok() {
        println!("Generation de {size} notes dans '{}'", config.output);
    } else {
        println!("Generating {size} notes in '{}'", config.output);
    }
    let progress = get_progress_bar(size * 3);

    // Parsing the commits
    let parsed_lines = split_all(&contents, Some(&progress));
    // println!("{} commits found", parsed_lines.len());

    // Recording them to the database
    let pool = data_access::connect().await?;
    let _ = data_access::record_commits(&tag, &pool, parsed_lines, Some(&progress)).await?;
    // println!("{} new lines recorded", line_recorded);
    // Writing the release note
    // println!("Writing the release note in '{}'...", config.output);
    let notes = generate_release_notes(&tag, &pool, Some(&progress)).await?;
    // println!("{}",notes);
    let _ = write_release_note(&config.output, notes)?;
    progress.finish_with_message("Done");

    Ok(())
}

fn split_all(contents: &str, progress: Option<&ProgressBar>) -> Vec<ParsedLine> {
    let mut res: Vec<ParsedLine> = Vec::new();
    for line in contents.lines() {
        res.push(split_one(line.trim()));
        if let Some(p) = progress {
            p.inc(1);
        }
    }
    res
}

fn split_one(line: &str) -> ParsedLine {
    let mut kind = String::from("other");
    let mut title = String::from("other");
    let content;
    let hash;
    let parts: Vec<&str> = line.split(':').collect();

    if parts.len() >= 3 {
        if parts[0].contains('(') {
            let sc: Vec<&str> = parts[0].split('(').collect();
            kind = sc[0].to_lowercase().trim().to_string();
            title = sc[1].replace(')', " ").trim().to_lowercase().to_string();
        } else {
            kind = parts[0].trim().to_lowercase().to_string();
        }
        content = parts[1].trim().to_string();
        hash = parts[2].trim().to_string();
    } else {
        content = parts[0].trim().to_string();
        hash = parts[1].trim().to_string();
    }

    ParsedLine {
        kind,
        title,
        content,
        hash,
    }
}

fn get_progress_bar(size: u64) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}]", // "{spinner:.green} [{elapsed_precise}] [{bar:60.cyan/blue}] {pos}/{len}"
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );
    pb
}

fn beautify_kind(kind: &str) -> anyhow::Result<&str> {
    let result;
    let mut kinds: HashMap<&str, &str> = HashMap::from([
        ("feat", "New features"),
        ("fix", "Bug fix"),
        ("chore", "Chore"),
        ("refactor", "Refactoring"),
        ("docs", "Documentation"),
        ("style", "Code Style"),
        ("test", "Test"),
        ("perf", "Performances"),
        ("ci", "Continuous Integration (CI)"),
        ("build", "Build System"),
        ("revert", "Reverts"),
        ("update", "Updates"),
        ("other", ""),
    ]);

    if env::var("LANG_FR").is_ok() {
        kinds = HashMap::from([
            ("feat", "Nouvelles fonctionnalités"),
            ("fix", "Correction d'erreur"),
            ("chore", "Chore"),
            ("refactor", "Refactoring"),
            ("docs", "Documentation"),
            ("style", "Style de Code"),
            ("test", "Test"),
            ("perf", "Performances"),
            ("ci", "Déploiement Continue"),
            ("build", " Système de Build"),
            ("revert", "Annulations"),
            ("update", "Mise à jour"),
            ("other", ""),
        ]);
    }

    if kinds.contains_key(kind) {
        result = kinds[kind];
    } else {
        panic!("kind {kind} not found");
    }
    Ok(result)
}

fn beautify_title(title: &str) -> String {
    let result;
    if title.len() <= 3 {
        result = title.to_uppercase();
    } else {
        let cap = title.to_uppercase();
        let mut res = String::from(&cap[0..1].to_string());
        res.push_str(&title[1..]);
        result = res;
    }
    result
}

async fn generate_release_notes(
    tag: &str,
    pool: &SqlitePool,
    progress: Option<&ProgressBar>,
) -> anyhow::Result<String> {
    let order = vec![
        "other".to_string(),
        "feat".to_string(),
        "fix".to_string(),
        "update".to_string(),
        "chore".to_string(),
        "refactor".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "test".to_string(),
        "perf".to_string(),
        "ci".to_string(),
        "build".to_string(),
        "revert".to_string(),
    ];
    let mut notes = String::new();
    let ki = data_access::get_kinds(tag, &pool).await?;
    let mut kinds: Vec<String> = vec![];
    for k in ki {
        kinds.push(k.kind);
    }

    for kind in order {
        if !kinds.contains(&kind) {
            continue;
        }
        notes.push_str(&format!("# {}\n", beautify_kind(&kind)?));

        let titles = data_access::get_titles(tag, &kind, &pool).await?;
        for title in &titles {
            if title.title != String::from("other") {
                notes.push_str(&format!("## {}\n", beautify_title(&title.title)));
            }

            let commits = data_access::get_commits(tag, title, &pool).await?;
            for commit in &commits {
                notes.push_str(&format!("- {}\n", commit.content));
                if let Some(p) = progress {
                    p.inc(1);
                }
            }
        }
    }

    Ok(notes)
}

fn write_release_note(file_path: &str, notes: String) -> anyhow::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_fmt(format_args!("{}", notes))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::result::Result::Ok;
    use tokio;

    #[test]
    fn split_one_work() {
        let contents =
            "feat (Reward): Added one more reward :13883a342dfe858a234d5366a855b49ddc0c534b";
        let pars = ParsedLine {
            kind: String::from("feat"),
            title: String::from("reward"),
            content: String::from("Added one more reward"),
            hash: String::from("13883a342dfe858a234d5366a855b49ddc0c534b"),
        };
        assert_eq!(pars, split_one(contents));
    }

    #[test]
    fn split_all_work() {
        let contents = "\
        feat (Reward): Added one more reward :13883a342dfe858a234d5366a855b49ddc0c534b
        feat (Reward): Added two more rewards :dd187eebf6321df5b541185dd0fd110b1b384712
        update: Added more balance to the game :9f0b66d57b97a33333681128f70396db7c2b3f53
        feat (Tank): added one tank type :06b9582c4a3a27a27e3a90c4444d8cc40ddf17e8
        fix (ci): fixed release notes path :478faab0a38cc5eb15b36915981ed538005dc9fb";
        let res = vec![
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("reward"),
                content: String::from("Added one more reward"),
                hash: String::from("13883a342dfe858a234d5366a855b49ddc0c534b"),
            },
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("reward"),
                content: String::from("Added two more rewards"),
                hash: String::from("dd187eebf6321df5b541185dd0fd110b1b384712"),
            },
            ParsedLine {
                kind: String::from("update"),
                title: String::from("other"),
                content: String::from("Added more balance to the game"),
                hash: String::from("9f0b66d57b97a33333681128f70396db7c2b3f53"),
            },
            ParsedLine {
                kind: String::from("feat"),
                title: String::from("tank"),
                content: String::from("added one tank type"),
                hash: String::from("06b9582c4a3a27a27e3a90c4444d8cc40ddf17e8"),
            },
            ParsedLine {
                kind: String::from("fix"),
                title: String::from("ci"),
                content: String::from("fixed release notes path"),
                hash: String::from("478faab0a38cc5eb15b36915981ed538005dc9fb"),
            },
        ];

        assert_eq!(res, split_all(contents, None))
    }

    async fn run_test(contents: &str) -> anyhow::Result<String> {
        let tag = "tag";
        let result;
        let parsed_lines = split_all(&contents, None);
        let pool = data_access::connect_test().await?;
        let _ = data_access::record_commits(tag, &pool, parsed_lines, None).await?;
        result = generate_release_notes(tag, &pool, None).await?;
        Ok(result)
    }

    #[tokio::test]
    // #[ignore = "need to find a way to run async tests"]
    async fn release_notes() {
        let contents = fs::read_to_string("./tests/logs.txt").expect("check input file");

        let notes = fs::read_to_string("./tests/parsed.md").expect("check output file");

        let res = run_test(&contents).await;
        let result;
        match res {
            Ok(res) => result = res,
            Err(error) => {
                println!("error while testing {error}");
                result = String::new()
            }
        }

        assert_eq!(result, notes)
    }
}
