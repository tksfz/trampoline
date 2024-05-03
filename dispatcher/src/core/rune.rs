use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use rune::alloc::clone::TryClone;
use rune::{Context, Diagnostics, Source, Sources, Vm};
use rune::termcolor::{ColorChoice, StandardStream};
use std::sync::Arc;

use crate::data::DynamicTaskMessage;

use super::handler::Handler;
use super::{HandleResult, WorkerResponse};


pub struct RuneScript {
    pub script: Source,
}

impl RuneScript {
    async fn yeah(&self) -> Result<()> {
        // https://docs.rs/rune-modules/0.13.2/rune_modules/http/
        let mut context = Context::with_default_modules()?;
        context.install(rune_modules::http::module(true)?)?;
        context.install(rune_modules::json::module(true)?)?;
        let runtime = Arc::new(context.runtime()?);
        
        let mut sources = Sources::new();
        sources.insert(self.script.try_clone()?)?;
        
        let mut diagnostics = Diagnostics::new();
        
        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();
        
        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources)?;
        }
        
        let unit = result?;
        let vm = Vm::new(runtime, Arc::new(unit));
        
        // https://rune-rs.github.io/book/multithreading.html
        let execution = vm.try_clone()?.send_execute(["main"], ())?;
        let _t1 = tokio::spawn(async move {
            execution.async_complete().await.unwrap();
            println!("timer ticked");
        });

        //println!("{}", output);
        Ok(())
    }
}

#[async_trait]
impl Handler for RuneScript {
    async fn handle(&self, _client: &Client, _task: &DynamicTaskMessage) -> Result<HandleResult> {
        self.yeah().await?;
        Ok(HandleResult::Continue {
            status: StatusCode::OK,
            response: WorkerResponse {
                tasks: vec![]
            }
        })
    }
}