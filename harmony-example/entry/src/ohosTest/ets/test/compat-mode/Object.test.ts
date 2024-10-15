import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";

export default () => {
  describe("CompatObjectTest", () => {
    test("setProperty", (t) => {
      const obj = {};
      const key = "jsPropertyKey";
      bindings.testSetProperty(obj, key);
      t.is(obj[key], "Rust object property");
    });

    test("testGetProperty", (t) => {
      const name = "JsSymbol";
      const value = "JsValue";
      const obj = {
        [name]: value
      };
      t.is(bindings.testGetProperty(obj, name), value);
    });

    test("setNamedProperty", (t) => {
      const obj = {};
      const property = "hello";
      bindings.testSetNamedProperty(obj, property);
      const keys = Object.keys(obj);
      const [key] = keys;
      t.is(keys.length, 1);
      t.is(obj[key], property);
    });

    test("testGetNamedProperty", (t) => {
      const obj = {
        p: "JsSymbol"
      };
      t.is(bindings.testGetNamedProperty(obj), obj.p);
    });

    test("testHasNamedProperty", (t) => {
      const obj = {
        a: 1,
        b: undefined
      };

      t.true(bindings.testHasNamedProperty(obj, "a"));
      t.true(bindings.testHasNamedProperty(obj, "b"));
      t.false(bindings.testHasNamedProperty(obj, "c"));
    });

    test("testHasOwnProperty", (t) => {
      const obj = {
        a: "1",
        b: undefined
      };

      const child = Object.create(obj, {
        d: {
          value: 1,
          enumerable: true,
          configurable: true
        }
      });

      t.false(bindings.testHasOwnProperty(child, "a"));
      t.false(bindings.testHasOwnProperty(child, "b"));
      t.true(bindings.testHasOwnProperty(child, "d"));
    });

    test("testHasOwnPropertyJs", (t) => {
      const obj = {
        a: "1",
        b: undefined
      };

      const child = Object.create(obj);

      child.c = "k1";

      t.false(bindings.testHasOwnPropertyJs(child, "a"));
      t.false(bindings.testHasOwnPropertyJs(child, "b"));
      t.true(bindings.testHasOwnPropertyJs(child, "c"));
    });

    test("testHasProperty", (t) => {
      const obj = {
        a: "1",
        b: undefined
      };

      const child = Object.create(obj);

      child.c = "k1";

      t.true(bindings.testHasProperty(child, "a"));
      t.true(bindings.testHasProperty(child, "b"));
      t.true(bindings.testHasProperty(child, "c"));
      t.false(bindings.testHasProperty(child, "__NOT_EXISTED__"));
    });

    test("testHasPropertJs", (t) => {
      const key = "JsString";
      const obj = {
        [key]: 1,
        a: 0,
        b: undefined,
        2: "c"
      };
      t.true(bindings.testHasPropertyJs(obj, key));
      t.true(bindings.testHasPropertyJs(obj, "a"));
      t.true(bindings.testHasPropertyJs(obj, "b"));
      t.true(bindings.testHasPropertyJs(obj, 2));
      t.false(bindings.testHasPropertyJs(obj, {}));
    });

    test("testDeleteProperty", (t) => {
      const k2 = 2;
      const k3 = "foo";
      const obj = {
        [k2]: 2,
        k4: 4
      };
      Object.defineProperty(obj, k3, {
        configurable: false,
        enumerable: true,
        value: "k3"
      });
      t.true(bindings.testDeleteProperty(obj, k2));
      t.false(bindings.testDeleteProperty(obj, k3));
      t.true(bindings.testDeleteProperty(obj, "k4"));
      t.true(bindings.testDeleteProperty(obj, "__NOT_EXISTED__"));
      t.deepEqual(obj, { [k3]: "k3" });
    });

    test("testDeleteNamedProperty", (t) => {
      const k1 = "k1";
      const k2 = "k2";
      const k3 = "foo";
      const obj = {
        [k1]: 1,
        [k2]: 2,
        k4: 4
      };
      Object.defineProperty(obj, k3, {
        configurable: false,
        enumerable: true,
        value: "k3"
      });
      t.true(bindings.testDeleteNamedProperty(obj, k1));
      t.true(bindings.testDeleteNamedProperty(obj, k2));
      t.false(bindings.testDeleteNamedProperty(obj, k3));
      t.true(bindings.testDeleteNamedProperty(obj, "k4"));
      t.true(bindings.testDeleteNamedProperty(obj, "__NOT_EXISTED__"));
      t.true(bindings.testDeleteNamedProperty(obj, k1));
      t.deepEqual(obj, { [k3]: "k3" });
    });

    test("testGetPropertyNames", (t) => {
      const k2 = 2;
      const k3 = "k3";
      const obj = {
        [k2]: 1,
        [k3]: 1
      };
      const ret = bindings
        .testGetPropertyNames(obj)
        .map((v: string | number) => v.toString());

      t.deepEqual(ret, ["2", "k3"]);
    });

    test("testGetPrototype", (t) => {
      class A {}
      class B extends A {}
      const obj = new B();
      t.is(bindings.testGetPrototype(obj), Object.getPrototypeOf(obj));
    });

    test("testSetElement", (t) => {
      const arr: any[] = [];
      bindings.testSetElement(arr, 1, 1);
      bindings.testSetElement(arr, 5, "foo");
      t.deepEqual(arr, [, 1, , , , "foo"]);
    });

    test("testHasElement", (t) => {
      const arr: number[] = [];
      arr[1] = 1;
      arr[4] = 0;
      t.false(bindings.testHasElement(arr, 0));
      t.true(bindings.testHasElement(arr, 1));
      t.false(bindings.testHasElement(arr, 2));
      t.false(bindings.testHasElement(arr, 3));
      t.true(bindings.testHasElement(arr, 4));
    });

    test("testGetElement", (t) => {
      const arr = [1, 2];
      t.is(bindings.testGetElement(arr, 0), arr[0]);
      t.is(bindings.testGetElement(arr, 1), arr[1]);
    });

    test("testDeleteElement", (t) => {
      const arr = [0, 1, 2, 3];
      bindings.testDeleteElement(arr, 1);
      bindings.testDeleteElement(arr, 2);
      t.deepEqual(arr, [0, , , 3]);
    });

    test("testDefineProperties", (t) => {
      const obj: any = {};
      bindings.testDefineProperties(obj);
      t.is(obj.count, 0);
      obj.add(10);
      t.is(obj.count, 10);
      const descriptor = Object.getOwnPropertyDescriptor(obj, "ro");
      t.is(descriptor?.value ?? descriptor?.get?.(), "readonly");
    });

    test("is promise", (t) => {
      t.false(bindings.testIsPromise(1));
      t.false(bindings.testIsPromise("hello"));
      t.false(bindings.testIsPromise({}));
      t.false(bindings.testIsPromise(new Date()));

      t.true(bindings.testIsPromise(Promise.resolve()));
      t.true(bindings.testIsPromise(Promise.reject().catch(() => {})));
      t.true(
        bindings.testIsPromise(
          new Promise<void>((resolve) => {
            resolve();
          })
        )
      );
    });
  });
};
