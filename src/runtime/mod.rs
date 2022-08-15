use std::env::{self, current_dir};
use std::path::PathBuf;

use crate::LoopData;
use calloop::futures::{Executor, Scheduler};
use calloop::LoopHandle;
use deno_core::error::AnyError;
use deno_core::ModuleSpecifier;
use deno_runtime::worker::MainWorker;

mod main;

pub struct Runtime {
    pub main_worker: MainWorker,
    pub main_module: ModuleSpecifier,
    pub data: LoopData,
}

impl Runtime {
    pub fn new(data: LoopData) -> Self {
        let mut config_path: PathBuf;
        if cfg!(feature = "devel") {
            config_path = match env::var("WM_PREFIX") {
                Ok(x) => PathBuf::from(x),
                Err(_) => current_dir().unwrap(),
            };
            config_path.push("src");
            config_path.push("wm");
            config_path.push("main.js");
        } else {
            let xdg_dirs = xdg::BaseDirectories::with_prefix("electrum").unwrap();
            config_path = xdg_dirs.get_config_file("main.js");
        }

        println!("{}", config_path.display());

        let main_module = deno_core::resolve_path(config_path.to_str().unwrap())
            .expect("failed to resolve main module");
        let main_worker = main::new(main_module.clone());

        Runtime {
            main_worker,
            main_module,
            data,
        }
    }

    pub async fn run(mut self) -> Result<(), AnyError> {
        self.main_worker
            .execute_main_module(&self.main_module)
            .await?;
        self.main_worker.run_event_loop(false).await?;
        Ok(())
    }

    pub fn run_with_calloop(self, handle: LoopHandle<LoopData>) {
        let (exec, sched): (
            Executor<Result<(), AnyError>>,
            Scheduler<Result<(), AnyError>>,
        ) = calloop::futures::executor().unwrap();

        handle
            .insert_source(exec, |evt, _metadata, _shared| {
                evt.unwrap();
            })
            .unwrap();

        sched.schedule(self.run()).unwrap();
    }
}
