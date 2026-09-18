export const DEFAULT_HOTKEY = 'F9';

export interface HotkeyKeyboardEvent {
  code: string;
  key: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  repeat: boolean;
}

type HotkeyCaptureResult =
  | { kind: 'captured'; hotkey: string }
  | { kind: 'pending' }
  | { kind: 'error'; key?: string };

const MODIFIER_CODES = new Set([
  'AltLeft',
  'AltRight',
  'ControlLeft',
  'ControlRight',
  'MetaLeft',
  'MetaRight',
  'ShiftLeft',
  'ShiftRight',
]);

const SUPPORTED_NAMED_CODES = new Set([
  'AudioVolumeDown',
  'AudioVolumeMute',
  'AudioVolumeUp',
  'Backquote',
  'Backslash',
  'Backspace',
  'BracketLeft',
  'BracketRight',
  'CapsLock',
  'Comma',
  'Delete',
  'End',
  'Enter',
  'Equal',
  'Escape',
  'Home',
  'Insert',
  'MediaPause',
  'MediaPlay',
  'MediaPlayPause',
  'MediaStop',
  'MediaTrackNext',
  'MediaTrackPrevious',
  'Minus',
  'NumLock',
  'NumpadAdd',
  'NumpadDecimal',
  'NumpadDivide',
  'NumpadEnter',
  'NumpadEqual',
  'NumpadMultiply',
  'NumpadSubtract',
  'PageDown',
  'PageUp',
  'Pause',
  'Period',
  'PrintScreen',
  'Quote',
  'ScrollLock',
  'Semicolon',
  'Slash',
  'Space',
  'Tab',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
]);

function isSupportedPrimaryCode(code: string): boolean {
  return (
    /^Key[A-Z]$/.test(code) ||
    /^Digit[0-9]$/.test(code) ||
    /^F(?:[1-9]|1[0-9]|2[0-4])$/.test(code) ||
    /^Numpad[0-9]$/.test(code) ||
    SUPPORTED_NAMED_CODES.has(code)
  );
}

export function hotkeyFromKeyboardEvent(
  event: HotkeyKeyboardEvent,
): HotkeyCaptureResult {
  if (event.repeat || MODIFIER_CODES.has(event.code)) {
    return { kind: 'pending' };
  }

  if (!isSupportedPrimaryCode(event.code)) {
    const rawKey = event.key || event.code;
    return {
      kind: 'error',
      key: rawKey,
    };
  }

  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Win');
  parts.push(event.code);

  return { kind: 'captured', hotkey: parts.join('+') };
}
