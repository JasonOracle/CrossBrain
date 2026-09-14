//! dryrun_slug.rs — TASK-07 验收脚本（开发期工具，不参与发布产物）。
//!
//! 单元测试验证的是「函数自己说自己对」，这里补两件测试做不到的事：
//! 1. **外部交叉验证**：hash 段必须与系统 `sha256sum` 的结果一致——
//!    证明我们算的确实是标准 SHA-256，而不是某个自洽的私有实现
//! 2. **人工可读的对照表**：把真实中文文件名跑一遍，肉眼确认产物形态
//!
//! 运行：`cargo run --example dryrun_slug`

use crossbrain_lib::adapters::{ensure_crossbrain_slug, format_skill_md};
use crossbrain_lib::slug::{generate_parts, SLUG_PREFIX};

fn main() {
    let mut failures = 0u32;
    let mut check = |ok: bool, msg: &str| {
        if ok {
            println!("   [OK] {msg}");
        } else {
            println!("   [!!] {msg}");
            failures += 1;
        }
    };

    println!("=== 1. 外部交叉验证：hash 段是否等于标准 SHA-256 前 6 位 ===");
    // 参照值由 `printf 'test' | sha256sum` 得到：9f86d081884c7d659a2feaa0c55ad015...
    let slug = generate_parts("Vue3 组件性能优化.md", "test");
    check(
        slug.hash() == "9f86d0",
        &format!(
            "sha256(\"test\") 前 6 位应为 9f86d0（外部 sha256sum 参照），实际 {}",
            slug.hash()
        ),
    );
    check(
        slug.dir_name() == "crossbrain-vue3-9f86d0",
        &format!("完整标识符应为 crossbrain-vue3-9f86d0，实际 {}", slug.dir_name()),
    );

    // 空串也是标准向量：SHA-256("") = e3b0c44298fc1c149afbf4c8996fb924...
    let empty = generate_parts("x.md", "");
    check(
        empty.hash() == "e3b0c4",
        &format!("sha256(\"\") 前 6 位应为 e3b0c4，实际 {}", empty.hash()),
    );

    println!("\n=== 2. 手动验证（TASK-07 验收清单） ===");
    let sample = generate_parts("Vue3 组件性能优化.md", "test");
    check(
        sample.dir_name().starts_with(SLUG_PREFIX),
        "输出以 crossbrain- 开头",
    );
    let twice = (
        generate_parts("Rust · 生命周期.md", "同一份内容"),
        generate_parts("Rust · 生命周期.md", "同一份内容"),
    );
    check(
        twice.0 == twice.1 && twice.0.dir_name() == twice.1.dir_name(),
        "同输入两次输出完全相同（幂等）",
    );
    let cn = generate_parts("Rust · 生命周期与借用检查器.md", "内容");
    check(cn.dir_name().is_ascii(), "含中文文件名 → 输出全为 ASCII");
    check(
        cn.dir_name()
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
        "仅含 [a-z0-9-]（可直接作目录名）",
    );

    println!("\n=== 3. 跨模块契约：产出的名字必须能过 Adapter 命名空间校验 ===");
    for name in [
        "Vue3 组件性能优化.md",
        "优化.md",
        "../../etc/passwd.md",
        "a/b\\c:d*e.md",
        ".md",
        "",
    ] {
        let s = generate_parts(name, "内容");
        let ok = ensure_crossbrain_slug(s.dir_name()).is_ok();
        check(ok, &format!("「{name}」→ {} 通过校验", s.dir_name()));
    }

    println!("\n=== 4. 真实文件名对照表 ===");
    println!(
        "   {:<38} {:<28} {}",
        "输入文件名", "目录名（跨平台安全）", "frontmatter name"
    );
    println!("   {}", "-".repeat(96));
    for name in [
        "Vue3 组件性能优化.md",
        "Rust · 生命周期与借用检查器.md",
        "C++ #include 踩坑记录.md",
        "Vue3 组件性能优化.md",
        "组件性能优化.md",
        "v1.2 迁移说明.md",
        "Note.MD",
        "abcdefghij klmnopqrst uvwxyz 0123456789.md",
    ] {
        let s = generate_parts(name, "内容");
        println!(
            "   {:<38} {:<28} {}",
            name,
            s.dir_name(),
            s.skill_name()
        );
    }

    println!("\n=== 5. 与 Adapter 的衔接：frontmatter 实际生成结果 ===");
    let s = generate_parts("Vue3 组件性能优化.md", "内容");
    let body = "# Vue3 组件性能优化：长列表虚拟滚动\n\n正文……";
    println!("--- 写入 skills/{}/SKILL.md ---", s.dir_name());
    println!("{}", format_skill_md(s.skill_name(), body));
    check(
        format_skill_md(s.skill_name(), body).starts_with("---\nname: vue3\n"),
        "frontmatter 的 name 使用技能名（不含 crossbrain- 前缀）",
    );

    println!("\n=== 6. 不变性保证（ARCHITECTURE 第 4 节） ===");
    let base = generate_parts("note.md", "# 标题\n正文");
    let renamed = generate_parts("note2.md", "# 标题\n正文");
    let edited = generate_parts("note.md", "# 标题\n正文\n新增");
    check(
        base.hash() == renamed.hash() && base.kebab() != renamed.kebab(),
        "只改文件名：hash 不变、kebab 变",
    );
    check(
        base.kebab() == edited.kebab() && base.hash() != edited.hash(),
        "只改内容：kebab 不变、hash 变（旧目录成为孤儿）",
    );
    check(
        generate_parts("note.md", "# 标题\n正文").dir_name() == base.dir_name(),
        "都不改：同一目录（不产生孤儿）",
    );

    println!();
    if failures == 0 {
        println!("全部通过。");
    } else {
        println!("失败 {failures} 项。");
        std::process::exit(1);
    }
}
