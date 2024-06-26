// `AbortSignal`,`AbortController` are defined here to prevent a dependency on the `dom` library which disagrees with node runtime.
// The definition for `AbortSignal` is taken from @types/node-fetch (https://github.com/DefinitelyTyped/DefinitelyTyped) for
// maximal compatibility with node-fetch.
// Original node-fetch definitions are under MIT License.
// this content is forked from node-abort-controller
pub const ABORT_TS: &str = r#"export class AbortSignal {
  aborted: boolean;
  reason?: any;

  addEventListener: (
    type: "abort",
    listener: (this: AbortSignal, event: any) => any,
    options?:
      | boolean
      | {
          capture?: boolean;
          once?: boolean;
          passive?: boolean;
        }
  ) => void;

  removeEventListener: (
    type: "abort",
    listener: (this: AbortSignal, event: any) => any,
    options?:
      | boolean
      | {
          capture?: boolean;
        }
  ) => void;

  dispatchEvent: (event: any) => void;

  onabort: null | ((this: AbortSignal, event: any) => void);

  throwIfAborted(): void;

  static abort(reason?: any): AbortSignal;

  static timeout(time: number): AbortSignal;
}
"#;
