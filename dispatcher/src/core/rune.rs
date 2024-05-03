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
    fn yeah(&self) -> Result<()> {
        let context = Context::with_default_modules()?;
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
        let mut vm = Vm::new(runtime, Arc::new(unit));
        
        let output = vm.call(["main"], ())?;
        let _output: () = rune::from_value(output)?;
        
        //println!("{}", output);
        Ok(())
    }
}

#[async_trait]
impl Handler for RuneScript {
    async fn handle(&self, _client: &Client, _task: &DynamicTaskMessage) -> Result<HandleResult> {
        self.yeah()?;
        Ok(HandleResult::Continue {
            status: StatusCode::OK,
            response: WorkerResponse {
                tasks: vec![]
            }
        })
    }
}