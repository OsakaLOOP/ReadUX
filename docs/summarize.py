import os
from pathlib import Path
from datetime import datetime          # ← 新增
from openai import OpenAI

# ── 配置 ──────────────────────────────────────────────
BASE_DIR = Path("./docs")                  # Markdown 目录
OUTPUT_FILE = Path("./summary_merged.md")  # 合并输出文件
MODEL = "deepseek-flash"
API_KEY = "sk-fd804fa3264b4a779ef1f021379387d9"  
MAX_OUTPUT_TOKENS = 20000                   # 多个文件摘要，留足输出空间
MAX_INPUT_CHARS = 1048576                   # 输入总长度上限（按需调整）
# ──────────────────────────────────────────────────────

client = OpenAI(
    api_key=API_KEY,
    base_url="https://api.deepseek.com",
)

def get_ctime(st: os.stat_result) -> float:
    """跨平台获取创建时间：macOS 用 st_birthtime，Windows 用 st_ctime。"""
    return getattr(st, "st_birthtime", st.st_ctime)


def fmt_ts(ts: float) -> str:
    return datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S")

def load_and_concat_markdown(base_dir: Path) -> tuple[str, list[tuple[str, str, str]]]:
    """读取目录下所有 .md 文件，拼接为带文件名+日期的单一字符串。

    返回 (拼接文本, [(文件名, 创建时间, 修改时间), ...])
    """
    md_files = sorted(base_dir.rglob("*.md"))
    if not md_files:
        return "", []

    chunks: list[str] = []
    metas: list[tuple[str, str, str]] = []
    total = 0

    for fp in md_files:
        rel_name = str(fp.relative_to(base_dir))
        try:
            st = fp.stat()
            ctime_str = fmt_ts(get_ctime(st))
            mtime_str = fmt_ts(st.st_mtime)
        except OSError as e:
            print(f"⚠️ 无法读取文件信息：{rel_name}（{e}）")
            ctime_str = mtime_str = "未知"

        header = (
            f"<<< 文件：{rel_name} | 创建：{ctime_str} | 修改：{mtime_str} >>>"
        )

        try:
            text = fp.read_text(encoding="utf-8").strip()
        except Exception as e:
            chunks.append(f"{header}\n[读取失败：{e}]")
            metas.append((rel_name, ctime_str, mtime_str))
            continue

        if not text:
            text = "（空文件）"

        block = f"{header}\n{text}\n"
        if total + len(block) > MAX_INPUT_CHARS:
            block = block[: MAX_INPUT_CHARS - total] + "\n...(内容超长，已截断)"
            chunks.append(block)
            metas.append((rel_name, ctime_str, mtime_str))
            print("⚠️ 输入达到长度上限，后续内容已截断。")
            break

        chunks.append(block)
        metas.append((rel_name, ctime_str, mtime_str))
        total += len(block)

    return "\n\n".join(chunks), metas


def summarize_all(concatenated: str, metas: list[tuple[str, str, str]]) -> str:
    """一次调用 deepseek-flash，让模型按小标题逐一输出每篇摘要及日期。"""
    system_instruction = (
        "你是一个文档摘要助手。用户会提供多个 Markdown 文档，来自同一个项目的docs文件夹."
        "每个文档以 `<<< 文件：文件名 | 创建：时间 | 修改：时间 >>>` 作为分隔标记。\n"
        "请为每个文档生成不超过 100 字的摘要，并按以下格式输出，"
        "不要输出任何额外解释或开场白：\n\n"
        "## 文件名\n"
        "- 创建：YYYY-MM-DD HH:MM:SS\n"
        "- 修改：YYYY-MM-DD HH:MM:SS\n"
        "摘要正文\n\n"
        "（日期必须原样取自分隔标记，不得编造、改写或省略；每个文档一个二级标题。）"
    )

    file_list = "\n".join(
        f"- {name} | 创建：{c} | 修改：{m}" for name, c, m in metas
    )
    
    further_instruction = (
        "\n\n--- 补充要求（优先级最高，覆盖以上任何冲突） ---\n"
        "1. 确保内容足够详细，保留相关核心词、术语。\n"
        "2. 符合原文档结构；若原文档有小标题，应保留该小标题结构，"
        "此时每个小标题下允许总结 20 字，总字数不限。\n"
        "3. 每篇文档摘要之后，请判断该文档对应的所处阶段"
        "（如：规划、开发、测试、上线、维护、废弃等）"
        "以及可能的过时状态（如：最新、可能过时、已过时，并简述判断理由）。"
    )
     
    user_prompt = (
        f"共有 {len(metas)} 个文档，文件名及日期如下：\n{file_list}\n\n"
        f"以下是各文档内容，请按要求逐一总结：\n\n{concatenated}"
        f"{further_instruction}"
    )
    
    

    response = client.responses.create(
        model=MODEL,
        instructions=system_instruction,
        input=user_prompt,
        max_output_tokens=MAX_OUTPUT_TOKENS,
        temperature=0.3,
    )
    return response.output_text.strip()

def main():
    concatenated, metas = load_and_concat_markdown(BASE_DIR)   # ← 变量名 names→metas
    if not concatenated:
        print(f"在 {BASE_DIR} 下没有找到可处理的 .md 文件。")
        return

    print(f"已拼接 {len(metas)} 个 Markdown 文件，正在调用 API…")
    try:
        result = summarize_all(concatenated, metas)            # ← 传 metas
    except Exception as e:
        print(f"API 调用失败：{e}")
        return

    lines = ["# Markdown 文档摘要合集\n", result, ""]
    OUTPUT_FILE.write_text("\n".join(lines), encoding="utf-8")
    print(f"完成，输出已写入：{OUTPUT_FILE.resolve()}")

if __name__ == "__main__":
    main()