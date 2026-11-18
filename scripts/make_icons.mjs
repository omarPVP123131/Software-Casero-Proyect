// Genera iconos del bundle (RGBA sin compresión PNG, sin dependencias).
// Se ejecuta una vez con `node scripts/make_icons.mjs`.
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";

const crcTable = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}
function png(size, pixel) {
  const raw = Buffer.alloc((size * 4 + 1) * size);
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0;
    for (let x = 0; x < size; x++) {
      const [r, g, b, a] = pixel(x, y, size);
      const o = y * (size * 4 + 1) + 1 + x * 4;
      raw[o] = r; raw[o + 1] = g; raw[o + 2] = b; raw[o + 3] = a;
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// Logo PhotoLab: fondo #0B1020, haz diagonal con gradiente λ (violeta→cian→ámbar)
// y tres electrones. El trazo es analítico: funciona a cualquier tamaño.
function logo(x, y, size) {
  const u = x / size, v = y / size;
  let r = 11, g = 16, b = 32, a = 255;
  // gradiente radial tenue
  const dx = u - 0.5, dy = v - 0.42;
  const glow = Math.max(0, 1 - Math.hypot(dx, dy) * 1.6) * 26;
  r += glow * 0.4; g += glow * 0.6; b += glow;
  // haz: línea y = 0.62 - 0.18*(x) aprox
  const beamY = 0.28 + 0.44 * u;
  const d = Math.abs(v - beamY);
  if (d < 0.045) {
    const t = u;
    const color = t < 0.5
      ? [120 + 100 * t, 90, 220 - 120 * t]
      : [220, 160 + 80 * (1 - t), 60];
    const k = (1 - d / 0.045) ** 2;
    r = r * (1 - k) + color[0] * k;
    g = g * (1 - k) + color[1] * k;
    b = b * (1 - k) + color[2] * k;
  }
  // electrodos
  if ((u > 0.18 && u < 0.24 && v > 0.24 && v < 0.76) || (u > 0.76 && u < 0.82 && v > 0.24 && v < 0.76)) {
    r = 168; g = 178; b = 196;
  }
  // electrones
  for (const [ex, ey] of [[0.42, 0.47], [0.56, 0.44], [0.64, 0.52]]) {
    if (Math.hypot(u - ex, v - ey) < 0.028) { r = 90; g = 220; b = 255; }
  }
  return [Math.round(r), Math.round(g), Math.round(b), a];
}

mkdirSync("src-tauri/icons", { recursive: true });
for (const size of [32, 128, 256, 512]) {
  writeFileSync(`src-tauri/icons/${size}x${size}.png`, png(size, logo));
}
writeFileSync("src-tauri/icons/icon.png", png(512, logo));
// .ico multi-imagen (BMP payload de 32bpp) para Windows.
function ico(images) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(images.length, 4);
  const entries = [];
  const payloads = [];
  let offset = 6 + 16 * images.length;
  for (const { size, data } of images) {
    const e = Buffer.alloc(16);
    e[0] = size === 256 ? 0 : size; e[1] = size === 256 ? 0 : size;
    e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6);
    e.writeUInt32LE(data.length, 8); e.writeUInt32LE(offset, 12);
    entries.push(e); payloads.push(data); offset += data.length;
  }
  return Buffer.concat([header, ...entries, ...payloads]);
}
function bmpPayload(size) {
  const rowSize = size * 4;
  const mask = Buffer.alloc(Math.ceil(size / 32) * 4 * size);
  const head = Buffer.alloc(40);
  head.writeUInt32LE(40, 0); head.writeInt32LE(size, 4); head.writeInt32LE(size * 2, 8);
  head.writeUInt16LE(1, 12); head.writeUInt16LE(32, 14);
  head.writeUInt32LE(rowSize * size + mask.length, 20);
  const pixels = Buffer.alloc(rowSize * size);
  for (let yy = 0; yy < size; yy++) {
    for (let xx = 0; xx < size; xx++) {
      const [r, g, b, a] = logo(xx, size - 1 - yy, size);
      const o = (yy * size + xx) * 4;
      pixels[o] = b; pixels[o + 1] = g; pixels[o + 2] = r; pixels[o + 3] = a;
    }
  }
  return Buffer.concat([head, pixels, mask]);
}
writeFileSync("src-tauri/icons/icon.ico", ico([
  { size: 32, data: bmpPayload(32) },
  { size: 64, data: bmpPayload(64) },
  { size: 256, data: bmpPayload(256) },
]));
// icns mínimo (PNG embebido, tipo ic09 = 512px) para macOS.
function icns(pngBuf) {
  const type = Buffer.from("ic09");
  const len = Buffer.alloc(4); len.writeUInt32BE(8 + pngBuf.length);
  const body = Buffer.concat([type, len, pngBuf]);
  const head = Buffer.alloc(8); head.write("icns", 0, "ascii"); head.writeUInt32BE(8 + body.length, 4);
  return Buffer.concat([head, body]);
}
writeFileSync("src-tauri/icons/icon.icns", icns(png(512, logo)));
console.log("icons written");
