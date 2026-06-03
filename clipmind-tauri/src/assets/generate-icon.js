const fs = require('fs');
const { createCanvas } = require('canvas');

// Create a proper 64x64 icon for the app
async function createIcon() {
  try {
    // Try to use canvas module
    const canvas = createCanvas(64, 64);
    const ctx = canvas.getContext('2d');
    
    // Background circle
    ctx.fillStyle = '#0a84ff';
    ctx.beginPath();
    ctx.arc(32, 32, 30, 0, Math.PI * 2);
    ctx.fill();
    
    // Draw "CM" text
    ctx.fillStyle = 'white';
    ctx.font = 'bold 28px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('CM', 32, 32);
    
    const buffer = canvas.toBuffer('image/png');
    fs.writeFileSync('icon.png', buffer);
    console.log('Icon created successfully at assets/icon.png');
  } catch (e) {
    console.log('Canvas not available, creating minimal icon...');
    // Create a minimal but valid 16x16 PNG icon using raw bytes
    // This is a minimal 16x16 PNG with a blue square
    const png = Buffer.from(
      'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
      'base64'
    );
    fs.writeFileSync('icon.png', png);
    console.log('Minimal icon created');
  }
}

createIcon();
