if (!(globalThis as ESObject).console) {
  (globalThis as ESObject).console = {
    log: (_msg: ESObject) => {},
    info: (_msg: ESObject) => {},
    warn: (_msg: ESObject) => {},
    error: (_msg: ESObject) => {},
  };
}

if (!(globalThis as ESObject).setTimeout) {
  let nextTimerId = 1;
  const timers = new Map<number, boolean>();
  (globalThis as ESObject).setTimeout = (handler: ESObject, timeoutMs: number = 0) => {
    const id = nextTimerId++;
    timers.set(id, true);
    if (typeof handler === "function") {
      Promise.resolve().then(() => {
        if (!timers.has(id)) {
          return;
        }
        timers.delete(id);
        handler();
      });
    }
    return id;
  };
  (globalThis as ESObject).__ohosTimerMap__ = timers;
}

if (!(globalThis as ESObject).clearTimeout) {
  (globalThis as ESObject).clearTimeout = (id: number) => {
    const timers = (globalThis as ESObject).__ohosTimerMap__;
    if (timers && typeof timers.delete === "function") {
      timers.delete(id);
    }
  };
}

function getOrInitNapiMetrics(): ESObject {
  const g = globalThis as ESObject;
  if (!g.__ohosNapiMetrics__) {
    g.__ohosNapiMetrics__ = {
      calls: 0,
      modules: [],
    };
  }
  return g.__ohosNapiMetrics__;
}

const originalRequireNapiPreview = (globalThis as ESObject).requireNapiPreview;
if (typeof originalRequireNapiPreview === "function") {
  const proxyCache = new WeakMap();
  const rawValueCache = new WeakMap();

  const unwrapValue = (value: ESObject): ESObject => {
    if (value === null || value === undefined) {
      return value;
    }
    const valueType = typeof value;
    if ((valueType === "object" || valueType === "function") && rawValueCache.has(value)) {
      return rawValueCache.get(value);
    }
    return value;
  };

  const wrapValue = (value: ESObject): ESObject => {

    if (value === null || value === undefined) {
      return value;
    }
    const valueType = typeof value;
    if (valueType !== "object" && valueType !== "function") {
      return value;
    }
    if (proxyCache.has(value)) {
      return proxyCache.get(value);
    }

    const proxy = new Proxy(value as ESObject, {
      get(target: ESObject, prop: ESObject, receiver: ESObject) {
        const inner = Reflect.get(target, prop, target);
        return wrapValue(inner);
      },
      set(target: ESObject, prop: ESObject, value: ESObject, receiver: ESObject) {
        return Reflect.set(target, prop, unwrapValue(value), target);
      },
      apply(target: ESObject, thisArg: ESObject, argArray: ESObject[]) {
        const metrics = getOrInitNapiMetrics();
        metrics.calls += 1;
        const rawThisArg = unwrapValue(thisArg);
        const rawArgArray = argArray.map((item) => unwrapValue(item));
        const ret = Reflect.apply(target as ESObject, rawThisArg, rawArgArray);
        return wrapValue(ret);
      },
      construct(target: ESObject, argArray: ESObject[], newTarget: ESObject) {
        const metrics = getOrInitNapiMetrics();
        metrics.calls += 1;
        const rawArgArray = argArray.map((item) => unwrapValue(item));
        const ret = Reflect.construct(target as ESObject, rawArgArray, target as ESObject);
        return wrapValue(ret);
      },
    });

    proxyCache.set(value, proxy);
    rawValueCache.set(proxy, value);
    return proxy;
  };

  (globalThis as ESObject).requireNapiPreview = (moduleName: string, isAppModule: boolean) => {
    const metrics = getOrInitNapiMetrics();
    if (!metrics.modules.includes(moduleName)) {
      metrics.modules.push(moduleName);
    }
    const loaded = originalRequireNapiPreview(moduleName, isAppModule);
    return wrapValue(loaded);
  };
}
