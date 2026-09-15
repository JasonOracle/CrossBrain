//! dryrun_sync.rs — 同步编排层的真实结构干跑验证（开发期工具，不参与发布产物）。
//!
//! # 为什么需要它
//!
//! `sync.rs` 的单元测试只能验证纯逻辑（未安装要跳过、空规则不推 L0…），
//! 因为它们不能在真实路径上跑——本机 `~/.claude/` 与 `~/.gemini/config/`
//! 里有**真实用户数据**：5 路硬链接共享的全局记忆、用户自建的技能。
//!
//! 本工具把这两个目录**完整复制**到系统临时目录，用 `with_base_dir()` 注入副本，
//! 然后跑一遍真实编排流程。跑完再比对真实目录的快照——
//! 必须**一个字节都没变**。
//!
//! 运行：`cargo run --example dryrun_sync`

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crossbrain_lib::adapters::antigravity::AntigravityAdapter;
use crossbrain_lib::adapters::claude_code::ClaudeCodeAdapter;
use crossbrain_lib::adapters::codex::CodexAdapter;
use crossbrain_lib::slug;
use crossbrain_lib::sync::{scan_knowledge, KnowledgeItem, ToolDescriptor, ToolSyncStatus};
use crossbrain_lib::{paths, sync};

/// 累加失败数的检查器。
struct Checks {
    failures: u32,
}

impl Checks {
    fn new() -> Self {
        Self { failures: 0 }
    }

    fn check(&mut self, ok: bool, msg: &str) {
        if ok {
            println!("   ✓ {msg}");
        } else {
            println!("   ✗ {msg}");
            self.failures += 1;
        }
    }
}

/// 一条快照记录：相对路径、字节数、内容摘要。
///
/// 摘要用 `DefaultHasher` 即可——它只在本进程内前后比对，
/// 不需要跨版本稳定（这一点与 slug 的 SHA-256 恰恰相反）。
type Snapshot = Vec<(String, u64, u64)>;

fn main() {
    println!("=== CrossBrain 同步编排层干跑验证（真实结构副本）===");

    let home = dirs_home();
    let real_claude = home.join(".claude");
    let real_gemini = home.join(".gemini").join("config");
    let real_codex = home.join(".codex");

    // ⚠️ 只快照我们真正会写的那些位置。
    // 整个 ~/.claude/ 可能包含大量缓存与插件，全量哈希既慢又无意义。
    let real_profile = home.join(".ai-profile");
    let watched: Vec<PathBuf> = vec![
        real_claude.join("CLAUDE.md"),
        real_claude.join("CLAUDE.md.crossbrain-backup"),
        // 还原前现场快照（TASK-19）——正常情况下不该出现在真实目录里，
        // 一旦出现说明有代码路径在用户机器上执行了还原，必须被这次回归抓到
        real_claude.join("CLAUDE.md.crossbrain-before-restore"),
        real_claude.join("skills"),
        real_gemini.join("rules"),
        real_gemini.join("skills"),
        // Codex（TASK-18 增补）——它的 AGENTS.md 是用户正在使用的全局指令文件
        real_codex.join("AGENTS.md"),
        real_codex.join("AGENTS.md.crossbrain-backup"),
        real_codex.join("AGENTS.md.crossbrain-before-restore"),
        real_codex.join("skills"),
        // SSOT 侧（TASK-12 增补）：全局规则与知识库目录是用户内容，
        // 任何一次干跑都不许碰它们
        real_profile.join("global").join("rules.md"),
        real_profile.join("knowledge"),
    ];

    let before = snapshot_all(&watched);
    println!("\n[1] 真实数据快照：{} 项（用于跑完后比对零改动）", before.len());

    // ── 构建隔离副本 ──
    let tmp = temp_root();
    let _ = fs::remove_dir_all(&tmp);
    let fake_claude = tmp.join("claude");
    let fake_gemini = tmp.join("gemini-config");
    let fake_codex = tmp.join("codex");

    println!("\n[2] 复制真实目录结构到副本 {}", tmp.display());
    if let Err(e) = copy_dir(&real_claude, &fake_claude) {
        println!("   ✗ 复制 ~/.claude 失败：{e}");
        std::process::exit(1);
    }
    if let Err(e) = copy_dir(&real_gemini, &fake_gemini) {
        println!("   ✗ 复制 ~/.gemini/config 失败：{e}");
        std::process::exit(1);
    }
    // ~/.codex/ 里有 .tmp / logs_*.sqlite / image-cache 等大量无关内容，
    // 全量复制既慢又无意义——只复制 CrossBrain 可能触碰的两处。
    if let Err(e) = copy_codex_subset(&real_codex, &fake_codex) {
        println!("   ✗ 复制 ~/.codex 子集失败：{e}");
        std::process::exit(1);
    }
    let original_claude_md = fs::read_to_string(fake_claude.join("CLAUDE.md")).unwrap_or_default();
    println!(
        "   ✓ 副本就绪（CLAUDE.md {} 字节，skills {} 项）",
        original_claude_md.len(),
        count_entries(&fake_claude.join("skills"))
    );

    // ⚠️ 副本的**预状态**决定断言走哪条分支。
    //
    // 早期版本假设真实 CLAUDE.md 从未同步过（首注场景），据此断言
    // 「备份 == 同步前的原始内容」。2026-09-15 用户真机同步过一次后，
    // 真实文件已带标记块，这个假设永久失效——干跑从此全红。
    //
    // 两种预状态的**不变式不同**，都必须验证：
    //   首次注入 → 备份是这次新建的，内容 = 注入前的原始文件
    //   已注入过 → 备份是历史干净快照，同步（更新分支）绝不能碰它（ADR-15）
    let had_marker_pre = original_claude_md.contains("<!-- CrossBrain:Start -->");
    let backup_pre_path = fake_claude.join("CLAUDE.md.crossbrain-backup");
    let backup_pre = fs::read_to_string(&backup_pre_path).ok();
    println!(
        "   · 副本预状态：{}｜备份文件：{}",
        if had_marker_pre { "已注入过标记块（走更新分支）" } else { "未注入过标记块（走首注分支）" },
        if backup_pre.is_some() { "已存在" } else { "不存在" }
    );

    // 副本 skills/ 下**用户自建内容**的清单（跑完后必须一模一样）。
    // 不硬编码技能名：用户随时可能增删自己的技能，写死名字只会造成假失败。
    let skills_targets: Vec<(&str, PathBuf)> = vec![
        ("Claude Code", fake_claude.join("skills")),
        ("Antigravity", fake_gemini.join("skills")),
        ("Codex", fake_codex.join("skills")),
    ];
    let user_entries_before: Vec<Vec<String>> = skills_targets
        .iter()
        .map(|(_, dir)| user_entries(dir))
        .collect();

    // ⚠️ spike 孤儿的**存在性必须在跑之前记录**。
    // 跑完再查只会得到「不存在」——那既可能是清理成功，也可能是本来就没有，
    // 这个断言就永远为真、永远测不出东西。
    let spike_before: Vec<bool> = skills_targets
        .iter()
        .map(|(_, dir)| dir.join("crossbrain-spike-test").exists())
        .collect();

    // ── 工具清单：adapter 全部指向副本 ──
    let tools = vec![
        ToolDescriptor {
            tool_id: "claude_code",
            display_name: "Claude Code",
            push_path: fake_claude.clone(),
            adapter: Box::new(ClaudeCodeAdapter::with_base_dir(fake_claude.clone())),
        },
        ToolDescriptor {
            tool_id: "antigravity",
            display_name: "Antigravity IDE",
            push_path: fake_gemini.clone(),
            adapter: Box::new(AntigravityAdapter::with_base_dir(fake_gemini.clone())),
        },
        ToolDescriptor {
            tool_id: "codex",
            display_name: "Codex",
            push_path: fake_codex.clone(),
            adapter: Box::new(CodexAdapter::with_base_dir(fake_codex.clone())),
        },
    ];

    // ── 内容取材 ──
    let rules = "# 我的编码偏好\n\n- 回答用中文\n- 先给结论，再给理由\n";
    let mut knowledge = scan_knowledge().unwrap_or_default();
    if knowledge.is_empty() {
        println!("\n[3] 真实 knowledge/ 为空 → 使用 2 条内置样例");
        knowledge = vec![
            make_item(
                "Vue3 组件性能优化.md",
                "# Vue3 组件性能优化：长列表虚拟滚动\n\n正文内容……",
            ),
            make_item(
                "Rust 生命周期.md",
                "# Rust 生命周期与借用检查器常见错误\n\n正文内容……",
            ),
        ];
    } else {
        println!("\n[3] 使用真实 knowledge/ 中的 {} 条技能", knowledge.len());
    }

    // ── 跑编排 ──
    println!("\n[4] 执行 run_for_tools（adapter 全部指向副本）");
    let mut events: Vec<(String, ToolSyncStatus)> = Vec::new();
    let report = sync::run_for_tools(&tools, rules, &knowledge, &mut |progress| {
        events.push((progress.tool_id.clone(), progress.status.clone()));
        println!(
            "      进度事件：{} → {:?}（{}）",
            progress.display_name, progress.status, progress.message
        );
    });

    let mut c = Checks::new();

    println!("\n[5] 编排结果断言");
    c.check(report.ok, "整体同步成功");
    c.check(report.error.is_none(), "无中止原因");
    c.check(report.rules_synced, "全局规则已推送");
    c.check(
        report.skill_count == knowledge.len(),
        &format!("技能条数正确（{}）", knowledge.len()),
    );
    c.check(
        events.len() == 3,
        &format!("进度事件逐个工具推送（收到 {} 条，期望 3 条）", events.len()),
    );
    c.check(
        report
            .tools
            .iter()
            .all(|t| t.status == ToolSyncStatus::Ok),
        "三个工具均为成功",
    );

    // ── 副本内的实际落盘效果 ──
    println!("\n[6] 副本内的落盘效果");

    let claude_md = fs::read_to_string(fake_claude.join("CLAUDE.md")).unwrap_or_default();
    c.check(
        claude_md.contains("<!-- CrossBrain:Start -->"),
        "CLAUDE.md 已注入标记块",
    );
    c.check(
        claude_md.contains(&first_line(&original_claude_md)),
        "CLAUDE.md 原有内容保留（标记块外未被覆盖）",
    );
    let backup = fake_claude.join("CLAUDE.md.crossbrain-backup");
    let backup_after = fs::read_to_string(&backup).ok();
    if had_marker_pre {
        // 更新分支：备份在首注时就建好了，同步只是改标记块内容——备份一字不动
        c.check(
            backup_pre.is_some() && backup_after == backup_pre,
            "更新分支：同步不覆盖既有备份（跑前跑后一字不变，ADR-15）",
        );
    } else {
        // 首注分支：这次同步新建了备份，内容必须是注入前的原始文件
        c.check(
            backup_after.as_deref() == Some(original_claude_md.as_str()),
            "首注分支：备份内容等于注入前的原始 CLAUDE.md",
        );
    }

    let l0 = fake_gemini.join("rules").join("crossbrain-L0.md");
    c.check(
        fs::read_to_string(&l0).unwrap_or_default().contains("我的编码偏好"),
        "Antigravity L0 独立文件已写入",
    );
    c.check(
        fake_gemini.join("rules").join("user_global.md").is_file(),
        "用户的 user_global.md 未被触碰",
    );

    // Codex：AGENTS.md 走**标记块注入 + 内联规则全文**——这是它与 Claude 的关键差异。
    // 若这里退化成 @ 引用（或写成独立文件），同步会静默失效，Codex 什么都读不到。
    let codex_agents = fs::read_to_string(fake_codex.join("AGENTS.md")).unwrap_or_default();
    c.check(
        codex_agents.contains("<!-- CrossBrain:Start -->"),
        "Codex AGENTS.md 已注入标记块",
    );
    c.check(
        codex_agents.contains("我的编码偏好"),
        "Codex 标记块内联了规则全文（Codex 不认识 Claude 的 @ 语法）",
    );
    c.check(
        !codex_agents.contains("@~/.ai-profile/AGENTS.md"),
        "Codex 侧未混入 Claude 专有的 @ 引用语法",
    );
    if real_codex.join("skills").join(".system").is_dir() {
        c.check(
            fake_codex.join("skills").join(".system").is_dir(),
            "Codex 内置的 skills/.system/ 未被触碰",
        );
    } else {
        println!("   · 真实 ~/.codex/skills/.system/ 不存在，跳过该项");
    }

    for (label, dir) in [
        ("Claude Code", fake_claude.join("skills")),
        ("Antigravity", fake_gemini.join("skills")),
        ("Codex", fake_codex.join("skills")),
    ] {
        let injected = crossbrain_dirs(&dir);
        c.check(
            injected.len() == knowledge.len(),
            &format!(
                "{label} 技能目录数正确（{} 个：{}）",
                injected.len(),
                injected.join(", ")
            ),
        );
        let has_skill_md = knowledge.iter().all(|k| {
            fs::read_to_string(dir.join(&k.dir_name).join("SKILL.md"))
                .map(|text| text.starts_with("---\nname: "))
                .unwrap_or(false)
        });
        c.check(has_skill_md, &format!("{label} 每个技能都有合规的 SKILL.md"));
    }

    // ── 孤儿清理：删掉 Spike 遗留，保留用户技能 ──
    println!("\n[7] 孤儿清理的正确性");
    for (i, (label, dir)) in skills_targets.iter().enumerate() {
        let spike = dir.join("crossbrain-spike-test");
        if spike_before[i] {
            c.check(
                !spike.exists(),
                &format!("{label} 的 crossbrain-spike-test 孤儿已被清理（跑前存在）"),
            );
        } else {
            println!("   · {label} 副本内无 spike 遗留，跳过该项（断言无意义）");
        }

        // 用户自建内容必须零改动——这是最不能出错的一条。
        // 断言「跑前跑后清单完全一致」而不是比对写死的技能名：
        // 每个 tools 目录装哪些技能因人而异，写死名字只会造成假失败。
        let after = user_entries(dir);
        c.check(
            after == user_entries_before[i],
            &format!(
                "{label} 用户自建内容零改动（{} 项：{}）",
                after.len(),
                after.join(", ")
            ),
        );
    }

    // ── 真实数据零改动 ──
    println!("\n[8] 真实数据安全回归");
    let after = snapshot_all(&watched);
    c.check(
        before == after,
        &format!(
            "真实目录零改动（前 {} 项 / 后 {} 项，逐项比对）",
            before.len(),
            after.len()
        ),
    );
    if before != after {
        for (b, a) in before.iter().zip(after.iter()) {
            if b != a {
                println!("       差异：{b:?} → {a:?}");
            }
        }
    }

    // ── 工具检测清单（向导步骤 2 的数据源）──
    //
    // ⚠️ 这一节读的是**真实路径**（只做 `is_dir()` 判断，不写入），
    // 目的是验证「向导到底会显示几个工具」——dryrun 前面的同步跑在副本上，
    // 证明不了检测清单的条数。
    println!("\n[9] 工具检测清单（向导步骤 2 展示的就是这份数据）");
    let detected = sync::detect_tools();
    for t in &detected {
        println!(
            "   {} {}｜已安装={}｜{}",
            if t.installed { "✓" } else { "·" },
            t.display_name,
            t.installed,
            t.push_path
        );
    }
    c.check(
        detected.len() == 3,
        &format!("检测工具数为 3（实际 {} 个）", detected.len()),
    );
    c.check(
        detected
            .iter()
            .any(|t| t.tool_id == "codex" && t.installed),
        "Codex 在检测清单中且标记为已安装",
    );

    // ── AGENTS.override.md 遮蔽 → 必须作为「失败」上报，而不是被编排层吞掉 ──
    //
    // ⚠️ 单元测试只证明了「Adapter 会报错」；这一节证明的是**这条错误能走到报告里**。
    // 若 `sync_one_tool` 把 `sync_l0` 的错误吞掉，用户看到的仍是「同步成功」，
    // 那这道防线等于不存在。
    println!("\n[10] Codex 的 AGENTS.override.md 遮蔽防护（端到端）");
    let override_dir = tmp.join("codex-override");
    fs::create_dir_all(&override_dir).unwrap();
    fs::write(
        override_dir.join("AGENTS.override.md"),
        "# 用户自己的覆盖指令\n",
    )
    .unwrap();

    let override_tools = vec![ToolDescriptor {
        tool_id: "codex",
        display_name: "Codex",
        push_path: override_dir.clone(),
        adapter: Box::new(CodexAdapter::with_base_dir(&override_dir)),
    }];
    let override_report = sync::run_for_tools(&override_tools, rules, &[], &mut |p| {
        println!("      进度事件：{} → {:?}", p.display_name, p.status);
    });

    let only = &override_report.tools[0];
    c.check(
        only.status == ToolSyncStatus::Failed,
        &format!("被遮蔽时该工具必须报失败（实际 {:?}）", only.status),
    );
    c.check(
        only.message.contains("AGENTS.override.md"),
        &format!("失败原因必须点名该文件（实际：{}）", only.message),
    );
    c.check(
        !override_report.ok,
        "整体报告必须标记为不成功，不能静默通过",
    );
    c.check(
        !override_dir.join("AGENTS.md").exists(),
        "被遮蔽时不得写入 AGENTS.md（写了也没人读）",
    );

    // ── 备份查询与还原（TASK-19 / ADR-15：「可逆」）──
    //
    // 复用同一个副本：此时 Claude 与 Codex 的副本里都已经注入过标记块、
    // 并留下了首次注入前的备份，于是可以端到端跑通「点一下还原」这条链路。
    println!("\n[11] 备份查询与还原（跑在副本上）");
    let backups = sync::list_backups_for(&tools);
    for b in &backups {
        println!(
            "   {} {}｜备份={}｜{}｜{}",
            if b.exists { "✓" } else { "·" },
            b.display_name,
            b.exists,
            b.target_file,
            b.modified_at.as_deref().unwrap_or("（无时间）")
        );
    }
    c.check(
        backups.len() == 2,
        &format!("只列出有备份概念的工具（实际 {} 个）", backups.len()),
    );
    c.check(
        !backups.iter().any(|b| b.tool_id == "antigravity"),
        "写独立文件的工具不该出现在备份列表里（它从不改动用户既有文件）",
    );
    c.check(
        backups
            .iter()
            .find(|b| b.tool_id == "claude_code")
            .map(|b| b.exists)
            .unwrap_or(false),
        "Claude Code 副本里应已有备份",
    );

    let backup_file = fake_claude.join("CLAUDE.md.crossbrain-backup");
    let backup_before = fs::read_to_string(&backup_file).unwrap_or_default();
    let target_before_restore =
        fs::read_to_string(fake_claude.join("CLAUDE.md")).unwrap_or_default();

    // ① 还原：内容回到备份，且备份本身一字不变
    match sync::restore_backup_for(&tools, "claude_code") {
        Ok(msg) => {
            println!("      还原返回：{msg}");
            c.check(msg.contains("还原"), "成功文案应面向用户");
        }
        Err(e) => c.check(false, &format!("还原 Claude Code 失败：{e}")),
    }
    let restored = fs::read_to_string(fake_claude.join("CLAUDE.md")).unwrap_or_default();
    c.check(restored == backup_before, "还原后的内容必须等于备份内容");
    if had_marker_pre {
        // 真机同步过：备份是「历史干净快照」，而同步前的原始内容已带标记块，
        // 两者本来就不同——「还原后 == 原始内容」只对首注场景成立。
        // 更新场景要保证的（还原写入的确实是备份内容）上一条已经断过了。
        println!("   · 预状态已注入过标记块——还原恢复的是历史备份，跳过「等于原始内容」断言");
    } else {
        c.check(
            restored == original_claude_md,
            "还原后的内容必须等于同步前的原始内容",
        );
    }
    c.check(
        fs::read_to_string(&backup_file).unwrap_or_default() == backup_before,
        "备份在还原后必须一字不变（它是唯一干净快照）",
    );
    let pre_restore = fake_claude.join("CLAUDE.md.crossbrain-before-restore");
    c.check(
        pre_restore.exists(),
        "还原前应把当时的现场另存为 .crossbrain-before-restore",
    );
    c.check(
        fs::read_to_string(&pre_restore).unwrap_or_default() == target_before_restore,
        "现场快照里应当是还原前的实际内容",
    );

    // ② 再还原一次必须幂等：内容已与备份一致 → 不该再写现场快照，
    //    否则第一次留下的真实现场会被「已还原后的内容」覆盖掉
    let pre_restore_content = fs::read_to_string(&pre_restore).unwrap_or_default();
    match sync::restore_backup_for(&tools, "claude_code") {
        Ok(msg) => c.check(
            !msg.contains("另存"),
            &format!("第二次还原不该再另存现场（实际文案：{msg}）"),
        ),
        Err(e) => c.check(false, &format!("第二次还原失败：{e}")),
    }
    c.check(
        fs::read_to_string(&pre_restore).unwrap_or_default() == pre_restore_content,
        "第二次还原覆盖了第一次留下的现场快照",
    );

    // ③ 没有备份概念的工具要给明确解释，而不是抛一个文件系统错误
    match sync::restore_backup_for(&tools, "antigravity") {
        Err(msg) => c.check(
            msg.contains("不会修改"),
            &format!("Antigravity 应解释「没有需要还原的内容」（实际：{msg}）"),
        ),
        Ok(_) => c.check(false, "Antigravity 不该有可还原的内容"),
    }

    // ④ 未知工具：可读文案，不 panic、不外泄系统细节
    match sync::restore_backup_for(&tools, "no-such-tool") {
        Err(msg) => c.check(
            !msg.contains("os error"),
            &format!("未知工具的文案应可读（实际：{msg}）"),
        ),
        Ok(_) => c.check(false, "未知工具不该返回成功"),
    }

    // ⑤ 技能知识库 CRUD（TASK-12）——跑在 `tmp/knowledge/` 上，
    //    真实的 `~/.ai-profile/knowledge/` 一个字节都不许动（已加入快照比对）
    println!("\n[12] 技能知识库 CRUD（跑在副本目录上）");
    let fake_knowledge = tmp.join("knowledge");
    c.check(
        sync::list_knowledge_cards_for(&fake_knowledge)
            .map(|cards| cards.is_empty())
            .unwrap_or(false),
        "不存在的知识目录应返回空列表而不是错误",
    );

    match sync::create_knowledge_for(&fake_knowledge, "Vue3 组件性能优化") {
        Ok(file_name) => {
            println!("      新建返回：{file_name}");
            c.check(
                file_name == "vue3.md",
                &format!("新建文件名 = 标题的 kebab 段 + .md（实际：{file_name}）"),
            );
        }
        Err(e) => c.check(false, &format!("新建技能知识失败：{e}")),
    }

    // 同名标题必须去重，绝不能静默覆盖第一篇
    match (sync::create_knowledge_for(&fake_knowledge, "Vue3 组件性能优化"), sync::create_knowledge_for(&fake_knowledge, "优化")) {
        (Ok(second), Ok(third)) => {
            println!("      去重返回：{second} / {third}");
            c.check(
                second == "vue3-2.md",
                &format!("同名标题自动加序号（实际：{second}）"),
            );
            c.check(
                third == "item.md",
                &format!("纯中文标题退化为兜底 kebab 段（实际：{third}）"),
            );
        }
        _ => c.check(false, "同名/纯中文标题的新建不应失败"),
    }

    match sync::list_knowledge_cards_for(&fake_knowledge) {
        Ok(cards) => {
            c.check(cards.len() == 3, &format!("列表应有 3 篇（实际 {} 篇）", cards.len()));
            if let Some(card) = cards.iter().find(|c| c.file_name == "vue3.md") {
                c.check(card.title == "vue3", "卡片标题 = 文件名去掉 .md");
                c.check(
                    card.summary == "[技术/语言] · [具体场景]",
                    &format!("卡片摘要 = 第一行非空内容（实际：{}）", card.summary),
                );
                c.check(
                    card.dir_name.starts_with("crossbrain-vue3-"),
                    &format!("注入目录名形态正确且随卡片展示（实际：{}）", card.dir_name),
                );
                c.check(card.char_count > 0, "卡片字数已统计");
            } else {
                c.check(false, "列表里找不到 vue3.md");
            }
        }
        Err(e) => c.check(false, &format!("列表读取失败：{e}")),
    }

    // 保存新内容 → 摘要与注入目录名（hash 段）随之更新
    let dir_name_before = sync::list_knowledge_cards_for(&fake_knowledge)
        .ok()
        .and_then(|cards| cards.iter().find(|c| c.file_name == "vue3.md").map(|c| c.dir_name.clone()))
        .unwrap_or_default();
    match sync::save_knowledge_file_for(&fake_knowledge, "vue3.md", "# 新主题\n新正文") {
        Ok(()) => match sync::list_knowledge_cards_for(&fake_knowledge) {
            Ok(cards) => match cards.iter().find(|c| c.file_name == "vue3.md") {
                Some(card) => {
                    c.check(card.summary == "新主题", "保存后摘要随内容更新");
                    c.check(
                        card.dir_name != dir_name_before,
                        "内容变化 → 注入目录名随之变化（hash 段承担区分度）",
                    );
                }
                None => c.check(false, "保存后列表里找不到 vue3.md"),
            },
            Err(e) => c.check(false, &format!("保存后列表读取失败：{e}")),
        },
        Err(e) => c.check(false, &format!("保存失败：{e}")),
    }

    // 文件名是路径拼接的输入：穿越与分隔符必须被拒绝
    c.check(
        sync::read_knowledge_file_for(&fake_knowledge, "../rules.md").is_err(),
        "上跳序列的文件名被拒绝",
    );
    c.check(
        sync::delete_knowledge_file_for(&fake_knowledge, "a/b.md").is_err(),
        "带路径分隔符的文件名被拒绝",
    );
    c.check(
        sync::save_knowledge_file_for(&fake_knowledge, "note.txt", "x").is_err(),
        "非 .md 扩展名被拒绝",
    );

    // 删除 + 幂等
    match sync::delete_knowledge_file_for(&fake_knowledge, "vue3.md") {
        Ok(()) => {
            c.check(!fake_knowledge.join("vue3.md").exists(), "删除后文件消失");
            c.check(
                sync::delete_knowledge_file_for(&fake_knowledge, "vue3.md").is_ok(),
                "重复删除按幂等成功处理",
            );
        }
        Err(e) => c.check(false, &format!("删除失败：{e}")),
    }

    // ── 收尾 ──
    let _ = fs::remove_dir_all(&tmp);

    println!("\n=== 干跑结束：{} 项检查未通过 ===", c.failures);
    if c.failures > 0 {
        std::process::exit(1);
    }
}

// ============================================================================
// 辅助
// ============================================================================

/// 真实的 `~/.ai-profile/` 与 `~/.claude/` 都基于用户 home 目录。
///
/// 这里复用 `paths` 的规则：不硬编盘符，交给 `dirs` 解析。
fn dirs_home() -> PathBuf {
    paths::profile_root()
        .parent()
        .expect("profile_root 必须位于 home 之下")
        .to_path_buf()
}

fn make_item(file_name: &str, body: &str) -> KnowledgeItem {
    let s = slug::generate_parts(file_name, body);
    KnowledgeItem {
        file_name: file_name.to_string(),
        dir_name: s.dir_name().to_string(),
        skill_name: s.skill_name().to_string(),
        body: body.to_string(),
    }
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("crossbrain-dryrun-sync-{}", std::process::id()))
}

fn first_line(text: &str) -> String {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

fn count_entries(dir: &Path) -> usize {
    fs::read_dir(dir).map(|it| it.count()).unwrap_or(0)
}

/// 列出目录下所有 `crossbrain-` 前缀的子目录名（已排序）。
fn crossbrain_dirs(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| name.starts_with("crossbrain-"))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// 列出目录下**非** `crossbrain-` 前缀的条目名（即用户自己的东西），已排序。
///
/// 用它来断言「清理只动了 CrossBrain 自己的目录」——比起比对写死的技能名，
/// 这个清单能适应任何用户的目录内容。
fn user_entries(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| !name.starts_with("crossbrain-"))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// 递归复制目录（**不跟随符号链接**，避免把链接目标整个拷进来）。
fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

/// 只复制 `~/.codex/` 中 CrossBrain 可能触碰的部分。
///
/// `~/.codex/` 里有 `.tmp/`、`logs_*.sqlite`、`image-cache/`、`plugins/` 等
/// 大量与同步无关的内容，全量复制既慢又无意义。
/// 我们实际会读写的只有两处：
/// - `AGENTS.md` —— L0 落点（当前实测 0 字节）
/// - `skills/` —— L2 落点，**含内置的 `.system/`**（正是要验证它不被误删）
fn copy_codex_subset(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    let agents = src.join("AGENTS.md");
    if agents.is_file() {
        fs::copy(&agents, dst.join("AGENTS.md"))?;
    }

    // 备份也要带上：真机上它真实存在（首注时创建），
    // 副本里缺了它，「备份与还原」区看到的就不是一个真实的状态。
    let backup = src.join("AGENTS.md.crossbrain-backup");
    if backup.is_file() {
        fs::copy(&backup, dst.join("AGENTS.md.crossbrain-backup"))?;
    }

    let skills = src.join("skills");
    if skills.is_dir() {
        copy_dir(&skills, &dst.join("skills"))?;
    }

    Ok(())
}

fn snapshot_all(items: &[PathBuf]) -> Snapshot {
    let mut out = Vec::new();
    for item in items {
        if item.is_file() {
            out.push((
                item.to_string_lossy().to_string(),
                fs::metadata(item).map(|m| m.len()).unwrap_or(0),
                hash_file(item),
            ));
        } else if item.is_dir() {
            collect_dir(item, item, &mut out);
        }
    }
    out.sort();
    out
}

fn collect_dir(root: &Path, dir: &Path, out: &mut Snapshot) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(ty) = entry.file_type() else { continue };
        if ty.is_dir() {
            collect_dir(root, &path, out);
        } else if ty.is_file() {
            out.push((
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string(),
                entry.metadata().map(|m| m.len()).unwrap_or(0),
                hash_file(&path),
            ));
        }
    }
}

fn hash_file(path: &Path) -> u64 {
    match fs::read(path) {
        Ok(bytes) => {
            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            hasher.finish()
        }
        Err(_) => 0,
    }
}
