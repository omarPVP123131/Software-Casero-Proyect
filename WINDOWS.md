# Compilar y probar en Windows

PhotoLab es una app **Tauri 2**: el motor de física es Rust nativo y la
interfaz se dibuja en el WebView del sistema. En Windows eso significa
**WebView2** (el motor de Edge), presente en Windows 10/11 al día.

## Opción A — compilar tú mismo (recomendado para revisar)

Requisitos, una sola vez:

1. **Visual Studio Build Tools** con la carga de trabajo
   *Desarrollo para el escritorio con C++* (trae MSVC y el Windows SDK).
2. **Rust**: descarga `rustup-init.exe` de <https://rustup.rs>. Si ya lo
   tenías instalado y el toolchain activo es `*-gnu`, cambia al MSVC:
   `rustup default stable-x86_64-pc-windows-msvc`. La toolchain gnu falla al
   enlazar (`export ordinal too large`).
3. **Node.js 20** LTS.
4. **WebView2 Runtime**: si no lo tienes,
   <https://developer.microsoft.com/microsoft-edge/webview2/>.

Con eso instalado, desde la raíz del repositorio:

```powershell
cargo install wasm-pack            # solo la primera vez
rustup target add wasm32-unknown-unknown
cd frontend
npm ci
npm run build:wasm                 # genera el paquete de física WASM (funciona en Windows)
npm run gen:types                  # opcional: regenera tipos TS
cd ..
frontend\node_modules\.bin\tauri build      # ~5-10 min la primera vez
```

> `tauri build` **debe** lanzarse desde la raíz del repositorio (no desde
> `frontend/`), porque `src-tauri/` y `frontend/` son hermanos. Si usas
> PowerShell, `.\frontend\node_modules\.bin\tauri.cmd build` también vale.

> La feature `custom-protocol` de `src-tauri/Cargo.toml` es la que incrusta
> la interfaz en el ejecutable. No compiles con `cargo build --release` a
> secas: obtendrías un binario que busca `http://localhost:1420` y abre en
> blanco. `tauri build` la activa automáticamente.

Salidas:

- `target\release\photolab.exe` — ejecutable suelto.
- `target\release\bundle\msi\PhotoLab_2.0.0_x64_en-US.msi` — instalador MSI.
- `target\release\bundle\nsis\PhotoLab_2.0.0_x64-setup.exe` — instalador NSIS
  (descarga WebView2 si falta).

Para desarrollo con recarga en caliente:

```powershell
frontend\node_modules\.bin\tauri dev
```

## Opción B — dejar que lo compile GitHub

El workflow `.github/workflows/windows.yml` hace exactamente lo anterior en
`windows-latest`. Se lanza a mano (Actions → Windows → Run workflow) o al
empujar un tag `v*`. Al terminar, el artefacto `photolab-windows` trae el MSI
y el instalador NSIS.

## Qué revisar en Windows

- La ventana abre con el panel izquierdo **sin scroll**: el conmutador
  «Detalle · auto» elige el modo compacto por debajo de 900 px de alto.
- Los iconos del panel son SVG (trazo uniforme), ya no glifos de fuente.
- `?` abre la lista de atajos; `Esc` la cierra.
- Al pasar el ratón por el diagrama de energías aparece la explicación de
  Φ, E y Kmax con los valores en vivo.
- **Arrastrar los bordes** de los paneles laterales debe funcionar y
  recordarse entre sesiones.
- La barra de estado debe decir `motor OK` y, en la pestaña Celda, los
  electrones animarse a ~60 fps.

### Si algo falla

| Síntoma | Causa probable |
|---|---|
| Ventana en blanco | Falta WebView2 Runtime |
| `link.exe not found` | Faltan las Visual Studio Build Tools (C++) |
| Pantalla negra y `motor OK` ausente | El `.wasm` no se generó: ejecuta `npm run build:wasm` |
| Texto borroso | Escalado de Windows a 125 %; la UI es responsive, prueba a cambiar el zoom con `A+` |

## Diferencia conocida entre Linux y Windows

En Linux la app usa `webkit2gtk`; en Windows, WebView2. El motor de física es
el mismo binario Rust compilado nativo, así que los **números deben coincidir
exactamente**. Si ves diferencias de tipografía o de redondeo de 1 px en el
layout, son del motor de render, no del modelo físico.
