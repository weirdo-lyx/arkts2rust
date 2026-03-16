import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def strip_code_fences(text):
    t = text.strip()
    if not t.startswith("```"):
        return t
    lines = t.splitlines()
    if lines and lines[0].startswith("```"):
        lines = lines[1:]
    if lines and lines[-1].startswith("```"):
        lines = lines[:-1]
    return "\n".join(lines).strip()


def call_llm(prompt, model, base_url, api_key, timeout):
    url = base_url.rstrip("/") + "/v1/chat/completions"
    payload = {
        "model": model,
        "temperature": 0,
        "messages": [
            {"role": "user", "content": prompt},
        ],
    }
    body = json.dumps(payload).encode("utf-8")
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    }
    req = Request(url, data=body, headers=headers, method="POST")
    try:
        with urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except (URLError, HTTPError) as e:
        raise RuntimeError(f"LLM request failed: {e}") from e
    choices = data.get("choices", [])
    if not choices:
        raise RuntimeError("LLM returned empty choices")
    content = choices[0].get("message", {}).get("content", "")
    if not content:
        raise RuntimeError("LLM returned empty content")
    return strip_code_fences(content)


def build_prompt(arkts_src, rust_src, rustc_err):
    return "\n".join(
        [
            "你是 Rust 编译修复器。",
            "输入包含：",
            "1) ArkTS 源码",
            "2) 规则生成的 Rust 源码",
            "3) rustc 编译错误信息",
            "",
            "输出只包含修复后的 Rust 源码，不要解释。",
            "要求：",
            "- 最小修改原则",
            "- 不新增无关功能",
            "- 保持函数与变量命名",
            "",
            "【ArkTS 源码】",
            arkts_src,
            "",
            "【规则生成 Rust】",
            rust_src,
            "",
            "【rustc 错误】",
            rustc_err,
        ]
    )


def generate_rust(input_path, output_path, repo_root, arkts2rust_bin):
    if arkts2rust_bin:
        cmd = [arkts2rust_bin, input_path, "-o", output_path]
        result = run(cmd, cwd=repo_root)
    else:
        cmd = ["cargo", "run", "--quiet", "--", input_path, "-o", output_path]
        result = run(cmd, cwd=repo_root)
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "arkts2rust failed")


def rustc_check(rust_src, rustc_bin):
    with tempfile.TemporaryDirectory() as tmp:
        src_path = Path(tmp) / "output.rs"
        exe_path = Path(tmp) / "output"
        src_path.write_text(rust_src, encoding="utf-8")
        cmd = [rustc_bin, str(src_path), "-o", str(exe_path)]
        result = run(cmd)
        return result.returncode == 0, result.stderr.strip()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--max-retries", type=int, default=3)
    parser.add_argument("--arkts2rust-bin", default="")
    parser.add_argument("--rustc-bin", default="rustc")
    parser.add_argument("--llm-base-url", default=os.getenv("ARKTS2RUST_LLM_BASE_URL", "https://api.openai.com"))
    parser.add_argument("--llm-api-key", default=os.getenv("ARKTS2RUST_LLM_API_KEY", ""))
    parser.add_argument("--llm-model", default=os.getenv("ARKTS2RUST_LLM_MODEL", "gpt-4o-mini"))
    parser.add_argument("--timeout", type=int, default=60)
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    out_path = Path(args.out).resolve()
    repo_root = Path(__file__).resolve().parents[1]

    arkts_src = input_path.read_text(encoding="utf-8")

    with tempfile.TemporaryDirectory() as tmp:
        rust_path = Path(tmp) / "baseline.rs"
        generate_rust(str(input_path), str(rust_path), repo_root, args.arkts2rust_bin or None)
        rust_src = rust_path.read_text(encoding="utf-8")

    last_err = ""
    for attempt in range(args.max_retries + 1):
        ok, err = rustc_check(rust_src, args.rustc_bin)
        if ok:
            out_path.write_text(rust_src, encoding="utf-8")
            print(f"success: compiled after {attempt} repair(s)")
            return
        if attempt == args.max_retries:
            print("failed: reached max retries")
            print(err)
            sys.exit(1)
        if not args.llm_api_key:
            print("failed: missing LLM api key")
            print(err)
            sys.exit(1)
        if err == last_err:
            print("failed: repeated error without progress")
            print(err)
            sys.exit(1)
        prompt = build_prompt(arkts_src, rust_src, err)
        rust_src = call_llm(prompt, args.llm_model, args.llm_base_url, args.llm_api_key, args.timeout)
        last_err = err


if __name__ == "__main__":
    main()
