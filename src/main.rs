mod server;
mod state;
mod models;
mod utils;

//use async_std::prelude::*;


#[async_std::main]
async fn main() -> tide::Result<()> {
    // dotenv::dotenv().ok();
    // pretty_env_logger::init();
    let state = state::new_state("admin");
    
    state.write()
      .await  
      .start().await?;

    server::new(state.clone())
        .listen("127.0.0.1:8080")
        .await?;

    state.read()
      .await
      .stop().await?;
    
    Ok(())   
    

      // let state = state::new_state("admin");
    // let mut engine = state.write().unwrap();
//
    // let (serve, process) = server::new(state.clone())
        // .listen("127.0.0.1:8080")
        // .join(engine.start())
        // .await;
//
    // match (serve, process) {
        // (Ok(_), Ok(_)) => Ok(()),
        // (Err(e), Ok(_)) => { Err(e.into())},
        // (Ok(_), Err(e)) => Err(e),
        // (Err(_), Err(e)) => Err(e),
    // }
//
    // server::new()
    //     .listen("127.0.0.1:8080")
    //     .await?;
}
