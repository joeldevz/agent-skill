#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

const extension = os.platform() === 'win32' ? '.exe' : '';
const binPath = path.join(__dirname, `skill-cli${extension}`);

if (!require('fs').existsSync(binPath)) {
    console.error('❌ El binario no se encuentra. Intenta ejecutar: npm install --force');
    process.exit(1);
}

// Ejecutar el binario y pasarle todos los argumentos
const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });

child.on('close', (code) => {
    process.exit(code);
});