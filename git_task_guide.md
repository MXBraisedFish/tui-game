# Git 任务指南

---

## 一、让 AI 干活的六个口令

| 我说              | 它干什么                                            |
| ----------------- | --------------------------------------------------- |
| **git 任务**      | 提交所有改动 → 推 dev（**默认**，不说别的就用这个） |
| **同步 git 任务** | 提交所有改动 → 推 dev → 再把白名单内容同步到 main   |
| **实验 git 任务** | 提交所有改动 → 推 test                              |
| **网页 git 任务** | 提交所有改动 → 推 web                               |
| **插件 git 任务** | 提交所有改动 → 推 ide                               |
| **全部 git 任务** | 提交所有改动 → 上述所有任务合并执行                 |

`ide` 是孤立分支（与 dev 没有共同历史），只放 IDE 配套插件的代码，目前只有初始目录 `vscode/`。

---

## 二、手动提交（自己来）

**先看改了啥**

```bash
git status
git diff --stat
```

**提交并推 dev（最常用）**

```bash
git add -A
git commit -m "这次改了什么"
git push origin dev
```

**提交并推 test（web 就换成 web）**

```bash
git add -A
git commit -m "这次改了什么"
git push origin test
```

**代理没开的时候，推送会失败，用这行绕过去**

```bash
git -c http.proxy= -c https.proxy= push origin dev
```

---

## 三、切换分支

**看现在在哪、有哪些分支**

```bash
git branch -a
```

**切分支**

```bash
git checkout dev
git checkout main
git checkout test
git checkout web
```

**有没提交的改动时，切换前先存起来，回头再拿出来**

```bash
git stash
git checkout main
git stash pop
```

---

## 四、把 dev 同步到 main（白名单）

main 只留这些：`assets/`、`crates/`、`dev_docs/`、`README-i18n/`、`scripts/`、`src/`、`Cargo.lock`、`Cargo.toml`、`LICENSE`、`README.md`、`rustfmt.toml`

**用临时工作区操作，不影响当前目录（不用切分支）**

```bash
# 1. 拉一个临时工作区出来检出 main
git worktree add temp\main-worktree main

# 2. 把 dev 的白名单内容搬过去
git -C temp\main-worktree checkout dev -- assets crates dev_docs README-i18n scripts src Cargo.lock Cargo.toml LICENSE README.md rustfmt.toml

# 3. 删掉 main 里不该有的东西（报错说找不到就说明本来没有，不用管）
git -C temp\main-worktree rm -r -q .github .gitignore .gitattributes test_package build.rs

# 4. 提交并推送
git -C temp\main-worktree commit -m "chore: main 同步白名单内容"
git -C temp\main-worktree -c http.proxy= -c https.proxy= push origin main

# 5. 收工，把临时工作区删掉
git worktree remove temp\main-worktree --force
git worktree prune
```

**查一下临时工作区还在不在**

```bash
git worktree list
```

---

## 五、常用的补救命令

**提交信息写错了，还没推送，改一下**

```bash
git commit --amend -m "改好的说明"
```

**提交错了，想撤销这一步（代码不丢，改动回到工作区）**

```bash
git reset --soft HEAD~1
```

**推送被拒（远端比本地新），先同步再推**

```bash
git -c http.proxy= -c https.proxy= fetch origin
git rebase origin/dev
git push origin dev
```

**看最近几次提交**

```bash
git log --oneline -5
```

**看某次提交改了哪些文件**

```bash
git show --stat <提交号>
```

---

**提交书写格式**

提交信息分英文、中文两部分，英文在前、中文在后，两部分之间用单独一行 `---` 分隔。不加单行主题，提交信息本体就是下面的结构。

**英文部分**

```markdown
# Commit Log

## Overview

- Committed at: 年/月/日 时:分
- Summary:
- Changes:
  + 0
  - 0
- Main files:
（文件树）

## Details

### Feat

- 子内容

### Fix

- 子内容
```

**中文部分**

```markdown
# 提交日志

## 信息总览

- 提交时间：年/月/日 时:分
- 提交总结：
- 修改数量：
  + 0
  - 0
- 主要文件：
（文件树）

## 详细信息

### 新增

- 子内容

### 修复

- 子内容
```

**书写要求**

- 不使用 emoji，纯文字。
- 中文部分除专业术语、代码、文件名与路径外一律用中文；子标题只用「新增」「修复」这类中文，不写 `feat`、`fix`。
- 英文部分的子标题按类型表首字母大写：`Feat`、`Fix`、`Docs`……，不写全小写类型名。
- 子标题统一按下方类型表的顺序书写；某一类没有内容就整节省略，不留空标题。
- `修改数量` 的 `+` / `-` 填本次提交的增删行数（`git diff --cached --numstat` 汇总）。
- `主要文件` 用文件树列出本次改动的文件。

**子标题与类型对照**

| 英文       | 中文     | 适用内容                   |
| ---------- | -------- | -------------------------- |
| `feat`     | 新增     | 新功能、新接口             |
| `fix`      | 修复     | 修正已有行为中的缺陷       |
| `docs`     | 文档     | 文档、教程、示例说明       |
| `refactor` | 重构     | 调整实现，保持对外行为     |
| `perf`     | 优化     | 性能、内存或资源占用改善   |
| `test`     | 测试     | 测试用例、夹具、测试工具   |
| `style`    | 格式     | 排版、缩进等非行为变化     |
| `build`    | 构建     | 依赖、编译、打包配置       |
| `ci`       | 持续集成 | 自动检查、构建及发布工作流 |
| `chore`    | 杂项     | 不属于上述类型的维护工作   |
| `revert`   | 回退     | 撤销既有变化               |
