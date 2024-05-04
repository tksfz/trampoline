use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use rune::alloc::clone::TryClone;
use rune::runtime::{RuntimeContext, VmResult};
use rune::{Any, Context, ContextError, Diagnostics, Module, Source, Sources, Unit, Value, Vm};
use rune::termcolor::{ColorChoice, StandardStream};
use std::collections::HashMap;
use std::sync::Arc;

use crate::data::DynamicTaskMessage;

use super::handler::Handler;
use super::{HandleResult, WorkerResponse};


#[derive(Debug, Any)]
#[rune(constructor)]
struct TrampolineTask {
    #[rune(get, set)]
    type_name: String,
    #[rune(get, set)]
    task: HashMap<String, Value>,
}

pub struct RuneScript {
    runtime: Arc<RuntimeContext>,
    unit: Arc<Unit>,
}

impl RuneScript {
    pub fn trampoline_module() -> Result<Module, ContextError> {
        let mut module = Module::default();
        module.ty::<TrampolineTask>()?;
        Ok(module)
    }
    
    pub fn new(script: Source) -> Result<RuneScript> {
        // https://docs.rs/rune-modules/0.13.2/rune_modules/http/
        let mut context = Context::with_default_modules()?;
        context.install(rune_modules::http::module(true)?)?;
        context.install(rune_modules::json::module(true)?)?;


        context.install(Self::trampoline_module()?)?;

        let runtime = Arc::new(context.runtime()?);
        
        let mut sources = Sources::new();
        sources.insert(script.try_clone()?)?;
        
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
        Ok(RuneScript { runtime, unit: Arc::new(unit) })
    }

    async fn execute(&self, _client: &Client, task: &DynamicTaskMessage) -> Result<HandleResult> {
        let vm = Vm::new(self.runtime.clone(), self.unit.clone());
        
        // TODO: figure out how to pass Rune json value directly
        let json = serde_json::to_string(&task.task)?;

        // https://rune-rs.github.io/book/multithreading.html
        let execution = vm.try_clone()?.send_execute(["handle"], (task.type_name.clone(), json))?;
        // https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        let _t1 = tokio::spawn(async move {
            // get_script_result is syntactically necessary here to deal with the intermediate Result
            // (e.g. in lieu of a try block)
            let r = Self::get_script_result(execution.async_complete().await);
            match r {
                Ok(x) => println!("script returned value {:?}", x),
                Err(y) => println!("script got error {}", y)
            }
        });
        Ok(HandleResult::Continue { status: StatusCode::OK, response: WorkerResponse { tasks: vec![] } })
    }

    fn get_script_result(r: VmResult<Value>) -> Result<Vec<TrampolineTask>> {
        rune::from_value::<Result<Vec<TrampolineTask>>>(r.into_result()?)?
    }

}

#[async_trait]
impl Handler for RuneScript {
    async fn handle(&self, client: &Client, task: &DynamicTaskMessage) -> Result<HandleResult> {
        self.execute(client, task).await?;
        Ok(HandleResult::Continue {
            status: StatusCode::OK,
            response: WorkerResponse {
                tasks: vec![]
            }
        })
    }
}