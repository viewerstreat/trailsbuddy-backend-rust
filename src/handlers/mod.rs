pub mod clip;
pub mod contest;
pub mod default;
pub mod favourite;
pub mod global_404;
pub mod movie;
pub mod notification;
pub mod ping;
pub mod play_tracker;
pub mod question;
pub mod temp_api;
pub mod upload;
pub mod user;
pub mod wallet;

pub use clip::add_view::add_clip_view_handler;
pub use clip::create::create_clip_handler;
pub use clip::get_clip::get_clips_handler;

pub use contest::activate::activate_contest_handler;
pub use contest::activate::inactivate_contest_handler;
pub use contest::create::create_contest_handler;
pub use contest::get::get_contest_handler;

pub use default::default_route_handler;

pub use favourite::create::add_favourite_handler;
pub use favourite::get::get_favourite_handler;

pub use global_404::global_404_handler;

pub use movie::add_view::add_movie_view_handler;
pub use movie::create::create_movie_handler;
pub use movie::details::movie_details_handler;
pub use movie::get_movie::get_movie_handler;
pub use movie::liked_by_me::is_liked_by_me_handler;

pub use notification::clear_noti::clear_all_noti_handler;
pub use notification::clear_noti::clear_noti_handler;
pub use notification::create_broadcast_noti::create_broadcast_noti_handler;
pub use notification::get_noti::get_noti_handler;
pub use notification::mark_read::mark_all_read_noti_handler;
pub use notification::mark_read::mark_read_noti_handler;

pub use ping::ping_handler;

pub use play_tracker::answer::answer_play_tracker_handler;
pub use play_tracker::finish::finish_play_tracker_handler;
pub use play_tracker::get::get_play_tracker_handler;
pub use play_tracker::start::get_next_ques_handler;
pub use play_tracker::start::start_play_tracker_handler;

pub use question::create::create_question_handler;
pub use question::delete::delete_question_handler;
pub use question::get::get_question_handler;
pub use question::update::update_question_handler;

pub use temp_api::temp_api_get_otp;
pub use temp_api::temp_api_get_token;

pub use upload::multipart::complete_multipart_handler;
pub use upload::multipart::create_multipart_handler;
pub use upload::multipart::upload_part_multipart_handler;
pub use upload::single::upload_handler;

pub use user::admin_login::admin_generate_otp;
pub use user::admin_login::admin_login_handler;
pub use user::admin_login::admin_signup_handler;
pub use user::check_otp::check_otp_handler;
pub use user::create::create_user_handler;
pub use user::get_leaderboard::get_leaderboard_handler;
pub use user::login::login_handler;
pub use user::referral::create_special_code_handler;
pub use user::referral::use_referral_code_handler;
pub use user::renew_token::renew_token_handler;
pub use user::update::update_user_handler;
pub use user::update_fcm_token::update_fcm_token_handler;
pub use user::verify::verify_user_handler;

pub use wallet::add_bal::add_bal_end_handler;
pub use wallet::add_bal::add_bal_init_handler;
pub use wallet::get_bal::get_bal_handler;
pub use wallet::helper::*;
pub use wallet::pay_contest::pay_contest_handler;
pub use wallet::withdraw_bal::withdraw_bal_end_handler;
pub use wallet::withdraw_bal::withdraw_bal_init_handler;
