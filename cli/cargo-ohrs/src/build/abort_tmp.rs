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

  dispatchEvent: (event: any) => boolean;

  onabort: null | ((this: AbortSignal, event: any) => void);

  throwIfAborted(): void;

  static abort(reason?: any): AbortSignal;

  static timeout(time: number): AbortSignal;
}
"#;
