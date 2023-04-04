//! Core macros implemented for coffee.
#[macro_export]
macro_rules! error {
    ($($msg:tt)*) => {{
        let msg = format!($($msg)*);
        CoffeeError::new(1, &msg)
    }};
}

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
            if $verbose {
                let _ = cmd
                    .spawn()
                    .expect("Unable to run the command")
                    .wait()
                    .await?;
            } else {
                let _ = cmd.output().await?;
            }
        }
    };

    ($root:expr, $script:expr) => {
        sh!($root, $script, false)
    };
}

pub use error;
pub use sh;
