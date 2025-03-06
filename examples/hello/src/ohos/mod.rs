use napi_derive_ohos::napi;
use napi_ohos::{
  ark::{ArkRuntime, EventLoopMode},
  bindgen_prelude::{FnArgs, Function},
  threadsafe_function::ThreadsafeFunction,
  Env, JsNumber, Result,
};
use ohos_hilog_binding::hilog_info;
use std::{
  sync::{
    mpsc::{self, Sender},
    LazyLock, Mutex,
  },
  thread,
  time::Duration,
};

static TX: LazyLock<Mutex<Option<Sender<bool>>>> = LazyLock::new(|| Mutex::new(None));

#[napi]
pub fn run_ble(cb: ThreadsafeFunction<JsNumber, ()>) -> Result<()> {
  let (tx, rx) = mpsc::channel::<bool>();

  let mut tx_option = TX.lock().unwrap();
  *tx_option = Some(tx);

  let _handle = thread::spawn(move || {
    let runtime = ArkRuntime::new().unwrap();
    let module = runtime.load_without_info("@ohos.bluetooth.access").unwrap();

    let func: Function<JsNumber, ()> = runtime
      .env
      .create_function_from_closure("stateChange", move |ctx| {
        hilog_info!("arkruntime_ble");
        let arg: JsNumber = ctx.first_arg().unwrap();
        cb.call(
          Ok(arg),
          napi_ohos::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
        Ok(())
      })
      .unwrap();

    module
      .call::<_, FnArgs<(String, Function<JsNumber, ()>)>, ()>(
        "on",
        (String::from("stateChange"), func).into(),
      )
      .unwrap();

    loop {
      runtime.run_loop(EventLoopMode::NonBlocking).unwrap();

      if let Ok(_) = rx.try_recv() {
        break;
      }
      thread::sleep(Duration::from_millis(10));
    }
  });

  Ok(())
}

#[napi]
pub fn stop() -> Result<()> {
  let tx_lock = TX.lock().unwrap();

  if let Some(tx) = tx_lock.as_ref() {
    tx.send(true).unwrap();
    return Ok(());
  }
  Ok(())
}

#[napi]
pub fn load_log(env: Env) -> Result<()> {
  let _log = env.load("@ohos.hilog")?;
  Ok(())
}
