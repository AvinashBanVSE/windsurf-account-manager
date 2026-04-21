use crate::services::auth_context::{AuthContext, AuthHeaderExt};
use crate::utils::{AppError, AppResult};
use base64::{Engine, engine::general_purpose};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

const WINDSURF_BASE_URL: &str = "https://web-backend.windsurf.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSeatsResult {
    pub success: bool,
    pub attempts: Vec<AttemptResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttemptResult {
    pub attempt: usize,
    pub status_code: Option<u16>,
    pub raw_response: Option<String>,
    pub error: Option<String>,
    pub timestamp: String,
}


pub struct WindsurfService {
    client: Arc<reqwest::Client>,
}

impl WindsurfService {
    pub fn new() -> Self {
        // Use globally shared HTTP client to avoid creating new instance for each request
        Self {
            client: super::get_http_client(),
        }
    }

    fn build_request_body(&self, token: &str, seat_count: i32) -> Vec<u8> {
        // UpdateSeats body format: 0x0a + token length(varint) + token + 0x10 + seat_count
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut body = vec![0x0a];
        
        // Token length (using varint encoding)
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            // For JWT token (usually >1000 bytes), need two-byte varint
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }
        
        // Token content
        body.extend_from_slice(token_bytes);
        
        // Seat count (field 2, varint)
        body.push(0x10);
        body.push(seat_count as u8);
        
        body
    }

    /// Build update plan request body
    /// 
    /// Protobuf structure (UpdatePlanRequest):
    /// - Field 1 (LengthDelimited): auth_token (string)
    /// - Field 2 (Varint): price (StripePrice enum)
    /// - Field 3 (Varint): preview (bool) - preview mode
    /// - Field 4 (Varint): payment_period (PaymentPeriod enum: 1=monthly, 2=yearly)
    /// - Field 5 (Varint): teams_tier (TeamsTier enum: 1-11)
    fn build_update_plan_body(&self, token: &str, plan_type: &str, payment_period: u8, preview: bool) -> Vec<u8> {
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();

        let mut body = vec![0x0a];

        // Token length (using varint encoding)
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }

        body.extend_from_slice(token_bytes);
        
        // Field 2: price (StripePrice)
        // 1 = STRIPE_PRICE_TEAMS_MONTHLY (monthly price)
        // 2 = STRIPE_PRICE_TEAMS_YEARLY (yearly price)
        body.push(0x10);
        body.push(if payment_period == 2 { 0x02 } else { 0x01 });
        
        // Field 3: preview (bool) - 0x18 = field 3 varint
        if preview {
            body.push(0x18);
            body.push(0x01);
        }
        
        // Field 4: payment_period (0x20 = field 4 varint)
        // 1 = PAYMENT_PERIOD_MONTH (monthly)
        // 2 = PAYMENT_PERIOD_YEAR (yearly)
        body.push(0x20);
        body.push(if payment_period == 2 { 0x02 } else { 0x01 });
        
        // Field 5: teams_tier (0x28 = field 5 varint)
        body.push(0x28);

        // Add different suffix bytes based on subscription type (TeamsTier enum values)
        match plan_type.to_lowercase().as_str() {
            "free" => body.push(0x00),                     // 0 = TEAMS_TIER_UNSPECIFIED (Free)
            "teams" => body.push(0x01),                    // 1 = TEAMS_TIER_TEAMS
            "pro" => body.push(0x02),                      // 2 = TEAMS_TIER_PRO
            "enterprise_saas" => body.push(0x03),          // 3 = TEAMS_TIER_ENTERPRISE_SAAS
            "hybrid" => body.push(0x04),                   // 4 = TEAMS_TIER_HYBRID
            "enterprise_self_hosted" => body.push(0x05),   // 5 = TEAMS_TIER_ENTERPRISE_SELF_HOSTED
            "waitlist_pro" => body.push(0x06),             // 6 = TEAMS_TIER_WAITLIST_PRO
            "teams_ultimate" => body.push(0x07),           // 7 = TEAMS_TIER_TEAMS_ULTIMATE
            "pro_ultimate" => body.push(0x08),             // 8 = TEAMS_TIER_PRO_ULTIMATE
            "trial" => body.push(0x09),                    // 9 = TEAMS_TIER_TRIAL
            "enterprise_self_serve" => body.push(0x0a),    // 10 = TEAMS_TIER_ENTERPRISE_SELF_SERVE
            "enterprise_saas_pooled" => body.push(0x0b),   // 11 = TEAMS_TIER_ENTERPRISE_SAAS_POOLED
            "devin_enterprise" => body.push(0x0c),         // 12 = TEAMS_TIER_DEVIN_ENTERPRISE
            "devin_teams" => body.push(0x0e),              // 14 = TEAMS_TIER_DEVIN_TEAMS
            "devin_teams_v2" => body.push(0x0f),           // 15 = TEAMS_TIER_DEVIN_TEAMS_V2
            "devin_pro" => body.push(0x10),                // 16 = TEAMS_TIER_DEVIN_PRO
            "devin_max" => body.push(0x11),                // 17 = TEAMS_TIER_DEVIN_MAX
            "max" => body.push(0x12),                      // 18 = TEAMS_TIER_MAX
            "devin_free" => body.push(0x13),               // 19 = TEAMS_TIER_DEVIN_FREE
            "devin_trial" => body.push(0x14),              // 20 = TEAMS_TIER_DEVIN_TRIAL
            "enterprise" | _ => body.push(0x0a),           // Default to ENTERPRISE_SELF_SERVE
        }

        body
    }

    /// Build cancel subscription request body
    ///
    /// Protobuf structure：
    /// - Field 1 (LengthDelimited): Firebase ID Token
    /// - Field 2 (Varint): 1 (indicates cancel operation)
    /// - Field 5 (LengthDelimited): cancellation reason string
    fn build_cancel_plan_body(&self, token: &str, reason: &str) -> Vec<u8> {
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        let reason_bytes = reason.as_bytes();
        let reason_length = reason_bytes.len();

        let mut body = vec![0x0a]; // Field 1, wire type 2 (LengthDelimited)

        // Token length (using varint encoding)
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }

        // Token content
        body.extend_from_slice(token_bytes);

        // Field 2: int32 = 1 (indicates cancel operation)
        body.push(0x10); // Field 2, wire type 0 (Varint)
        body.push(0x01); // value = 1

        // Field 5: cancellation reason string
        body.push(0x2a); // Field 5, wire type 2 (LengthDelimited)

        // Reason string length
        if reason_length < 128 {
            body.push(reason_length as u8);
        } else {
            body.push(((reason_length & 0x7F) | 0x80) as u8);
            body.push((reason_length >> 7) as u8);
        }

        // Reason string content
        body.extend_from_slice(reason_bytes);

        body
    }

    /// Build resume subscription request body
    ///
    /// Protobuf structure：
    /// - Field 1 (LengthDelimited): Firebase ID Token
    /// - Field 3 (Varint): 1 (indicates resume operation)
    fn build_resume_plan_body(&self, token: &str) -> Vec<u8> {
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();

        let mut body = vec![0x0a]; // Field 1, wire type 2 (LengthDelimited)

        // Token length (using varint encoding)
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }

        // Token content
        body.extend_from_slice(token_bytes);

        // Field 3: int32 = 1 (indicates resume operation)
        body.push(0x18); // Field 3, wire type 0 (Varint)
        body.push(0x01); // value = 1

        body
    }

    fn build_subscribe_to_plan_body(
        &self, 
        token: &str, 
        success_url: &str, 
        cancel_url: &str, 
        teams_tier: i32,
        payment_period: i32,
        start_trial: bool,
        team_name: Option<&str>,
        seats: Option<i32>,
        turnstile_token: Option<&str>
    ) -> Vec<u8> {
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        let success_url_bytes = success_url.as_bytes();
        let success_url_length = success_url_bytes.len();
        let cancel_url_bytes = cancel_url.as_bytes();
        let cancel_url_length = cancel_url_bytes.len();

        let mut body = Vec::new();

        // Field 1: auth_token (string, field number 1, wire type 2)
        body.push(0x0a); // field 1, wire type 2 (length-delimited)
        let mut len = token_length;
        while len >= 0x80 {
            body.push(((len & 0x7F) | 0x80) as u8);
            len >>= 7;
        }
        body.push(len as u8);
        body.extend_from_slice(token_bytes);

        // Field 3: start_trial (bool, field number 3, wire type 0)
        if start_trial {
            body.push(0x18); // field 3, wire type 0 (0x18 = (3 << 3) | 0)
            body.push(0x01); // value = true
        }

        // Field 4: Success URL (string, field number 4, wire type 2)
        body.push(0x22); // field 4, wire type 2 (0x22 = (4 << 3) | 2)
        body.push(success_url_length as u8);
        body.extend_from_slice(success_url_bytes);

        // Field 5: Cancel URL (string, field number 5, wire type 2)
        body.push(0x2a); // field 5, wire type 2 (0x2a = (5 << 3) | 2)
        body.push(cancel_url_length as u8);
        body.extend_from_slice(cancel_url_bytes);

        // Field 6: seats (int64, field number 6, wire type 0)
        // All team/enterprise plans require seats, individual plans (Pro/Max/Trial/Free, etc.) do not set
        if matches!(teams_tier, 1 | 3 | 4 | 5 | 7 | 10 | 11 | 12 | 14 | 15) {
            let seat_count = seats.unwrap_or(1);
            if seat_count > 0 {
                body.push(0x30); // field 6, wire type 0 (0x30 = (6 << 3) | 0)
                body.push(seat_count as u8);
            }
        }

        // Field 7: team_name (string, field number 7, wire type 2) - Teams/Enterprise required
        if let Some(name) = team_name {
            if !name.is_empty() {
                let name_bytes = name.as_bytes();
                body.push(0x3a); // field 7, wire type 2 (0x3a = (7 << 3) | 2)
                body.push(name_bytes.len() as u8);
                body.extend_from_slice(name_bytes);
            }
        }

        // Field 8: teams_tier (enum, field number 8, wire type 0)
        body.push(0x40); // field 8, wire type 0 (varint)
        body.push(teams_tier as u8);

        // Field 9: payment_period (enum, field number 9, wire type 0)
        body.push(0x48); // field 9, wire type 0 (varint)
        body.push(payment_period as u8);

        // Field 10: turnstile_token (string, field number 10, wire type 2) - required for all plans when start_trial=true
        if let Some(turnstile) = turnstile_token {
            let turnstile_bytes = turnstile.as_bytes();
            body.push(0x52); // field 10, wire type 2 (0x52 = (10 << 3) | 2)
            let mut tlen = turnstile_bytes.len();
            while tlen >= 0x80 {
                body.push(((tlen & 0x7F) | 0x80) as u8);
                tlen >>= 7;
            }
            body.push(tlen as u8);
            body.extend_from_slice(turnstile_bytes);
        }

        body
    }

    pub async fn update_seats(&self, ctx: &AuthContext, seat_count: i32, retry_times: i32) -> AppResult<UpdateSeatsResult> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdateSeats", WINDSURF_BASE_URL);
        
        let mut attempts = Vec::new();
        let mut success = false;
        
        for i in 0..retry_times {
            let body = self.build_request_body(token, seat_count);
            
            let result = self.client
                .post(&url)
                .body(body)
                .header("accept", "*/*")
                .header("accept-language", "zh-CN,zh;q=0.9")
                .header("cache-control", "no-cache")
                .header("connect-protocol-version", "1")
                .header("content-type", "application/proto")
                .header("pragma", "no-cache")
                .header("priority", "u=1, i")
                .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
                .header("sec-ch-ua-mobile", "?0")
                .header("sec-ch-ua-platform", r#""Windows""#)
                .header("sec-fetch-dest", "empty")
                .header("sec-fetch-mode", "cors")
                .header("sec-fetch-site", "same-site")
                .with_auth(ctx)
                .header("x-debug-email", "")
                .header("x-debug-team-name", "")
                .header("Referer", "https://windsurf.com/")
                .send()
                .await;
            
            match result {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let response_bytes = response.bytes().await.unwrap_or_default();
                    
                    // Try to parse response
                    let mut raw_response = String::from_utf8_lossy(&response_bytes).to_string();
                    let mut parsed_data = None;
                    
                    // 200 or 204 both indicate success
                    if status_code == 200 || status_code == 204 {
                        // Try to parse Protobuf response
                        if response_bytes.len() > 0 {
                            match crate::services::proto_parser::ProtobufParser::parse_update_seats_response(&response_bytes) {
                                Ok(parsed) => {
                                    println!("[UpdateSeats] Successfully parsed response: {:?}", parsed);
                                    parsed_data = Some(parsed.clone());
                                    
                                    // Check parsed success status
                                    if let Some(parsed_success) = parsed.get("success").and_then(|v| v.as_bool()) {
                                        success = parsed_success;
                                    } else {
                                        success = true; // If no explicit failure flag, treat as success
                                    }
                                    
                                    // Construct more detailed response
                                    raw_response = parsed.to_string();
                                },
                                Err(e) => {
                                    println!("[UpdateSeats] Failed to parse response: {}", e);
                                    // Parse failed but status code is 200/204, still treat as success
                                    success = true;
                                }
                            }
                        } else {
                            success = true; // 204 No Content
                        }
                    }
                    
                    // Construct attempt result
                    let mut attempt_result = AttemptResult {
                        attempt: i as usize + 1,
                        status_code: Some(status_code),
                        raw_response: Some(raw_response),
                        error: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    
                    // If there is parsed data, add to result
                    if let Some(data) = parsed_data {
                        // Store parsed data as JSON string
                        if let Ok(json_str) = serde_json::to_string_pretty(&data) {
                            attempt_result.raw_response = Some(json_str);
                        }
                    }
                    
                    attempts.push(attempt_result);
                    
                    // If successful, return directly, no need to continue retrying
                    if success {
                        break;
                    }
                },
                Err(e) => {
                    attempts.push(AttemptResult {
                        attempt: i as usize + 1,
                        status_code: None,
                        raw_response: None,
                        error: Some(e.to_string()),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
            
            // Slight delay between two requests
            if i < retry_times - 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        
        Ok(UpdateSeatsResult {
            success,
            attempts,
        })
    }

    pub async fn get_team_credit_entries(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetTeamCreditEntries", WINDSURF_BASE_URL);
        
        // GetTeamCreditEntries body format: 0x0a + token length + token
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut full_body = vec![0x0a];
        
        // Token length (using varint encoding)
        if token_length < 128 {
            full_body.push(token_length as u8);
        } else {
            full_body.push(((token_length & 0x7F) | 0x80) as u8);
            full_body.push((token_length >> 7) as u8);
        }
        
        full_body.extend_from_slice(token_bytes);
        
        println!("[GetTeamCreditEntries] Sending request to {}", url);
        println!("[GetTeamCreditEntries] Token length: {} bytes", token_length);
        println!("[GetTeamCreditEntries] Request body length: {} bytes", full_body.len());
        
        // Print first few bytes for debugging
        if full_body.len() >= 3 {
            println!("[GetTeamCreditEntries] Body prefix: {:02x} {:02x} {:02x}", full_body[0], full_body[1], full_body[2]);
        }
        
        let result = self.client
            .post(&url)
            .body(full_body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .with_auth(ctx)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await;
        
        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                println!("[GetTeamCreditEntries] Response status: {}", status_code);
                
                let response_bytes = response.bytes().await.unwrap_or_default();
                println!("[GetTeamCreditEntries] Response size: {} bytes", response_bytes.len());
                
                if status_code == 200 {
                    // Empty response may indicate no credit entries
                    if response_bytes.len() == 0 {
                        println!("[GetTeamCreditEntries] Empty response - no credit entries found");
                        return Ok(json!({
                            "success": true,
                            "entries": [],
                            "total_entries": 0,
                            "message": "This team has no credit entries yet"
                        }));
                    }
                    // Print first 100 bytes of response for debugging
                    let preview = if response_bytes.starts_with(b"data:application/proto;base64,") {
                        "Base64 encoded response"
                    } else {
                        "Binary response"
                    };
                    println!("[GetTeamCreditEntries] Response format: {}", preview);
                    
                    // Try to parse Protobuf response
                    match crate::services::proto_parser::ProtobufParser::parse_get_team_credit_entries_response(&response_bytes) {
                        Ok(parsed) => {
                            println!("[GetTeamCreditEntries] Successfully parsed credit entries response");
                            println!("[GetTeamCreditEntries] Total entries: {}", 
                                parsed.get("total_entries").and_then(|v| v.as_i64()).unwrap_or(0));
                            Ok(parsed)
                        },
                        Err(e) => {
                            println!("[GetTeamCreditEntries] Failed to parse response: {}", e);
                            // Return raw response for debugging
                            let raw_response = if response_bytes.starts_with(b"data:application/proto;base64,") {
                                String::from_utf8_lossy(&response_bytes).to_string()
                            } else {
                                format!("data:application/proto;base64,{}", general_purpose::STANDARD.encode(&response_bytes))
                            };
                            Ok(json!({
                                "success": false,
                                "error": format!("Parse error: {}", e),
                                "raw_response": raw_response
                            }))
                        }
                    }
                } else {
                    println!("[GetTeamCreditEntries] Unexpected status code: {}", status_code);
                    Ok(json!({
                        "success": false,
                        "status_code": status_code,
                        "error": format!("HTTP error: {}", status_code)
                    }))
                }
            },
            Err(e) => {
                println!("[GetTeamCreditEntries] Request failed: {}", e);
                Ok(json!({
                    "success": false,
                    "error": format!("Request failed: {}", e)
                }))
            }
        }
    }
    
    pub async fn get_team_billing(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetTeamBilling", WINDSURF_BASE_URL);
        
        // GetTeamBilling body format: 0x0a + token length + token
        // Note: Not 0x0a 0xa1 0x07, that's for UpdatePlan
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut full_body = vec![0x0a];
        
        // Token length (using varint encoding)
        if token_length < 128 {
            full_body.push(token_length as u8);
        } else {
            full_body.push(((token_length & 0x7F) | 0x80) as u8);
            full_body.push((token_length >> 7) as u8);
        }
        
        full_body.extend_from_slice(token_bytes);
        
        println!("[GetTeamBilling] Sending request to {}", url);
        
        let result = self.client
            .post(&url)
            .body(full_body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .with_auth(ctx)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await;
        
        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                println!("[GetTeamBilling] Response status: {}", status_code);
                
                let response_bytes = response.bytes().await.unwrap_or_default();
                println!("[GetTeamBilling] Response size: {} bytes", response_bytes.len());
                
                if status_code == 200 && response_bytes.len() > 0 {
                    // Try to parse Protobuf response
                    match crate::services::proto_parser::ProtobufParser::parse_get_team_billing_response(&response_bytes) {
                        Ok(parsed) => {
                            println!("[GetTeamBilling] Successfully parsed billing response");
                            Ok(parsed)
                        },
                        Err(e) => {
                            println!("[GetTeamBilling] Failed to parse response: {}", e);
                            Ok(json!({
                                "success": false,
                                "error": format!("Parse error: {}", e),
                                "raw_response": general_purpose::STANDARD.encode(&response_bytes)
                            }))
                        }
                    }
                } else {
                    Ok(json!({
                        "success": false,
                        "status_code": status_code,
                        "error": "Invalid response"
                    }))
                }
            },
            Err(e) => {
                println!("[GetTeamBilling] Request failed: {}", e);
                Ok(json!({
                    "success": false,
                    "error": e.to_string()
                }))
            }
        }
    }

    /// Update subscription plan
    /// 
    /// # Arguments
    /// * `token` - Firebase ID Token
    /// * `plan_type` - Plan type (teams, pro, enterprise, etc.)
    /// * `payment_period` - Payment period (1=monthly, 2=yearly)
    /// * `preview` - Preview mode (true=preview only, no actual execution)
    pub async fn update_plan(&self, ctx: &AuthContext, plan_type: &str, payment_period: u8, preview: bool) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdatePlan", WINDSURF_BASE_URL);
        
        // Validate payment_period
        let period = if payment_period == 2 { 2 } else { 1 };
        let period_name = if period == 2 { "yearly" } else { "monthly" };
        
        println!("[UpdatePlan] plan_type={}, period={}, preview={}", plan_type, period_name, preview);
        
        let body = self.build_update_plan_body(token, plan_type, period, preview);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;
        
        let status_code = response.status().as_u16();
        let response_bytes = response.bytes().await.unwrap_or_default();
        
        println!("[UpdatePlan] Response status: {}, size: {} bytes", status_code, response_bytes.len());
        
        // Try to parse Protobuf response
        if status_code == 200 && response_bytes.len() > 0 {
            match crate::services::proto_parser::ProtobufParser::parse_update_plan_response(&response_bytes) {
                Ok(parsed) => {
                    println!("[UpdatePlan] Successfully parsed response");
                    
                    // Check if there is payment failure reason
                    let payment_failure = parsed.get("payment_failure_reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    let applied_changes = parsed.get("applied_changes")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    
                    return Ok(serde_json::json!({
                        "success": payment_failure.is_empty() && (preview || applied_changes),
                        "preview": preview,
                        "plan_type": plan_type,
                        "payment_period": period,
                        "payment_period_name": period_name,
                        "status_code": status_code,
                        "applied_changes": applied_changes,
                        "payment_failure_reason": if payment_failure.is_empty() { None } else { Some(payment_failure) },
                        "billing_update": parsed.get("billing_update"),
                        "requires_password_reset": parsed.get("requires_password_reset"),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }));
                },
                Err(e) => {
                    println!("[UpdatePlan] Failed to parse response: {}", e);
                }
            }
        }
        
        // Return raw response when parsing fails
        let raw_response = if response_bytes.starts_with(b"data:application/proto;base64,") {
            String::from_utf8_lossy(&response_bytes).to_string()
        } else {
            format!("data:application/proto;base64,{}", general_purpose::STANDARD.encode(&response_bytes))
        };
        
        Ok(serde_json::json!({
            "success": status_code == 200,
            "preview": preview,
            "plan_type": plan_type,
            "payment_period": period,
            "payment_period_name": period_name,
            "status_code": status_code,
            "raw_response": raw_response,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Cancel subscription
    ///
    /// # Arguments
    /// * `token` - Firebase ID Token
    /// * `reason` - Cancellation reason (e.g.: "too_expensive", "not_using", "missing_features", "switching_service", "other")
    ///
    /// # Returns
    /// Returns JSON object containing operation result
    pub async fn cancel_plan(&self, ctx: &AuthContext, reason: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/CancelPlan", WINDSURF_BASE_URL);

        println!("[CancelPlan] Canceling subscription with reason: {}", reason);

        let body = self.build_cancel_plan_body(token, reason);

        println!("[CancelPlan] Request body length: {} bytes", body.len());
        println!("[CancelPlan] Request body hex: {}", body.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-api-key", token)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let response_bytes = response.bytes().await.unwrap_or_default();
        let response_text = String::from_utf8_lossy(&response_bytes).to_string();

        println!("[CancelPlan] Response status: {}", status_code);
        println!("[CancelPlan] Response length: {} bytes", response_bytes.len());

        Ok(serde_json::json!({
            "success": status_code == 200,
            "reason": reason,
            "status_code": status_code,
            "raw_response": response_text,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Resume subscription
    ///
    /// # Arguments
    /// * `token` - Firebase ID Token
    ///
    /// # Returns
    /// Returns JSON object containing operation result
    pub async fn resume_plan(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/CancelPlan", WINDSURF_BASE_URL);

        println!("[ResumePlan] Resuming subscription");

        let body = self.build_resume_plan_body(token);

        println!("[ResumePlan] Request body length: {} bytes", body.len());
        println!("[ResumePlan] Request body hex: {}", body.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-api-key", token)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let response_bytes = response.bytes().await.unwrap_or_default();
        let response_text = String::from_utf8_lossy(&response_bytes).to_string();

        println!("[ResumePlan] Response status: {}", status_code);
        println!("[ResumePlan] Response length: {} bytes", response_bytes.len());

        Ok(serde_json::json!({
            "success": status_code == 200,
            "status_code": status_code,
            "raw_response": response_text,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Get one-time auth_token (for Windsurf desktop client OAuth callback login)
    ///
    /// # Background
    /// Windsurf desktop client triggers login via `windsurf://codeium.windsurf#access_token=<one_time_auth_token>`
    /// This one-time ticket is issued by backend `GetOneTimeAuthToken` RPC.
    ///
    /// # Authentication Compatibility
    /// - Firebase account: input `auth_token` = Firebase ID Token, request only needs `x-auth-token` header
    /// - Devin account: input `auth_token` = session_token in form `devin-session-token$...`;
    ///   Request must also include `X-Devin-Auth1-Token` / `X-Devin-Account-Id` /
    ///   `X-Devin-Primary-Org-Id` / `X-Devin-Session-Token` 4 extension headers
    ///
    /// `with_auth(ctx)` will automatically route based on `ctx.devin`, caller doesn't need to be aware of specific account system.
    ///
    /// # Returns
    /// Returns one-time auth_token string on success
    pub async fn get_one_time_auth_token(&self, ctx: &AuthContext) -> AppResult<String> {
        let token = ctx.token_str();
        let url = format!(
            "{}/exa.seat_management_pb.SeatManagementService/GetOneTimeAuthToken",
            WINDSURF_BASE_URL
        );

        // Request body: GetOneTimeAuthTokenRequest { auth_token = 1 }
        let body = self.encode_string_field(1, token);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!(
            "[GetOneTimeAuthToken] Status: {}, Size: {} bytes",
            status_code,
            response_body.len()
        );

        if status_code != 200 {
            let err_text = String::from_utf8_lossy(&response_body).to_string();
            return Err(AppError::Api(format!(
                "GetOneTimeAuthToken request failed: status={}, body={}",
                status_code, err_text
            )));
        }

        // Response body: GetOneTimeAuthTokenResponse { auth_token = 1 }
        let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
        let parsed = parser.parse_message()
            .map_err(|e| AppError::Api(format!("Failed to parse GetOneTimeAuthToken response: {}", e)))?;

        let auth_token = parsed.get("string_1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Api("auth_token field not found in GetOneTimeAuthToken response".to_string()))?;

        Ok(auth_token.to_string())
    }

    pub async fn get_current_user(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetCurrentUser", WINDSURF_BASE_URL);
        
        // Build request body: 0x0a + token length(varint) + token + 0x10 0x01 0x18 0x01 0x20 0x01
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut body = vec![0x0a];
        
        // Add varint-encoded token length
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }
        
        body.extend_from_slice(token_bytes);
        
        // Add additional fields
        body.extend_from_slice(&[0x10, 0x01, 0x18, 0x01, 0x20, 0x01]);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .with_auth(ctx)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await?;
        
        println!("[GetCurrentUser] Status code: {}", status_code);
        println!("[GetCurrentUser] Response size: {} bytes", response_body.len());
        
        if status_code == 200 {
            // Use proto_parser to parse response
            match super::proto_parser::parse_get_current_user_response(&response_body) {
                Ok(parsed_result) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "status_code": status_code,
                        "parsed_data": parsed_result["parsed_data"],
                        "user_info": parsed_result["user_info"],
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                },
                Err(parse_error) => {
                    // Parse failed, return raw response
                    let response_str = String::from_utf8_lossy(&response_body);
                    let base64_data = if response_str.starts_with("data:application/proto;base64,") {
                        &response_str[31..]
                    } else {
                        &response_str
                    };
                    
                    Ok(serde_json::json!({
                        "success": true,
                        "status_code": status_code,
                        "raw_response": base64_data.trim(),
                        "parse_error": parse_error,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                }
            }
        } else {
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get current user",
                "raw_response": String::from_utf8_lossy(&response_body).to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get plan status (credits/quota information)
    /// Lighter than GetCurrentUser, specifically for refreshing credit status
    pub async fn get_plan_status(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetPlanStatus", WINDSURF_BASE_URL);
        
        // Build request body: GetPlanStatusRequest { auth_token = 1 }
        // Format: 0x0a + token length(varint) + token
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut body = vec![0x0a];
        
        // Add varint-encoded token length
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }
        
        body.extend_from_slice(token_bytes);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .with_auth(ctx)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await?;
        
        println!("[GetPlanStatus] Status code: {}", status_code);
        println!("[GetPlanStatus] Response size: {} bytes", response_body.len());
        
        if status_code == 200 {
            // Use proto_parser to parse response
            match super::proto_parser::ProtobufParser::parse_get_plan_status_response(&response_body) {
                Ok(parsed_result) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "status_code": status_code,
                        "plan_status": parsed_result,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                },
                Err(parse_error) => {
                    // Parse failed, return raw response
                    let response_str = String::from_utf8_lossy(&response_body);
                    let base64_data = if response_str.starts_with("data:application/proto;base64,") {
                        &response_str[31..]
                    } else {
                        &response_str
                    };
                    
                    Ok(serde_json::json!({
                        "success": true,
                        "status_code": status_code,
                        "raw_response": base64_data.trim(),
                        "parse_error": parse_error,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                }
            }
        } else {
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get plan status",
                "raw_response": String::from_utf8_lossy(&response_body).to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    pub async fn reset_credits(&self, ctx: &AuthContext, seat_count: Option<i32>, last_seat_count: Option<i32>, seat_count_options: &[i32]) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        // Determine seat count to use
        let seat_count = if let Some(sc) = seat_count {
            sc
        } else if seat_count_options.is_empty() {
            // If no configuration options, use default value
            18
        } else if let Some(last) = last_seat_count {
            // Rotate through configured options
            if let Some(current_idx) = seat_count_options.iter().position(|&x| x == last) {
                // Find current seat count position in options, select next
                let next_idx = (current_idx + 1) % seat_count_options.len();
                seat_count_options[next_idx]
            } else {
                // If last used seat count not in options, use first option
                seat_count_options[0]
            }
        } else {
            // No previous record, use first option
            seat_count_options[0]
        };
        
        println!("[ResetCredits] Using seat count: {}", seat_count);
        
        // Execute one seat update to trigger credit reset
        let seats_result = self.update_seats(ctx, seat_count, 1).await?;
        
        // Return seat update result directly
        Ok(serde_json::json!({
            "success": seats_result.success,
            "seat_count_used": seat_count,
            "steps": {
                "update_seats": seats_result
            },
            "message": if seats_result.success {
                format!("Credit reset successful, seat count updated to {}", seat_count)
            } else {
                "Credit reset failed".to_string()
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Reset team member credits
    /// Reset credits by removing member then re-inviting (consistent with team management logic)
    pub async fn reset_member_credits(&self, master_ctx: &AuthContext, member_api_key: &str, member_name: &str, member_email: &str) -> AppResult<serde_json::Value> {
        println!("[ResetMemberCredits] Starting to reset member credits: {} ({})", member_name, member_email);
        
        // Step 1: Remove member
        let remove_result = self.remove_user_from_team(master_ctx, member_api_key).await;
        if let Err(e) = &remove_result {
            println!("[ResetMemberCredits] Failed to remove member: {}", e);
            return Ok(serde_json::json!({
                "success": false,
                "step": "remove",
                "error": format!("Failed to remove member: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));
        }
        
        let remove_data = remove_result.unwrap();
        if !remove_data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(serde_json::json!({
                "success": false,
                "step": "remove",
                "error": "Failed to remove member",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));
        }
        
        println!("[ResetMemberCredits] Member removed, starting re-invitation...");
        
        // Step 2: Re-invite
        let invite_result = self.grant_preapproval(master_ctx, vec![(member_name.to_string(), member_email.to_string())]).await;
        if let Err(e) = &invite_result {
            println!("[ResetMemberCredits] Failed to re-invite: {}", e);
            return Ok(serde_json::json!({
                "success": false,
                "step": "invite",
                "error": format!("Failed to re-invite: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));
        }
        
        let invite_data = invite_result.unwrap();
        if !invite_data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(serde_json::json!({
                "success": false,
                "step": "invite",
                "error": "Failed to re-invite",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));
        }
        
        println!("[ResetMemberCredits] Member credits reset successful: {}", member_email);
        
        Ok(serde_json::json!({
            "success": true,
            "message": format!("{} credits reset, waiting to accept invitation", member_name),
            "member_email": member_email,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Get trial card binding link
    ///
    /// # Arguments
    /// * `token` - JWT token
    /// * `teams_tier` - Team tier: 1=Teams, 2=Pro, 3=Enterprise
    /// * `payment_period` - Payment period: 1=monthly, 2=yearly
    /// * `team_name` - Team name (only required for Teams/Enterprise)
    /// * `seats` - Seat count (only required for Teams/Enterprise)
    /// * `turnstile_token` - Turnstile verification token (required for all plans when start_trial=true)
    ///
    /// # Returns
    /// Returns JSON object containing Stripe Checkout link
    pub async fn subscribe_to_plan(
        &self, 
        ctx: &AuthContext, 
        teams_tier: i32,
        payment_period: i32,
        start_trial: bool,
        team_name: Option<&str>,
        seats: Option<i32>,
        turnstile_token: Option<&str>
    ) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/SubscribeToPlan", WINDSURF_BASE_URL);

        // Debug log
        println!("[SubscribeToPlan] teams_tier={}, payment_period={}, start_trial={}, team_name={:?}, seats={:?}, has_turnstile={}", 
            teams_tier, payment_period, start_trial, team_name, seats, turnstile_token.is_some());

        // Set callback URL based on plan type
        let plan_tier_str = if teams_tier == 1 { "teams" } else { "pro" };
        let success_url = format!("https://windsurf.com/billing/payment-success?plan_tier={}", plan_tier_str);
        let cancel_url = format!("https://windsurf.com/plan?plan_cancelled=true&plan_tier={}", plan_tier_str);

        let body = self.build_subscribe_to_plan_body(
            token, 
            &success_url, 
            &cancel_url, 
            teams_tier,
            payment_period,
            start_trial,
            team_name,
            seats,
            turnstile_token
        );
        
        println!("[SubscribeToPlan] Request body size: {} bytes", body.len());

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("authorization", format!("Bearer {}", token))
            .with_auth(ctx)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await?;
        
        println!("[SubscribeToPlan] Response status code: {}, response body size: {} bytes", status_code, response_body.len());

        if status_code == 200 {
            // Response is directly Protobuf binary data
            match super::proto_parser::ProtobufParser::new(response_body.to_vec()).parse_message() {
                Ok(parsed) => {
                    // Extract string_1 field (Stripe Checkout link)
                    let stripe_url = parsed.get("string_1")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if !stripe_url.is_empty() {
                        return Ok(serde_json::json!({
                            "success": true,
                            "status_code": status_code,
                            "stripe_url": stripe_url,
                            "teams_tier": teams_tier,
                            "payment_period": payment_period,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    } else {
                        return Ok(serde_json::json!({
                            "success": false,
                            "status_code": status_code,
                            "error": "Stripe link not found in response",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    }
                },
                Err(e) => {
                    return Ok(serde_json::json!({
                        "success": false,
                        "status_code": status_code,
                        "error": format!("Failed to parse response: {}", e),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }));
                }
            }
        } else {
            let error_msg = String::from_utf8_lossy(&response_body).to_string();
            println!("[SubscribeToPlan] Error response: status={}, body={}", status_code, error_msg);

            // Parse error message, provide more friendly hints
            let friendly_error = if status_code == 400 {
                if error_msg.contains("invalid_argument") {
                    "Invalid request parameters, possibly invalid price ID or account does not support this operation".to_string()
                } else if error_msg.contains("turnstile") || error_msg.contains("Turnstile") {
                    "Turnstile verification failed, please verify again".to_string()
                } else {
                    format!("Invalid request format: {}", error_msg)
                }
            } else if status_code == 401 || status_code == 403 {
                "Authentication failed, please refresh Token and retry".to_string()
            } else if status_code == 404 {
                "API interface does not exist".to_string()
            } else {
                format!("Failed to get payment link: {}", error_msg)
            };

            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": friendly_error,
                "error_details": error_msg,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get team configuration
    pub async fn get_team_config(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetTeamConfigRecord", WINDSURF_BASE_URL);

        // Build request body (field 1 = auth_token)
        let mut body = Vec::new();
        let token_bytes = token.as_bytes();
        body.push(0x0A); // field 1, wire type 2 (length-delimited)
        // Write length
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        if status_code == 200 {
            // Parse response in generic format
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_text = String::from_utf8_lossy(&response_body).to_string();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get team configuration",
                "error_details": error_text,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Update team configuration
    pub async fn update_team_config(&self, ctx: &AuthContext, config: serde_json::Value) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdateTeamConfigExternal", WINDSURF_BASE_URL);

        // Build request body
        let mut body = Vec::new();
        
        // field 1 = auth_token
        let token_bytes = token.as_bytes();
        body.push(0x0A);
        // Write length
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        // Add various fields based on config
        // field 2 = allow_auto_run_commands (bool)
        if let Some(val) = config.get("allow_auto_run_commands").and_then(|v| v.as_bool()) {
            body.push(0x10); // field 2, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 3 = allow_mcp_servers (bool)
        if let Some(val) = config.get("allow_mcp_servers").and_then(|v| v.as_bool()) {
            body.push(0x18); // field 3, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 4 = allow_app_deployments (bool)
        if let Some(val) = config.get("allow_app_deployments").and_then(|v| v.as_bool()) {
            body.push(0x20); // field 4, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 5 = allow_github_reviews (bool)
        if let Some(val) = config.get("allow_github_reviews").and_then(|v| v.as_bool()) {
            body.push(0x28); // field 5, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 6 = allow_github_description_edits (bool)
        if let Some(val) = config.get("allow_github_description_edits").and_then(|v| v.as_bool()) {
            body.push(0x30); // field 6, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 10 = allow_conversation_sharing (bool)
        if let Some(val) = config.get("allow_conversation_sharing").and_then(|v| v.as_bool()) {
            body.push(0x50); // field 10, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 11 = allow_sandbox_app_deployments (bool)
        if let Some(val) = config.get("allow_sandbox_app_deployments").and_then(|v| v.as_bool()) {
            body.push(0x58); // field 11, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 12 = allow_teams_app_deployments (bool)
        if let Some(val) = config.get("allow_teams_app_deployments").and_then(|v| v.as_bool()) {
            body.push(0x60); // field 12, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 13 = allow_attribution (bool)
        if let Some(val) = config.get("allow_attribution").and_then(|v| v.as_bool()) {
            body.push(0x68); // field 13, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 9 = allow_individual_level_analytics (bool)
        if let Some(val) = config.get("allow_individual_level_analytics").and_then(|v| v.as_bool()) {
            body.push(0x48); // field 9, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 16 = allow_browser_experimental_features (bool)
        if let Some(val) = config.get("allow_browser_experimental_features").and_then(|v| v.as_bool()) {
            body.push(0x80); body.push(0x01); // field 16, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 17 = allow_vibe_and_replace (bool)
        if let Some(val) = config.get("allow_vibe_and_replace").and_then(|v| v.as_bool()) {
            body.push(0x88); body.push(0x01); // field 17, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 18 = disable_deepwiki (bool)
        if let Some(val) = config.get("disable_deepwiki").and_then(|v| v.as_bool()) {
            body.push(0x90); body.push(0x01); // field 18, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 19 = disable_codemaps (bool)
        if let Some(val) = config.get("disable_codemaps").and_then(|v| v.as_bool()) {
            body.push(0x98); body.push(0x01); // field 19, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }
        
        // field 20 = allow_codemap_sharing (string)
        if let Some(val) = config.get("allow_codemap_sharing").and_then(|v| v.as_str()) {
            let val_bytes = val.as_bytes();
            body.push(0xA2); body.push(0x01); // field 20, wire type 2
            let len = val_bytes.len();
            if len < 128 {
                body.push(len as u8);
            } else {
                body.push((len & 0x7F | 0x80) as u8);
                body.push((len >> 7) as u8);
            }
            body.extend_from_slice(val_bytes);
        }
        
        // field 21 = disable_fast_context (bool)
        if let Some(val) = config.get("disable_fast_context").and_then(|v| v.as_bool()) {
            body.push(0xA8); body.push(0x01); // field 21, wire type 0
            body.push(if val { 0x01 } else { 0x00 });
        }

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Team configuration updated",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to update team configuration",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get available model list
    pub async fn get_cascade_model_configs(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.api_server_pb.ApiServerService/GetCascadeModelConfigsForSite", WINDSURF_BASE_URL);

        // Build request body (field 6 = auth_token)
        let mut body = Vec::new();
        let token_bytes = token.as_bytes();
        body.push(0x32); // field 6, wire type 2 (length-delimited)
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        if status_code == 200 {
            println!("[GetCascadeModelConfigs] Response size: {} bytes", response_body.len());
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|e| {
                println!("[GetCascadeModelConfigs] Parse error: {}", e);
                serde_json::json!({})
            });
            
            println!("[GetCascadeModelConfigs] Parsed keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            println!("[GetCascadeModelConfigs] default_off_models_for_teams (int_3): {:?}", parsed.get("int_3"));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            println!("[GetCascadeModelConfigs] Error status: {}", status_code);
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get model configuration",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get Command model list
    pub async fn get_command_model_configs(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.api_server_pb.ApiServerService/GetCommandModelConfigsForSite", WINDSURF_BASE_URL);

        // Build request body (field 1 = auth_token)
        let mut body = Vec::new();
        let token_bytes = token.as_bytes();
        body.push(0x0A); // field 1, wire type 2 (length-delimited)
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        if status_code == 200 {
            println!("[GetCommandModelConfigs] Response size: {} bytes", response_body.len());
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|e| {
                println!("[GetCommandModelConfigs] Parse error: {}", e);
                serde_json::json!({})
            });
            
            println!("[GetCommandModelConfigs] Parsed keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            println!("[GetCommandModelConfigs] Error status: {}", status_code);
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get Command model configuration",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get team model control configuration
    pub async fn get_team_organizational_controls(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.api_server_pb.ApiServerService/GetTeamOrganizationalControlsForSite", WINDSURF_BASE_URL);

        // Build request body (field 1 = auth_token)
        let mut body = Vec::new();
        let token_bytes = token.as_bytes();
        body.push(0x0A); // field 1, wire type 2
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        if status_code == 200 {
            println!("[GetTeamOrganizationalControls] Response size: {} bytes", response_body.len());
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|e| {
                println!("[GetTeamOrganizationalControls] Parse error: {}", e);
                serde_json::json!({})
            });
            
            println!("[GetTeamOrganizationalControls] Parsed keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            println!("[GetTeamOrganizationalControls] Full response: {}", serde_json::to_string_pretty(&parsed).unwrap_or_default());
            
            // Check fields in subMesssage_1 (controls)
            if let Some(controls) = parsed.get("subMesssage_1") {
                println!("[GetTeamOrganizationalControls] Controls keys: {:?}", controls.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                println!("[GetTeamOrganizationalControls] team_id (string_1): {:?}", controls.get("string_1"));
                println!("[GetTeamOrganizationalControls] cascade_model_labels (string_2): {:?}", controls.get("string_2"));
                println!("[GetTeamOrganizationalControls] command_model_labels (string_3): {:?}", controls.get("string_3"));
            }
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            println!("[GetTeamOrganizationalControls] Error status: {}", status_code);
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get team model configuration",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Update team model control configuration
    pub async fn upsert_team_organizational_controls(
        &self, 
        ctx: &AuthContext, 
        team_id: &str,
        cascade_models: Vec<String>,
        command_models: Vec<String>,
        extension_models: Vec<String>,
    ) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        println!("[UpsertTeamOrgControls] team_id={}, cascade={:?}, command={:?}, extension={:?}", 
            team_id, cascade_models, command_models, extension_models);
        
        // Validate team_id is not empty
        if team_id.is_empty() {
            return Ok(serde_json::json!({
                "success": false,
                "error": "Team ID is empty, cannot save model configuration",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));
        }
        
        let url = format!("{}/exa.api_server_pb.ApiServerService/UpsertTeamOrganizationalControlsForSite", WINDSURF_BASE_URL);

        // Build request body
        let mut body = Vec::new();
        
        // field 1 = TeamOrganizationalControls (nested message)
        let mut controls = Vec::new();
        
        // TeamOrganizationalControls.team_id (field 1)
        let team_id_bytes = team_id.as_bytes();
        controls.push(0x0A); // field 1, wire type 2
        controls.push(team_id_bytes.len() as u8);
        controls.extend_from_slice(team_id_bytes);
        
        // TeamOrganizationalControls.cascade_model_labels (field 2, repeated)
        for model in &cascade_models {
            let model_bytes = model.as_bytes();
            controls.push(0x12); // field 2, wire type 2
            controls.push(model_bytes.len() as u8);
            controls.extend_from_slice(model_bytes);
        }
        
        // TeamOrganizationalControls.command_model_labels (field 3, repeated)
        for model in &command_models {
            let model_bytes = model.as_bytes();
            controls.push(0x1A); // field 3, wire type 2
            controls.push(model_bytes.len() as u8);
            controls.extend_from_slice(model_bytes);
        }
        
        // TeamOrganizationalControls.extension_model_labels (field 6, repeated)
        for model in &extension_models {
            let model_bytes = model.as_bytes();
            controls.push(0x32); // field 6, wire type 2
            controls.push(model_bytes.len() as u8);
            controls.extend_from_slice(model_bytes);
        }
        
        // Write controls to body (field 1)
        body.push(0x0A); // field 1, wire type 2
        let controls_len = controls.len();
        if controls_len < 128 {
            body.push(controls_len as u8);
        } else if controls_len < 16384 {
            body.push((controls_len & 0x7F | 0x80) as u8);
            body.push((controls_len >> 7) as u8);
        } else {
            body.push((controls_len & 0x7F | 0x80) as u8);
            body.push(((controls_len >> 7) & 0x7F | 0x80) as u8);
            body.push((controls_len >> 14) as u8);
        }
        body.extend_from_slice(&controls);
        
        // field 2 = auth_token
        let token_bytes = token.as_bytes();
        body.push(0x12); // field 2, wire type 2
        let len = token_bytes.len();
        if len < 128 {
            body.push(len as u8);
        } else {
            body.push((len & 0x7F | 0x80) as u8);
            body.push((len >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);

        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Model configuration updated",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to update model configuration",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get available MCP plugin list
    pub async fn get_available_mcp_plugins(&self, api_key: &str) -> AppResult<serde_json::Value> {
        let url = format!("{}/exa.cascade_plugins_pb.CascadePluginsService/GetAvailableCascadePlugins", WINDSURF_BASE_URL);

        let request_body = serde_json::json!({
            "metadata": {
                "ideName": "windsurf",
                "extensionVersion": "1.0.0",
                "apiKey": api_key,
                "os": "unknown",
                "ideVersion": "1.3.7"
            }
        });

        let response = self.client
            .post(&url)
            .json(&request_body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/json")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.text().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        if status_code == 200 {
            // Parse JSON response
            let parsed: serde_json::Value = serde_json::from_str(&response_body)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get MCP plugin list",
                "error_details": response_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Delete user (DeleteUser API)
    /// DeleteUserRequest: auth_token=1, api_key=3
    pub async fn delete_user(&self, ctx: &AuthContext, api_key: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/DeleteUser", WINDSURF_BASE_URL);

        // Build protobuf request body
        // field 1: auth_token (string)
        // field 3: api_key (string)
        let mut request_body = Vec::new();
        
        // Field 1: auth_token
        let token_bytes = token.as_bytes();
        request_body.push(0x0a); // field 1, wire type 2 (length-delimited)
        let token_len = token_bytes.len();
        if token_len < 128 {
            request_body.push(token_len as u8);
        } else {
            request_body.push((token_len & 0x7F | 0x80) as u8);
            request_body.push((token_len >> 7) as u8);
        }
        request_body.extend_from_slice(token_bytes);
        
        // Field 3: api_key
        let api_key_bytes = api_key.as_bytes();
        request_body.push(0x1a); // field 3, wire type 2 (length-delimited)
        let api_key_len = api_key_bytes.len();
        if api_key_len < 128 {
            request_body.push(api_key_len as u8);
        } else {
            request_body.push((api_key_len & 0x7F | 0x80) as u8);
            request_body.push((api_key_len >> 7) as u8);
        }
        request_body.extend_from_slice(api_key_bytes);

        log::info!("[DeleteUser] Request body size: {} bytes", request_body.len());

        let response = self.client
            .post(&url)
            .body(request_body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Chromium";v="136", "Google Chrome";v="136", "Not_A Brand";v="99""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        log::info!("[DeleteUser] Response status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "User deleted",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            
            log::error!("[DeleteUser] Error: {}", error_body);
            
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to delete user",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    // ==================== Team Member Management API ====================

    /// Helper method: encode varint-length string field
    fn encode_string_field(&self, field_num: u8, value: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let bytes = value.as_bytes();
        let len = bytes.len();
        
        // field tag: (field_num << 3) | 2 (wire type 2 = length-delimited)
        result.push((field_num << 3) | 2);
        
        // varint length
        if len < 128 {
            result.push(len as u8);
        } else {
            result.push((len & 0x7F | 0x80) as u8);
            result.push((len >> 7) as u8);
        }
        
        result.extend_from_slice(bytes);
        result
    }

    /// Get team member list (GetUsers API)
    /// Requires admin permissions
    pub async fn get_team_members(&self, ctx: &AuthContext, group_id: Option<&str>) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetUsers", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        
        // field 2: group_id (optional)
        if let Some(gid) = group_id {
            body.extend(self.encode_string_field(2, gid));
        }
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!("[GetTeamMembers] Status: {}, Size: {} bytes", status_code, response_body.len());

        if status_code == 200 && !response_body.is_empty() {
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            // Transparent error response: print and return raw response body for troubleshooting permission/parameter/authentication issues"
            let raw_body_text = String::from_utf8_lossy(&response_body).to_string();
            println!(
                "[GetTeamMembers] Error response: status={}, body={}",
                status_code, raw_body_text
            );
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get team members",
                "raw_response": raw_body_text,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Invite member to join team (GrantPreapproval API)
    /// Requires admin permission
    pub async fn grant_preapproval(&self, ctx: &AuthContext, users: Vec<(String, String)>) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GrantPreapproval", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        
        // field 2: repeated PreapprovalUserItem
        for (name, email) in &users {
            let mut item = Vec::new();
            item.extend(self.encode_string_field(1, name));
            item.extend(self.encode_string_field(2, email));
            
            // Nested message: field 2, wire type 2
            body.push(0x12);
            let item_len = item.len();
            if item_len < 128 {
                body.push(item_len as u8);
            } else {
                body.push((item_len & 0x7F | 0x80) as u8);
                body.push((item_len >> 7) as u8);
            }
            body.extend(item);
        }
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!("[GrantPreapproval] Status: {}, Size: {} bytes", status_code, response_body.len());

        if status_code == 200 {
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "invited_count": users.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_text = String::from_utf8_lossy(&response_body).to_string();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to invite member",
                "error_details": error_text,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Remove member from team (RemoveUserFromTeam API)
    /// Requires admin permission
    pub async fn remove_user_from_team(&self, ctx: &AuthContext, api_key: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/RemoveUserFromTeam", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, api_key));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[RemoveUserFromTeam] Status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Member removed",
                "removed_api_key": api_key,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to remove member",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Revoke preapproval invitation (RevokePreapproval API)
    /// Requires admin permission
    pub async fn revoke_preapproval(&self, ctx: &AuthContext, approval_id: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/RevokePreapproval", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, approval_id));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[RevokePreapproval] Status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Invitation revoked",
                "approval_id": approval_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to revoke invitation",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get all pending preapproval invitations (GetPreapprovals API)
    /// Requires admin permission
    pub async fn get_preapprovals(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetPreapprovals", WINDSURF_BASE_URL);
        
        let body = self.encode_string_field(1, token);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!("[GetPreapprovals] Status: {}, Size: {} bytes", status_code, response_body.len());

        if status_code == 200 {
            if response_body.is_empty() {
                return Ok(serde_json::json!({
                    "success": true,
                    "preapprovals": [],
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }));
            }
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get preapproval list",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get current user's pending invitation (GetPreapprovalForUser API)
    /// Regular user permissions
    pub async fn get_preapproval_for_user(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetPreapprovalForUser", WINDSURF_BASE_URL);
        
        let body = self.encode_string_field(1, token);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!("[GetPreapprovalForUser] Status: {}, Size: {} bytes", status_code, response_body.len());

        if status_code == 200 {
            if response_body.is_empty() {
                return Ok(serde_json::json!({
                    "success": true,
                    "has_pending_invitation": false,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }));
            }
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "has_pending_invitation": true,
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            // 404 usually means no pending invitation
            if status_code == 404 {
                return Ok(serde_json::json!({
                    "success": true,
                    "has_pending_invitation": false,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }));
            }
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get invitation information",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Accept team invitation (AcceptPreapproval API)
    /// Regular user permissions
    pub async fn accept_preapproval(&self, ctx: &AuthContext, approval_id: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/AcceptPreapproval", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, approval_id));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[AcceptPreapproval] Status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Invitation accepted, successfully joined team",
                "approval_id": approval_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to accept invitation",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Reject team invitation (RejectPreapproval API)
    /// Regular user permissions
    pub async fn reject_preapproval(&self, ctx: &AuthContext, approval_id: &str) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/RejectPreapproval", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, approval_id));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[RejectPreapproval] Status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": "Invitation rejected",
                "approval_id": approval_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to reject invitation",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Request to join team (RequestTeamAccess API)
    /// Regular user requests to join team via invitation link
    pub async fn request_team_access(&self, api_key: &str, invite_id: &str) -> AppResult<serde_json::Value> {
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/RequestTeamAccess", WINDSURF_BASE_URL);
        
        let mut body = self.encode_string_field(1, api_key);
        body.extend(self.encode_string_field(2, invite_id));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(e.to_string()))?;

        println!("[RequestTeamAccess] Status: {}, Size: {} bytes", status_code, response_body.len());

        if status_code == 200 {
            if response_body.is_empty() {
                return Ok(serde_json::json!({
                    "success": true,
                    "message": "Join request submitted, waiting for admin approval",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }));
            }
            
            let mut parser = super::proto_parser::ProtobufParser::new(response_body.to_vec());
            let parsed = parser.parse_message().unwrap_or_else(|_| serde_json::json!({}));
            
            Ok(serde_json::json!({
                "success": true,
                "message": "Join request submitted",
                "data": parsed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_text = String::from_utf8_lossy(&response_body).to_string();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to request to join team",
                "error_details": error_text,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Update user team status (UpdateUserTeamStatus API)
    /// Admin approves user's join request
    /// status: 2=APPROVED(approved), 3=REJECTED(rejected)
    pub async fn update_user_team_status(&self, ctx: &AuthContext, user_api_key: &str, status: u8) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdateUserTeamStatus", WINDSURF_BASE_URL);
        
        // Build nested message: { api_key: string, status: int }
        let mut inner_msg = self.encode_string_field(1, user_api_key);
        // field 2 (status), wire type 0 (varint)
        inner_msg.push(0x10);
        inner_msg.push(status);
        
        // Build outer message
        let mut body = self.encode_string_field(1, token);
        // field 2, wire type 2 (nested message)
        body.push(0x12);
        let inner_len = inner_msg.len();
        if inner_len < 128 {
            body.push(inner_len as u8);
        } else {
            body.push((inner_len & 0x7F | 0x80) as u8);
            body.push((inner_len >> 7) as u8);
        }
        body.extend(inner_msg);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[UpdateUserTeamStatus] Status: {}", status_code);

        let status_text = match status {
            2 => "Join approved",
            3 => "Join rejected",
            _ => "Status updated",
        };

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": status_text,
                "user_api_key": user_api_key,
                "new_status": status,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to update user status",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    // ==================== Auto Top-up Management API ====================

    /// Helper method: encode varint
    fn encode_varint(&self, value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        let mut val = value;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if val == 0 {
                break;
            }
        }
        result
    }

    /// Update auto top-up settings (UpdateCreditTopUpSettings API)
    /// Requires admin permission
    pub async fn update_credit_top_up_settings(
        &self,
        ctx: &AuthContext,
        enabled: bool,
        monthly_top_up_amount: i32,
        top_up_increment: i32,
    ) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdateCreditTopUpSettings", WINDSURF_BASE_URL);
        
        // Build protobuf message
        let mut body = self.encode_string_field(1, token);
        
        // field 2: enabled (bool as varint)
        body.push(0x10); // field 2, wire type 0
        body.push(if enabled { 1 } else { 0 });
        
        // field 3: monthly_top_up_amount (int32 as varint)
        body.push(0x18); // field 3, wire type 0
        body.extend(self.encode_varint(monthly_top_up_amount as u64));
        
        // field 4: top_up_increment (int32 as varint)
        body.push(0x20); // field 4, wire type 0
        body.extend(self.encode_varint(top_up_increment as u64));
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[UpdateCreditTopUpSettings] Status: {}", status_code);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": if enabled { "Auto top-up enabled" } else { "Auto top-up disabled" },
                "enabled": enabled,
                "monthly_top_up_amount": monthly_top_up_amount,
                "top_up_increment": top_up_increment,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to update auto top-up settings",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Get auto top-up settings (extracted from subMessage_10 in GetPlanStatus response)
    /// subMessage_10 contains:
    /// - int_1: status
    /// - int_2: enabled (1=enabled)
    /// - int_3: monthly_top_up_amount (unit: cents/100)
    /// - int_5: top_up_increment (unit: cents)
    pub async fn get_credit_top_up_settings(&self, ctx: &AuthContext) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetPlanStatus", WINDSURF_BASE_URL);
        
        // Build request body
        let token_bytes = token.as_bytes();
        let token_length = token_bytes.len();
        
        let mut body = vec![0x0a];
        if token_length < 128 {
            body.push(token_length as u8);
        } else {
            body.push(((token_length & 0x7F) | 0x80) as u8);
            body.push((token_length >> 7) as u8);
        }
        body.extend_from_slice(token_bytes);
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await?;
        
        println!("[GetCreditTopUpSettings] Status: {}", status_code);
        
        if status_code == 200 {
            // Parse protobuf response
            match super::proto_parser::ProtobufParser::parse_get_plan_status_response(&response_body) {
                Ok(parsed) => {
                    // Extract auto top-up settings from raw_data.subMesssage_1.subMesssage_10
                    let top_up_status = parsed
                        .get("raw_data")
                        .and_then(|d| d.get("subMesssage_1"))
                        .and_then(|s| s.get("subMesssage_10"));
                    
                    if let Some(top_up) = top_up_status {
                        let enabled = top_up["int_2"].as_i64().unwrap_or(0) == 1;
                        // API returned value unit is already in cents, use directly
                        let monthly_top_up_amount = top_up["int_3"].as_i64().unwrap_or(0) as i32;
                        let top_up_increment = top_up["int_5"].as_i64().unwrap_or(0) as i32;
                        
                        return Ok(serde_json::json!({
                            "success": true,
                            "top_up_enabled": enabled,
                            "monthly_top_up_amount": monthly_top_up_amount,
                            "top_up_increment": top_up_increment as i32,
                            "top_up_spent": 0,
                            "raw_top_up_status": top_up,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    }
                    
                    // If no top_up_status, return full parsed result for debugging
                    Ok(serde_json::json!({
                        "success": true,
                        "top_up_enabled": false,
                        "monthly_top_up_amount": 0,
                        "top_up_increment": 0,
                        "top_up_spent": 0,
                        "note": "No top_up_status found in response",
                        "raw_parsed": parsed,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                },
                Err(e) => {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to parse response: {}", e),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }))
                }
            }
        } else {
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to get auto top-up settings",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Update member's Windsurf access permissions (UpdateCodeiumAccess API)
    /// disable_access: true = disable access, false = enable access
    pub async fn update_codeium_access(&self, ctx: &AuthContext, api_key: &str, disable_access: bool) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/UpdateCodeiumAccess", WINDSURF_BASE_URL);
        
        // Build request body: auth_token(1) + api_key(2) + disable_codeium_access(3)
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, api_key));
        // bool field encoding: field_num << 3 | 0, then value (0 or 1)
        body.push((3 << 3) | 0); // field 3, wire type 0 (varint)
        body.push(if disable_access { 1 } else { 0 });
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();

        if status_code == 200 {
            println!("[UpdateCodeiumAccess] Status: 200, disable={}", disable_access);
            Ok(serde_json::json!({
                "success": true,
                "message": if disable_access { "Windsurf access disabled" } else { "Windsurf access enabled" },
                "api_key": api_key,
                "disabled": disable_access,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            // Transparent error response: print status code, context summary, raw response body
            // Context summary only exposes first 8 characters of token / api_key to avoid leaking full credentials in logs
            let raw_body = response.bytes().await
                .map_err(|e| AppError::Api(e.to_string()))?;
            let raw_body_text = String::from_utf8_lossy(&raw_body).to_string();

            let token_kind = if ctx.devin.is_some() { "devin" } else { "firebase" };
            let token_preview: String = token.chars().take(16).collect();
            let api_key_preview: String = api_key.chars().take(8).collect();

            println!(
                "[UpdateCodeiumAccess] Error response: status={}, disable={}, token_kind={}, token_prefix={}..., api_key_prefix={}..., body={}",
                status_code,
                disable_access,
                token_kind,
                token_preview,
                api_key_preview,
                raw_body_text
            );

            // Connect Protocol error response is usually JSON like {"code":"permission_denied","message":"..."}
            // Try to parse for frontend friendly error display
            let parsed_error = serde_json::from_slice::<serde_json::Value>(&raw_body).ok();

            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to update access permissions",
                "error_details": raw_body_text,
                "parsed_error": parsed_error,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Add user role (AddUserRole API)
    /// role: role name, e.g., "admin", "billing.admin", etc.
    pub async fn add_user_role(&self, ctx: &AuthContext, api_key: &str, role: &str, group_id: Option<&str>) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/AddUserRole", WINDSURF_BASE_URL);
        
        // Build request body: auth_token(1) + api_key(2) + role(3) + group_id(4, optional)
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, api_key));
        body.extend(self.encode_string_field(3, role));
        if let Some(gid) = group_id {
            body.extend(self.encode_string_field(4, gid));
        }
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[AddUserRole] Status: {}, role={}", status_code, role);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Role added: {}", role),
                "api_key": api_key,
                "role": role,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to add role",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }

    /// Remove user role (RemoveUserRole API)
    pub async fn remove_user_role(&self, ctx: &AuthContext, api_key: &str, role: &str, group_id: Option<&str>) -> AppResult<serde_json::Value> {
        let token = ctx.token_str();
        let url = format!("{}/exa.seat_management_pb.SeatManagementService/RemoveUserRole", WINDSURF_BASE_URL);
        
        // Build request body: auth_token(1) + api_key(2) + role(3) + group_id(4, optional)
        let mut body = self.encode_string_field(1, token);
        body.extend(self.encode_string_field(2, api_key));
        body.extend(self.encode_string_field(3, role));
        if let Some(gid) = group_id {
            body.extend(self.encode_string_field(4, gid));
        }
        
        let response = self.client
            .post(&url)
            .body(body)
            .header("accept", "*/*")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .with_auth(ctx)
            .header("Referer", "https://windsurf.com/")
            .send()
            .await
            .map_err(|e| AppError::Api(e.to_string()))?;

        let status_code = response.status().as_u16();
        println!("[RemoveUserRole] Status: {}, role={}", status_code, role);

        if status_code == 200 {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Role removed: {}", role),
                "api_key": api_key,
                "role": role,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            let error_body = response.bytes().await
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "success": false,
                "status_code": status_code,
                "error": "Failed to remove role",
                "error_details": error_body,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }
}

