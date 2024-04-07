use std::{env, fs, path::Path};

use anyhow::bail;
use clap::{Parser, Subcommand};
use colorz::Colorize;

mod lockfile;
mod pathutil;
mod styles;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(disable_help_subcommand = true)]
#[command(styles=styles::usage_style())]
struct Cli {
    /// The dir to link the packages to
    #[arg(short, long, value_name = "DIR", default_value = "~")]
    target: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ///  Link packages to the target dir
    Link {
        packages: Vec<String>,

        /// Overwrite existing symbolic links
        #[clap(long, short)]
        force: bool,
    },

    /// Unlink the specified packages
    Unlink {
        packages: Vec<String>,

        /// Force deletion of symbolic links
        #[clap(long, short)]
        force: bool,
    },

    /// List the linked packages
    List {},

    /// Delete all symlinks from target dir
    Prune {},
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        // Link packages to the target dir
        Some(Commands::Link { packages, force }) => {
            if packages.is_empty() {
                bail!("No packages specified")
            }

            // Get the packages to link
            let pkgs: Vec<String> = if packages.contains(&String::from("*")) {
                get_packages()
            } else {
                packages.clone()
            };

            // Check if the target dir exists
            let target_dir = match cli.target.clone() {
                Some(target) if target != "~" => target,
                _ => dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
            };
            if !Path::new(&target_dir).is_dir() {
                bail!("Target dir does not exist: {}", target_dir);
            }
            let target_dir = Path::new(&target_dir);

            if link_packages(target_dir, &pkgs, force.to_owned()).is_err() {
                bail!("Failed to link packages")
            }
            Ok(())
        }

        // Unlink the specified packages
        Some(Commands::Unlink { packages, force }) => {
            if packages.is_empty() {
                bail!("No packages specified")
            }
            if let Err(e) = unlink_packages(packages, force) {
                eprintln!("{}", e);
            }
            Ok(())
        }

        // List the linked packages
        Some(Commands::List {}) => {
            let packages = lockfile::read_snowlock()?;
            packages.list_symlinks();
            Ok(())
        }

        // Delete all symlinks from target dir
        Some(Commands::Prune {}) => {
            let mut packages = lockfile::read_snowlock()?;
            packages
                .get_packages()
                .iter()
                .for_each(|p| match fs::remove_file(&p.symlink) {
                    Ok(_) => {
                        println!("{} {}", "Removed".red(), p.symlink);
                    }
                    Err(e) => {
                        eprintln!("Failed to remove symlink: {}", e);
                    }
                });

            // Remove lockfile
            lockfile::delete_lockfile()?;
            Ok(())
        }

        // If no command is specified, print the help message
        None => Ok(()),
    }
}

/// Get all the packages in the current dir
fn get_packages() -> Vec<String> {
    fs::read_dir(".")
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_type()
                        .ok()
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false)
                })
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| !name.starts_with('*'))
                .collect()
        })
        .unwrap_or_else(|_| vec![])
}

/// Link all of the packages
fn link_packages(target_dir: &Path, pkgs: &[String], force: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    pkgs.iter().try_for_each(|pkg| {
        let dir_path = current_dir.join(pkg);
        let pkg_dir = dir_path.to_str().unwrap();
        link_package(target_dir, pkg_dir, &dir_path, force)
    })?;
    Ok(())
}

/// Link the package
fn link_package(
    target_dir: &Path,
    pkg_dir: &str,
    dir_path: &Path,
    force: bool,
) -> anyhow::Result<()> {
    // extract package name from pkg_dir
    let pkg_name = pkg_dir.split('/').last().unwrap();

    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let path1 = entry?.path();
            let path2 = path1.clone();
            if path1.is_dir() {
                let target = target_dir.join(path1.strip_prefix(pkg_dir).unwrap());

                // If the target file is a symbolic link
                if target.is_symlink() {
                    if !force {
                        continue;
                    }
                    // if force is specified, remove the existing symbolic link
                    // and continue the process
                    fs::remove_file(&target)?;
                }

                // If the target file is a directory,
                // recursively search the next level of directories
                if target.is_dir() {
                    link_package(target_dir, pkg_dir, &path1, force)?;
                    continue;
                }

                // if target does not exist, create the symbolic link
                if !target.exists() {
                    if let Err(e) = make_symlink(&path1, &target, pkg_name) {
                        eprintln!("{}", e);
                    }
                    continue;
                }
            }

            if path2.is_file() {
                let target = target_dir.join(path1.strip_prefix(pkg_dir).unwrap());
                if target.exists() && (!force || fs::remove_file(&target).is_err()) {
                    continue;
                }
                if let Err(e) = make_symlink(&path1, &target, pkg_name) {
                    eprintln!("{}", e);
                }
                continue;
            }
        }
    }
    Ok(())
}

/// Unlink the specified packages
fn unlink_packages(packages: &[String], force: &bool) -> anyhow::Result<()> {
    if packages.is_empty() {
        bail!("No packages specified");
    }

    let mut linked_packages = lockfile::read_snowlock()?;
    for pkg in packages {
        if let Some(package) = linked_packages.find_package(pkg) {
            let real_path = pathutil::get_symlink_real_path(&package.symlink).unwrap();
            let is_same_file = pathutil::is_same_file(Path::new(&package.origin), &real_path);

            if *force || matches!(is_same_file, Ok(true) | Err(_)) {
                remove_symlink(&package.symlink)?;
                println!("{} {}", "Unlinking".red(), &package.symlink);
                linked_packages.remove_package(pkg);
            } else {
                bail!(
                    "The symlink is not pointing to the package: {}",
                    &package.symlink
                );
            }
        }
    }

    lockfile::write_snowlock(&linked_packages)?;
    Ok(())
}

/// Create symlink and update lockfile
fn make_symlink(origin: &Path, target: &Path, pkg_name: &str) -> anyhow::Result<()> {
    let origin_str = origin.to_str().unwrap();
    let symlink_str = target.to_str().unwrap();
    if let Ok(relpath) = pathutil::relative(origin_str, symlink_str) {
        std::os::unix::fs::symlink(relpath, target)?;
        println!("{} {}", "Symlinked".green(), symlink_str);

        let mut packages = lockfile::read_snowlock()?;
        packages.add_package(pkg_name, origin_str, symlink_str);
        lockfile::write_snowlock(&packages)?;
    }
    Ok(())
}

/// Remove a symlink file
fn remove_symlink(symlink_path: &str) -> anyhow::Result<()> {
    fs::remove_file(symlink_path)?;
    Ok(())
}
