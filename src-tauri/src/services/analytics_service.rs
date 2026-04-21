use crate::utils::{AppError, AppResult};
use std::sync::Arc;

const WINDSURF_BASE_URL: &str = "https://web-backend.windsurf.com";

pub struct AnalyticsService {
    client: Arc<reqwest::Client>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        // Use globally shared HTTP client to avoid creating new instance for each request
        Self {
            client: super::get_http_client(),
        }
    }

    /// Build GetAnalytics request body
    ///
    /// Request format:
    /// - field 2 (repeated QueryRequest): query request array
    /// - field 3 (start_timestamp): start time
    /// - field 4 (end_timestamp): end time
    /// Note: api_key is not in request body, authenticated via x-auth-token header
    ///
    /// QueryRequest field numbers:
    /// - 1: completion_stats
    /// - 2: completions_by_day
    /// - 3: completions_by_language
    /// - 10: chats_by_day
    /// - 12: chats_by_model
    /// - 15: percent_code_written
    /// - 17: chat_stats
    /// - 20: cascade_stats (used in official website request 3)
    /// - 23: cascade_lines
    /// - 24: cascade_tool_usage
    /// - 25: cascade_runs
    /// - 29: daily_active_user_counts
    /// - 31: cascade_summary (used in official website request 2, includes conversations, credits, commands, etc.)
    /// Build complete request body (includes all query types)
    fn build_get_analytics_body(&self, start_timestamp: i64, end_timestamp: i64, is_team: bool) -> Vec<u8> {
        let mut body = Vec::new();

        // ===== Cascade-related queries =====
        
        // Field 2: QueryRequest - cascade_stats (field 20 in QueryRequest) - Official website request 3
        // Tag: (20 << 3) | 2 = 0xA2 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xA2, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_lines (field 23 in QueryRequest)
        // Tag: (23 << 3) | 2 = 0xBA 0x01, empty message = 0x00
        body.extend_from_slice(&[0x12, 0x03, 0xBA, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_tool_usage (field 24 in QueryRequest)
        // Tag: (24 << 3) | 2 = 0xC2 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xC2, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_runs (field 25 in QueryRequest)
        // Tag: (25 << 3) | 2 = 0xCA 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xCA, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_summary (field 31 in QueryRequest) - official website request 2
        // Includes conversations, credits, commands, workflows, memories, terminal messages, etc.
        // Tag: (31 << 3) | 2 = 0xFA 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xFA, 0x01, 0x00]);

        // ===== Basic statistics queries (missing data will be ignored) =====
        // completion_stats (field 1)
        body.extend_from_slice(&[0x12, 0x02, 0x0A, 0x00]);
        // completions_by_language (field 3)
        body.extend_from_slice(&[0x12, 0x02, 0x1A, 0x00]);
        // chats_by_model (field 12)
        body.extend_from_slice(&[0x12, 0x02, 0x62, 0x00]);
        
        // ===== Queries with time_zone parameter =====
        let tz = b"Asia/Shanghai";
        let tz_msg_len = 2 + tz.len(); // 0x0A + len + tz_bytes
        let query_msg_len = 2 + tz_msg_len; // field_tag + len + inner_msg
        
        // completions_by_day (field 2)
        body.push(0x12); // QueryRequest tag
        body.push(query_msg_len as u8);
        body.push(0x12); // completions_by_day field tag (2 << 3 | 2)
        body.push(tz_msg_len as u8);
        body.push(0x0A); // time_zone field tag (1 << 3 | 2)
        body.push(tz.len() as u8);
        body.extend_from_slice(tz);

        // chats_by_day (field 10)
        body.push(0x12); // QueryRequest tag
        body.push(query_msg_len as u8);
        body.push(0x52); // chats_by_day field tag (10 << 3 | 2)
        body.push(tz_msg_len as u8);
        body.push(0x0A); // time_zone field tag
        body.push(tz.len() as u8);
        body.extend_from_slice(tz);

        // ===== Queries available only for team accounts =====
        if is_team {
            // percent_code_written (field 15)
            body.extend_from_slice(&[0x12, 0x02, 0x7A, 0x00]);
        }

        // Field 3: start_timestamp (google.protobuf.Timestamp)
        // Tag: (3 << 3) | 2 = 0x1A
        body.push(0x1A);
        let start_ts_bytes = Self::encode_timestamp(start_timestamp);
        body.push(start_ts_bytes.len() as u8);
        body.extend_from_slice(&start_ts_bytes);

        // Field 4: end_timestamp (google.protobuf.Timestamp)
        // Tag: (4 << 3) | 2 = 0x22
        body.push(0x22);
        let end_ts_bytes = Self::encode_timestamp(end_timestamp);
        body.push(end_ts_bytes.len() as u8);
        body.extend_from_slice(&end_ts_bytes);

        // Note: api_key is not in request body, authentication is completed via x-auth-token header

        body
    }

    /// Build simplified request body (only cascade data, for fallback)
    fn build_cascade_only_body(&self, start_timestamp: i64, end_timestamp: i64) -> Vec<u8> {
        let mut body = Vec::new();

        // Only request cascade-related queries (consistent with official website)
        body.extend_from_slice(&[0x12, 0x03, 0xA2, 0x01, 0x00]); // cascade_stats (field 20)
        body.extend_from_slice(&[0x12, 0x03, 0xBA, 0x01, 0x00]); // cascade_lines (field 23)
        body.extend_from_slice(&[0x12, 0x03, 0xC2, 0x01, 0x00]); // cascade_tool_usage (field 24)
        body.extend_from_slice(&[0x12, 0x03, 0xCA, 0x01, 0x00]); // cascade_runs (field 25)
        body.extend_from_slice(&[0x12, 0x03, 0xFA, 0x01, 0x00]); // cascade_summary (field 31)

        // start_timestamp (without nanos)
        body.push(0x1A);
        let start_ts_bytes = Self::encode_timestamp(start_timestamp);
        body.push(start_ts_bytes.len() as u8);
        body.extend_from_slice(&start_ts_bytes);

        // end_timestamp (with nanos, consistent with official website)
        body.push(0x22);
        let end_ts_bytes = Self::encode_timestamp_with_nanos(end_timestamp, 999999000);
        body.push(end_ts_bytes.len() as u8);
        body.extend_from_slice(&end_ts_bytes);

        body
    }

    /// Build request body without timestamp (final fallback, imitating official website request 1)
    fn build_no_timestamp_body(&self) -> Vec<u8> {
        let mut body = Vec::new();
        // cascade-related queries (without timestamp)
        body.extend_from_slice(&[0x12, 0x03, 0xA2, 0x01, 0x00]); // cascade_stats (field 20)
        body.extend_from_slice(&[0x12, 0x03, 0xCA, 0x01, 0x00]); // cascade_runs (field 25)
        body.extend_from_slice(&[0x12, 0x03, 0xFA, 0x01, 0x00]); // cascade_summary (field 31)
        body
    }

    /// Encode Timestamp to Protobuf format (only seconds, for start_timestamp)
    fn encode_timestamp(timestamp: i64) -> Vec<u8> {
        let mut ts_body = Vec::new();
        // Field 1: seconds - Tag: (1 << 3) | 0 = 0x08
        ts_body.push(0x08);
        ts_body.extend_from_slice(&Self::encode_varint(timestamp as u64));
        ts_body
    }

    /// Encode Timestamp to Protobuf format (includes nanos, for end_timestamp)
    /// Official website's end_timestamp includes nanos field, value is usually a fixed value
    fn encode_timestamp_with_nanos(timestamp: i64, nanos: i32) -> Vec<u8> {
        let mut ts_body = Vec::new();
        // Field 1: seconds
        ts_body.push(0x08);
        ts_body.extend_from_slice(&Self::encode_varint(timestamp as u64));
        // Field 2: nanos - Tag: (2 << 3) | 0 = 0x10
        if nanos != 0 {
            ts_body.push(0x10);
            ts_body.extend_from_slice(&Self::encode_varint(nanos as u64));
        }
        ts_body
    }

    /// Encode varint
    fn encode_varint(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if value == 0 {
                break;
            }
        }
        result
    }

    /// Call GetAnalytics API
    /// auth_token: Firebase JWT Token (for x-auth-token header)
    /// is_team: whether it's a team account, team accounts will additionally request percent_code_written
    pub async fn get_analytics(&self, auth_token: &str, start_timestamp: i64, end_timestamp: i64, is_team: bool) -> AppResult<Vec<u8>> {
        let url = format!("{}/exa.user_analytics_pb.UserAnalyticsService/GetAnalytics", WINDSURF_BASE_URL);
        
        let body = self.build_get_analytics_body(start_timestamp, end_timestamp, is_team);

        println!("[GetAnalytics] Calling API with time range: {} - {}", start_timestamp, end_timestamp);
        println!("[GetAnalytics] Request body length: {} bytes", body.len());
        println!("[GetAnalytics] Request body hex: {}", body.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        
        let response = self.client
            .post(&url)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("sec-ch-ua", "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-auth-token", auth_token)
            .header("Referer", "https://windsurf.com/")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Api(format!("Request failed: {}", e)))?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(format!("Failed to read response: {}", e)))?
            .to_vec();
        
        println!("[GetAnalytics] Response status: {}", status_code);
        println!("[GetAnalytics] Response body length: {} bytes", response_body.len());

        if status_code != 200 {
            // Try to parse error response
            if let Ok(error_text) = String::from_utf8(response_body.clone()) {
                println!("[GetAnalytics] Error response: {}", error_text);
                return Err(AppError::Api(format!("API returned status code {}: {}", status_code, error_text)));
            }
            return Err(AppError::Api(format!("API returned status code: {}", status_code)));
        }
        
        Ok(response_body)
    }

    /// Fallback request: only get cascade data (used when complete request fails)
    pub async fn get_analytics_cascade_only(&self, auth_token: &str, start_timestamp: i64, end_timestamp: i64) -> AppResult<Vec<u8>> {
        let url = format!("{}/exa.user_analytics_pb.UserAnalyticsService/GetAnalytics", WINDSURF_BASE_URL);
        
        let body = self.build_cascade_only_body(start_timestamp, end_timestamp);

        println!("[GetAnalytics-CascadeOnly] Fallback request with cascade-only queries");
        println!("[GetAnalytics-CascadeOnly] Request body length: {} bytes", body.len());
        
        let response = self.client
            .post(&url)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("sec-ch-ua", "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-auth-token", auth_token)
            .header("Referer", "https://windsurf.com/")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Api(format!("Request failed: {}", e)))?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(format!("Failed to read response: {}", e)))?
            .to_vec();
        
        println!("[GetAnalytics-CascadeOnly] Response status: {}", status_code);

        if status_code != 200 {
            if let Ok(error_text) = String::from_utf8(response_body.clone()) {
                println!("[GetAnalytics-CascadeOnly] Error response: {}", error_text);
                return Err(AppError::Api(format!("API returned status code {}: {}", status_code, error_text)));
            }
            return Err(AppError::Api(format!("API returned status code: {}", status_code)));
        }
        
        Ok(response_body)
    }

    /// Final fallback request: no timestamp, only cascade_runs (imitating official website request 1)
    pub async fn get_analytics_no_timestamp(&self, auth_token: &str) -> AppResult<Vec<u8>> {
        let url = format!("{}/exa.user_analytics_pb.UserAnalyticsService/GetAnalytics", WINDSURF_BASE_URL);
        
        let body = self.build_no_timestamp_body();

        println!("[GetAnalytics-NoTimestamp] Final fallback with no timestamp");
        println!("[GetAnalytics-NoTimestamp] Request body: {:?}", body.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        
        let response = self.client
            .post(&url)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("sec-ch-ua", "\"Chromium\";v=\"143\", \"Google Chrome\";v=\"143\", \"Not A(Brand\";v=\"24\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-auth-token", auth_token)
            .header("Referer", "https://windsurf.com/")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Api(format!("Request failed: {}", e)))?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(format!("Failed to read response: {}", e)))?
            .to_vec();
        
        println!("[GetAnalytics-NoTimestamp] Response status: {}", status_code);

        if status_code != 200 {
            if let Ok(error_text) = String::from_utf8(response_body.clone()) {
                println!("[GetAnalytics-NoTimestamp] Error response: {}", error_text);
                return Err(AppError::Api(format!("API returned status code {}: {}", status_code, error_text)));
            }
            return Err(AppError::Api(format!("API returned status code: {}", status_code)));
        }
        
        Ok(response_body)
    }
}
