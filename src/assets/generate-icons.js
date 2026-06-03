const fs = require('fs');
const path = require('path');
const { createCanvas } = require('canvas');

function createTemplateIcon(size) {
  const canvas = createCanvas(size, size);
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, size, size);

  const s = size / 32;

  // Clipboard outline (template style: black with alpha, no fill)
  ctx.strokeStyle = 'black';
  ctx.lineWidth = 2 * s;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.fillStyle = 'black';

  // Clipboard body (rounded rect)
  const x = 6 * s, y = 4 * s, w = 20 * s, h = 26 * s, r = 3 * s;
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, y + h - r);
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
  ctx.lineTo(x + r, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
  ctx.stroke();

  // Clipboard clip (top)
  const cx = 16 * s, cy = 3 * s, cw = 10 * s, ch = 6 * s;
  ctx.beginPath();
  ctx.moveTo(cx - cw/2 + 1.5*s, cy);
  ctx.lineTo(cx - cw/2, cy);
  ctx.lineTo(cx - cw/2, cy + ch);
  ctx.lineTo(cx + cw/2, cy + ch);
  ctx.lineTo(cx + cw/2, cy);
  ctx.closePath();
  ctx.stroke();

  // Lines on clipboard
  ctx.beginPath();
  ctx.moveTo(10 * s, 14 * s);
  ctx.lineTo(22 * s, 14 * s);
  ctx.moveTo(10 * s, 18 * s);
  ctx.lineTo(22 * s, 18 * s);
  ctx.moveTo(10 * s, 22 * s);
  ctx.lineTo(18 * s, 22 * s);
  ctx.stroke();

  return canvas.toBuffer('image/png');
}

function createAppIcon(size) {
  const canvas = createCanvas(size, size);
  const ctx = canvas.getContext('2d');

  // Background gradient (Tahoe-style blue to purple)
  const gradient = ctx.createLinearGradient(0, 0, size, size);
  gradient.addColorStop(0, '#0a84ff');
  gradient.addColorStop(0.5, '#5e5ce6');
  gradient.addColorStop(1, '#bf5af2');

  // Rounded square (macOS app icon style)
  const r = size * 0.22; // 22% radius (Tahoe style)
  ctx.beginPath();
  ctx.moveTo(r, 0);
  ctx.lineTo(size - r, 0);
  ctx.quadraticCurveTo(size, 0, size, r);
  ctx.lineTo(size, size - r);
  ctx.quadraticCurveTo(size, size, size - r, size);
  ctx.lineTo(r, size);
  ctx.quadraticCurveTo(0, size, 0, size - r);
  ctx.lineTo(0, r);
  ctx.quadraticCurveTo(0, 0, r, 0);
  ctx.closePath();
  ctx.fillStyle = gradient;
  ctx.fill();

  // Inner highlight (glass effect)
  const highlight = ctx.createLinearGradient(0, 0, 0, size * 0.5);
  highlight.addColorStop(0, 'rgba(255, 255, 255, 0.25)');
  highlight.addColorStop(1, 'rgba(255, 255, 255, 0)');
  ctx.fillStyle = highlight;
  ctx.fill();

  // Clipboard icon in white
  const s = size / 100;
  ctx.strokeStyle = 'white';
  ctx.lineWidth = 6 * s;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.fillStyle = 'white';

  // Clipboard body
  const x = 25 * s, y = 18 * s, w = 50 * s, h = 70 * s, radius = 6 * s;
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.lineTo(x + w - radius, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + radius);
  ctx.lineTo(x + w, y + h - radius);
  ctx.quadraticCurveTo(x + w, y + h, x + w - radius, y + h);
  ctx.lineTo(x + radius, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - radius);
  ctx.lineTo(x, y + radius);
  ctx.quadraticCurveTo(x, y, x + radius, y);
  ctx.closePath();
  ctx.stroke();

  // Clip
  const cx = 50 * s, cy = 14 * s, cw = 24 * s, ch = 12 * s;
  ctx.beginPath();
  ctx.moveTo(cx - cw/2 + 3 * s, cy);
  ctx.lineTo(cx - cw/2, cy);
  ctx.lineTo(cx - cw/2, cy + ch);
  ctx.lineTo(cx + cw/2, cy + ch);
  ctx.lineTo(cx + cw/2, cy);
  ctx.closePath();
  ctx.stroke();

  // Lines
  ctx.beginPath();
  ctx.moveTo(34 * s, 42 * s);
  ctx.lineTo(66 * s, 42 * s);
  ctx.moveTo(34 * s, 52 * s);
  ctx.lineTo(66 * s, 52 * s);
  ctx.moveTo(34 * s, 62 * s);
  ctx.lineTo(58 * s, 62 * s);
  ctx.stroke();

  return canvas.toBuffer('image/png');
}

function main() {
  const assetsDir = __dirname;

  // Tray template icons (B/W with alpha, for macOS menu bar)
  const tray16 = createTemplateIcon(16);
  const tray32 = createTemplateIcon(32);
  fs.writeFileSync(path.join(assetsDir, 'trayIconTemplate.png'), tray16);
  fs.writeFileSync(path.join(assetsDir, 'trayIconTemplate@2x.png'), tray32);
  console.log('Generated tray template icons (16x16, 32x32)');

  // Window icon (template, used for the window itself)
  const win32 = createTemplateIcon(32);
  fs.writeFileSync(path.join(assetsDir, 'windowIconTemplate.png'), win32);
  console.log('Generated window template icon (32x32)');

  // Full app icon set
  const sizes = [16, 32, 64, 128, 256, 512, 1024];
  for (const size of sizes) {
    const buf = createAppIcon(size);
    fs.writeFileSync(path.join(assetsDir, `icon-${size}.png`), buf);
  }
  console.log(`Generated app icons: ${sizes.join(', ')}`);

  // Main icon.png (used by electron-builder)
  fs.writeFileSync(path.join(assetsDir, 'icon.png'), createAppIcon(512));
  console.log('Generated main icon.png (512x512)');

  // iconset folder for macOS .icns generation
  const iconsetDir = path.join(assetsDir, 'icon.iconset');
  if (!fs.existsSync(iconsetDir)) {
    fs.mkdirSync(iconsetDir);
  }
  fs.writeFileSync(path.join(iconsetDir, 'icon_16x16.png'), createAppIcon(16));
  fs.writeFileSync(path.join(iconsetDir, 'icon_16x16@2x.png'), createAppIcon(32));
  fs.writeFileSync(path.join(iconsetDir, 'icon_32x32.png'), createAppIcon(32));
  fs.writeFileSync(path.join(iconsetDir, 'icon_32x32@2x.png'), createAppIcon(64));
  fs.writeFileSync(path.join(iconsetDir, 'icon_128x128.png'), createAppIcon(128));
  fs.writeFileSync(path.join(iconsetDir, 'icon_128x128@2x.png'), createAppIcon(256));
  fs.writeFileSync(path.join(iconsetDir, 'icon_256x256.png'), createAppIcon(256));
  fs.writeFileSync(path.join(iconsetDir, 'icon_256x256@2x.png'), createAppIcon(512));
  fs.writeFileSync(path.join(iconsetDir, 'icon_512x512.png'), createAppIcon(512));
  fs.writeFileSync(path.join(iconsetDir, 'icon_512x512@2x.png'), createAppIcon(1024));
  console.log('Generated icon.iconset for macOS .icns');
}

main();
