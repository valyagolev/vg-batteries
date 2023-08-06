use std::{process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::{Child, Command},
    sync::Mutex,
};

use tokio::task::JoinSet;

#[derive(Debug, Clone)]
pub struct Cmdline {
    pub command: String,
    pub args: Vec<String>,
    pub path: String,
}

impl Cmdline {
    pub fn new(command: &str, args: &Vec<&str>, path: &str) -> Self {
        Self {
            command: command.to_owned(),
            args: args.iter().map(|s| (*s).to_owned()).collect(),
            path: path.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ProcessRunner {
    pub worker_id: String,
    pub should_restart: Mutex<bool>,
    pub name: String,
    pub cmdline: Cmdline,
    pub current: Mutex<Option<Child>>,
    pub stdout: Arc<Mutex<String>>,
    pub stderr: Arc<Mutex<String>>,
    pub tasks: Mutex<JoinSet<anyhow::Result<()>>>,
}

impl ProcessRunner {
    pub fn new(worker_id: String, name: String, cmdline: Cmdline) -> anyhow::Result<Self> {
        let mut tasks = JoinSet::new();
        tasks.spawn(Self::reap_orphans(name.clone()));

        let pr = Self {
            worker_id,
            should_restart: Mutex::new(true),
            name,
            cmdline,
            current: Mutex::new(None),
            stdout: Arc::new(Mutex::new(String::new())),
            stderr: Arc::new(Mutex::new(String::new())),
            tasks: Mutex::new(tasks),
        };

        Ok(pr)
    }

    pub async fn reap_orphans(name: String) -> anyhow::Result<()> {
        let Ok(last_rememebered_pid) = std::fs::read_to_string(format!("./{}.pid", name)) else {
            return Ok(());
        };

        println!("Found pid-file for {}", name);

        let last_rememebered_pid: u32 = last_rememebered_pid.parse()?;

        let Ok(cmdline) =
            std::fs::read_to_string(format!("/proc/{}/cmdline", last_rememebered_pid))
        else {
            println!("pid-file for {} is stale, deleting", name);

            std::fs::remove_file(format!("./{}.pid", name))?;
            return Ok(());
        };

        print!(
            "reaping orphan pid={} cmdline: {}",
            last_rememebered_pid, cmdline
        );

        let result = Command::new("kill")
            .arg("-9")
            .arg(last_rememebered_pid.to_string())
            .output()
            .await?;
        println!("result: {:?}", result);

        std::fs::remove_file(format!("./{}.pid", name))?;

        Ok(())
    }

    pub async fn wait_until_stop(self: Arc<Self>) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let mut cur = self.current.lock().await;

            if cur.is_some() {
                let res = cur.as_mut().unwrap().try_wait()?;

                if let Some(res) = res {
                    println!("process exited: {:?}", res);
                    *cur = None;
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }
    }

    pub async fn start_and_loop(self: Arc<Self>) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            if !*self.should_restart.lock().await {
                println!("not restarting");
                return Ok(());
            }

            self.start().await?;
            self.clone().wait_until_stop().await?;
        }
    }

    pub async fn disable_restart(&self) -> anyhow::Result<()> {
        *self.should_restart.lock().await = false;

        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        let mut cur = self.current.lock().await;

        if let Some(ref mut child) = &mut cur.as_mut() {
            let pid = child.id().expect("pid?");

            let last_rememebered_pid = std::fs::read_to_string(format!("./{}.pid", self.name))?;

            if pid.to_string() != last_rememebered_pid {
                println!("pid mismatch, not deleting .pid");
            } else {
                std::fs::remove_file(format!("./{}.pid", self.name))?;
            }

            print!("stopping process");
            child.kill().await?;
            child.wait().await?;

            *cur = None;

            self.drain_tasks().await;
        }

        Ok(())
    }

    pub async fn drain_tasks(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.abort_all();

        while let Some(_) = tasks.join_next().await {
            println!("waiting for pipe reading tasks to finish");
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let Cmdline {
            command,
            args,
            path,
        } = &self.cmdline;

        self.stop().await?;
        self.drain_tasks().await;

        *self.stdout.lock().await = String::new();
        *self.stderr.lock().await = String::new();

        println!(
            "starting process: {} {} {:?}",
            command,
            args.join(" "),
            path
        );

        let mut cur = self.current.lock().await;

        *cur = Some(
            Command::new(command)
                .env("WORKER_ID", &self.worker_id)
                .args(args)
                .current_dir(path)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?,
        );

        let proc = cur.as_mut().unwrap();
        let pid = proc.id().expect("pid?");

        std::fs::write(format!("./{}.pid", self.name), pid.to_string())?;

        let stdout = proc.stdout.take().unwrap();
        let stderr = proc.stderr.take().unwrap();

        let mut tasks = self.tasks.lock().await;
        tasks.spawn(Self::run_pipe_reader(
            self.name.clone(),
            stdout,
            self.stdout.clone(),
        ));
        tasks.spawn(Self::run_pipe_reader(
            self.name.clone(),
            stderr,
            self.stderr.clone(),
        ));

        Ok(())
    }

    pub async fn run_pipe_reader(
        process_name: String,
        pipe: impl AsyncRead + Unpin,
        data: Arc<Mutex<String>>,
    ) -> anyhow::Result<()> {
        let reader = BufReader::new(pipe);

        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            println!("from {}: {}", process_name, line);
            let mut data = data.lock().await;
            data.push_str(&line);
            data.push('\n');
        }

        Ok(())
    }

    pub async fn get_status(&self) -> anyhow::Result<serde_json::Value> {
        let mut status = serde_json::json!({
            "process_id": self.worker_id,
            "name": self.name,
            "running": false,
            "stdout": *self.stdout.lock().await,
            "stderr": *self.stderr.lock().await,
        });

        let mut cur = self.current.lock().await;

        if let Some(ref mut child) = &mut cur.as_mut() {
            status["running"] = serde_json::json!(child.try_wait()?.is_none());
            status["pid"] = serde_json::json!(child.id());
        }

        Ok(status)
    }
}
