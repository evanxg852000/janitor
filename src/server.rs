use tide::prelude::*;
use tide::{Server, sse, Request};

use crate::state::State;
use crate::models::Monitor;
use crate::utils::{RespUtil, Page};


use include_dir::{include_dir, Dir};
const EMBEDED_ASSETS: Dir = include_dir!("./ui");

pub(crate) fn new(state: State) -> Server<State> {
  tide::log::start();
  let mut app: Server<State> = Server::with_state(state);
    
  app.at("/").all(|_| async move {
    let path = "index.html".to_string();
    let file = EMBEDED_ASSETS.get_file(&path);
    RespUtil::static_ok(file.unwrap().contents(), &path)
  });

  app.at("/static/*path").get(|req: Request<State>| async move {
    let path = req.param::<String>("path")?;
    let file = EMBEDED_ASSETS.get_file(&path);
    if file.is_none() {
      return RespUtil::static_not_found();
    }
    
    RespUtil::static_ok(file.unwrap().contents(), &path)
  });

  app.at("/info").all(|_| async move {
    Ok(json!({
      "message": "Welcome to Janitor, the status.io engine!",
      "product": "status.io",
      "component": "Janitor",
      "version": option_env!("CARGO_PKG_VERSION"),
      "meta": { "authors": [
          "Evance Soumaoro <evanxg852000@gmail.com>"]
      },
      "statistics": {
      },
    }))
  });

  app.at("/heartbeat/:id").get(|req: Request<State>| async move {
    // heartbeat endpoint
    let id = req.param::<String>("id")?;
    let secret = req.header("X-Auth-Secret").map_or(None, |v| Some(v.as_str().to_string()));
    match req
      .state()
      .read()
      .await
      .heartbeat(id, secret) {
      true => RespUtil::ok(json!({"message": "Yeah! heartbeat received!"})),
      _ => RespUtil::no_auth(json!({"message": "Oops! not authorised!"})),
    }
  });

  app.at("/events/only/:id")
    .get(|req: Request<State>| async move {
      let id = req.param::<String>("id")?;
      if req.state()
        .read()
        .await
        .exists(id) {
          Ok(sse::upgrade(req, |req: Request<State>, sender| async move {
            //loop through rx
            let id = req.param::<String>("id")?;
            req.state().write().await.subscribe(id, sender).await;
            Ok(())
          }))
      } else {
        RespUtil::not_found(json!({"message": "Oops! channel not found."})) 
      }
  });
  
  app.at("events/all")
    .get(|req: Request<State>| async move {
      Ok(sse::upgrade(req, |req: Request<State>, sender| async move {
        req.state()
          .write()
          .await
          .subscribe_all(sender).await;
        Ok(())
      }))
  });


      // sse::upgrade(req: Request<State>, |req: Request<State>, sender| async move {
//      endpoint for sending error event
    // let id = req.param::<String>("id")?;
    // let chan = req.state()
      // .read()
      // .unwrap()
      // .subscribe(id);
//
    // match chan {
      // Some(rx) => {
  //      loop through rx
        // sender.send("heartbeat", "02 failed", None).await?;
      // },
      // _ => (),
    // };
    // Ok(())
  // }));

  app.at("/monitors/:id").post(|mut req: Request<State>| async move {
    //upsert monitor item
    let id = req.param::<String>("id")?;
    let item: Monitor = req.body_json().await?;
    if item.id != id {
      return RespUtil::bad_data(json!({ "message": "Oops! id mismatch."}));
    }

    let msg = match req.state()
      .write()
      .await
      .upsert(id, item) {
      true => "Yeah! item created.",
      _ => "Ok! item updated."
    };
    RespUtil::ok(json!({"message": msg}))
  });

  app.at("/monitors/:id").delete(|req: Request<State>| async move {
    // delete monitor item
    let id = req.param::<String>("id")?;
    match req
      .state()
      .write()
      .await
      .delete(id) {
      true => RespUtil::ok(json!({"message": "Yeah! item deleted."})),
      _ => RespUtil::not_found(json!({"message": "Oops! item not found."})),
    }
  });

  app.at("/monitors").get(|req: Request<State>| async move {
    // list all registered monitors
    let args: Page = req.query()?;
    let (count, data) =  req.state()
      .read()
      .await
      .list(args.page, args.size);
    Ok(json!({
      "data": data,
      "meta": {
        "total": count,
        "page": args.page,
        "size": args.size,
      }
    }))
  });

  app
}

