use reqwest::blocking::Client;
use serde_json::{json};
use std::io::{Error, ErrorKind, Result, Write};
use colored::Colorize;
use serde::{Deserialize, Serialize};

const PROMPT: &str = "根据以下的git差异内容，生成一个简洁的提交信息，生成信息为纯文本，不包含markdown \"`\"等标识符，且生成内容不超过100 tokens。根据更改的性质，以以下其中一个前缀开头：'build'（构建系统），'chore'（杂务），'ci'（持续集成），'docs'（文档），'feat'（新功能），'fix'（修复），'perf'（性能），'refactor'（重构），'style'（样式），'test'（测试）：";


#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String, // 生成该 completion 的模型名
    object: String,
    system_fingerprint: String, // This fingerprint represents the backend configuration that the model runs with.
    choices: Vec<OpenAIResponseChoice>,
    usage: OpenAIResponseUsage, // 该对话补全请求的用量信息
    created: i64, // 创建聊天完成时的 Unix 时间戳（以秒为单位）
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: i64, // 该 completion 在模型生成的 completion 的选择列表中的索引。
    message: OpenAIResponseChoiceMessage,
    finish_reason: String, // 模型停止生成 token 的原因:stop/length/content_filter
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoiceMessage {
    role: String, // 角色:assistant
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

#[derive(Debug)]
pub struct LLMResult {
    pub commit_message: String,
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
}

pub fn openai_request(diff_content: &str) -> Result<LLMResult> {
    let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    let openai_url = String::from("https://api.deepseek.com");
    let openai_model = String::from("deepseek-chat");

    if openai_api_key.is_empty() || openai_url.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "OpenAI API key or URL is empty",
        ));
    }

    let client = Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", openai_url))
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&json!({
            "model": openai_model,
            "messages": [
                {
                    "role": "system",
                    "content": PROMPT,
                },
                {
                    "role": "user",
                    "content": diff_content
                }
            ],
            "max_tokens": 100
        }))
        .send()
        .expect("Error sending request");

    return if response.status().is_success() {
        let _response_json = OpenAIResponse {
            id: "".to_string(),
            model: "".to_string(),
            object: "".to_string(),
            system_fingerprint: "".to_string(),
            choices: vec![],
            usage: OpenAIResponseUsage {
                completion_tokens: 0,
                prompt_tokens: 0,
                total_tokens: 0,
            },
            created: 0,
        };
        let response_json: OpenAIResponse = response.json().expect("Error parsing response");

        if response_json.choices.get(0).unwrap().message.content != "".to_string() {
            Ok(
                LLMResult {
                    commit_message: response_json.choices.get(0).unwrap().message.content.clone().trim().to_string(),
                    total_tokens: response_json.usage.total_tokens,
                    prompt_tokens: response_json.usage.prompt_tokens,
                    completion_tokens: response_json.usage.completion_tokens,
                }
            )
        } else {
            eprintln!("Error: Could not parse response");
            return Err(Error::new(ErrorKind::Other, "Error: Could not parse response"));
        }
    } else {
        eprintln!(
            "Error: Request failed with status code: {}",
            response.status()
        );
        return Err(Error::new(ErrorKind::Other, response.text().unwrap()));
    };
}

pub fn confirm_commit(commit_message: &str) -> bool {
    // 屏幕上打印提交信息，询问用户是否确认提交，如果 回车 及提交，否则取消
    println!("{}", commit_message.cyan().bold());

    print!("Are you sure you want to commit? (Y/n) ");
    let mut input = String::new();

    // flush
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");

    return if input.trim() == "y" || input.trim() == "Y" || input.trim() == "" {
        true
    } else {
        false
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let diff_content = "diff --git a/src/ai/git.rs b/src/ai/git.rs\nindex 40149a0..f24f5c0 100644\n--- a/src/ai/git.rs\n+++ b/src/ai/git.rs\n@@ -17,6 +17,8 @@ fn git_stage_filenames() -> Vec<String> {\n }\n \n fn git_stage_diff() -> String {\n+    std::process::Command::new(\"cd\").args([\"/Users/fujianbang/Workspace/fujianbang/gitbuddy/src\"]).output().unwrap();\n+\n     let output = std::process::Command::new(\"git\")\n         .args([\"diff\", \"--cached\", \"--no-ext-diff\", \"--diff-algorithm=minimal\"]).output().unwrap();\n \ndiff --git a/src/llm/mod.rs b/src/llm/mod.rs\nnew file mode 100644\nindex 0000000..92ba671\n--- /dev/null\n+++ b/src/llm/mod.rs\n@@ -0,0 +1,69 @@\n+use std::collections::HashMap;\n+use reqwest::blocking::Client;\n+use serde_json::{json, Value};\n+use std::io::{Error, ErrorKind, Result};\n+\n+const PROMPT: &str = \"根据以下的git差异内容，生成一个简洁的提交信息。根据更改的性质，以以下其中一个前缀开头：'build'（构建系统），'chore'（杂务），'ci'（持续集成），'docs'（文档），'feat'（新功能），'fix'（修复），'perf'（性能），'refactor'（重构），'style'（样式），'test'（测试）：\";\n+\n+pub fn openai_request(diff_content: &str) -> Result<()> {\n+    let mut openai_api_key = String::from(\"sk-1c01228156d0400da5b97a0ff5815312\");\n+    let mut openai_url = String::from(\"https://api.deepseek.com\");\n+    let openai_model = String::from(\"deepseek-chat\");\n+\n+    if openai_api_key.is_empty() || openai_url.is_empty() {\n+        eprintln!(\"Error: OpenAI API key or URL is empty\");\n+        return Err(Error::new(\n+            ErrorKind::NotFound,\n+            \"OpenAI API key or URL is empty\",\n+        ));\n+    }\n+\n+    let client = Client::new();\n+    let response = client\n+        .post(format!(\"{}/v1/chat/completions\", openai_url))\n+        .header(\"Authorization\", format!(\"Bearer {}\", openai_api_key))\n+        .json(&json!({\n+            \"model\": openai_model,\n+            \"messages\": [\n+                {\n+                    \"role\": \"system\",\n+                    \"content\": PROMPT,\n+                },\n+                {\n+                    \"role\": \"user\",\n+                    \"content\": diff_content\n+                }\n+            ],\n+            \"max_tokens\": 60\n+        }))\n+        .send()\n+        .expect(\"Error sending request\");\n+\n+    if response.status().is_success() {\n+        let response_json: Value = response.json().expect(\"Error parsing response\");\n+        if let Some(text) = response_json[\"choices\"][0][\"message\"][\"content\"].as_str() {\n+            println!(\"{}\", text);\n+        } else {\n+            eprintln!(\"Error: Could not parse response\");\n+        }\n+    } else {\n+        eprintln!(\n+            \"Error: Request failed with status code: {}\",\n+            response.status()\n+        );\n+    }\n+\n+    Ok(())\n+}\n+\n+#[cfg(test)]\n+mod test {\n+    use super::*;\n+\n+    #[test]\n+    fn test() {\n+        let diff_content = \"diff --git a/src/ai/mod.rs b/src/ai/mod.rs\";\n+\n+        openai_request(diff_content).unwrap();\n+    }\n+}\ndiff --git a/src/main.rs b/src/main.rs\nindex 541e7b5..c4cdf6c 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -4,6 +4,7 @@ use clap::Parser;\n use crate::ai::{get_command, handle_command};\n \n mod ai;\n+mod llm;\n \n fn main() {\n     let matches = Command::new(\"gitbuddy\")\n";

        let r = openai_request(diff_content).unwrap();
        println!("{:?}", r);
    }
}
