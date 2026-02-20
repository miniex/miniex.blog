use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Lang enum
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Debug)]
pub enum Lang {
    Ko,
    Ja,
    #[default]
    En,
}

impl Lang {
    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Ko => "ko",
            Lang::Ja => "ja",
            Lang::En => "en",
        }
    }

    pub fn parse(s: &str) -> Lang {
        match s.to_lowercase().as_str() {
            "ko" | "ko-kr" | "ko_kr" => Lang::Ko,
            "ja" | "ja-jp" | "ja_jp" => Lang::Ja,
            "en" | "en-us" | "en-gb" | "en_us" | "en_gb" => Lang::En,
            _ => Lang::En,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Ko => "KO",
            Lang::Ja => "JA",
            Lang::En => "EN",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::Ko => "한국어",
            Lang::Ja => "日本語",
            Lang::En => "English",
        }
    }

    pub fn all() -> &'static [Lang] {
        &[Lang::Ko, Lang::Ja, Lang::En]
    }

    pub fn from_accept_language(header_value: &str) -> Lang {
        let mut best_lang = Lang::En;
        let mut best_q: f32 = -1.0;

        for part in header_value.split(',') {
            let part = part.trim();
            let (lang_tag, q) = if let Some(idx) = part.find(";q=") {
                let q_val = part[idx + 3..].trim().parse::<f32>().unwrap_or(0.0);
                (part[..idx].trim(), q_val)
            } else {
                (part, 1.0)
            };

            let primary = lang_tag
                .split('-')
                .next()
                .unwrap_or(lang_tag)
                .to_lowercase();

            let candidate = match primary.as_str() {
                "ko" => Some(Lang::Ko),
                "ja" => Some(Lang::Ja),
                "en" => Some(Lang::En),
                _ => None,
            };

            if let Some(lang) = candidate {
                if q > best_q {
                    best_q = q;
                    best_lang = lang;
                }
            }
        }

        best_lang
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// LangExtractor (Axum FromRequestParts)
// ---------------------------------------------------------------------------

pub struct LangExtractor(pub Lang);

#[async_trait]
impl<S> FromRequestParts<S> for LangExtractor
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Check cookie header for lang=xx
        if let Some(cookie_header) = parts.headers.get(header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(val) = cookie.strip_prefix("lang=") {
                        let lang = Lang::parse(val.trim());
                        return Ok(LangExtractor(lang));
                    }
                }
            }
        }

        // 2. Parse Accept-Language header
        if let Some(accept_lang) = parts.headers.get(header::ACCEPT_LANGUAGE) {
            if let Ok(val) = accept_lang.to_str() {
                return Ok(LangExtractor(Lang::from_accept_language(val)));
            }
        }

        // 3. Fallback to English
        Ok(LangExtractor(Lang::En))
    }
}

// ---------------------------------------------------------------------------
// Translations struct
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Translations {
    // Navigation
    pub nav_posts: &'static str,
    pub nav_blog: &'static str,
    pub nav_review: &'static str,
    pub nav_diary: &'static str,
    pub nav_series: &'static str,
    pub nav_guestbook: &'static str,
    pub nav_about_me: &'static str,
    pub nav_light_mode: &'static str,
    pub nav_dark_mode: &'static str,

    // Index
    pub index_recent_posts: &'static str,
    pub index_no_recent_posts: &'static str,
    pub index_check_back: &'static str,

    // List pages
    pub blog_title: &'static str,
    pub review_title: &'static str,
    pub diary_title: &'static str,
    pub filter_by_category: &'static str,
    pub filter_all: &'static str,
    pub no_posts_available: &'static str,

    // Post card
    pub card_read_more: &'static str,

    // Post detail
    pub post_back: &'static str,
    pub post_min_read: &'static str,
    pub post_share_article: &'static str,
    pub post_back_to_top: &'static str,
    pub post_toc_title: &'static str,
    pub post_not_found_title: &'static str,
    pub post_not_found_subtitle: &'static str,
    pub post_not_found_message: &'static str,
    pub post_return_home: &'static str,

    // Comments
    pub comments_title: &'static str,
    pub comments_name: &'static str,
    pub comments_password: &'static str,
    pub comments_password_hint: &'static str,
    pub comments_password_placeholder: &'static str,
    pub comments_comment: &'static str,
    pub comments_placeholder: &'static str,
    pub comments_submit: &'static str,
    pub comments_be_first: &'static str,
    pub comments_enter_both: &'static str,
    pub comments_failed_create: &'static str,
    pub comments_error: &'static str,
    pub comments_enter_password_edit: &'static str,
    pub comments_edit_prompt: &'static str,
    pub comments_wrong_password: &'static str,
    pub comments_failed_edit: &'static str,
    pub comments_enter_password_delete: &'static str,
    pub comments_confirm_delete: &'static str,

    // Series
    pub series_title: &'static str,
    pub series_subtitle: &'static str,
    pub series_completed: &'static str,
    pub series_ongoing: &'static str,
    pub series_posts_count: &'static str,
    pub series_updated: &'static str,
    pub series_view: &'static str,
    pub series_no_series: &'static str,
    pub series_no_series_message: &'static str,
    pub series_last_updated: &'static str,
    pub series_part: &'static str,
    pub series_no_posts: &'static str,
    pub series_no_posts_message: &'static str,
    pub series_view_all: &'static str,
    pub series_previous: &'static str,
    pub series_next: &'static str,

    // Guestbook
    pub guestbook_title: &'static str,
    pub guestbook_subtitle: &'static str,
    pub guestbook_write_new: &'static str,
    pub guestbook_name: &'static str,
    pub guestbook_name_placeholder: &'static str,
    pub guestbook_password: &'static str,
    pub guestbook_password_hint: &'static str,
    pub guestbook_password_placeholder: &'static str,
    pub guestbook_message: &'static str,
    pub guestbook_message_placeholder: &'static str,
    pub guestbook_submit: &'static str,
    pub guestbook_recent: &'static str,
    pub guestbook_no_entries: &'static str,
    pub guestbook_no_entries_message: &'static str,
    pub guestbook_enter_both: &'static str,
    pub guestbook_failed: &'static str,

    // Error
    pub error_title: &'static str,
    pub error_subtitle: &'static str,
    pub error_message: &'static str,
    pub error_return_home: &'static str,

    // Footer
    pub footer_copyright: String,

    // Search
    pub search_placeholder: &'static str,
    pub search_no_results: &'static str,
    pub search_results_for: &'static str,

    // Language
    pub lang_switch_label: &'static str,

    // Code highlight
    pub code_copy: &'static str,
    pub code_copied: &'static str,

    // Post dates
    pub post_created: &'static str,
    pub post_updated: &'static str,

    // Graph
    pub graph_before: &'static str,
    pub graph_after: &'static str,

    // Sort
    pub sort_newest_first: &'static str,
    pub sort_oldest_first: &'static str,
    pub sort_recently_updated: &'static str,
    pub sort_least_updated: &'static str,

    // Post stats
    pub post_views: &'static str,
    pub post_likes: &'static str,

    // Visitor stats
    pub visitor_today: &'static str,
    pub visitor_total: &'static str,
    pub visitor_visitors: &'static str,

    // Rate limit
    pub rate_limit: &'static str,
}

impl Translations {
    pub fn for_lang(lang: Lang) -> Self {
        match lang {
            Lang::Ko => Self::ko(),
            Lang::Ja => Self::ja(),
            Lang::En => Self::en(),
        }
    }

    fn copyright() -> String {
        let year = Utc::now().year();
        format!(
            "Copyright \u{00a9} 2024 - {} All rights reserved by Han Damin",
            year
        )
    }

    fn en() -> Self {
        Translations {
            // Navigation
            nav_posts: "Posts",
            nav_blog: "Blog",
            nav_review: "Review",
            nav_diary: "Diary",
            nav_series: "Series",
            nav_guestbook: "Guestbook",
            nav_about_me: "About",
            nav_light_mode: "Light Mode",
            nav_dark_mode: "Dark Mode",

            // Index
            index_recent_posts: "Recent Posts",
            index_no_recent_posts: "No posts yet.",
            index_check_back: "New content is on the way!",

            // List pages
            blog_title: "Blog",
            review_title: "Reviews",
            diary_title: "Diary",
            filter_by_category: "Category",
            filter_all: "All",
            no_posts_available: "No posts yet",

            // Post card
            card_read_more: "Read more",

            // Post detail
            post_back: "Back",
            post_min_read: "min read",
            post_share_article: "Share this article",
            post_back_to_top: "Back to top",
            post_toc_title: "Table of Contents",
            post_not_found_title: "404",
            post_not_found_subtitle: "Page Not Found",
            post_not_found_message: "The page you're looking for doesn't exist. Check the URL or head back to the homepage.",
            post_return_home: "Go Home",

            // Comments
            comments_title: "Comments",
            comments_name: "Name",
            comments_password: "Password",
            comments_password_hint: "(optional, for editing)",
            comments_password_placeholder: "Password for editing later",
            comments_comment: "Comment",
            comments_placeholder: "Leave a comment...",
            comments_submit: "Post Comment",
            comments_be_first: "Be the first to comment!",
            comments_enter_both: "Please enter both name and comment.",
            comments_failed_create: "Failed to post comment.",
            comments_error: "Something went wrong.",
            comments_enter_password_edit: "Enter your password to edit:",
            comments_edit_prompt: "Edit your comment:",
            comments_wrong_password: "Wrong password or comment not found.",
            comments_failed_edit: "Failed to edit comment.",
            comments_enter_password_delete: "Enter your password to delete:",
            comments_confirm_delete: "Are you sure you want to delete this comment?",

            // Series
            series_title: "Series",
            series_subtitle: "Related posts organized into series for in-depth reading.",
            series_completed: "Completed",
            series_ongoing: "Ongoing",
            series_posts_count: "posts",
            series_updated: "Updated:",
            series_view: "View Series",
            series_no_series: "No series yet",
            series_no_series_message: "Series will appear here once created. Stay tuned!",
            series_last_updated: "Last updated:",
            series_part: "Part",
            series_no_posts: "No posts in this series yet",
            series_no_posts_message: "Posts for this series are coming soon!",
            series_view_all: "All Series",
            series_previous: "Previous",
            series_next: "Next",

            // Guestbook
            guestbook_title: "Guestbook",
            guestbook_subtitle: "Say hello, share a thought, or leave a note. All messages are welcome!",
            guestbook_write_new: "Write a Message",
            guestbook_name: "Name",
            guestbook_name_placeholder: "Your name",
            guestbook_password: "Password",
            guestbook_password_hint: "(optional, for editing later)",
            guestbook_password_placeholder: "Password for editing later",
            guestbook_message: "Message",
            guestbook_message_placeholder: "Write your message...",
            guestbook_submit: "Post",
            guestbook_recent: "Recent Messages",
            guestbook_no_entries: "No messages yet",
            guestbook_no_entries_message: "Be the first to leave a message!",
            guestbook_enter_both: "Please enter both name and message.",
            guestbook_failed: "Failed to post. Please try again.",

            // Error
            error_title: "404",
            error_subtitle: "Page Not Found",
            error_message: "The page you're looking for doesn't exist. Check the URL or head back to the homepage.",
            error_return_home: "Go Home",

            // Footer
            footer_copyright: Self::copyright(),

            // Search
            search_placeholder: "Search...",
            search_no_results: "No results found.",
            search_results_for: "Results for",

            // Language
            lang_switch_label: "Language",

            // Code highlight
            code_copy: "Copy",
            code_copied: "Copied!",

            // Post dates
            post_created: "Created",
            post_updated: "Updated",

            // Graph
            graph_before: "Before",
            graph_after: "After",

            // Sort
            sort_newest_first: "Newest first",
            sort_oldest_first: "Oldest first",
            sort_recently_updated: "Recently updated",
            sort_least_updated: "Least recently updated",

            // Post stats
            post_views: "views",
            post_likes: "likes",

            // Visitor stats
            visitor_today: "Today",
            visitor_total: "Total",
            visitor_visitors: "visitors",

            // Rate limit
            rate_limit: "Too many requests. Please wait a moment.",
        }
    }

    fn ko() -> Self {
        Translations {
            // Navigation
            nav_posts: "글",
            nav_blog: "블로그",
            nav_review: "리뷰",
            nav_diary: "일기",
            nav_series: "시리즈",
            nav_guestbook: "방명록",
            nav_about_me: "소개",
            nav_light_mode: "라이트 모드",
            nav_dark_mode: "다크 모드",

            // Index
            index_recent_posts: "최근 글",
            index_no_recent_posts: "아직 작성된 글이 없습니다.",
            index_check_back: "곧 새로운 글이 올라올 예정입니다!",

            // List pages
            blog_title: "블로그",
            review_title: "리뷰",
            diary_title: "일기",
            filter_by_category: "카테고리별 보기",
            filter_all: "전체",
            no_posts_available: "아직 작성된 글이 없습니다",

            // Post card
            card_read_more: "자세히 보기",

            // Post detail
            post_back: "뒤로",
            post_min_read: "분 소요",
            post_share_article: "이 글 공유하기",
            post_back_to_top: "맨 위로",
            post_toc_title: "목차",
            post_not_found_title: "404",
            post_not_found_subtitle: "페이지를 찾을 수 없습니다",
            post_not_found_message:
                "요청하신 페이지를 찾을 수 없습니다. 주소를 확인하시거나 홈으로 돌아가 주세요.",
            post_return_home: "홈으로 돌아가기",

            // Comments
            comments_title: "댓글",
            comments_name: "이름",
            comments_password: "비밀번호",
            comments_password_hint: "(선택, 수정 시 필요)",
            comments_password_placeholder: "수정 시 사용할 비밀번호",
            comments_comment: "댓글",
            comments_placeholder: "댓글을 남겨 주세요...",
            comments_submit: "댓글 달기",
            comments_be_first: "첫 댓글을 남겨 보세요!",
            comments_enter_both: "이름과 댓글을 모두 입력해 주세요.",
            comments_failed_create: "댓글 작성에 실패했습니다.",
            comments_error: "오류가 발생했습니다.",
            comments_enter_password_edit: "댓글을 수정하려면 비밀번호를 입력하세요:",
            comments_edit_prompt: "댓글을 수정하세요:",
            comments_wrong_password: "비밀번호가 틀렸거나 댓글을 찾을 수 없습니다.",
            comments_failed_edit: "댓글 수정에 실패했습니다.",
            comments_enter_password_delete: "댓글을 삭제하려면 비밀번호를 입력하세요:",
            comments_confirm_delete: "정말 이 댓글을 삭제하시겠습니까?",

            // Series
            series_title: "시리즈",
            series_subtitle: "관련 글을 주제별로 묶어 시리즈로 정리했습니다.",
            series_completed: "완결",
            series_ongoing: "연재중",
            series_posts_count: "편",
            series_updated: "수정일:",
            series_view: "시리즈 보기",
            series_no_series: "아직 시리즈가 없습니다",
            series_no_series_message:
                "시리즈가 만들어지면 여기에 나타납니다. 조금만 기다려 주세요!",
            series_last_updated: "마지막 수정:",
            series_part: "파트",
            series_no_posts: "이 시리즈에는 아직 글이 없습니다",
            series_no_posts_message: "이 시리즈의 글이 곧 올라올 예정입니다!",
            series_view_all: "전체 시리즈 보기",
            series_previous: "이전",
            series_next: "다음",

            // Guestbook
            guestbook_title: "방명록",
            guestbook_subtitle: "자유롭게 인사하거나 하고 싶은 말을 남겨 주세요!",
            guestbook_write_new: "새 글 작성",
            guestbook_name: "이름",
            guestbook_name_placeholder: "이름을 입력하세요",
            guestbook_password: "비밀번호",
            guestbook_password_hint: "(선택, 나중에 수정 시 필요)",
            guestbook_password_placeholder: "수정 시 사용할 비밀번호",
            guestbook_message: "메시지",
            guestbook_message_placeholder: "하고 싶은 말을 적어 주세요...",
            guestbook_submit: "글 남기기",
            guestbook_recent: "최근 방명록",
            guestbook_no_entries: "아직 방명록이 없습니다",
            guestbook_no_entries_message: "첫 번째 메시지를 남겨 보세요!",
            guestbook_enter_both: "이름과 메시지를 모두 입력해 주세요.",
            guestbook_failed: "작성에 실패했습니다. 다시 시도해 주세요.",

            // Error
            error_title: "404",
            error_subtitle: "페이지를 찾을 수 없습니다",
            error_message:
                "요청하신 페이지를 찾을 수 없습니다. 주소를 확인하시거나 홈으로 돌아가 주세요.",
            error_return_home: "홈으로 돌아가기",

            // Footer
            footer_copyright: Self::copyright(),

            // Search
            search_placeholder: "글 검색...",
            search_no_results: "검색 결과가 없습니다.",
            search_results_for: "검색 결과:",

            // Language
            lang_switch_label: "언어",

            // Code highlight
            code_copy: "복사",
            code_copied: "복사 완료!",

            // Post dates
            post_created: "작성",
            post_updated: "수정",

            // Graph
            graph_before: "변환 전",
            graph_after: "변환 후",

            // Sort
            sort_newest_first: "최신순",
            sort_oldest_first: "오래된순",
            sort_recently_updated: "최근 수정순",
            sort_least_updated: "오래된 수정순",

            // Post stats
            post_views: "조회",
            post_likes: "좋아요",

            // Visitor stats
            visitor_today: "오늘",
            visitor_total: "전체",
            visitor_visitors: "방문자",

            // Rate limit
            rate_limit: "요청이 너무 많습니다. 잠시 후 다시 시도해 주세요.",
        }
    }

    fn ja() -> Self {
        Translations {
            // Navigation
            nav_posts: "記事",
            nav_blog: "ブログ",
            nav_review: "レビュー",
            nav_diary: "日記",
            nav_series: "シリーズ",
            nav_guestbook: "ゲストブック",
            nav_about_me: "プロフィール",
            nav_light_mode: "ライトモード",
            nav_dark_mode: "ダークモード",

            // Index
            index_recent_posts: "最近の記事",
            index_no_recent_posts: "まだ記事がありません。",
            index_check_back: "新しい記事をお楽しみに！",

            // List pages
            blog_title: "ブログ",
            review_title: "レビュー",
            diary_title: "日記",
            filter_by_category: "カテゴリで絞り込み",
            filter_all: "すべて",
            no_posts_available: "まだ記事がありません",

            // Post card
            card_read_more: "続きを読む",

            // Post detail
            post_back: "戻る",
            post_min_read: "分で読めます",
            post_share_article: "この記事をシェアする",
            post_back_to_top: "トップへ戻る",
            post_toc_title: "目次",
            post_not_found_title: "404",
            post_not_found_subtitle: "ページが見つかりません",
            post_not_found_message: "お探しのページは見つかりませんでした。URLをご確認いただくか、ホームへお戻りください。",
            post_return_home: "ホームへ戻る",

            // Comments
            comments_title: "コメント",
            comments_name: "名前",
            comments_password: "パスワード",
            comments_password_hint: "(任意、編集用)",
            comments_password_placeholder: "編集用パスワード",
            comments_comment: "コメント",
            comments_placeholder: "コメントを入力...",
            comments_submit: "コメントする",
            comments_be_first: "最初のコメントを書いてみましょう！",
            comments_enter_both: "名前とコメントを入力してください。",
            comments_failed_create: "コメントの投稿に失敗しました。",
            comments_error: "エラーが発生しました。",
            comments_enter_password_edit: "コメントを編集するにはパスワードを入力してください：",
            comments_edit_prompt: "コメントを編集：",
            comments_wrong_password: "パスワードが正しくないか、コメントが見つかりません。",
            comments_failed_edit: "コメントの編集に失敗しました。",
            comments_enter_password_delete: "コメントを削除するにはパスワードを入力してください：",
            comments_confirm_delete: "このコメントを削除してもよろしいですか？",

            // Series
            series_title: "シリーズ",
            series_subtitle: "関連する記事をシリーズにまとめました。じっくり読んでみてください。",
            series_completed: "完結",
            series_ongoing: "連載中",
            series_posts_count: "記事",
            series_updated: "更新日:",
            series_view: "シリーズを見る",
            series_no_series: "まだシリーズがありません",
            series_no_series_message: "シリーズが作成されるとここに表示されます。お楽しみに！",
            series_last_updated: "最終更新:",
            series_part: "パート",
            series_no_posts: "このシリーズにはまだ記事がありません",
            series_no_posts_message: "このシリーズの記事は近日公開予定です！",
            series_view_all: "すべてのシリーズを見る",
            series_previous: "前へ",
            series_next: "次へ",

            // Guestbook
            guestbook_title: "ゲストブック",
            guestbook_subtitle: "お気軽にメッセージを残してください！ひとことでも大歓迎です。",
            guestbook_write_new: "メッセージを書く",
            guestbook_name: "名前",
            guestbook_name_placeholder: "お名前を入力",
            guestbook_password: "パスワード",
            guestbook_password_hint: "(任意、編集用)",
            guestbook_password_placeholder: "編集用パスワード",
            guestbook_message: "メッセージ",
            guestbook_message_placeholder: "メッセージを入力...",
            guestbook_submit: "投稿する",
            guestbook_recent: "最近の書き込み",
            guestbook_no_entries: "まだ書き込みがありません",
            guestbook_no_entries_message: "最初のメッセージを残してみましょう！",
            guestbook_enter_both: "名前とメッセージを入力してください。",
            guestbook_failed: "投稿に失敗しました。もう一度お試しください。",

            // Error
            error_title: "404",
            error_subtitle: "ページが見つかりません",
            error_message: "お探しのページは見つかりませんでした。URLをご確認いただくか、ホームへお戻りください。",
            error_return_home: "ホームへ戻る",

            // Footer
            footer_copyright: Self::copyright(),

            // Search
            search_placeholder: "記事を検索...",
            search_no_results: "検索結果が見つかりませんでした。",
            search_results_for: "検索結果:",

            // Language
            lang_switch_label: "言語",

            // Code highlight
            code_copy: "コピー",
            code_copied: "コピーしました！",

            // Post dates
            post_created: "作成",
            post_updated: "更新",

            // Graph
            graph_before: "変換前",
            graph_after: "変換後",

            // Sort
            sort_newest_first: "新しい順",
            sort_oldest_first: "古い順",
            sort_recently_updated: "最近更新順",
            sort_least_updated: "更新が古い順",

            // Post stats
            post_views: "閲覧",
            post_likes: "いいね",

            // Visitor stats
            visitor_today: "今日",
            visitor_total: "合計",
            visitor_visitors: "訪問者",

            // Rate limit
            rate_limit: "リクエストが多すぎます。しばらくしてからもう一度お試しください。",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lang_parse_ko() {
        assert_eq!(Lang::parse("ko"), Lang::Ko);
    }

    #[test]
    fn test_lang_parse_ko_kr() {
        assert_eq!(Lang::parse("ko-KR"), Lang::Ko);
    }

    #[test]
    fn test_lang_parse_ko_kr_underscore() {
        assert_eq!(Lang::parse("ko_kr"), Lang::Ko);
    }

    #[test]
    fn test_lang_parse_ja() {
        assert_eq!(Lang::parse("ja"), Lang::Ja);
    }

    #[test]
    fn test_lang_parse_ja_jp() {
        assert_eq!(Lang::parse("ja-JP"), Lang::Ja);
    }

    #[test]
    fn test_lang_parse_en() {
        assert_eq!(Lang::parse("en"), Lang::En);
    }

    #[test]
    fn test_lang_parse_en_us() {
        assert_eq!(Lang::parse("en-US"), Lang::En);
    }

    #[test]
    fn test_lang_parse_en_gb() {
        assert_eq!(Lang::parse("en-GB"), Lang::En);
    }

    #[test]
    fn test_lang_parse_unknown_defaults_en() {
        assert_eq!(Lang::parse("fr"), Lang::En);
        assert_eq!(Lang::parse("de"), Lang::En);
        assert_eq!(Lang::parse(""), Lang::En);
    }

    #[test]
    fn test_from_accept_language_simple_ko() {
        assert_eq!(Lang::from_accept_language("ko"), Lang::Ko);
    }

    #[test]
    fn test_from_accept_language_with_quality() {
        assert_eq!(
            Lang::from_accept_language("en;q=0.5, ko;q=0.9, ja;q=0.3"),
            Lang::Ko
        );
    }

    #[test]
    fn test_from_accept_language_complex_rfc() {
        assert_eq!(
            Lang::from_accept_language("ja-JP,ja;q=0.9,en-US;q=0.8,en;q=0.7"),
            Lang::Ja
        );
    }

    #[test]
    fn test_from_accept_language_default_quality() {
        // No q= means q=1.0
        assert_eq!(Lang::from_accept_language("ko-KR, en;q=0.5"), Lang::Ko);
    }

    #[test]
    fn test_from_accept_language_unknown_only() {
        assert_eq!(Lang::from_accept_language("fr, de;q=0.5"), Lang::En);
    }

    #[test]
    fn test_lang_as_str() {
        assert_eq!(Lang::Ko.as_str(), "ko");
        assert_eq!(Lang::Ja.as_str(), "ja");
        assert_eq!(Lang::En.as_str(), "en");
    }

    #[test]
    fn test_lang_code() {
        assert_eq!(Lang::Ko.code(), "KO");
        assert_eq!(Lang::Ja.code(), "JA");
        assert_eq!(Lang::En.code(), "EN");
    }

    #[test]
    fn test_lang_display() {
        assert_eq!(format!("{}", Lang::Ko), "ko");
        assert_eq!(format!("{}", Lang::En), "en");
    }
}
