use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    io::{Error, ErrorKind},
    path::Path,
};

pub fn write_files(
    target_path: &Path,
    lib_path: &Path,
    input_provider: &dyn InputProvider,
    year: u16,
    day: u8,
    force: bool,
) -> Result<(), Error> {
    let lib_path = lib_path
        .to_str()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Can't convert lib path to str"))?;
    let variables: HashMap<&str, &dyn Display> = HashMap::from([
        ("AOC_PATH", &lib_path as &dyn Display),
        ("YEAR", &year),
        ("DAY", &day),
    ]);

    if target_path.exists() && !force {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "The target directory '{}' exists. Use the --force option to overwrite.",
                target_path.to_string_lossy()
            ),
        ));
    }

    let src_path = target_path.join("src");
    println!("Creating directories for {}", src_path.to_string_lossy());
    fs::create_dir_all(src_path.as_path())?;

    // input file from web
    write_file(
        input_provider.load_input(year, day)?.as_str(),
        &HashMap::new(),
        target_path.join("input.txt").as_path(),
    )?;

    // other files from templates
    write_file(
        GITIGNORE,
        &variables,
        target_path.join(".gitignore").as_path(),
    )?;
    write_file(
        CARGO_TOML,
        &variables,
        target_path.join("Cargo.toml").as_path(),
    )?;
    write_file(MAIN_RS, &variables, src_path.join("main.rs").as_path())?;
    write_file(LIB_RS, &variables, src_path.join("lib.rs").as_path())?;

    Ok(())
}

pub trait InputProvider {
    fn load_input(&self, year: u16, day: u8) -> Result<String, Error>;
}

#[derive(Debug)]
pub struct InputLoader<'a> {
    pub session: &'a str,
}

impl<'a> InputProvider for InputLoader<'a> {
    fn load_input(&self, year: u16, day: u8) -> Result<String, Error> {
        reqwest::blocking::Client::new()
            .get(format!("https://adventofcode.com/{}/day/{}/input", year, day).as_str())
            .header("Cookie", format!("session={}", self.session))
            .send()
            .map_err(|err| Error::new(ErrorKind::Other, err))?
            .text()
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

fn write_file(
    template: &str,
    variables: &HashMap<&str, &dyn Display>,
    path: &Path,
) -> Result<(), Error> {
    let mut content = template.to_string();
    for (&name, &value) in variables {
        content = content.replace(
            format!("{{{}}}", name).as_str(),
            format!("{}", value).as_str(),
        );
    }

    println!("Writing file {} ...", path.to_string_lossy());
    fs::write(path, content)?;

    Ok(())
}

const MAIN_RS: &str = r###"use mr_kaffee_aoc::{err::PuzzleError, GenericPuzzle};
use mr_kaffee_{YEAR}_{DAY}::*;

fn main() -> Result<(), PuzzleError> {
    puzzle().solve_report_err()
}
"###;

const LIB_RS: &str = r###"use mr_kaffee_aoc::{Puzzle, Star};
use input::*;

/// the puzzle
pub fn puzzle() -> Puzzle<PuzzleData, usize, usize, usize, usize> {
    Puzzle {
        year: {YEAR},
        day: {DAY},
        input: include_str!("../input.txt"),
        star1: Some(Star {
            name: "Star 1",
            f: &star_1,
            exp: None,
        }),
        star2: Some(Star {
            name: "Star 2",
            f: &star_2,
            exp: None,
        }),
    }
}

pub mod input {
    use core::fmt;
    use std::{convert::Infallible, str::FromStr};

    #[derive(Debug)]
    pub struct PuzzleData {
        input: String,
    }

    impl FromStr for PuzzleData {
        type Err = Infallible;

        /// parse the puzzle input
        ///
        /// # Examples
        /// ```
        /// # use mr_kaffee_{YEAR}_{DAY}::input::*;
        /// let data = "Hello World".parse::<PuzzleData>().unwrap();
        /// assert_eq!("Hello World", format!("{}", data));
        /// ```    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(PuzzleData { input: s.into() })
        }
    }

    impl fmt::Display for PuzzleData {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.input.fmt(f)
        }
    }
}

pub fn star_1(data: &PuzzleData) -> usize {
    println!("{}", data);
    0
}

pub fn star_2(data: &PuzzleData) -> usize {
    println!("{:?}", data);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mr_kaffee_aoc::err::PuzzleError;

    const CONTENT: &str = r#"Hello World!
Freedom"#;

    #[test]
    pub fn test_puzzle_data_from_str() -> Result<(), PuzzleError> {
        let data = CONTENT.parse::<PuzzleData>()?;
        assert_eq!(format!("{}", data), CONTENT.to_string());
        Ok(())
    }
}
"###;

const CARGO_TOML: &str = r###"[package]
name = "mr-kaffee-{YEAR}-{DAY}"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]

mr-kaffee-aoc = { path = "{AOC_PATH}" }
"###;

const GITIGNORE: &str = r###"**/target
"###;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_dir_all;
    use std::process::Command;
    use std::str;

    struct TestInputProvider {}

    impl InputProvider for TestInputProvider {
        fn load_input(&self, year: u16, day: u8) -> Result<String, Error> {
            Ok(format!("Test input for {}/{}\n", year, day))
        }
    }

    /// create test files and execute tests and program with cargo
    #[test]
    pub fn test_write_files() {
        let target_path = Path::new("target/test_write_file");
        let lib_path = Path::new("../../../aoc");
        let input_provider = TestInputProvider {};
        let year = 2022;
        let day = 25;
        let force = true;

        // write files
        let result = write_files(target_path, lib_path, &input_provider, year, day, force);
        assert!(matches!(result, Ok(_)));

        // run tests using `cargo test`
        let result = Command::new("cargo")
            .arg("test")
            .current_dir(target_path)
            .output();
        assert!(
            matches!(result, Ok(_)),
            "'cargo test' did not execute successful"
        );
        let result = result.unwrap();
        println!(
            "{}",
            str::from_utf8(&result.stdout)
                .expect("Could not convert stdout of 'cargo test' to string")
        );
        assert_eq!(
            result.status.code(),
            Some(0),
            "'cargo test' returned with non-zero status"
        );

        // run program using `cargo run`
        let result = Command::new("cargo")
            .arg("run")
            .current_dir(target_path)
            .output();
        assert!(
            matches!(result, Ok(_)),
            "'cargo run' did not execute successful"
        );
        let result = result.unwrap();
        println!(
            "{}",
            str::from_utf8(&result.stdout)
                .expect("Could not convert stdout of 'cargo run' to string")
        );
        assert_eq!(
            result.status.code(),
            Some(0),
            "'cargo run' returned with non-zero status"
        );

        // clean up, if it fails, 'cargo clean' will do the job
        let _ = remove_dir_all(target_path);
    }
}
