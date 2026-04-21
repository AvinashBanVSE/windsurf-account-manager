//! Unified authentication context (AuthContext)
//!
//! Design background:
//! - Firebase accounts only need `x-auth-token` header (value = Firebase idToken).
//! - Devin accounts need 5 headers: `x-auth-token`, `x-devin-session-token`,
//!   `x-devin-account-id`, `x-devin-auth1-token`, `x-devin-primary-org-id`,
//!   where `x-auth-token` and `x-devin-session-token` values are session_token with
//!   `devin-session-token$` prefix (backend returns this form directly, no app-level concatenation needed).
//!
//! This module provides a one-time, type-safe authentication context abstraction for all
//! `WindsurfService` gRPC methods to consume. Firebase and Devin branches are automatically
//! routed at the request header construction layer, business methods no longer need to
//! distinguish auth_provider.
//!
//! Quick-fail principle: when constructing `AuthContext::from_account`, only validate the main
//! `token` field; the 3 extended fields for Devin accounts (account_id / auth1_token / primary_org_id)
//! are changed to optional to support the "session_token-only import" scenario (users can copy
//! `devin-session-token$...` directly from browser to create an account, extended fields are
//! temporarily missing, can be filled later if refresh_devin_session is needed).
//!
//! Header injection layer `with_auth` conditionally writes corresponding `x-devin-*` headers
//! based on field existence. Accounts with only session_token only send two headers:
//! `x-auth-token` + `x-devin-session-token`, consistent with the official website frontend
//! `createDevinAuth1TokenInterceptor` behavior when localStorage is missing.

use crate::models::account::Account;

/// Extended authentication information specific to Devin accounts
///
/// All fields are original values for gRPC request headers, without any further processing.
/// All fields are optional to support the "session_token-only import" scenario — in this case
/// only the main `x-auth-token` and `x-devin-session-token` are sent, the other 3 `x-devin-*`
/// headers are not sent by default.
#[derive(Debug, Clone, Default)]
pub struct DevinAuthContext {
    /// `x-devin-account-id` header value (Devin account ID, format like `account-<32 hex>`)
    pub account_id: Option<String>,
    /// `x-devin-auth1-token` header value (Auth1 primary token, used for session refresh)
    pub auth1_token: Option<String>,
    /// `x-devin-primary-org-id` header value (Devin primary organization ID)
    pub primary_org_id: Option<String>,
}

/// Authentication context for Windsurf gRPC requests
///
/// Unifies `x-auth-token` and optional Devin extended header set.
/// Consumed by `WindsurfService::apply_auth_headers` during request construction.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Primary authentication token:
    /// - Firebase accounts: Firebase idToken
    /// - Devin accounts: session_token with `devin-session-token$` prefix
    pub token: String,
    /// Additional header set for Devin accounts; `None` for Firebase accounts
    pub devin: Option<DevinAuthContext>,
}

impl AuthContext {
    /// Construct from raw Firebase idToken (Firebase account specific, or when manual control is needed)
    pub fn firebase(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            devin: None,
        }
    }

    /// Construct from Account, routed by `auth_provider`
    ///
    /// - Firebase accounts: only require `token` field to exist
    /// - Devin accounts: only require `token` (i.e., session_token); the other 3 Devin extended fields
    ///   (account_id / auth1_token / primary_org_id) are all **optional**. When missing, corresponding
    ///   `x-devin-*` headers are not sent, daily APIs (GetCurrentUser, GetPlanStatus, etc. session_token-driven
    ///   interfaces) still work; only operations that explicitly depend on auth1_token like refresh_devin_session will fail.
    pub fn from_account(account: &Account) -> Result<Self, String> {
        let token = account
            .token
            .clone()
            .ok_or_else(|| "Account missing token".to_string())?;

        if account.is_devin_account() {
            Ok(Self {
                token,
                devin: Some(DevinAuthContext {
                    account_id: account.devin_account_id.clone(),
                    auth1_token: account.devin_auth1_token.clone(),
                    primary_org_id: account.devin_primary_org_id.clone(),
                }),
            })
        } else {
            Ok(Self::firebase(token))
        }
    }

    /// Construct directly from a pure Devin `session_token` (session_token-only import path specific)
    ///
    /// `token` is expected to already have `devin-session-token$` prefix.
    /// All 3 extended fields are left empty, `with_auth` only sends two headers:
    /// `x-auth-token` + `x-devin-session-token`.
    pub fn devin_session_only(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            devin: Some(DevinAuthContext::default()),
        }
    }

    /// Convenience read-only: check if current context is a Devin account
    pub fn is_devin(&self) -> bool {
        self.devin.is_some()
    }

    /// Convenience read-only: return `&str` view of primary authentication token,
    /// retained for compatibility with code paths that heavily use `&str` during protobuf body construction
    pub fn token_str(&self) -> &str {
        &self.token
    }
}

/// Provides unified authentication header injection extension for `reqwest::RequestBuilder`
///
/// Usage: in existing chain, replace
/// ```ignore
/// .header("x-auth-token", token)
/// ```
/// with
/// ```ignore
/// .with_auth(ctx)
/// ```
/// to automatically:
/// 1. Write `x-auth-token` main header
/// 2. If Devin account, append 4 extended headers: `x-devin-account-id`, `x-devin-auth1-token`,
///    `x-devin-primary-org-id`, `x-devin-session-token`
pub trait AuthHeaderExt {
    /// Automatically write authentication header family based on `AuthContext`
    fn with_auth(self, ctx: &AuthContext) -> Self;
}

impl AuthHeaderExt for reqwest::RequestBuilder {
    fn with_auth(self, ctx: &AuthContext) -> Self {
        let mut req = self.header("x-auth-token", &ctx.token);
        if let Some(devin) = &ctx.devin {
            // x-devin-session-token is always sent (value equals ctx.token, i.e., main session_token)
            req = req.header("x-devin-session-token", &ctx.token);
            // The other 3 extended headers are only sent when fields exist — aligned with official website frontend
            // `createDevinAuth1TokenInterceptor` behavior when localStorage is missing
            if let Some(account_id) = &devin.account_id {
                req = req.header("x-devin-account-id", account_id);
            }
            if let Some(auth1_token) = &devin.auth1_token {
                req = req.header("x-devin-auth1-token", auth1_token);
            }
            if let Some(primary_org_id) = &devin.primary_org_id {
                req = req.header("x-devin-primary-org-id", primary_org_id);
            }
        }
        req
    }
}
