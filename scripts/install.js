const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Configuración
const REPO = "joeldevz/agent-skill";
const VERSION = "v0.0.5"; // ¡CAMBIA ESTO PARA QUE COINCIDA CON TU TAG DE GITHUB!
const BIN_NAME = "skillctl";

// Detectar plataforma
const platform = os.platform(); // 'darwin', 'linux', 'win32'
const arch = os.arch(); // 'x64', 'arm64'

let urlPlatform = "";
let extension = "";

if (platform === 'win32') {
    urlPlatform = 'win';
    extension = '.exe';
} else if (platform === 'darwin') {
    urlPlatform = 'macos';
} else if (platform === 'linux') {
    urlPlatform = 'linux';
} else {
    console.error('Sistema operativo no soportado');
    process.exit(1);
}

// Construir URL de descarga (GitHub Releases)
const fileName = `${BIN_NAME}-${urlPlatform}${extension}`;
const downloadUrl = `https://github.com/${REPO}/releases/download/${VERSION}/${fileName}`;

const binDir = path.join(__dirname, '..', 'bin');
if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir);
}

const destPath = path.join(binDir, `skillctl${extension}`);

console.log(`⬇️  Descargando ${BIN_NAME} desde: ${downloadUrl}`);

async function download() {
    try {
        const response = await axios({
            method: 'get',
            url: downloadUrl,
            responseType: 'stream'
        });

        const writer = fs.createWriteStream(destPath);
        response.data.pipe(writer);

        return new Promise((resolve, reject) => {
            writer.on('finish', () => {
                // Dar permisos de ejecución en Linux/Mac
                if (platform !== 'win32') {
                    fs.chmodSync(destPath, 0o755);
                }
                console.log('✅ Instalación completada.');
                resolve();
            });
            writer.on('error', reject);
        });
    } catch (error) {
        console.error('❌ Error descargando el binario. Asegúrate de que la Release exista en GitHub.');
        console.error(error.message);
        process.exit(1);
    }
}

download();