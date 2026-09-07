interface Window {
  __TAURI_INTERNALS__?: {
    invoke?: unknown;
    transformCallback?: unknown;
  };
  EyeDropper?: {
    new (): {
      open(): Promise<{ sRGBHex: string }>;
    };
  };
}
