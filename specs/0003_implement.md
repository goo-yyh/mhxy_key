# 无敌小铃铛实施文档

- 文档编号：0003
- 日期：2026-06-27
- 依据文档：`specs/0002_demand.md`
- 应用名称：无敌小铃铛
- Bundle Identifier：`com.yuyuehui.mhxykey`
- 图标资产：`assets/app-icon.png`
- 目标平台：macOS、Windows

## 1. 技术方案

本项目采用 Tauri v2 + React + TypeScript + Ant Design。

核心职责划分：

1. 前端负责配置编辑、状态展示、快捷键捕获交互、导入导出入口。
2. Rust 后端负责配置持久化、快捷键注册、动作执行、托盘菜单、退出清理。
3. 全局快捷键监听只在 Rust 后端实现。
4. 鼠标键盘模拟只在 Rust 后端实现。
5. 配置文件只由 Rust 后端读写，前端不把 `localStorage` 作为配置来源。

关键依赖：

| 类型 | 依赖 | 用途 |
| --- | --- | --- |
| 桌面容器 | Tauri v2 | 跨平台桌面应用、窗口、托盘、打包。 |
| 前端 | React + TypeScript + Vite | 单页配置界面。 |
| UI | Ant Design | 表单、表格、弹窗、按钮、提示。 |
| 全局快捷键 | `tauri-plugin-global-shortcut` | 注册触发快捷键、配置启停快捷键、全局启停快捷键。 |
| 文件选择 | `tauri-plugin-dialog` | 导入、导出时选择 JSON 文件路径。 |
| 输入模拟 | `enigo` | 跨平台键盘和鼠标模拟。 |
| 原子写入 | `atomic-write-file` | 保存配置时避免写入中断导致 JSON 损坏。 |
| 序列化 | `serde`、`serde_json` | 配置模型序列化。 |
| 错误处理 | `thiserror` | 后端错误类型。 |
| ID 和时间 | `uuid`、`chrono` | 配置 ID、创建时间、更新时间。 |

实现时需要把 `enigo` 封装在项目自己的 `InputSimulator` 内部接口后面，避免第三方库 API 变化扩散到业务代码。

## 2. 初始化步骤

仓库当前已有 `specs/` 和 `assets/`，初始化应用时保留这些文件。

建议步骤：

```bash
npm create vite@latest . -- --template react-ts
npm install
npm install antd @ant-design/icons
npm install @tauri-apps/api
npm install --save-dev @tauri-apps/cli
npm run tauri init
npm run tauri add global-shortcut
npm run tauri add dialog
```

如果脚手架因为当前目录非空而拒绝初始化，则在临时目录创建 Vite + Tauri 项目，再把生成的 `package.json`、`index.html`、`src/`、`src-tauri/`、`vite.config.ts` 等文件复制回当前仓库。

Tauri 配置要求：

1. `productName` 设置为 `无敌小铃铛`。
2. `identifier` 设置为 `com.yuyuehui.mhxykey`。
3. `build.beforeDevCommand` 设置为 `npm run dev`。
4. `build.beforeBuildCommand` 设置为 `npm run build`。
5. `build.frontendDist` 设置为 `../dist`。
6. 主窗口标题设置为 `无敌小铃铛`。
7. 主窗口建议尺寸：`980 x 680`。
8. `src-tauri/Cargo.toml` 的 `tauri` 依赖启用 `tray-icon` 特性。

图标生成：

```bash
npm run tauri icon assets/app-icon.png
```

生成后确认 `src-tauri/icons/icon.ico`、`src-tauri/icons/icon.icns` 和 PNG 图标存在，并在 `tauri.conf.json` 中引用生成图标。

## 3. 目录结构

建议目录：

```text
.
├── assets/
│   └── app-icon.png
├── specs/
│   ├── 0002_demand.md
│   └── 0003_implement.md
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── api/
│   │   └── tauri.ts
│   ├── components/
│   │   ├── ActionEditor.tsx
│   │   ├── ConfigEditorModal.tsx
│   │   ├── ConfigTable.tsx
│   │   ├── GlobalToolbar.tsx
│   │   └── HotkeyInput.tsx
│   ├── hooks/
│   │   ├── useAppEvents.ts
│   │   └── useConfigStore.ts
│   ├── types/
│   │   └── config.ts
│   └── styles/
│       └── app.css
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── lib.rs
│       ├── commands.rs
│       ├── config_store.rs
│       ├── errors.rs
│       ├── executor.rs
│       ├── hotkeys.rs
│       ├── input_simulator.rs
│       ├── models.rs
│       ├── tray.rs
│       └── validation.rs
└── .github/
    └── workflows/
        └── windows-build.yml
```

## 4. 数据模型

前后端保持同一份 JSON 结构。Rust 为真实来源，前端类型从该结构手动同步。

### 4.1 配置根对象

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub global_enabled: bool,
    pub global_toggle_hotkey: Option<Hotkey>,
    pub configs: Vec<MacroConfig>,
}
```

默认值：

1. `version = 1`
2. `global_enabled = true`
3. `global_toggle_hotkey = None`
4. `configs = []`

### 4.2 单个配置

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_hotkey: Hotkey,
    pub toggle_hotkey: Option<Hotkey>,
    pub actions: Vec<Action>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 4.3 快捷键

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    pub modifiers: Vec<HotkeyModifier>,
    pub code: HotkeyCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyModifier {
    Shift,
    Control,
    Alt,
    Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyCode {
    KeyA,
    KeyB,
    KeyC,
    Digit0,
    Digit1,
    F1,
    F2,
    Escape,
    Enter,
    Tab,
    Space,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}
```

实现时先覆盖字母、数字、功能键、常用控制键和方向键。`HotkeyInput` 捕获到其他键时显示“不支持该按键”。

快捷键规范化规则：

1. `modifiers` 固定排序：`Control`、`Alt`、`Shift`、`Meta`。
2. 去重后再保存。
3. 至少包含一个 `code`。
4. `code` 不允许来自鼠标事件。
5. 冲突判断使用规范化后的 `Hotkey`。

平台展示规则：

1. macOS：`Meta` 显示为 `Command`，`Alt` 显示为 `Option`，`Control` 显示为 `Control`。
2. Windows：`Meta` 显示为 `Win`，`Alt` 显示为 `Alt`，`Control` 显示为 `Ctrl`。

### 4.4 动作

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    KeyCombo {
        keys: Hotkey,
        #[serde(default)]
        delay_after_ms: u64,
    },
    MouseClick {
        button: MouseButton,
        #[serde(default = "default_click_count")]
        click_count: u8,
        #[serde(default)]
        delay_after_ms: u64,
    },
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
```

校验规则：

1. `actions` 至少一项。
2. `MouseClick.click_count` 只能是 `1` 或 `2`。
3. `Delay.duration_ms` 范围建议为 `1..=60000`。
4. `delay_after_ms` 范围建议为 `0..=60000`。

## 5. 后端架构

### 5.1 全局状态

```rust
pub struct AppState {
    pub config_store: ConfigStore,
    pub hotkey_manager: HotkeyManager,
    pub executor: ActionExecutor,
    pub tray: TrayController,
}
```

建议使用：

1. `Arc<RwLock<AppConfig>>` 保存当前配置快照。
2. `Arc<Mutex<RuntimeState>>` 保存运行态信息。
3. `tauri::async_runtime::spawn` 启动串行动作执行 worker。
4. `mpsc` 队列传递执行请求，避免在快捷键回调中阻塞。

运行态结构：

```rust
pub struct RuntimeState {
    pub registered_hotkeys: HashMap<Hotkey, HotkeyRole>,
    pub config_errors: HashMap<String, RuntimeErrorInfo>,
    pub last_trigger_at: HashMap<String, Instant>,
    pub running_configs: HashSet<String>,
    pub suppress_shortcuts_until: Option<Instant>,
    pub shutdown_requested: bool,
}

pub enum HotkeyRole {
    Trigger { config_id: String },
    ToggleConfig { config_id: String },
    ToggleGlobal,
}
```

### 5.2 配置存储

模块：`config_store.rs`

职责：

1. 计算配置路径。
2. 启动时加载配置。
3. 保存配置。
4. 导入配置。
5. 导出配置。
6. 配置文件损坏时备份并返回空配置。

配置路径：

1. 使用 Tauri app data 目录。
2. 文件名：`config.json`。
3. 损坏备份名：`config.corrupt.<timestamp>.json`。

保存流程：

1. 调用 `validation::validate_app_config`。
2. `serde_json::to_string_pretty` 序列化。
3. 使用 `atomic-write-file` 写入目标文件。
4. 写入成功后更新内存快照。
5. 向前端发送 `config://changed` 事件。
6. 刷新托盘菜单。
7. 刷新快捷键注册。

读取流程：

1. 如果文件不存在，创建默认配置并保存。
2. 如果文件存在，读取并反序列化。
3. 如果反序列化失败，备份损坏文件，创建默认配置，并发送 `config://load_failed` 事件。

导入流程：

1. 从用户选择的 JSON 文件读取内容。
2. 反序列化为 `AppConfig`。
3. 根据导入模式执行覆盖或追加。
4. 追加模式下为冲突 `id` 生成新 UUID。
5. 对合并后的配置做完整校验。
6. 校验通过后保存。
7. 校验失败时返回错误，不修改当前配置。

### 5.3 快捷键管理

模块：`hotkeys.rs`

职责：

1. 把内部 `Hotkey` 转换为 `tauri-plugin-global-shortcut` 的 `Shortcut`。
2. 注册触发快捷键。
3. 注册配置启停快捷键。
4. 注册全局启停快捷键。
5. 注销失效快捷键。
6. 处理快捷键事件并分发到对应角色。

注册策略：

1. 全局启停快捷键只要配置存在就注册。
2. 配置启停快捷键只要配置存在就注册。
3. 配置触发快捷键只有在 `global_enabled = true` 且 `config.enabled = true` 时注册。
4. 全局总开关关闭时，只注销配置触发快捷键，保留全局启停和配置启停快捷键。
5. 单个配置关闭时，只注销该配置触发快捷键。
6. 编辑配置后统一执行差异刷新：旧快捷键不再需要则注销，新快捷键需要则注册。

事件处理：

1. 收到 `ToggleGlobal` 后调用 `set_global_enabled(!current)`。
2. 收到 `ToggleConfig` 后调用 `set_config_enabled(config_id, !current)`。
3. 收到 `Trigger` 后做防抖、运行中检查、抑制窗口检查，然后入队执行动作。

防抖：

1. 每个配置维护 `last_trigger_at`。
2. 同一配置 300 ms 内重复触发直接忽略。

错误处理：

1. 快捷键注册失败时，记录到 `RuntimeState.config_errors`。
2. 向前端发送 `hotkey://register_failed`。
3. 刷新托盘菜单和配置列表状态。

### 5.4 动作执行器

模块：`executor.rs`

职责：

1. 串行消费动作执行请求。
2. 调用 `InputSimulator` 执行键盘和鼠标动作。
3. 执行等待。
4. 支持取消。
5. 出错时释放输入状态并上报。

执行队列：

```rust
pub struct ExecutionRequest {
    pub config_id: String,
    pub actions: Vec<Action>,
    pub generation: u64,
}
```

执行流程：

1. 快捷键触发后复制当前配置动作列表。
2. 检查该配置是否已经在 `running_configs` 中。
3. 未运行则加入队列，并把配置 ID 加入 `running_configs`。
4. worker 串行取出请求。
5. 设置 `suppress_shortcuts_until = now + execution_window`。
6. 按顺序执行每个动作。
7. 每个动作前检查取消标记。
8. 动作执行成功后处理 `delay_after_ms`。
9. 完成后从 `running_configs` 移除。
10. 发送 `action://finished` 或 `action://failed` 事件。

取消策略：

1. 全局总开关关闭时递增 `generation`，队列中旧请求失效。
2. 单个配置关闭时把该配置标记为 cancelled。
3. 程序退出时设置 `shutdown_requested = true` 并清空队列。

输入抑制：

1. 动作执行期间忽略所有触发类快捷键事件。
2. 配置启停和全局启停快捷键可在执行动作之间生效。
3. 动作结束后延迟 50 ms 再解除抑制，降低模拟按键重新触发的风险。

### 5.5 输入模拟

模块：`input_simulator.rs`

接口：

```rust
pub trait InputSimulator {
    fn key_combo(&mut self, keys: &Hotkey) -> Result<(), AppError>;
    fn mouse_click(&mut self, button: MouseButton, click_count: u8) -> Result<(), AppError>;
    fn release_all(&mut self);
}
```

`EnigoInputSimulator` 实现：

1. 初始化 `Enigo::new(&Settings::default())`。
2. 键盘组合键执行顺序：
   - 按下所有 modifier。
   - 点击普通键。
   - 反向释放所有 modifier。
3. 鼠标单击：
   - 对应 `Button::Left`、`Button::Right`、`Button::Middle`。
   - 执行一次 click。
4. 鼠标双击：
   - 连续执行两次 click。
   - 两次点击之间建议间隔 40 ms。
5. 出现错误时调用 `release_all`。

`release_all` 要求：

1. 执行器维护本次动作中已经按下的 modifier。
2. 发生错误或取消时按反向顺序释放。
3. 本期没有长按动作，但仍保留该保护逻辑，避免组合键执行中断导致 modifier 残留。

### 5.6 托盘控制

模块：`tray.rs`

职责：

1. 创建托盘图标。
2. 设置托盘 tooltip：`无敌小铃铛`。
3. 构建托盘菜单。
4. 处理菜单点击。
5. 配置状态变化后重建菜单。

菜单结构：

```text
打开主窗口
全局总开关：开启/关闭
---
配置：<配置名 1>：开启/关闭
配置：<配置名 2>：开启/关闭
---
退出
```

行为：

1. 点击托盘图标恢复主窗口。
2. 右击托盘图标显示菜单。
3. 点击“打开主窗口”显示并聚焦主窗口。
4. 点击“全局总开关”调用 `set_global_enabled`。
5. 点击配置项调用 `set_config_enabled`。
6. 点击“退出”调用统一退出清理流程。

### 5.7 窗口关闭和最小化

实现位置：

1. 前端在 `App.tsx` 注册窗口事件。
2. 后端提供 `hide_main_window`、`exit_app` 命令。

最小化：

1. 用户点击最小化按钮时拦截窗口最小化事件。
2. 调用后端命令隐藏主窗口。
3. 程序继续运行，配置继续生效。

关闭：

1. 用户点击关闭按钮时执行退出。
2. 关闭请求进入后端 `exit_app`。
3. `exit_app` 调用统一清理流程。
4. 清理完成后退出 Tauri 进程。

统一退出清理流程：

1. 设置 `shutdown_requested = true`。
2. 停止接受新的动作执行请求。
3. 清空执行队列。
4. 取消正在执行的动作。
5. 注销所有全局快捷键。
6. 释放输入状态。
7. 保存最新配置。
8. 销毁托盘或让 Tauri 退出时自动清理。
9. 调用 `app.exit(0)`。

## 6. 前端架构

### 6.1 状态管理

使用轻量本地状态即可，不引入 Redux。

`useConfigStore` 维护：

1. `config: AppConfig | null`
2. `loading: boolean`
3. `saving: boolean`
4. `errors: Record<string, string>`
5. `globalError?: string`

页面初始化：

1. 调用 `get_config`。
2. 注册后端事件监听。
3. 事件触发时刷新配置或局部更新状态。

### 6.2 后端 API 封装

文件：`src/api/tauri.ts`

```ts
export async function getConfig(): Promise<AppConfig>;
export async function saveConfig(config: AppConfig): Promise<AppConfig>;
export async function createConfig(input: MacroConfigInput): Promise<AppConfig>;
export async function updateConfig(id: string, input: MacroConfigInput): Promise<AppConfig>;
export async function deleteConfig(id: string): Promise<AppConfig>;
export async function setGlobalEnabled(enabled: boolean): Promise<AppConfig>;
export async function setGlobalToggleHotkey(hotkey: Hotkey | null): Promise<AppConfig>;
export async function setConfigEnabled(id: string, enabled: boolean): Promise<AppConfig>;
export async function importConfig(path: string, mode: "replace" | "append"): Promise<AppConfig>;
export async function exportConfig(path: string): Promise<void>;
export async function testHotkey(hotkey: Hotkey): Promise<void>;
export async function hideMainWindow(): Promise<void>;
export async function exitApp(): Promise<void>;
```

### 6.3 页面组件

`App.tsx`：

1. 页面壳。
2. 加载配置。
3. 注册窗口 close/minimize 相关处理。
4. 注册后端事件。

`GlobalToolbar.tsx`：

1. 应用名称。
2. 全局总开关。
3. 全局启停快捷键。
4. 启用配置数量。
5. 新建、导入、导出按钮。

`ConfigTable.tsx`：

1. 展示配置列表。
2. 展示快捷键、动作摘要和错误状态。
3. 提供启停、编辑、删除操作。

`ConfigEditorModal.tsx`：

1. 新建和编辑共用。
2. 表单字段包括名称、启用状态、触发快捷键、配置启停快捷键、动作序列。
3. 保存前做前端基础校验，后端仍执行完整校验。

`HotkeyInput.tsx`：

1. 点击后进入捕获状态。
2. 监听 `keydown`。
3. `preventDefault` 防止浏览器默认行为。
4. 转换为内部 `Hotkey`。
5. 不处理鼠标事件。
6. 显示平台化快捷键文本。
7. 支持清空可选快捷键。

`ActionEditor.tsx`：

1. 使用列表编辑动作序列。
2. 支持新增键盘组合键、鼠标点击、等待。
3. 支持调整顺序。
4. 支持删除动作。
5. 鼠标点击提供按钮类型和点击次数选择。
6. 等待动作提供毫秒输入。
7. 每个动作提供 `delayAfterMs` 输入。

### 6.4 UI 约束

1. 页面保持单页配置工具风格。
2. 使用 Ant Design 的 `Layout`、`Card`、`Table`、`Modal`、`Form`、`Switch`、`Button`、`Tag`、`Space`、`Alert`、`Tooltip`。
3. 主界面不要做营销式首页。
4. 顶部工具条紧凑，配置表格为主要内容。
5. 错误提示短文本展示，不堆叠长日志。
6. 动作摘要格式示例：`Command + A -> 等待 100ms -> 左键单击`。

## 7. 后端命令

所有命令返回统一结果：成功时返回数据，失败时返回可序列化错误。

错误结构：

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub config_id: Option<String>,
}
```

命令列表：

| 命令 | 入参 | 返回 | 说明 |
| --- | --- | --- | --- |
| `get_config` | 无 | `AppConfig` | 读取当前内存配置。 |
| `save_config` | `AppConfig` | `AppConfig` | 完整保存配置并刷新运行态。 |
| `create_config` | `MacroConfigInput` | `AppConfig` | 创建配置。 |
| `update_config` | `id`, `MacroConfigInput` | `AppConfig` | 更新配置。 |
| `delete_config` | `id` | `AppConfig` | 删除配置。 |
| `set_global_enabled` | `enabled` | `AppConfig` | 切换全局总开关。 |
| `set_global_toggle_hotkey` | `Hotkey \| null` | `AppConfig` | 设置全局启停快捷键。 |
| `set_config_enabled` | `id`, `enabled` | `AppConfig` | 切换单配置启用状态。 |
| `import_config` | `path`, `mode` | `AppConfig` | 导入配置。 |
| `export_config` | `path` | `()` | 导出配置。 |
| `test_hotkey` | `Hotkey` | `()` | 检查格式和冲突，不持久化。 |
| `hide_main_window` | 无 | `()` | 隐藏主窗口到托盘。 |
| `exit_app` | 无 | `()` | 统一清理后退出。 |

## 8. 后端事件

后端向前端发送事件：

| 事件 | Payload | 触发时机 |
| --- | --- | --- |
| `config://changed` | `AppConfig` | 配置保存、导入、启停变化后。 |
| `config://load_failed` | `{ message: string }` | 启动读取配置失败并已恢复为空配置。 |
| `hotkey://register_failed` | `{ configId?: string, hotkey: Hotkey, message: string }` | 快捷键注册失败。 |
| `permission://required` | `{ platform: string, message: string }` | 检测到缺少权限。 |
| `action://started` | `{ configId: string }` | 动作序列开始执行。 |
| `action://finished` | `{ configId: string }` | 动作序列执行完成。 |
| `action://failed` | `{ configId: string, message: string }` | 动作执行失败。 |
| `app://exiting` | `{}` | 进入退出清理流程。 |

## 9. 校验规则

模块：`validation.rs`

`validate_app_config(config)`：

1. `version == 1`。
2. 配置 ID 不重复。
3. 配置名称非空。
4. 每个配置 `actions` 非空。
5. 每个 `Hotkey` 至少有一个普通键。
6. 鼠标按键不能出现在 `Hotkey` 中。
7. 已启用配置的 `trigger_hotkey` 不能重复。
8. 所有配置启停快捷键不能重复。
9. 全局启停快捷键不能与任意配置快捷键重复。
10. 每个配置内 `trigger_hotkey` 不能与 `toggle_hotkey` 相同。
11. `MouseClick.click_count` 只能为 1 或 2。
12. `Delay.duration_ms` 在允许范围内。
13. `delay_after_ms` 在允许范围内。

校验失败时返回结构化错误，前端根据 `config_id` 定位到具体配置行。

## 10. 权限处理

macOS：

1. 首次执行输入模拟或快捷键注册失败时，提示用户检查辅助功能、输入监控权限。
2. 错误提示文案需要说明路径：系统设置 -> 隐私与安全性 -> 辅助功能 / 输入监控。
3. 用户开启权限后，允许在页面点击“重新注册快捷键”或切换总开关来重试。

Windows：

1. 普通应用窗口按普通权限支持。
2. 快捷键被占用时展示具体快捷键冲突提示。
3. 输入模拟失败时展示动作执行失败提示。

权限判断不要阻塞应用启动。启动时先加载界面，注册失败再展示错误状态。

## 11. Windows 打包

新增文件：`.github/workflows/windows-build.yml`

以 `/Users/yuyuehui/chenglu/.github/workflows/windows-build.yml` 为参考，使用以下结构：

```yaml
name: Build Windows Package

on:
  workflow_dispatch:
  push:
    tags:
      - "v*"

jobs:
  build-windows:
    name: Build Windows installer
    runs-on: windows-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "lts/*"
          cache: "npm"

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust build
        uses: swatinem/rust-cache@v2
        with:
          workspaces: "./src-tauri -> target"

      - name: Install dependencies
        run: npm ci

      - name: Build Windows package
        run: npm run tauri:build

      - name: Upload Windows installer
        uses: actions/upload-artifact@v4
        with:
          name: windows-nsis-installer
          path: |
            src-tauri/target/release/bundle/**/*.exe
          if-no-files-found: error
          retention-days: 14
```

本期只要求 artifact 上传，不要求代码签名和自动发布 GitHub Release。

## 12. 实施顺序

### 12.1 阶段一：项目骨架

1. 初始化 Vite + React + TypeScript。
2. 初始化 Tauri v2。
3. 安装 Ant Design。
4. 配置应用名称、identifier、窗口尺寸。
5. 生成并配置应用图标。
6. 确认 `npm run build` 通过。
7. 确认 `npm run tauri dev` 能启动空应用。

验收：

1. 桌面窗口标题显示 `无敌小铃铛`。
2. 应用使用生成图标。
3. 前端构建通过。

### 12.2 阶段二：模型、配置存储、校验

1. 实现 Rust 数据模型。
2. 实现配置默认值。
3. 实现 JSON 读取和原子写入。
4. 实现配置损坏备份。
5. 实现完整校验规则。
6. 暴露 `get_config`、`save_config`。
7. 添加单元测试。

验收：

1. 首次启动自动创建默认配置。
2. 配置保存后重启仍存在。
3. 损坏 JSON 不导致程序崩溃。

### 12.3 阶段三：前端配置 UI

1. 实现主页面布局。
2. 实现配置列表。
3. 实现新建、编辑、删除弹窗。
4. 实现快捷键捕获组件。
5. 实现动作序列编辑器。
6. 对接 `create_config`、`update_config`、`delete_config`。

验收：

1. 可以创建配置。
2. 可以配置触发快捷键。
3. 可以配置键盘组合键、等待、鼠标单击、鼠标双击动作。
4. 可以编辑、删除配置。

### 12.4 阶段四：快捷键注册和启停

1. 安装并初始化 `tauri-plugin-global-shortcut`。
2. 实现 `Hotkey` 到 Tauri shortcut 的转换。
3. 实现全局总开关。
4. 实现全局启停快捷键。
5. 实现配置启停快捷键。
6. 实现注册失败错误展示。
7. 实现快捷键防抖。

验收：

1. 全局总开关开启时，已启用配置触发快捷键生效。
2. 全局总开关关闭时，配置触发快捷键不生效。
3. 配置启停快捷键可以切换单个配置。
4. 全局启停快捷键可以切换全局总开关。

### 12.5 阶段五：动作执行

1. 安装并封装 `enigo`。
2. 实现 `InputSimulator`。
3. 实现串行动作执行队列。
4. 实现键盘组合键模拟。
5. 实现鼠标单击、双击。
6. 实现等待和 `delayAfterMs`。
7. 实现取消和退出释放。
8. 实现动作事件上报。

验收：

1. `Option + A` 可以触发 `Command + A -> 等待 100ms -> 鼠标左键单击`。
2. 同一配置运行中重复触发会被忽略。
3. 关闭配置会停止后续动作。
4. 程序退出前释放输入状态。

### 12.6 阶段六：托盘和窗口行为

1. 启用 Tauri `tray-icon` 特性。
2. 实现托盘图标和菜单。
3. 实现托盘打开主窗口。
4. 实现托盘切换全局总开关。
5. 实现托盘切换单个配置。
6. 实现最小化隐藏到托盘。
7. 实现关闭窗口退出程序。
8. 实现统一退出清理流程。

验收：

1. 最小化后窗口隐藏，托盘仍存在。
2. 托盘可以恢复窗口。
3. 托盘菜单可以切换状态。
4. 点击关闭后程序退出且快捷键失效。

### 12.7 阶段七：导入导出

1. 安装并初始化 `tauri-plugin-dialog`。
2. 前端使用保存文件对话框选择导出路径。
3. 前端使用打开文件对话框选择导入路径。
4. 后端实现导出 JSON。
5. 后端实现覆盖导入。
6. 后端实现追加导入。
7. 导入失败时展示错误并保持当前配置不变。

验收：

1. 可以导出 JSON。
2. 可以覆盖导入 JSON。
3. 可以追加导入 JSON。
4. 导入非法 JSON 不破坏当前配置。

### 12.8 阶段八：Windows 打包

1. 新增 `.github/workflows/windows-build.yml`。
2. 确认本地 `npm run build` 通过。
3. 确认本地 Rust 测试通过。
4. 推送 tag 或手动触发 workflow。
5. 下载并验证 `.exe` artifact。

验收：

1. workflow 可以手动触发。
2. `v*` tag 可以触发。
3. artifact 中存在 Windows 安装包。

## 13. 测试计划

### 13.1 Rust 单元测试

覆盖：

1. `Hotkey` 规范化。
2. 快捷键冲突检测。
3. 鼠标按键不能作为触发快捷键。
4. 动作序列不能为空。
5. 鼠标点击次数只能为 1 或 2。
6. 配置序列化、反序列化。
7. 导入覆盖。
8. 导入追加并处理 ID 冲突。
9. 损坏配置文件备份。

命令：

```bash
cd src-tauri
cargo test
```

### 13.2 前端检查

覆盖：

1. TypeScript 类型检查。
2. 前端构建。
3. 主要组件交互手工验证。

命令：

```bash
npm run build
```

### 13.3 端到端手工验证

macOS：

1. 启动应用。
2. 创建配置：触发 `Option + A`。
3. 动作：`Command + A`、等待 `100 ms`、左键单击。
4. 验证触发效果。
5. 验证全局总开关。
6. 验证配置启停快捷键。
7. 验证全局启停快捷键。
8. 验证最小化到托盘。
9. 验证关闭退出后快捷键不再生效。

Windows：

1. 安装 GitHub Actions 产出的 `.exe`。
2. 创建配置：触发 `Alt + A`。
3. 动作：`Ctrl + A`、等待 `100 ms`、左键单击。
4. 验证触发效果。
5. 验证托盘菜单。
6. 验证导入导出。

## 14. 风险和处理

| 风险 | 影响 | 处理 |
| --- | --- | --- |
| macOS 权限不足 | 快捷键或输入模拟失败 | 注册或执行失败时展示权限提示，用户授权后重试。 |
| 快捷键被系统占用 | 某配置不可用 | 配置级错误展示，不影响其他配置。 |
| 模拟输入再次触发快捷键 | 可能导致递归执行 | 执行动作期间设置 `suppress_shortcuts_until`，触发事件先检查抑制窗口。 |
| 输入模拟中断 | modifier 可能残留 | 执行器维护已按下按键，错误、取消、退出时释放。 |
| `enigo` API 变化 | 后续维护成本上升 | 通过 `InputSimulator` trait 隔离第三方 API。 |
| 配置文件写入中断 | 配置损坏 | 使用 `atomic-write-file` 原子写入，读取失败时备份损坏文件。 |
| 托盘菜单状态不同步 | 用户看到旧状态 | 每次配置变化、注册结果变化后重建托盘菜单。 |

## 15. 完成标准

完成实现时需要满足：

1. `npm run build` 通过。
2. `cd src-tauri && cargo test` 通过。
3. `npm run tauri dev` 可以正常启动。
4. 需求文档 `0002` 中所有验收项均通过手工验证。
5. Windows workflow 可以产出 `.exe` artifact。
6. 退出程序后，原先注册的快捷键全部失效。

## 16. 参考资料

1. [Tauri Global Shortcut 插件](https://v2.tauri.app/plugin/global-shortcut/)
2. [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
3. [Tauri Dialog 插件](https://v2.tauri.app/plugin/dialog/)
4. [Tauri GitHub Actions 打包](https://v2.tauri.app/distribute/pipelines/github/)
5. [Tauri App Icons](https://v2.tauri.app/develop/icons/)
6. [Enigo 输入模拟 crate](https://docs.rs/enigo/latest/enigo/)
7. [atomic-write-file crate](https://docs.rs/atomic-write-file/latest/atomic_write_file/)
