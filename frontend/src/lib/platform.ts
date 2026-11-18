// Atajos con el modificador correcto según la plataforma. La app se distribuye
// sobre todo en Windows, pero sigue siendo usable en macOS/Linux.
const ua = typeof navigator === "undefined" ? "" : navigator.userAgent;

export const isMac = /Mac|iPhone|iPad|iPod/i.test(ua);
export const modKey = isMac ? "⌘" : "Ctrl";
export const paletteShortcut = `${modKey}+K`;
export const undoShortcut = `${modKey}+Z`;
export const redoShortcut = isMac ? "⌘+Shift+Z" : "Ctrl+Y";
