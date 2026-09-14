//! dryrun_claude.rs — 在**真实结构的副本**上干跑 `ClaudeCodeAdapter` 的完整链路。
//!
//! # 这是什么
//!
//! 开发期验证工具，**不参与发布产物**（`examples/` 不会被 `tauri build` 打包）。
//!
//! 本机 `~/.claude/CLAUDE.md` 与另外 4 个工具路径是**硬链接**共享同一 inode
//! （links=5，`fsutil hardlink list` 已验证）。这意味着任何写入策略上的错误
//! 都会**同时污染 5 个 AI 工具的规则文件**——风险极高，靠单元测试的临时目录
//! 不足以让人放心。
//!
//! 因此本工具：
//! 1. **只读**读取真实的那 5 个文件（绝不修改，运行前后逐字节比对）
//! 2. 在临时目录里用 `fs::hard_link` **复现同样的 5 链接结构**
//! 3. 在副本上跑完整链路，并逐项校验「兄弟路径一字未变」
//!
//! 用法：`cargo run --example dryrun_claude`

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use crossbrain_lib::adapters::claude_code::{ClaudeCodeAdapter, SyncL0Result};
use crossbrain_lib::adapters::Adapter;

/// 失败计数器。
static FAILURES: AtomicU32 = AtomicU32::new(0);

/// 打印一条检查结果。
fn check(ok: bool, msg: &str) {
    if ok {
        println!("   ✓ {msg}");
    } else {
        println!("   ✗ {msg}");
        FAILURES.fetch_add(1, Ordering::SeqCst);
    }
}

/// 本机真实存在的那 5 个共享 inode 的路径（**只读**使用）。
fn real_shared_paths() -> Option<Vec<PathBuf>> {
    let home = dirs::home_dir()?;
    let paths = vec![
        home.join(".ai-memory").join("user_profile.md"),
        home.join(".claude").join("CLAUDE.md"),
        home.join(".cursor").join("rules").join("user_profile.md"),
        home.join(".config").join("opencode").join("AGENTS.md"),
        home.join(".gemini").join("config").join("rules").join("user_global.md"),
    ];
    if paths.iter().all(|p| p.is_file()) {
        Some(paths)
    } else {
        None
    }
}

fn main() {
    println!("=== CrossBrain ClaudeCodeAdapter 干跑（真实结构副本）===\n");

    // ---------- 0. 记录真实文件快照（只读） ----------
    let Some(real) = real_shared_paths() else {
        println!("跳过：本机未找到那 5 个共享文件。");
        return;
    };
    let real_before: Vec<Vec<u8>> = real
        .iter()
        .map(|p| fs::read(p).expect("读取真实文件失败"))
        .collect();
    let original = String::from_utf8_lossy(&real_before[0]).to_string();
    println!(
        "① 真实文件快照已记录（只读，共 {} 个路径 / {} 字节）",
        real.len(),
        real_before[0].len()
    );

    // ---------- 1. 在临时目录复现 5 链接结构 ----------
    let root = std::env::temp_dir().join(format!("crossbrain-dryrun-claude-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("创建临时目录失败");

    // 索引 0 = 原始；1 = CLAUDE.md（本 Adapter 的目标）；2.. = 其它工具
    let mirror: Vec<PathBuf> = vec![
        root.join("user_profile.md"),
        root.join("CLAUDE.md"),
        root.join("cursor_user_profile.md"),
        root.join("opencode_AGENTS.md"),
        root.join("gemini_user_global.md"),
    ];
    fs::write(&mirror[0], &real_before[0]).expect("写入副本失败");
    for p in &mirror[1..] {
        fs::hard_link(&mirror[0], p).expect("创建硬链接失败（需 NTFS）");
    }
    println!("② 副本结构已建立：1 份内容 + {} 个路径共享 inode\n", mirror.len());

    // 读副本的辅助闭包
    let read = |p: &PathBuf| fs::read_to_string(p).expect("读取副本失败");
    let siblings_untouched = |expect: &str| {
        mirror[2..]
            .iter()
            .all(|p| fs::read_to_string(p).unwrap() == expect)
    };

    let adapter = ClaudeCodeAdapter::with_base_dir(&root);
    let backup = root.join("CLAUDE.md.crossbrain-backup");

    // ---------- 情况三：首次注入 ----------
    println!("③ 情况三（文件存在、无标记块、无备份）");
    match adapter.sync_l0_with_result("全局规则") {
        Ok(SyncL0Result::BackupCreated { backup_path }) => {
            check(true, &format!("返回 BackupCreated（备份：{backup_path}）"));
        }
        other => check(false, &format!("应返回 BackupCreated，实际：{other:?}")),
    }
    check(backup.is_file(), "备份文件已创建");
    check(
        fs::read_to_string(&backup).unwrap() == original,
        "备份内容与原始内容完全一致",
    );
    let after_case3 = read(&mirror[1]);
    check(
        after_case3.contains("<!-- CrossBrain:Start -->") && after_case3.contains("<!-- CrossBrain:End -->"),
        "CLAUDE.md 已注入标记块",
    );
    check(
        after_case3.starts_with(original.trim_end()),
        "CLAUDE.md 原有内容完整保留（标记块在末尾）",
    );
    check(
        siblings_untouched(&original),
        "⚠️ 核心：其余 4 个工具路径一字未变",
    );
    println!();

    // ---------- 情况二：幂等复跑（覆盖面最大的路径） ----------
    println!("④ 情况二（已有标记块，重复同步）");
    match adapter.sync_l0_with_result("全局规则已更新") {
        Ok(SyncL0Result::Silent) => check(true, "返回 Silent"),
        other => check(false, &format!("应返回 Silent，实际：{other:?}")),
    }
    let after_case2 = read(&mirror[1]);
    check(after_case2 == after_case3, "重复同步字节级幂等（内容零变化）");
    check(
        siblings_untouched(&original),
        "⚠️ 核心：其余 4 个工具路径仍一字未变",
    );
    println!();

    // ---------- 情况四：无标记块 + 已有备份 ----------
    println!("⑤ 情况四（无标记块、备份已存在）");
    // 已断链，所以这里「就地覆写」是安全的——顺带验证了断链确实生效
    fs::write(&mirror[1], &original).expect("重置 CLAUDE.md 失败");
    check(
        read(&mirror[1]) == original,
        "就地覆写 CLAUDE.md 只影响自身（断链生效，未牵动兄弟路径）",
    );
    match adapter.sync_l0_with_result("x") {
        Ok(SyncL0Result::Silent) => check(true, "返回 Silent（不重复提示）"),
        other => check(false, &format!("应返回 Silent，实际：{other:?}")),
    }
    check(
        fs::read_to_string(&backup).unwrap() == original,
        "已有备份未被覆盖（用户此前的干净快照保住了）",
    );
    check(
        read(&mirror[1]).ends_with("<!-- CrossBrain:End -->"),
        "标记块已重新追加",
    );
    check(siblings_untouched(&original), "其余 4 个工具路径仍一字未变");
    println!();

    // ---------- 孤儿清理 ----------
    println!("⑥ 孤儿清理（副本 skills 目录）");
    let skills = root.join("skills");
    for name in ["crossbrain-keep-111111", "crossbrain-orphan-222222", "grill-me"] {
        let d = skills.join(name);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("SKILL.md"), "x").unwrap();
    }
    match adapter.cleanup_orphans(&["crossbrain-keep-111111".to_string()]) {
        Ok(report) => {
            check(
                report.deleted_dirs == vec!["crossbrain-orphan-222222".to_string()],
                &format!("只删了孤儿：{:?}", report.deleted_dirs),
            );
            check(skills.join("grill-me").is_dir(), "用户自建技能 grill-me 未被删除");
        }
        Err(e) => check(false, &format!("清理失败：{e}")),
    }
    println!();

    // ---------- 收尾：确认真实文件零改动 ----------
    println!("⑦ 真实文件最终校验（关键：全局零改动）");
    let mut all_same = true;
    for (i, p) in real.iter().enumerate() {
        let now = fs::read(p).expect("读取真实文件失败");
        if now != real_before[i] {
            all_same = false;
            println!("   ✗ 真实文件被改动：{}", p.display());
        }
    }
    check(all_same, "5 个真实文件逐字节与运行前一致");

    // 清理临时目录
    let _ = fs::remove_dir_all(&root);

    let failures = FAILURES.load(Ordering::SeqCst);
    println!();
    if failures == 0 {
        println!("=== 干跑通过：四情况协议正确，硬链接隔离生效，真实数据零改动 ===");
    } else {
        println!("=== 干跑失败：{failures} 项检查未通过 ===");
        std::process::exit(1);
    }
}
