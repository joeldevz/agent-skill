# Ejemplo de Uso: Skill CLI Tool

## Escenario: Configurar un Proyecto Nuevo

### 1. Inicializar el Proyecto

```bash
# Crear una nueva carpeta para tu proyecto
mkdir mi-proyecto-ia
cd mi-proyecto-ia

# Inicializar el gestor de skills
npx skill-cli-tool init
```

**Resultado:**

```
✅ Proyecto inicializado. Se ha creado 'skills.toml'.
🚀 Prueba ahora: npx skill-cli add <url> --skill <nombre>
```

**Archivos creados:**

- `skills.toml` - Manifiesto de skills
- `.cursor/skills/` - Carpeta para skills descargadas

### 2. Añadir Skills

```bash
# Añadir skill de TypeScript
npx skill-cli-tool add https://github.com/wshobson/agents --skill typescript

# Añadir skill de Python
npx skill-cli-tool add https://github.com/wshobson/agents --skill python

# Añadir skill personalizada
npx skill-cli-tool add https://github.com/tu-usuario/tu-repo --skill custom-skill
```

### 3. Ver Skills Instaladas

```bash
npx skill-cli-tool list
```

**Resultado:**

```
📦 Skills instaladas (2):
  • typescript (https://github.com/wshobson/agents)
    └─ Branch: main | Path: .cursor/skills/typescript/SKILL.md
  • python (https://github.com/wshobson/agents)
    └─ Branch: main | Path: .cursor/skills/python/SKILL.md
```

### 4. Sincronizar con Editores

```bash
# Sincronizar con Cursor
npx skill-cli-tool sync --editors cursor

# Sincronizar con múltiples editores
npx skill-cli-tool sync --editors cursor,antigravity,vscode
```

**Archivos generados:**

- `.cursorrules` - Configuración para Cursor
- `.antigravity` - Configuración para Antigravity
- `.github/copilot-instructions.md` - Instrucciones para GitHub Copilot

### 5. Actualizar Skills

```bash
# Actualizar todas las skills a sus últimas versiones
npx skill-cli-tool update
```

**Resultado:**

```
🔄 Buscando actualizaciones para 2 skills...
   ⬇️ Actualizando typescript...
   ⬇️ Actualizando python...
✅ Todas las skills están al día.
✅ .cursorrules actualizado.
✅ .antigravity actualizado.
```

## Estructura del Proyecto Resultante

```
mi-proyecto-ia/
├── .cursor/
│   └── skills/
│       ├── typescript/
│       │   └── SKILL.md
│       └── python/
│           └── SKILL.md
├── .github/
│   └── copilot-instructions.md
├── .cursorrules
├── .antigravity
└── skills.toml
```

## Contenido de `skills.toml`

```toml
# Manifiesto de Skills
version = "1.0"

[skills.typescript]
url = "https://github.com/wshobson/agents"
branch = "main"
local_path = ".cursor/skills/typescript/SKILL.md"
last_updated = "2026-02-08T22:30:00Z"

[skills.python]
url = "https://github.com/wshobson/agents"
branch = "main"
local_path = ".cursor/skills/python/SKILL.md"
last_updated = "2026-02-08T22:30:00Z"
```

## Integración con Cursor

Una vez sincronizado, Cursor automáticamente:

1. Lee el archivo `.cursorrules`
2. Carga las skills referenciadas
3. Usa las instrucciones de las skills en sus respuestas

**Ejemplo de `.cursorrules` generado:**

```markdown
# Rules generadas por Skill-CLI

## Skill: typescript

Reference: .cursor/skills/typescript/SKILL.md

## Skill: python

Reference: .cursor/skills/python/SKILL.md
```

## Flujo de Trabajo Diario

```bash
# Mañana: Actualizar skills
npx skill-cli-tool update

# Durante el día: Añadir nueva skill si es necesario
npx skill-cli-tool add <url> --skill <nombre>

# Verificar configuración
npx skill-cli-tool list
```

## Compartir Configuración con el Equipo

1. **Commitear `skills.toml`** al repositorio:

   ```bash
   git add skills.toml
   git commit -m "Add skill configuration"
   git push
   ```

2. **Otros miembros del equipo** solo necesitan:
   ```bash
   git pull
   npx skill-cli-tool update  # Descarga todas las skills del manifiesto
   ```

## Troubleshooting

### Problema: "El binario no se encuentra"

```bash
# Solución: Reinstalar el paquete
npm install --force skill-cli-tool
```

### Problema: "No se encontró skills.toml"

```bash
# Solución: Inicializar el proyecto
npx skill-cli-tool init
```

### Problema: Skills desactualizadas

```bash
# Solución: Forzar actualización
npx skill-cli-tool update
```

## Casos de Uso Avanzados

### Usar diferentes branches

```bash
# Editar skills.toml manualmente
[skills.typescript-beta]
url = "https://github.com/wshobson/agents"
branch = "beta"  # <-- Cambiar branch
local_path = ".cursor/skills/typescript-beta/SKILL.md"
last_updated = "2026-02-08T22:30:00Z"

# Actualizar
npx skill-cli-tool update
```

### Crear skills personalizadas

1. Crear un repositorio con la estructura:

   ```
   mi-skill/
   └── SKILL.md
   ```

2. Añadir la skill:
   ```bash
   npx skill-cli-tool add https://github.com/mi-usuario/mi-skill --skill mi-skill
   ```

## Próximos Pasos

- Explorar skills disponibles en GitHub
- Crear tus propias skills personalizadas
- Compartir configuración con tu equipo
- Automatizar actualizaciones en CI/CD
