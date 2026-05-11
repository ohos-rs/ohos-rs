import { describe, test } from "../utils/setup.test";

const { Fib, Fib2, Fib3 } = requireNapiPreview("example", true);

function getIterator(target: ESObject): ESObject {
  const symbolIterator = (target as ESObject)[Symbol.iterator];
  if (typeof symbolIterator === "function") {
    return symbolIterator.call(target);
  }
  if (typeof (target as ESObject).next === "function") {
    return target;
  }
  const legacyIterator = (target as ESObject).$_iterator;
  if (typeof legacyIterator === "function") {
    return legacyIterator.call(target);
  }
  return undefined;
}

export default function generatorTest() {
  describe("GeneratorTest", () => {
    for (const [index, factory] of [
      () => new Fib(),
      () => Fib2.create(0),
      () => new Fib3(0, 1),
    ].entries()) {
      test(`should be able to stop a generator #${index}`, (t) => {
        const fib = factory();
        const iterator = getIterator(fib);
        t.is(typeof iterator?.next, "function");
        t.deepEqual(iterator.next(), {
          done: false,
          value: 1,
        });
        iterator.next();
        iterator.next();
        iterator.next();
        iterator.next();
        t.deepEqual(iterator.next(), {
          done: false,
          value: 8,
        });
        t.deepEqual(iterator.return?.(), {
          done: true,
        });
        t.deepEqual(iterator.next(), {
          done: true,
        });
      });

      test(`should be able to throw to generator #${index}`, (t) => {
        const fib = factory();
        const iterator = getIterator(fib);
        t.is(typeof iterator?.next, "function");
        t.deepEqual(iterator.next(), {
          done: false,
          value: 1,
        });
        iterator.next();
        iterator.next();
        iterator.next();
        iterator.next();
        t.deepEqual(iterator.next(), {
          done: false,
          value: 8,
        });
        t.throws(() => iterator.throw!(new Error()));
        t.deepEqual(iterator.next(), {
          done: true,
        });
      });
    }
  });
}
