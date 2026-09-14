//! ipc_contract.rs — 前端与 Rust 之间的 IPC 契约测试。
//!
//! # 为什么必须有这个测试
//!
//! Tauri 的 IPC 是**字符串寻址**的：`invoke("save_global_rules")` 与
//! `#[tauri::command] fn save_global_rules` 之间没有任何编译期联系。
//!
//! 于是下列改动会让 `cargo test` 与 `pnpm build` **双双通过**，
//! 却在用户点下按钮的那一刻才失败：
//!
//! | 改动 | 运行期表现 |
//! |:---|:---|
//! | Rust 命令改名，前端没改 | 按钮点了没反应 |
//! | 事件名两侧不一致 | 同步「卡住」，进度永远不出现 |
//! | `Serialize` 字段改名 | 界面上一片空白或 `undefined` |
//!
//! 这几种故障的共同点是**没有任何报错**，排查成本极高。
//! 本测试把两侧的真实源码都读进来做交叉比对，把它们提前到编译期。
//!
//! 注意这里比对的是**源码文本**而不是手写清单——手写清单本身就是
//! 「两处各写一份」，正是要防的东西。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// 读取仓库内文件（路径相对 `src-tauri/` 的上级，即项目根）。
fn repo_file(rel: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 {} 失败：{e}", path.display()))
}

/// 剥掉 `//` 与 `/** */` 行注释。
///
/// 必要之举：`api.ts` 的文档注释里就写着 ``invoke("xxx")`` 作为说明，
/// 不剥掉的话会被当成真实调用抓进来。
fn strip_line_comments(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            !t.starts_with("//") && !t.starts_with('*')
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `lib.rs` 中 `invoke_handler!` 里注册的命令名。
fn registered_commands(lib_rs: &str) -> BTreeSet<String> {
    let start = lib_rs
        .find("generate_handler![")
        .expect("lib.rs 中找不到 generate_handler! —— 命令注册块可能被改名了");
    let rest = &lib_rs[start..];
    let end = rest.find(']').expect("generate_handler! 未闭合");

    let mut names = BTreeSet::new();
    for segment in rest[..end].split("commands::").skip(1) {
        let name: String = segment
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            names.insert(name);
        }
    }
    names
}

/// `api.ts` 中实际发出的 `invoke("...")` 命令名。
fn invoked_commands(api_ts: &str) -> BTreeSet<String> {
    let code = strip_line_comments(api_ts);
    let re = regex::Regex::new(r#"invoke\s*(?:<[^>]*>)?\s*\(\s*"([a-z0-9_]+)""#).unwrap();
    re.captures_iter(&code)
        .map(|c| c[1].to_string())
        .collect()
}

/// `api.ts` 中订阅的事件名。
fn subscribed_events(api_ts: &str) -> BTreeSet<String> {
    let code = strip_line_comments(api_ts);
    let re = regex::Regex::new(r#"listen\s*(?:<[^>]*>)?\s*\(\s*"([^"]+)""#).unwrap();
    re.captures_iter(&code)
        .map(|c| c[1].to_string())
        .collect()
}

/// 前端调用的每个命令都必须在 Rust 侧注册过。
///
/// 这是单向断言（不要求相等）：Rust 侧可以多注册暂时没用的命令，
/// 但前端**绝不能**调用一个不存在的命令。
#[test]
fn every_invoked_command_is_registered() {
    let registered = registered_commands(&repo_file("src-tauri/src/lib.rs"));
    let invoked = invoked_commands(&repo_file("src/api.ts"));

    assert!(
        !invoked.is_empty(),
        "没有从 api.ts 中解析到任何 invoke 调用——断言会永远为真，等于没测"
    );

    let missing: Vec<&String> = invoked.difference(&registered).collect();
    assert!(
        missing.is_empty(),
        "前端调用了未注册的命令 {missing:?}\n已注册：{registered:?}\n已调用：{invoked:?}"
    );
}

/// 同步进度事件名两侧必须完全一致。
///
/// 不一致的后果是「同步看起来卡住了」——进度事件发到无人订阅的频道上，
/// 前后端都不会报错。
#[test]
fn sync_progress_event_name_matches() {
    let frontend = subscribed_events(&repo_file("src/api.ts"));
    assert_eq!(
        frontend.len(),
        1,
        "api.ts 中的事件订阅数量与预期不符：{frontend:?}"
    );
    assert_eq!(
        frontend.iter().next().unwrap(),
        crossbrain_lib::sync::SYNC_PROGRESS_EVENT,
        "前端订阅的事件名与 Rust 侧常量不一致"
    );
}

/// DTO 字段名必须两侧对齐（Rust 用 snake_case，经 serde 转成 camelCase）。
#[test]
fn dto_fields_are_mirrored_on_both_sides() {
    let rust_source = format!(
        "{}\n{}",
        repo_file("src-tauri/src/commands.rs"),
        repo_file("src-tauri/src/sync.rs")
    );
    let api_ts = repo_file("src/api.ts");

    // (前端字段名, Rust 字段名)
    let pairs: &[(&str, &str)] = &[
        ("firstRun", "first_run"),
        ("lastSyncAt", "last_sync_at"),
        ("lastSyncOk", "last_sync_ok"),
        ("toolId", "tool_id"),
        ("displayName", "display_name"),
        ("pushPath", "push_path"),
        ("installed", "installed"),
        ("rulesSynced", "rules_synced"),
        ("skillCount", "skill_count"),
        ("tools", "tools"),
        ("status", "status"),
        ("message", "message"),
        // TASK-19 新增：断链说明的展示标记 + 备份现状
        ("linkNoticeShown", "link_notice_shown"),
        ("targetFile", "target_file"),
        ("exists", "exists"),
        ("sizeBytes", "size_bytes"),
        ("modifiedAt", "modified_at"),
    ];

    for (ts_name, rust_name) in pairs {
        assert!(
            api_ts.contains(ts_name),
            "api.ts 中找不到字段 {ts_name}——前端可能已被改动"
        );
        assert!(
            rust_source.contains(rust_name),
            "Rust 侧找不到字段 {rust_name}（前端期望的是 {ts_name}）"
        );
    }
}

/// camelCase 转换是靠 serde 属性达成的，属性丢了字段名就会不匹配。
#[test]
fn camel_case_rename_is_declared() {
    let rust_source = repo_file("src-tauri/src/sync.rs");
    let count = rust_source
        .matches(r#"#[serde(rename_all = "camelCase")]"#)
        .count();
    // 至少要有 SyncReport / ToolSyncResult / ToolInfo / ToolSyncStatus 四处
    assert!(
        count >= 4,
        "sync.rs 中 camelCase 重命名属性只有 {count} 处，序列化到前端会变成 snake_case"
    );
}

/// 取出某个前端文件 `<template>` 段落的内容。
///
/// 只查模板：`<script>` 与注释里出现内部术语是正常的，
/// 甚至必要——比如「这里不能写 L0」这种给后来者的说明。
fn template_of(rel: &str) -> String {
    let source = repo_file(rel);
    let start = source
        .find("<template>")
        .unwrap_or_else(|| panic!("{rel} 找不到 <template>"));
    let end = source
        .rfind("</template>")
        .unwrap_or_else(|| panic!("{rel} 找不到 </template>"));
    source[start..end].to_string()
}

/// 界面文案里不得出现任何内部术语（`PRD.md` 第 5 节的验收项）。
///
/// # 为什么列出所有面向用户的文件
///
/// 只查向导的话，后加的主工作台、弹窗组件就成了绕过这条约束的缺口——
/// 而「不暴露内部概念」是产品定位的一部分：一旦漏进去一个「slug」，
/// 用户就得先学会我们的实现细节才能用这个软件。
#[test]
fn wizard_copy_has_no_internal_terms() {
    // 术语清单与 PRD 第 5 节验收项一致
    let forbidden = ["L0", "L2", "SSOT", "slug", "Slug", "Adapter"];

    let user_facing = [
        "src/views/WizardView.vue",
        "src/views/MainView.vue",
        "src/components/LinkNoticeDialog.vue",
        // TASK-11：全局规则编辑器（挂载在主工作台的 Tab 里）
        "src/components/RuleEditor.vue",
    ];

    for rel in user_facing {
        let template = template_of(rel);
        for term in forbidden {
            assert!(
                !template.contains(term),
                "{rel} 的界面文案出现了内部术语「{term}」——用户看不懂，必须改写成日常说法"
            );
        }
    }
}

/// 命令**参数名**同样是字符串寻址的：Rust 侧 `tool_id` ← 前端 `toolId`。
///
/// 写错不会编译报错，只会让「还原」按钮点了没反应——
/// 与命令名写错是同一种故障，但上面的命令名测试覆盖不到它。
#[test]
fn restore_backup_argument_name_matches_on_both_sides() {
    let api = strip_line_comments(&repo_file("src/api.ts"));
    assert!(
        api.contains(r#"invoke<string>("restore_backup", { toolId })"#),
        "api.ts 中 restore_backup 的参数名已被改动，请同步更新本断言与 Rust 侧"
    );

    let commands = repo_file("src-tauri/src/commands.rs");
    assert!(
        commands.contains("pub fn restore_backup(tool_id: String)"),
        "Rust 侧 restore_backup 的参数名已改动，前端传的 toolId 会对不上"
    );
}

/// 断链说明的「已展示」标记必须发生在用户**确认之后**（TASK-19 / ADR-15）。
///
/// # 为什么用源码文本锁一条纯前端逻辑
///
/// 若把标记写进 `guard()`（弹窗弹出的那一刻），用户点「先不同步」或直接关掉窗口，
/// 这条说明就**永远不会再出现**——而它恰恰是验收项要求的事前告知。
/// 前端的组合式函数没有测试框架覆盖，而这种顺序错误在人工 review 里也极易漏掉，
/// 所以按本项目既有做法（命令名、事件名同样是字符串寻址）用源码比对钉死。
#[test]
fn link_notice_is_marked_only_after_confirm() {
    // 只剥行注释：本例要断言的是真实调用，注释里的解释不算
    let code = strip_line_comments(&repo_file("src/composables/useLinkNotice.ts"));

    let mark = code
        .find("markLinkNoticeShown(")
        .expect("useLinkNotice 里找不到标记调用——断链说明会被反复弹出");
    let confirm = code
        .find("function confirm()")
        .expect("useLinkNotice 里找不到 confirm()");

    assert!(
        mark > confirm,
        "「已展示」标记必须写在 confirm() 内（用户确认之后）；\
         写在 guard() 里会在弹窗那一刻就标记，用户点取消后说明将永远不再出现"
    );
    assert_eq!(
        code.matches("markLinkNoticeShown(").count(),
        1,
        "标记调用应只有一处且位于 confirm() 内——多处意味着存在别的时机也会落标记"
    );
}

/// 保存全局规则**绝不**顺带触发同步（TASK-11 / PRD §6.1）。
///
/// # 为什么单独锁这一条
///
/// 「编辑」与「同步」是两个独立动作，混在一起的实际后果是：
/// 用户只是想存个草稿，却把自己的 `CLAUDE.md` 等文件全改了一遍——
/// 而 TASK-19 恰恰花大力气把「每次改动都有备份、可还原」做出来，
/// 让保存去触发同步等于把这些不可逆副作用全部重新暴露出来。
///
/// 断言范围是**规则编辑器自己的两个文件**（状态机 + 组件），
/// 而不是 `MainView.vue` —— 主工作台里「立即同步」是合法的，
/// 在那个文件上做全文禁止会误伤。
#[test]
fn rule_editor_saving_never_triggers_sync() {
    for rel in [
        "src/composables/useRuleEditor.ts",
        "src/components/RuleEditor.vue",
    ] {
        let code = strip_line_comments(&repo_file(rel));
        assert!(
            !code.contains("runSync"),
            "{rel} 里出现了 runSync —— 保存动作会顺带触发同步，\
             违背「编辑与同步是两个独立动作」（PRD §6.1）"
        );
    }

    // 反向护栏：上面是「不允许出现」，若编辑器根本没接保存命令，
    // 前一条恒为真。这里确认保存调用确实存在。
    let code = strip_line_comments(&repo_file("src/composables/useRuleEditor.ts"));
    assert!(
        code.contains("saveGlobalRules("),
        "useRuleEditor 里找不到 saveGlobalRules 调用——保存链路断了，\
         上面的「不触发同步」断言会因此恒为真"
    );
}

/// 预览必须**转义**原始 HTML，而不是照原样插进 DOM（TASK-11）。
///
/// # 为什么这不是洁癖
///
/// `tauri.conf.json` 目前 `csp: null`。若预览把用户内容里的
/// `<script>` / `onerror` 当真标签插入，脚本会在 webview 里执行，
/// 而 webview 拥有 `invoke` 权限——等同本地文件任意读写。
/// markdown-it 默认 `html: false` 会把原始 HTML 转成纯文本（已实测），
/// 所以这里锁的就是「别有人图省事把这个开关打开」。
#[test]
fn markdown_preview_must_escape_raw_html() {
    let code = strip_line_comments(&repo_file("src/composables/useRuleEditor.ts"));

    // 先确认渲染器确实接上了，否则下面的断言恒为真
    assert!(
        code.contains("MarkdownIt("),
        "useRuleEditor 里找不到 MarkdownIt 实例——预览渲染链路断了"
    );

    // 去掉所有空白再比对，`html: true` / `html:true` 都逃不掉
    let compact: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !compact.contains("html:true"),
        "markdown-it 的 html 选项被打开了——预览会把原始 HTML 插进 DOM，\
         而当前 CSP 为 null，脚本会在 webview 里执行。\
         若确需打开，必须同时补 CSP 与 sanitizer，并更新本测试"
    );
}
