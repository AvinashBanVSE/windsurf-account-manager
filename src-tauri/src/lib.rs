mod models;
mod repository;
mod services;
mod commands;
mod utils;

use repository::DataStore;
use commands::{AutoResetStore, ResetRecordStore};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize data store
            let store = DataStore::new(app.handle())
                .expect("Failed to initialize data store");
            let store = Arc::new(store);
            
            // Inject data store into application state
            app.manage(store.clone());
            
            // Initialize auto reset config store
            let auto_reset_store = AutoResetStore::new(app.handle())
                .expect("Failed to initialize auto reset store");
            app.manage(Arc::new(auto_reset_store));
            
            // Initialize reset record store
            let reset_record_store = ResetRecordStore::new(app.handle())
                .expect("Failed to initialize reset record store");
            app.manage(Arc::new(reset_record_store));
            
            // Initialize proxy configuration
            let store_for_proxy = store.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(settings) = store_for_proxy.get_settings().await {
                    if settings.proxy_enabled || settings.proxy_url.is_some() {
                        println!("[Init] Loading proxy config: enabled={}, url={:?}", 
                            settings.proxy_enabled, settings.proxy_url);
                        services::update_proxy_config(
                            settings.proxy_enabled,
                            settings.proxy_url
                        );
                    }
                }
            });
            
            // Get version number and set window title
            let version = app.package_info().version.to_string();
            if let Some(window) = app.get_webview_window("main") {
                let title = format!("windsurf-account-manager-simple v{}", version);
                window.set_title(&title).ok();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Account management commands
            commands::add_account,
            commands::add_account_by_refresh_token,
            commands::get_all_accounts,
            commands::get_account,
            commands::update_account,
            commands::delete_account,
            commands::delete_accounts_batch,
            commands::search_accounts,
            commands::filter_accounts_by_group,
            commands::filter_accounts_by_tags,
            
            // API operation commands
            commands::login_account,
            commands::refresh_token,
            commands::get_plan_status,
            commands::reset_credits,
            commands::update_seats,
            commands::get_billing,
            commands::update_plan,
            commands::cancel_subscription,
            commands::resume_subscription,
            commands::get_account_info,
            commands::get_current_user,
            commands::batch_reset_credits,
            commands::batch_refresh_tokens,
            commands::get_team_credit_entries,
            commands::get_trial_payment_link,
            commands::get_team_config,
            commands::update_team_config,
            commands::get_cascade_model_configs,
            commands::get_command_model_configs,
            commands::get_team_organizational_controls,
            commands::upsert_team_organizational_controls,
            commands::get_available_mcp_plugins,
            commands::delete_windsurf_user,
            
            // Payment related commands
            commands::generate_virtual_card,
            commands::open_payment_window,
            commands::inject_card_info,
            commands::validate_card_number,
            commands::auto_fill_payment_form,
            commands::get_trial_payment_link_enhanced,
            commands::open_external_link,
            commands::open_external_link_incognito,
            commands::inject_auto_submit_script,
            commands::close_payment_window,
            commands::get_success_bins,
            commands::add_success_bin,
            commands::clear_success_bins,
            commands::get_random_success_bin,
            commands::reset_test_mode_progress,
            commands::get_test_mode_progress,
            
            // Protobuf parsing API commands (return parsed data)
            commands::get_current_user_parsed,
            commands::get_billing_parsed,
            commands::batch_get_users_parsed,

            // Analytics analysis commands
            commands::get_account_analytics,

            // Settings management commands
            commands::get_settings,
            commands::update_settings,
            commands::get_groups,
            commands::add_group,
            commands::delete_group,
            commands::rename_group,
            commands::get_tags,
            commands::add_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::batch_update_account_tags,
            commands::get_logs,
            commands::clear_logs,
            commands::get_stats,
            commands::export_data,
            
            // Account switching related commands
            commands::switch_account,
            commands::reset_machine_id,
            commands::check_admin_privileges,
            
            // Windsurf information commands
            commands::get_current_windsurf_info,
            
            // Application information commands
            commands::get_app_version,
            commands::get_app_title,
            commands::reset_http_client,
            
            // Seamless account switching patch commands
            commands::get_windsurf_path,
            commands::apply_seamless_patch,
            commands::restore_seamless_patch,
            commands::check_patch_status,
            commands::validate_windsurf_path,
            
            // Cunzhi commands
            commands::check_cunzhi_status,
            commands::install_cunzhi,
            commands::uninstall_cunzhi,
            
            // Data backup commands
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::delete_backup,
            commands::export_data_to_file,
            commands::import_data_from_file,
            commands::get_data_directory,
            
            // Sorting commands
            commands::get_sorted_accounts,
            commands::update_accounts_order,
            commands::update_sort_config,
            commands::get_sort_config,
            
            // Team management commands
            commands::get_team_members,
            commands::invite_team_members,
            commands::remove_team_member,
            commands::revoke_invitation,
            commands::get_pending_invitations,
            commands::get_my_pending_invitation,
            commands::accept_invitation,
            commands::reject_invitation,
            commands::request_team_access,
            commands::approve_team_join_request,
            // Auto top-up management
            commands::get_credit_top_up_settings,
            commands::update_credit_top_up_settings,
            // Member permission management
            commands::update_codeium_access,
            commands::add_user_role,
            commands::remove_user_role,
            commands::transfer_subscription,
            
            // Auto reset commands
            commands::get_auto_reset_configs,
            commands::add_auto_reset_config,
            commands::update_auto_reset_config,
            commands::delete_auto_reset_config,
            commands::check_and_auto_reset,
            commands::force_reset_config,
            commands::get_reset_records,
            commands::get_reset_stats,
            commands::clear_reset_records,

            // Devin Session password login
            commands::devin_check_connections,
            commands::devin_password_login,
            commands::devin_windsurf_post_auth,
            commands::add_account_by_devin_login,
            commands::add_account_by_devin_with_org,
            commands::refresh_devin_session,
            commands::add_account_by_devin_session_token,

            // Login method intelligent sniffing (Plan B: automatic sniffing + unified entry)
            commands::devin_check_user_login_method,
            commands::sniff_login_method,

            // Devin email registration / passwordless email login / forgot password
            commands::devin_email_start,
            commands::devin_email_complete,
            commands::devin_password_reset_start,
            commands::devin_password_reset_complete,
            commands::add_account_by_devin_register,
            commands::add_account_by_devin_email_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
