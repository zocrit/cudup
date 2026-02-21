pub mod check;
pub mod install;
pub mod list;
pub mod local;
pub mod manage;
pub mod uninstall;
pub mod use_version;

pub use check::check;
pub use install::install;
pub use list::list_available_versions;
pub use local::{local_activate, local_write};
pub use manage::{remove, setup};
pub use uninstall::uninstall;
pub use use_version::use_version;

use std::path::Path;

pub fn print_shell_exports(install_dir: &Path) {
    println!("export CUDA_HOME=\"{}\"", install_dir.display());
    println!("export PATH=\"$CUDA_HOME/bin${{PATH:+:$PATH}}\"");
    println!("export LD_LIBRARY_PATH=\"$CUDA_HOME/lib64${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\"");
}
