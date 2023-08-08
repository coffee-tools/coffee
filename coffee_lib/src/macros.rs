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
    ($root: expr, $script:expr, $verbose:expr) => {{
        let script = $script.trim();
        let cmds = script.split("\n"); // Check if the script contains `\`
        log::debug!("cmds: {:?}", cmds);
        for cmd in cmds {
            log::debug!("cmd {:?}", cmd);
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
                let mut content = String::from_utf8(command.stderr).unwrap();
                if content.trim().is_empty() {
                    content = String::from_utf8(command.stdout).unwrap();
                }
                return Err(CoffeeError::new(2, &content));
            }
        }
    }};

    ($root:expr, $script:expr) => {
        sh!($root, $script, false)
    };
}

/// get the repository's commit ID as a string.
#[macro_export]
macro_rules! commit_id {
    ($repo:expr) => {{
        $repo
            .head()
            .map_err(|err| error!("{}", err.message()))?
            .peel_to_commit()
            .map_err(|err| error!("{}", err.message()))?
            .id()
            .to_string()
    }};
}

/// get the repository's commit ID and the date of the last commit.
#[macro_export]
macro_rules! get_repo_info {
    ($repo:ident) => {{
        let commit_id = commit_id!($repo);

        let oid = git2::Oid::from_str(&commit_id).map_err(|err| error!("{}", err.message()))?;
        let commit = $repo
            .find_commit(oid)
            .map_err(|err| error!("{}", err.message()))?;
        let commit_time = commit.time();
        let timestamp = commit_time.seconds();
        let date_time = Utc.timestamp_opt(timestamp, 0).single();

        if let Some(date_time) = date_time {
            let formatted_date = date_time.format("%d/%m/%Y").to_string();
            (commit_id.clone(), formatted_date.clone())
        } else {
            return Err(error!("Invalid timestamp"));
        }
    }};
}

pub use {commit_id, error, get_repo_info, sh};
