// Re-exports from split font files (kept under 4MB each for git object limits)
import { FONT_DATA_REGULAR } from './font-data-regular';
import { FONT_DATA_BOLD } from './font-data-bold';
import { FONT_DATA_ITALIC } from './font-data-italic';

const FONT_DATA: Record<string, string> = {
  ...FONT_DATA_REGULAR,
  ...FONT_DATA_BOLD,
  ...FONT_DATA_ITALIC,
};

export function getGlyphPBF(fontstack: string, range: string): ArrayBuffer | null {
  const b64 = FONT_DATA[`${fontstack}/${range}`];
  if (!b64) return null;
  const bin = atob(b64);
  const buf = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
  return buf.buffer;
}
