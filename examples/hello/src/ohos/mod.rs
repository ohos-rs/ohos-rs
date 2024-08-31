use lazy_static::lazy_static;
use napi_derive_ohos::napi;
use napi_ohos::{
    ark::{ArkRuntime, EventLoopMode},
    bindgen_prelude::Function,
    threadsafe_function::ThreadsafeFunction,
    JsNumber, JsString, Result,
};
use ohos_hilog_binding::hilog_info;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

lazy_static! {
    static ref GLOBAL_RUNTIME: Arc<Mutex<Option<ArkRuntime>>> = Arc::new(Mutex::new(None));
    static ref TX: Arc<Mutex<Option<Sender<bool>>>> = Arc::new(Mutex::new(None));
}

#[napi]
pub fn run_ble(cb: ThreadsafeFunction<JsNumber, ()>) -> Result<()> {
    let global_runtime_clone = Arc::clone(&GLOBAL_RUNTIME);
    let (tx, rx) = mpsc::channel::<bool>();

    let mut tx_option = TX.lock().unwrap();
    *tx_option = Some(tx);

    let _handle = thread::spawn(move || {
        let mut runtime_guard = global_runtime_clone.lock().unwrap();
        if runtime_guard.is_none() {
            *runtime_guard = Some(ArkRuntime::new().unwrap());
        }

        let runtime = runtime_guard.as_ref().unwrap();
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

        let event_name = runtime.env.create_string("stateChange").unwrap();

        module
            .call::<_, (JsString, Function<JsNumber, ()>), ()>("on", (event_name, func))
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
