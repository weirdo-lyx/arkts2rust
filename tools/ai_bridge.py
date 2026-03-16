import sys
import os
import json
import urllib.request
import urllib.error

def log(msg):
    sys.stderr.write(f"[AI Bridge] {msg}\n")
    sys.stderr.flush()

def main():
    # 硬编码 API 密钥和配置
    api_key = ""
    base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
    model = "glm-5"

    log(f"Starting AI translation with model: {model}")
    
    # 从标准输入读取提示词
    prompt = sys.stdin.read()
    if not prompt.strip():
        log("Empty prompt received.")
        return

    log(f"Prompt length: {len(prompt)} characters")

    # 构造请求
    url = f"{base_url}/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }
    
    payload = {
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an ArkTS to Rust compiler assistant. Only output the valid Rust code, no markdown, no explanation."
            },
            {"role": "user", "content": prompt}
        ],
        "extra_body": {"enable_thinking": True}
    }

    try:
        log(f"Sending request to {url}...")
        req = urllib.request.Request(url, data=json.dumps(payload).encode("utf-8"), headers=headers, method="POST")
        
        with urllib.request.urlopen(req, timeout=60) as response:
            res_body = response.read().decode("utf-8")
            log(f"Response received: Status {response.status}")
            data = json.loads(res_body)
            
            if "choices" in data and len(data["choices"]) > 0:
                content = data["choices"][0]["message"]["content"]
                if "usage" in data:
                    log(f"Token usage: {data['usage']}")
                
                # 去掉可能的 markdown 块
                if content.startswith("```"):
                    lines = content.splitlines()
                    if lines[0].startswith("```"):
                        content = "\n".join(lines[1:-1])
                
                log("Translation successful, outputting Rust code...")
                print(content.strip())
                sys.stdout.flush()
    except urllib.error.HTTPError as e:
        log(f"HTTP ERROR: {e.code} - {e.read().decode('utf-8')}")
        sys.exit(1)
    except Exception as e:
        log(f"ERROR: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
