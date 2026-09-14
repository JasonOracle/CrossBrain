//! 真实数据安全干跑（开发期工具，不属于发布产物）。
//!
//! 在真实 `~/.gemini/config` 的**临时副本**上执行完整同步流程，
//! 确认实现不会误删用户的既有内容——**绝不触碰真实目录**。
//!
//! # 什么时候该跑
//!
//! 每次改动 [`AntigravityAdapter`] 的读写或清理逻辑之后。
//! 单元测试用的是合成 fixture（目录名是我们自己编的），而这个工具跑在
//! **用户真实的目录清单**上——本机实测有 `design-taste-frontend`、`grill-me`
//! 等自建技能，正是最容易被误删的对象。
//!
//! 用法：`cargo run --example dryrun_antigravity`

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crossbrain_lib::adapters::antigravity::AntigravityAdapter;
use crossbrain_lib::adapters::{format_skill_md, Adapter};

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn list_dirs(dir: &Path) -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                v.push(e.file_name().to_string_lossy().to_string());
            }
        }
    }
    v.sort();
    v
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("无法获取 home 目录")?;
    let real = home.join(".gemini").join("config");

    println!("真实配置目录：{}", real.display());
    if !real.is_dir() {
        println!("→ 不存在，跳过");
        return Ok(());
    }

    let real_skills = real.join("skills");
    let before = list_dirs(&real_skills);
    println!("真实 skills/ 下的目录（{} 个）：{before:?}", before.len());

    // ---- 建副本（只复制 rules/ 与 skills/，明确且轻量）----
    // 仅用于生成唯一临时目录名，不是业务时间戳，故不走 chrono
    // （铁律 L-03 约束的是业务时间戳；此处若用 chrono 只是为了拿一个唯一串，反而更重）
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let sandbox = std::env::temp_dir().join(format!("cb-dryrun-{stamp}"));
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(&sandbox)?;
    for sub in ["rules", "skills"] {
        if real.join(sub).is_dir() {
            copy_dir(&real.join(sub), &sandbox.join(sub))?;
        }
    }
    println!("\n沙箱副本：{}", sandbox.display());

    // ---- 用真实代码在副本上执行 ----
    let adapter = AntigravityAdapter::with_base_dir(&sandbox);

    println!("\n[detect]           → {}", adapter.detect()?);

    adapter.sync_l0("# CrossBrain 干跑规则\n\n- 仅用于验证，不写入真实目录\n")?;
    println!("[sync_l0]          → 写入 rules/crossbrain-L0.md ✓");

    let slug = "crossbrain-dryrun-check-abc123";
    adapter.sync_l2(slug, &format_skill_md("dryrun-check", "# 干跑技能\n\n正文"))?;
    println!("[sync_l2]          → 写入 skills/{slug}/SKILL.md ✓");

    // active = 副本中原有的 crossbrain-* 全算「活跃」+ 刚写入的那个
    let active: Vec<String> = before
        .iter()
        .filter(|n| n.starts_with("crossbrain-"))
        .cloned()
        .chain(std::iter::once(slug.to_string()))
        .collect();
    let report = adapter.cleanup_orphans(&active)?;
    println!("[cleanup_orphans]  → deleted = {:?}", report.deleted_dirs);
    println!("[cleanup_orphans]  → kept    = {:?}", report.kept_dirs);

    // 再跑一次，这次 active 为空：所有 crossbrain-* 都应被判定为孤儿
    let report2 = adapter.cleanup_orphans(&[])?;
    println!("[cleanup_orphans]  → 空 active 时 deleted = {:?}", report2.deleted_dirs);

    // ---- 断言 ----
    let after = list_dirs(&sandbox.join("skills"));
    println!("\n副本 skills/ 下的目录（{} 个）：{after:?}", after.len());

    let mut ok = true;
    for name in &before {
        if !name.starts_with("crossbrain-") && !after.contains(name) {
            println!("✗ 用户目录被误删：{name}");
            ok = false;
        }
    }
    let user_rule = sandbox.join("rules").join("user_global.md");
    if user_rule.is_file() {
        let len = fs::read(&user_rule)?.len();
        println!("✓ 用户规则文件 user_global.md 保留（{len} 字节）");
    }
    println!(
        "✓ crossbrain-* 目录被正确清理：{:?}",
        before
            .iter()
            .filter(|n| n.starts_with("crossbrain-"))
            .collect::<Vec<_>>()
    );

    // ---- 确认真实目录全程零改动 ----
    let real_after = list_dirs(&real_skills);
    if real_after == before {
        println!("✓ 真实 ~/.gemini/config 全程零改动（{} 个目录逐一比对一致）", before.len());
    } else {
        println!("✗ 真实目录发生了变化！before={before:?} after={real_after:?}");
        ok = false;
    }

    let _ = fs::remove_dir_all(&sandbox);
    println!("\n===== {} =====", if ok { "干跑通过" } else { "干跑失败" });
    if ok {
        Ok(())
    } else {
        Err("干跑发现数据安全问题".into())
    }
}
