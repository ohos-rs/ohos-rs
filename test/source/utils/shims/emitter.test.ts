const handlers = new Map<string, Array<ESObject>>();

function on(name: string, handler: ESObject) {
  const list = handlers.get(name) || [];
  list.push(handler);
  handlers.set(name, list);
}

function off(name: string, handler: ESObject) {
  const list = handlers.get(name) || [];
  handlers.set(
    name,
    list.filter((item) => item !== handler),
  );
}

function emit(name: string, payload: ESObject) {
  const list = handlers.get(name) || [];
  for (const handler of list) {
    handler(payload);
  }
}

export default {
  on,
  off,
  emit,
};
