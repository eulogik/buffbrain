const fs = require('fs');
const { createCanvas } = require('canvas');

// If canvas not available, create minimal PNG
try {
  const canvas = createCanvas(64, 64);
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#0a84ff';
  ctx.fillRect(0, 0, 64, 64);
  // Draw "CM" text
  ctx.fillStyle = 'white';
  ctx.font = 'bold 32px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('CM', 32, 32);
  const buffer = canvas.toBuffer('image/png');
  fs.writeFileSync('icon.png', buffer);
  console.log('Icon created with canvas');
} catch (e) {
  // Fallback: create a minimal valid PNG (1x1 pixel)
  const minimalPNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==', 'base64');
  fs.writeFileSync('icon.png', minimalPNG);
  console.log('Icon created (minimal fallback)');
}
