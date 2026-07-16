# Capability: clipboard-monitor

## Purpose

TBD

## Requirements

### Requirement: 剪贴板监控 SHALL 消除真实缺陷与重复记录
剪贴板监控 MUST 正确处理 HTML/Text 双通道 echo,确保同一份内容只入数据库一次;
同时 MUST 不在数据库锁持有期间 emit 事件;同时 MUST 不破坏前端正在进行的搜索。

#### Scenario: HTML 复制时不会同时记录为 text
- **WHEN** 用户复制一段 HTML 内容(monitor 通过 `clipboard.get().html()` 与后续 `clipboard.get_text()` 收到两条)
- **THEN** 数据库只保留一条 HTML 项,不会留下重复的纯文本条目

#### Scenario: clipboard-changed 事件不会破坏正在进行的搜索
- **WHEN** 用户在搜索框内有非空查询词,后台 monitor 检测到新剪贴内容并通过 `clipboard-changed` 事件通知前端
- **THEN** 前端搜索结果列表不会被全部历史 `loadHistory()` 调用覆盖

#### Scenario: 数据库锁不会跨 emit 调用持有
- **WHEN** monitor 线程写入新剪贴项并 emit `clipboard-changed`
- **THEN** emit 调用发生时数据库 Mutex 已释放,前端 invoke 不会被阻塞

### Requirement: 关闭主窗口的行为 SHALL 符合 minimize_to_tray 开关
`settings.minimize_to_tray = true` 时关闭按钮 SHALL 隐藏窗口且不退出;`false` 时 SHALL 退出进程。

#### Scenario: 用户点击关闭按钮且 minimize_to_tray=true
- **WHEN** 用户点击主窗口标题栏右侧的 ✕ 关闭按钮,且 `settings.minimize_to_tray = true`
- **THEN** 窗口隐藏,程序继续在后台运行,数据库与剪贴板监控不被终止

#### Scenario: 用户点击关闭按钮且 minimize_to_tray=false
- **WHEN** 用户点击主窗口标题栏右侧的 ✕ 关闭按钮,且 `settings.minimize_to_tray = false`
- **THEN** 应用正常退出,托盘图标与全局快捷键随之释放

### Requirement: 设置加载与持久化逻辑 SHALL 单一可信
`Settings` 字段加载与持久化 MUST 经过同一处实现;`sync_server` 清空 MUST 真正清空 DB 中的旧值。

#### Scenario: `get_settings` 命令返回与 `load_settings` 一致的结果
- **WHEN** 前端调用 `get_settings` 命令读取当前设置
- **THEN** 返回的 Settings 与 `Database::load_settings()` 的字段映射完全一致,不存在字段 fallback 漂移

#### Scenario: 清空 sync_server 时持久化的旧值被清除
- **WHEN** 用户在前端将 `settings.sync_server` 改为 `null` 并保存
- **THEN** 数据库 settings 表中 `sync_server` 行被清空;reload 设置时该字段为 `None`

### Requirement: 剪贴历史裁剪 SHALL 分批删除
`Database::prune_history` MUST 分批 DELETE 旧项,避免单事务过大。

#### Scenario: 把 max_history_size 从 5000 调到 50
- **WHEN** 用户保存新的 `max_history_size` 值,旧项数量远大于新上限
- **THEN** 删除过程分批进行(每批 ≤ 200 行),数据库事务时间可控

### Requirement: HTTP 调用 SHALL 共享客户端
所有对外 HTTP 请求 MUST 复用全局 `reqwest::Client`,不再各自构造 `Client::builder()`。

#### Scenario: `sync_http` 与 `check_update` 各自调用
- **WHEN** 后端触发同步或检查更新
- **THEN** 请求都通过共享 `HTTP_CLIENT` 实例发出

### Requirement: SyncStatus.status SHALL 使用枚举
`sync` 模块的 `SyncStatus.status` 字段 SHALL 使用定义良好的 enum 替代裸字符串字面量。

#### Scenario: trigger_sync 状态变更使用 enum
- **WHEN** 同步过程推进到 `syncing` / `synced` / `error` 等节点
- **THEN** 状态写入使用 enum,序列化输出仍是 JSON 字符串以兼容现有 IPC 契约

### Requirement: 死代码 SHALL 被清理
未被引用的依赖、样式与重复命令 MUST 被移除。

#### Scenario: Cargo.toml 中的 `dirs` 依赖被移除
- **WHEN** 实施清理任务
- **THEN** Cargo.toml 不再包含 `dirs = "5"`,`cargo build` 通过

#### Scenario: 前端从未调用的 `pasteItem` 被移除
- **WHEN** 实施清理任务
- **THEN** `src/services/api.ts` 不再 export `pasteItem`,`src/store/index.ts` 不再有 `pasteItem` 方法

#### Scenario: 未被引用的 CSS `!important` 覆盖被移除
- **WHEN** 实施清理任务
- **THEN** `src/index.css` 的死样式段被移除,且不影响现有组件外观

### Requirement: 文档描述 SHALL 与实现一致
`AGENTS.md` 中关于剪贴板轮询节流的描述 MUST 与 `clipboard/mod.rs::start` 的实际行为一致。

#### Scenario: AGENTS.md 节流描述校正
- **WHEN** 用户或 LLM 阅读 `AGENTS.md`
- **THEN** 描述与 `clipboard/mod.rs` 的 `if got_content { 200ms } else { 500ms }` 实际行为匹配