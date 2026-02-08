# Guía de Bootstrap: Primera Publicación

Esta guía te ayudará a publicar la primera versión del proyecto y resolver el problema del "huevo y la gallina".

## 🎯 Objetivo

Publicar la versión `v0.0.1` del proyecto para que los usuarios puedan ejecutar:

```bash
npx skill-cli-tool init
```

## 📋 Pre-requisitos

- [x] Cuenta de GitHub
- [x] Cuenta de NPM (crear en https://www.npmjs.com/signup)
- [x] Rust instalado (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- [x] Node.js instalado
- [x] Git configurado

## 🚀 Pasos para la Primera Publicación

### Paso 1: Compilar el Binario Rust

```bash
# Asegúrate de estar en la raíz del proyecto
cd /home/clasing/proyects/umibu/agent-skill

# Compilar en modo release
cargo build --release

# Verificar que el binario se creó correctamente
ls -lh target/release/skill-cli
```

**Resultado esperado:**

```
-rwxr-xr-x 1 user user 3.2M Feb  8 22:30 target/release/skill-cli
```

### Paso 2: Probar el Binario Localmente

```bash
# Ejecutar el binario directamente
./target/release/skill-cli --help

# Probar el comando init
./target/release/skill-cli init

# Verificar que se creó skills.toml
cat skills.toml
```

### Paso 3: Preparar el Repositorio

```bash
# Asegúrate de que todos los cambios están commiteados
git status

# Si hay cambios pendientes:
git add .
git commit -m "Prepare for v0.0.1 release"

# Subir a GitHub
git push origin main
```

### Paso 4: Crear el Release en GitHub (CRÍTICO)

Este es el paso más importante. **Sin este paso, el script de NPM no podrá descargar el binario.**

#### Opción A: Manual (Recomendado para la primera vez)

1. Ve a tu repositorio: https://github.com/joeldevz/agent-skill
2. Click en "Releases" (en la barra lateral derecha)
3. Click en "Draft a new release"
4. Completa el formulario:
   - **Tag version:** `v0.0.1`
   - **Release title:** `v0.0.1 - Initial Release`
   - **Description:**

     ````markdown
     ## 🎉 Primera versión de Skill CLI Tool

     Gestor de Skills para Agentes de IA.

     ### Características

     - ✅ Comando `init` para inicializar proyectos
     - ✅ Comando `list` para listar skills
     - ✅ Soporte para Cursor, Antigravity y VSCode

     ### Instalación

     ```bash
     npx skill-cli-tool init
     ```
     ````

     ```

     ```

5. **IMPORTANTE: Subir los binarios**
   - Click en "Attach binaries by dropping them here or selecting them"
   - Sube el archivo: `target/release/skill-cli`
   - **RENOMBRA el archivo a:** `skill-cli-linux` (sin extensión)

   > **Nota:** Para Windows y macOS, necesitarás compilar en esas plataformas o usar GitHub Actions (ver Opción B)

6. Click en "Publish release"

#### Opción B: Automático con GitHub Actions

```bash
# Crear y subir el tag
git tag v0.0.1
git push origin v0.0.1
```

Esto activará el workflow `.github/workflows/release.yml` que:

1. Compilará el binario en Linux, Windows y macOS
2. Subirá automáticamente los binarios al release

**Espera a que termine el workflow** (ve a la pestaña "Actions" en GitHub)

### Paso 5: Verificar que el Release está Correcto

1. Ve a: https://github.com/joeldevz/agent-skill/releases/tag/v0.0.1
2. Verifica que aparezcan los binarios:
   - `skill-cli-linux`
   - `skill-cli-win.exe` (si usaste GitHub Actions)
   - `skill-cli-macos` (si usaste GitHub Actions)

3. **Prueba la URL de descarga:**

   ```bash
   # Debería descargar el binario (no dar error 404)
   curl -L https://github.com/joeldevz/agent-skill/releases/download/v0.0.1/skill-cli-linux -o test-binary

   # Verificar que se descargó
   ls -lh test-binary

   # Limpiar
   rm test-binary
   ```

### Paso 6: Probar el Script de Instalación NPM

```bash
# Probar el script de descarga
node scripts/install.js
```

**Resultado esperado:**

```
⬇️  Descargando skill-cli desde: https://github.com/joeldevz/agent-skill/releases/download/v0.0.1/skill-cli-linux
✅ Instalación completada.
```

**Verificar:**

```bash
ls -lh bin/skill-cli
# Debería mostrar el binario descargado
```

### Paso 7: Probar el Wrapper Completo

```bash
# Probar el wrapper de Node.js
node bin/run.js --help
node bin/run.js init
node bin/run.js list
```

### Paso 8: Publicar en NPM

```bash
# Login en NPM (primera vez)
npm login
# Introduce tu usuario, contraseña y email

# Verificar que estás logueado
npm whoami

# Publicar el paquete
npm publish
```

**Resultado esperado:**

```
+ skill-cli-tool@0.0.1
```

### Paso 9: Probar la Instalación desde NPM

```bash
# Crear una carpeta de prueba
mkdir /tmp/test-skill-cli
cd /tmp/test-skill-cli

# Probar con npx (sin instalar)
npx skill-cli-tool init

# Verificar que funcionó
ls -la
# Debería mostrar skills.toml y .cursor/
```

## ✅ Checklist de Verificación

- [ ] Binario compilado correctamente
- [ ] Release v0.0.1 creado en GitHub
- [ ] Binarios subidos al release (al menos Linux)
- [ ] URL de descarga funciona (no da 404)
- [ ] Script `install.js` descarga correctamente
- [ ] Wrapper `run.js` ejecuta el binario
- [ ] Publicado en NPM
- [ ] `npx skill-cli-tool init` funciona

## 🐛 Troubleshooting

### Error: "404 Not Found" al descargar

**Causa:** El release no existe o los binarios no están subidos.

**Solución:**

1. Verifica que el release existe: https://github.com/joeldevz/agent-skill/releases
2. Verifica que el tag es exactamente `v0.0.1`
3. Verifica que el binario se llama exactamente `skill-cli-linux`

### Error: "Permission denied" al ejecutar el binario

**Causa:** El binario no tiene permisos de ejecución.

**Solución:**

```bash
chmod +x bin/skill-cli
```

### Error: "Package name already exists" en NPM

**Causa:** El nombre `skill-cli-tool` ya está tomado.

**Solución:**

1. Cambia el nombre en `package.json`:
   ```json
   "name": "@tu-usuario/skill-cli-tool"
   ```
2. Publica de nuevo:
   ```bash
   npm publish --access public
   ```

### Error: "GITHUB_TOKEN" en GitHub Actions

**Causa:** El token no tiene permisos suficientes.

**Solución:**

1. Ve a Settings → Actions → General
2. En "Workflow permissions", selecciona "Read and write permissions"
3. Guarda y vuelve a ejecutar el workflow

## 🎉 ¡Éxito!

Si llegaste aquí, tu proyecto está publicado y funcionando. Los usuarios ahora pueden ejecutar:

```bash
npx skill-cli-tool init
```

## 📚 Próximos Pasos

1. **Implementar el comando `add`** para añadir skills
2. **Añadir tests** para asegurar calidad
3. **Mejorar la documentación** con más ejemplos
4. **Crear un video tutorial** de uso
5. **Compartir en redes sociales** y comunidades

## 🔄 Para Futuras Versiones

Cuando quieras publicar `v0.0.2`:

1. Actualiza las versiones:
   - `Cargo.toml`: `version = "0.0.2"`
   - `package.json`: `"version": "0.0.2"`
   - `scripts/install.js`: `const VERSION = "v0.0.2"`

2. Compila y publica:
   ```bash
   cargo build --release
   git add .
   git commit -m "Bump version to 0.0.2"
   git tag v0.0.2
   git push origin main --tags
   npm publish
   ```

El workflow de GitHub Actions se encargará del resto automáticamente.
