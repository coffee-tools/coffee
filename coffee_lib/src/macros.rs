//! Core macros implemented for coffee.

/// return the Coffee Error
#[macro_export]
macro_rules! error {
    ($($msg:tt)*) => {{
        let msg = format!($($msg)*);
        CoffeeError::new(1, &msg)
    }};
}

/// sh macro is the macro that allow to run a
/// script as a sequence of commands.
#[macro_export]
macro_rules! sh {
    ($root: expr, $script:expr, $verbose:expr) => {
        let script = $script.trim();
        let cmds = script.split("\n"); // Check if the script contains `\`
        debug!("cmds: {:#?}", cmds);
        for cmd in cmds {
            debug!("cmd {:#?}", cmd);
            let cmd_tok: Vec<&str> = cmd.split(" ").collect();
            let command = cmd_tok.first().unwrap().to_string();
            let mut cmd = Command::new(command);
            cmd.args(&cmd_tok[1..cmd_tok.len()]);
            cmd.current_dir($root);
            let command = if $verbose {
                cmd.spawn()
                    .expect("Unable to run the command")
                    .wait_with_output()
                    .await?
            } else {
                cmd.output().await?
            };

            if !command.status.success() {
                return Err(CoffeeError::new(
                    2,
                    &String::from_utf8(command.stderr).unwrap(),
                ));
            }
        }
    };

    ($root:expr, $script:expr) => {
        sh!($root, $script, false)
    };
}

pub use error;
pub use sh;
