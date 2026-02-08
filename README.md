# Agent Skill Tool

Gestor de Skills para Agentes de IA (Cursor, Antigravity, VSCode Copilot)

## 🚀 Inicio Rápido para Usuarios

### Instalación y Uso

No necesitas instalar nada globalmente. Simplemente usa `npx`:

```bash
# 1. Inicializar un nuevo proyecto
npx agent-skill init

# 2. Añadir una skill
npx agent-skill add https://github.com/wshobson/agents --skill typescript

# 3. Listar skills instaladas
npx agent-skill list

# 4. Actualizar todas las skills
npx agent-skill update

# 5. Sincronizar con editores
npx agent-skill sync --editors cursor,antigravity
```

## 🔧 Desarrollo: Ciclo de Vida del Proyecto

### Estructura del Proyecto

```
agent-skill/
├── src/              # Código Rust
│   └── main.rs
├── bin/              # Wrapper Node.js
│   └── run.js
├── scripts/          # Scripts de instalación
│   └── install.js
├── Cargo.toml        # Configuración Rust
└── package.json      # Configuración NPM
```

### Paso 1: Compilar el Binario Rust

```bash
# Instalar Rust si no lo tienes
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilar en modo release
cargo build --release

# El binario estará en: target/release/agent-skill
```

### Paso 2: Crear el Release en GitHub (Primera vez)

1. **Asegúrate de que `Cargo.toml` tenga la versión `0.0.1`**
2. **Compila el binario** (paso anterior)
3. **Sube tus cambios a GitHub:**

   ```bash
   git add .
   git commit -m "Initial release v0.0.1"
   git push origin main
   ```

4. **Crea el Release en GitHub:**
   - Ve a tu repositorio en GitHub
   - Click en "Releases" → "Draft a new release"
   - Tag: `v0.0.1`
   - Title: `v0.0.1 - Initial Release`
   - **Sube los binarios compilados:**
     - `target/release/agent-skill` → Renombrar a `agent-skill-linux`
     - Para Windows: `target/release/agent-skill.exe` → Renombrar a `agent-skill-win.exe`
     - Para macOS: compilar en Mac y renombrar a `agent-skill-macos`
   - Publica la Release

### Paso 3: Probar la Descarga Localmente

Antes de publicar en NPM, prueba que el script de descarga funciona:

```bash
# Ejecutar el script de instalación manualmente
node scripts/install.js

# Verificar que se descargó el binario
ls -la bin/

# Probar el comando
node bin/run.js init
node bin/run.js list
```

### Paso 4: Publicar en NPM

```bash
# Login en NPM (primera vez)
npm login

# Publicar el paquete
npm publish
```

### Paso 5: Automatización con GitHub Actions

El archivo `.github/workflows/release.yml` automatiza la compilación y publicación:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: cargo build --release

      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./target/release/skill-cli${{ matrix.os == 'windows-latest' && '.exe' || '' }}
          asset_name: skill-cli-${{ matrix.os == 'ubuntu-latest' && 'linux' || matrix.os == 'windows-latest' && 'win.exe' || 'macos' }}
          asset_content_type: application/octet-stream
```

## 🔄 Flujo de Actualización de Versiones

1. **Actualiza la versión en ambos archivos:**
   - `Cargo.toml`: `version = "0.0.2"`
   - `package.json`: `"version": "0.0.2"`
   - `scripts/install.js`: `const VERSION = "v0.0.2"`

2. **Compila y crea el tag:**

   ```bash
   cargo build --release
   git add .
   git commit -m "Bump version to 0.0.2"
   git tag v0.0.2
   git push origin main --tags
   ```

3. **Crea el Release en GitHub** (o usa GitHub Actions)

4. **Publica en NPM:**
   ```bash
   npm publish
   ```

## 📦 Comandos Disponibles

| Comando                      | Descripción                                           |
| ---------------------------- | ----------------------------------------------------- |
| `init`                       | Inicializa un nuevo proyecto (crea `skills.toml`)     |
| `add <url> --skill <nombre>` | Añade una nueva skill                                 |
| `list`                       | Lista todas las skills instaladas                     |
| `update`                     | Actualiza todas las skills desde sus URLs             |
| `sync --editors <lista>`     | Sincroniza con editores (cursor, antigravity, vscode) |

## 🛠️ Desarrollo Local

```bash
# Compilar en modo desarrollo
cargo build

# Ejecutar directamente
cargo run -- init
cargo run -- list

# Probar el wrapper NPM
node bin/run.js init
```

## 📝 Notas Importantes

- **El binario debe estar en GitHub Releases** antes de que el script de NPM pueda descargarlo
- **La versión en `scripts/install.js`** debe coincidir con el tag de GitHub
- **Los nombres de los binarios** deben seguir el formato: `skill-cli-{platform}.exe`
  - Linux: `skill-cli-linux`
  - Windows: `skill-cli-win.exe`
  - macOS: `skill-cli-macos`

## 🤝 Contribuir

**IMPORTANTE:** Este es un proyecto de código propietario. Las contribuciones están sujetas a términos especiales:

1. Todas las contribuciones se considerarán cedidas al titular de los derechos de autor
2. Al contribuir, aceptas que tu código se licenciará bajo la misma licencia propietaria
3. No se permite el uso de contribuciones en proyectos competidores
4. Contacta con el titular de los derechos antes de realizar contribuciones significativas

Para contribuir:

1. Contacta primero para discutir los cambios propuestos
2. Fork el proyecto (solo para desarrollo autorizado)
3. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
4. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
5. Push a la rama (`git push origin feature/AmazingFeature`)
6. Abre un Pull Request (sujeto a revisión y aceptación de términos)

## 📄 Licencia

**LICENCIA PROPIETARIA - Todos los derechos reservados**

Este software está protegido por derechos de autor y se distribuye bajo una licencia propietaria restrictiva.

**Restricciones principales:**

- ❌ No se permite uso comercial sin licencia
- ❌ No se permite modificación o redistribución
- ❌ No se permite ingeniería inversa
- ✅ Uso personal y no comercial permitido

Para uso comercial o permisos especiales, consulta el archivo `LICENSE` o contacta al titular de los derechos.

Ver el archivo [LICENSE](LICENSE) para los términos completos.
