use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;
use std::cell::RefCell;

mod models;
use models::{AppState, Word, WordStats, CycleStats, L2DMessage};

// 全局变量用于存储当前单词，供 JS 调用
thread_local! {
    static CURRENT_WORD: RefCell<Option<String>> = RefCell::new(None);
}

// 导出给 JS 的函数：获取当前单词
#[wasm_bindgen]
pub fn get_current_word() -> Option<String> {
    CURRENT_WORD.with(|word| word.borrow().clone())
}

// 导出给 JS 的函数：设置当前单词
#[wasm_bindgen]
pub fn set_current_word(word: String) {
    CURRENT_WORD.with(|w| *w.borrow_mut() = Some(word));
}

const STORAGE_KEY: &str = "cet6_app_state_v2";

// --- Actions for Reducer ---
#[derive(Debug, Clone)]
pub enum AppAction {
    // 初始化
    SetAllWords(Vec<Word>),
    LoadState(AppState),

    // 导航
    NextNewWord,
    PrevNewWord,
    NextReviewWord,
    PrevReviewWord,

    // 单词标记
    MarkMastered,
    MarkDifficult,

    // 答题相关
    SubmitAnswer(String, String), // (user_answer, correct_answer)

    // 动态复习库管理
    GenerateReviewPool,
    CycleSettlement,

    // 状态锁
    SetLock(bool),

    // L2D消息
    SendL2DMessage(L2DMessage),
}

// --- 辅助函数 ---
fn get_current_timestamp() -> i64 {
    js_sys::Date::now() as i64
}

// --- Reducer Logic ---
impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();

        match action {
            AppAction::SetAllWords(words) => {
                next_state.all_words = words;

                // 如果是冷启动（已掌握和难词库都为空），生成初始动态复习库
                if next_state.mastered_words.is_empty() && next_state.difficult_words.is_empty() {
                    // 显示L2D提示
                    send_l2d_message("请先标记一些单词为已掌握或难词，以开始智能复习！");
                } else if next_state.dynamic_review_pool.is_empty() {
                    // 生成初始动态复习库
                    generate_review_pool(&mut next_state);
                }
            }

            AppAction::LoadState(mut state) => {
                let all_words = next_state.all_words.clone();
                state.all_words = all_words;
                next_state = state;
            }

            AppAction::NextNewWord => {
                let mut idx = next_state.new_words_index;
                let mut found = false;

                while idx < next_state.all_words.len().saturating_sub(1) {
                    idx += 1;
                    if let Some(word) = next_state.all_words.get(idx) {
                        // 跳过已标记的单词
                        if !next_state.mastered_words.contains(&word.word) &&
                           !next_state.difficult_words.contains(&word.word) {
                            next_state.new_words_index = idx;
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    send_l2d_message("已经是最后一个生词了！");
                } else {
                    // 找到生词后朗读
                    speak_current_word();
                }
            }

            AppAction::PrevNewWord => {
                let mut idx = next_state.new_words_index;
                let mut found = false;

                while idx > 0 {
                    idx -= 1;
                    if let Some(word) = next_state.all_words.get(idx) {
                        // 跳过已标记的单词
                        if !next_state.mastered_words.contains(&word.word) &&
                           !next_state.difficult_words.contains(&word.word) {
                            next_state.new_words_index = idx;
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    send_l2d_message("已经是第一个生词了！");
                } else {
                    // 找到生词后朗读
                    speak_current_word();
                }
            }

            AppAction::NextReviewWord => {
                if !next_state.dynamic_review_pool.is_empty() {
                    next_state.dynamic_review_index =
                        (next_state.dynamic_review_index + 1) % next_state.dynamic_review_pool.len();
                }
            }

            AppAction::PrevReviewWord => {
                if !next_state.dynamic_review_pool.is_empty() {
                    if next_state.dynamic_review_index == 0 {
                        next_state.dynamic_review_index = next_state.dynamic_review_pool.len() - 1;
                    } else {
                        next_state.dynamic_review_index -= 1;
                    }
                }
            }

            AppAction::MarkMastered => {
                if let Some(word) = next_state.all_words.get(next_state.new_words_index) {
                    let word_str = word.word.clone();

                    // 如果已经在已掌握库，则移除
                    if let Some(pos) = next_state.mastered_words.iter().position(|w| w == &word_str) {
                        next_state.mastered_words.remove(pos);
                        send_l2d_message(&format!("「{}」已从已掌握库移除", word_str));
                    } else {
                        // 添加到已掌握库
                        next_state.mastered_words.push(word_str.clone());

                        // 如果在难词库，则移除
                        if let Some(pos) = next_state.difficult_words.iter().position(|w| w == &word_str) {
                            next_state.difficult_words.remove(pos);
                            send_l2d_message(&format!("「{}」从难词库流入已掌握库", word_str));
                        } else {
                            send_l2d_message(&format!("「{}」已标记为已掌握", word_str));
                        }

                        // 初始化统计信息
                        next_state.word_stats.entry(word_str.clone())
                            .or_insert_with(WordStats::default);
                    }

                    // 重新生成动态复习库
                    generate_review_pool(&mut next_state);

                    // 标记后朗读当前单词
                    speak_current_word();
                }
            }

            AppAction::MarkDifficult => {
                if let Some(word) = next_state.all_words.get(next_state.new_words_index) {
                    let word_str = word.word.clone();

                    // 如果已经在难词库，则移除
                    if let Some(pos) = next_state.difficult_words.iter().position(|w| w == &word_str) {
                        next_state.difficult_words.remove(pos);
                        send_l2d_message(&format!("「{}」已从难词库移除", word_str));
                    } else {
                        // 添加到难词库
                        next_state.difficult_words.push(word_str.clone());

                        // 如果在已掌握库，则移除
                        if let Some(pos) = next_state.mastered_words.iter().position(|w| w == &word_str) {
                            next_state.mastered_words.remove(pos);
                            send_l2d_message(&format!("「{}」从已掌握库流入难词库", word_str));
                        } else {
                            send_l2d_message(&format!("「{}」已标记为难词", word_str));
                        }

                        // 初始化统计信息
                        next_state.word_stats.entry(word_str.clone())
                            .or_insert_with(WordStats::default);
                    }

                    // 重新生成动态复习库
                    generate_review_pool(&mut next_state);

                    // 标记后朗读当前单词
                    speak_current_word();
                }
            }

            AppAction::SubmitAnswer(user_answer, correct_answer) => {
                let is_correct = user_answer.to_lowercase() == correct_answer.to_lowercase();

                // 获取当前复习单词
                if let Some(word_str) = next_state.dynamic_review_pool.get(next_state.dynamic_review_index) {
                    let word_str = word_str.clone();

                    // 更新统计信息
                    let stats = next_state.word_stats.entry(word_str.clone())
                        .or_insert_with(WordStats::default);

                    // 更新周期内统计
                    if !stats.cycle_reviewed {
                        stats.cycle_reviewed = true;
                        stats.cycle_first_answer_correct = Some(is_correct);
                        next_state.cycle_stats.reviewed_words += 1;
                        if is_correct {
                            next_state.cycle_stats.correct_count += 1;
                        }
                    }
                    stats.cycle_attempts += 1;

                    // 更新总统计
                    stats.total_reviews += 1;
                    stats.last_review_timestamp = get_current_timestamp();

                    if is_correct {
                        stats.total_correct += 1;
                        stats.consecutive_correct_answers += 1;

                        // 显示正确表情（3号表情 - clever）
                        show_correct_expression();

                        // 更新 EasinessFactor
                        // 1-2次答对：EF不变或微增0.05
                        // 3次以上：正常增加0.15
                        if stats.consecutive_correct_answers <= 2 {
                            stats.easiness_factor += 0.05;
                        } else {
                            stats.easiness_factor += 0.15;
                        }

                        // 计算新的复习间隔
                        // 首次答对：1天
                        // 第二次答对：4天
                        // 之后：NewInterval = OldInterval * EF
                        if stats.consecutive_correct_answers == 1 {
                            stats.interval = 1;
                        } else if stats.consecutive_correct_answers == 2 {
                            stats.interval = 4;
                        } else {
                            stats.interval = (stats.interval as f32 * stats.easiness_factor).round() as i32;
                        }

                        // 检查是否需要流动：难词连续答对3次流入已掌握
                        if next_state.difficult_words.contains(&word_str) &&
                           stats.consecutive_correct_answers >= 3 {
                            // 从难词库移除
                            if let Some(pos) = next_state.difficult_words.iter().position(|w| w == &word_str) {
                                next_state.difficult_words.remove(pos);
                            }
                            // 添加到已掌握库
                            next_state.mastered_words.push(word_str.clone());
                            send_l2d_message(&format!("太棒了！「{}」连续答对3次，从难词库流入已掌握库！下次复习间隔：{}天",
                                word_str, stats.interval));
                        } else {
                            send_l2d_message(&format!("正确！「{}」连续答对{}次，下次复习间隔：{}天",
                                word_str, stats.consecutive_correct_answers, stats.interval));
                        }
                    } else {
                        // 答错处理
                        stats.consecutive_correct_answers = 0;
                        stats.total_incorrect_answers += 1;

                        // 显示错误表情（在4,5,6,8,9中随机选择）
                        show_error_expression();

                        // 降低 EasinessFactor，但不低于1.3
                        stats.easiness_factor = (stats.easiness_factor - 0.2).max(1.3);

                        // 重置间隔为1天
                        stats.interval = 1;

                        // 检查是否需要流动：已掌握的单词答错立即流入难词库
                        if next_state.mastered_words.contains(&word_str) {
                            // 从已掌握库移除
                            if let Some(pos) = next_state.mastered_words.iter().position(|w| w == &word_str) {
                                next_state.mastered_words.remove(pos);
                            }
                            // 添加到难词库
                            next_state.difficult_words.push(word_str.clone());
                            send_l2d_message(&format!("「{}」答错了，从已掌握库流入难词库。正确答案：{}",
                                word_str, correct_answer));
                        } else {
                            send_l2d_message(&format!("「{}」答错了，正确答案：{}。复习间隔重置为1天",
                                word_str, correct_answer));
                        }
                    }

                    // 提交答案后朗读当前单词（显示对比时）
                    speak_current_word();

                    // 检查是否需要周期结算
                    if next_state.cycle_stats.reviewed_words >= next_state.dynamic_review_pool.len() {
                        // 所有单词都至少被测试过一次，触发周期结算
                        perform_cycle_settlement(&mut next_state);
                    }
                }
            }

            AppAction::GenerateReviewPool => {
                generate_review_pool(&mut next_state);
            }

            AppAction::CycleSettlement => {
                perform_cycle_settlement(&mut next_state);
            }

            AppAction::SetLock(locked) => {
                next_state.is_locked = locked;
            }

            AppAction::SendL2DMessage(msg) => {
                // 这里实际发送L2D消息
                match msg {
                    L2DMessage::Success(text) |
                    L2DMessage::Error(text) |
                    L2DMessage::Flow(text) |
                    L2DMessage::System(text) => {
                        send_l2d_message(&text);
                    }
                    L2DMessage::Quiz(text) => {
                        // 随机问答有较低优先级
                        send_l2d_message(&text);
                    }
                }
            }
        }

        // 保存状态
        let _ = LocalStorage::set(STORAGE_KEY, &next_state);
        next_state.into()
    }
}

// 单词优先级信息（用于排序）
#[derive(Debug, Clone)]
struct WordPriority {
    word: String,
    due_date_factor: f32,
    tie_breaker_score: f32,
}

// 生成动态复习库
fn generate_review_pool(state: &mut AppState) {
    let now = get_current_timestamp();
    let now_days = now / (1000 * 60 * 60 * 24);

    // 1. 收集所有单词（已掌握 + 难词）
    let mut all_available_words = Vec::new();

    // 添加已掌握的单词
    for word in &state.mastered_words {
        all_available_words.push(word.clone());
    }

    // 添加难词
    for word in &state.difficult_words {
        all_available_words.push(word.clone());
    }

    // 2. 检查可复习单词总数
    if all_available_words.is_empty() {
        send_l2d_message("没有可复习的单词，请先标记一些单词！");
        state.dynamic_review_pool.clear();
        state.cache_pool.clear();
        state.dynamic_review_index = 0;
        return;
    }

    // 3. 计算每个单词的优先级
    let word_priorities: Vec<WordPriority> = all_available_words.iter().map(|word| {
        let stats = state.word_stats.get(word).cloned().unwrap_or_default();

        // 计算上次复习距今的天数
        let last_review_days = if stats.last_review_timestamp == 0 {
            // 从未复习过，视为无限久以前（高优先级）
            1000000
        } else {
            stats.last_review_timestamp / (1000 * 60 * 60 * 24)
        };

        let days_since_last_review = (now_days - last_review_days).max(0) as f32;
        let interval = stats.interval.max(1) as f32;

        // 计算到期因子 DueDateFactor = (当前日期 - LastReviewDate) / Interval
        let due_date_factor = days_since_last_review / interval;

        // 计算决胜分 TieBreakerScore = TotalIncorrectAnswers / (ConsecutiveCorrectAnswers + 1)
        let tie_breaker_score = stats.total_incorrect_answers as f32
            / (stats.consecutive_correct_answers + 1) as f32;

        WordPriority {
            word: word.clone(),
            due_date_factor,
            tie_breaker_score,
        }
    }).collect();

    // 4. 分离已到期和未到期的单词
    let mut due_words: Vec<WordPriority> = word_priorities.iter()
        .filter(|wp| wp.due_date_factor >= 1.0)
        .cloned()
        .collect();

    let mut not_due_words: Vec<WordPriority> = word_priorities.iter()
        .filter(|wp| wp.due_date_factor < 1.0)
        .cloned()
        .collect();

    // 5. 排序
    // 已到期的单词按 TieBreakerScore 降序排序（错误多的优先）
    due_words.sort_by(|a, b| {
        b.tie_breaker_score.partial_cmp(&a.tie_breaker_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 未到期的单词按 DueDateFactor 降序排序（最接近到期的优先）
    not_due_words.sort_by(|a, b| {
        b.due_date_factor.partial_cmp(&a.due_date_factor)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 6. 分配到复习池和缓存池
    // 复习池 = 已到期的单词
    state.dynamic_review_pool = due_words.iter()
        .map(|wp| wp.word.clone())
        .collect();

    // 缓存池 = 未到期的单词
    state.cache_pool = not_due_words.iter()
        .map(|wp| wp.word.clone())
        .collect();

    state.dynamic_review_index = 0;

    // 重置周期统计
    state.cycle_stats = CycleStats {
        total_words: state.dynamic_review_pool.len(),
        reviewed_words: 0,
        correct_count: 0,
        accuracy_rate: 0.0,
    };

    // 重置所有单词的周期内状态
    for stats in state.word_stats.values_mut() {
        stats.cycle_reviewed = false;
        stats.cycle_first_answer_correct = None;
        stats.cycle_attempts = 0;
    }

    send_l2d_message(&format!("复习池已更新：{}个到期单词，{}个未到期单词（缓存池）",
        state.dynamic_review_pool.len(), state.cache_pool.len()));
}

// 周期结算
fn perform_cycle_settlement(state: &mut AppState) {
    state.is_locked = true;

    // 计算正确率
    if state.cycle_stats.reviewed_words > 0 {
        state.cycle_stats.accuracy_rate =
            state.cycle_stats.correct_count as f32 / state.cycle_stats.reviewed_words as f32;
    }

    // 根据正确率调整下轮复习库大小
    let accuracy = state.cycle_stats.accuracy_rate;
    if accuracy >= 1.0 {
        state.review_pool_target_size *= 2;
        send_l2d_message(&format!("完美表现！正确率100%，下轮复习量翻倍至{}个", state.review_pool_target_size));
    } else if accuracy >= 0.5 {
        state.review_pool_target_size += 1;
        send_l2d_message(&format!("表现不错！正确率{:.0}%，下轮复习量增至{}个",
            accuracy * 100.0, state.review_pool_target_size));
    } else {
        state.review_pool_target_size = (state.review_pool_target_size as f32 / 2.0).ceil() as usize;
        state.review_pool_target_size = state.review_pool_target_size.max(1);
        send_l2d_message(&format!("继续加油！正确率{:.0}%，下轮复习量减至{}个",
            accuracy * 100.0, state.review_pool_target_size));
    }

    // 当前动态复习库变为缓存库
    state.cache_pool = state.dynamic_review_pool.clone();

    // 生成新的动态复习库
    generate_review_pool(state);

    state.last_cycle_timestamp = get_current_timestamp();
    state.is_locked = false;
}

// 发送L2D消息
fn send_l2d_message(message: &str) {
    let window = web_sys::window().unwrap();
    let oml2d = js_sys::Reflect::get(&window, &"oml2dInstance".into()).unwrap();

    if !oml2d.is_undefined() {
        // 调用tipsMessage方法
        let tips_method = js_sys::Reflect::get(&oml2d, &"tipsMessage".into()).unwrap();
        if tips_method.is_function() {
            let function = tips_method.dyn_ref::<js_sys::Function>().unwrap();
            let _ = function.call2(&oml2d, &JsValue::from_str(message), &JsValue::from(5000));
        }
    }
}

// 显示正确答案表情
fn show_correct_expression() {
    let window = web_sys::window().unwrap();
    if let Ok(func) = js_sys::Reflect::get(&window, &"showCorrectExpression".into()) {
        if func.is_function() {
            if let Some(function) = func.dyn_ref::<js_sys::Function>() {
                let _ = function.call0(&window);
            }
        }
    }
}

// 显示错误答案表情
fn show_error_expression() {
    let window = web_sys::window().unwrap();
    if let Ok(func) = js_sys::Reflect::get(&window, &"showErrorExpression".into()) {
        if func.is_function() {
            if let Some(function) = func.dyn_ref::<js_sys::Function>() {
                let _ = function.call0(&window);
            }
        }
    }
}

// 调用 Live2D 朗读当前单词（延迟调用以等待 DOM 更新）
fn speak_current_word() {
    let window = web_sys::window().unwrap();

    // 创建一个延迟调用的闭包
    let callback = Closure::once(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Ok(func) = js_sys::Reflect::get(&window, &"speakWord".into()) {
                if func.is_function() {
                    if let Some(function) = func.dyn_ref::<js_sys::Function>() {
                        let _ = function.call0(&window);
                    }
                }
            }
        }
    }) as Box<dyn FnOnce()>);

    // 延迟 100ms 调用，等待 DOM 更新
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        100
    );

    callback.forget();
}

// 主组件
#[function_component(App)]
fn app() -> Html {
    let app_state = use_reducer(AppState::default);
    let is_flipped = use_state(|| false);
    let user_input = use_state(|| String::new());
    let show_answer = use_state(|| false);
    let input_ref = use_node_ref();
    let loading = use_state(|| true);
    let current_mode = use_state(|| "new".to_string()); // "new" or "review"

    // 初始化：加载状态和单词数据
    {
        let app_state = app_state.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // 加载保存的状态
                if let Ok(state) = LocalStorage::get::<AppState>(STORAGE_KEY) {
                    app_state.dispatch(AppAction::LoadState(state));
                }

                // 加载单词数据
                match Request::get("4-CET6-顺序.json").send().await {
                    Ok(response) => {
                        if let Ok(data) = response.json::<Vec<Word>>().await {
                            app_state.dispatch(AppAction::SetAllWords(data));
                        }
                    }
                    Err(e) => web_sys::console::log_1(&format!("网络请求失败: {:?}", e).into()),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // 自动聚焦逻辑：在复习模式下，输入框始终保持焦点
    {
        let input_ref = input_ref.clone();
        let show_answer = show_answer.clone();
        let current_mode = current_mode.clone();
        use_effect_with((*show_answer, *current_mode == "review"), move |(show_answer, is_review)| {
            if *is_review && !*show_answer {
                // 未显示答案时聚焦输入框
                if let Some(element) = input_ref.cast::<web_sys::HtmlInputElement>() {
                    let _ = element.focus();
                }
            }
            || ()
        });
    }

    // 显示答案后，自动聚焦到结果区域以接收键盘事件
    {
        let show_answer = show_answer.clone();
        let current_mode = current_mode.clone();
        use_effect_with((*show_answer, *current_mode == "review"), move |(show_answer, is_review)| {
            if *is_review && *show_answer {
                // 显示答案后，查找并聚焦结果区域
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(result_div) = document.query_selector(".quiz-result").ok().flatten() {
                            if let Some(element) = result_div.dyn_ref::<web_sys::HtmlElement>() {
                                let _ = element.focus();
                            }
                        }
                    }
                }
            }
            || ()
        });
    }

    // 键盘事件处理
    {
        let current_mode = current_mode.clone();
        let is_flipped = is_flipped.clone();
        use_effect_with(app_state.clone(), move |app_state| {
            let document = web_sys::window().unwrap().document().unwrap();
            let app_state = app_state.clone();
            let current_mode = current_mode.clone();
            let is_flipped = is_flipped.clone();

            let keydown_handler = Closure::<dyn Fn(KeyboardEvent)>::new(move |event: KeyboardEvent| {
                if app_state.is_locked {
                    return;
                }

                match event.key().as_str() {
                    "ArrowLeft" => {
                        event.prevent_default();
                        current_mode.set("new".to_string());
                        is_flipped.set(false);  // 重置卡片状态
                        app_state.dispatch(AppAction::PrevNewWord);
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        current_mode.set("new".to_string());
                        is_flipped.set(false);  // 重置卡片状态
                        app_state.dispatch(AppAction::NextNewWord);
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        if app_state.dynamic_review_pool.is_empty() {
                            send_l2d_message("复习池为空，请先标记一些单词！");
                        } else {
                            current_mode.set("review".to_string());
                            app_state.dispatch(AppAction::PrevReviewWord);
                        }
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        if app_state.dynamic_review_pool.is_empty() {
                            send_l2d_message("复习池为空，请先标记一些单词！");
                        } else {
                            current_mode.set("review".to_string());
                            app_state.dispatch(AppAction::NextReviewWord);
                        }
                    }
                    "PageUp" => {
                        event.prevent_default();
                        app_state.dispatch(AppAction::MarkMastered);
                    }
                    "PageDown" => {
                        event.prevent_default();
                        app_state.dispatch(AppAction::MarkDifficult);
                    }
                    _ => {}
                }
            });

            document.add_event_listener_with_callback(
                "keydown",
                keydown_handler.as_ref().unchecked_ref()
            ).unwrap();

            // 清理函数
            move || {
                let _ = document.remove_event_listener_with_callback(
                    "keydown",
                    keydown_handler.as_ref().unchecked_ref()
                );
                keydown_handler.forget();
            }
        });
    }

    // 事件处理器
    let flip_card = {
        let is_flipped = is_flipped.clone();
        Callback::from(move |_| is_flipped.set(!*is_flipped))
    };

    let mark_mastered = {
        let app_state = app_state.clone();
        Callback::from(move |_| {
            if !app_state.is_locked {
                app_state.dispatch(AppAction::MarkMastered)
            }
        })
    };

    let mark_difficult = {
        let app_state = app_state.clone();
        Callback::from(move |_| {
            if !app_state.is_locked {
                app_state.dispatch(AppAction::MarkDifficult)
            }
        })
    };

    let on_input_change = {
        let user_input = user_input.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            user_input.set(input.value());
        })
    };

    let next_question = {
        let show_answer = show_answer.clone();
        let user_input = user_input.clone();
        let app_state = app_state.clone();
        Callback::from(move |_| {
            if !app_state.is_locked {
                show_answer.set(false);
                user_input.set(String::new());
                app_state.dispatch(AppAction::NextReviewWord);
            }
        })
    };

    let submit_answer = {
        let app_state = app_state.clone();
        let user_input = user_input.clone();
        let show_answer = show_answer.clone();
        Callback::from(move |_| {
            if !app_state.is_locked {
                // 获取当前复习单词
                if let Some(word_str) = app_state.dynamic_review_pool.get(app_state.dynamic_review_index) {
                    // 查找完整的单词对象
                    if let Some(word) = app_state.all_words.iter().find(|w| w.word == *word_str) {
                        app_state.dispatch(AppAction::SubmitAnswer(
                            (*user_input).clone(),
                            word.word.clone()
                        ));
                        show_answer.set(true);
                    }
                }
            }
        })
    };

    let on_keypress = {
        let submit = submit_answer.clone();
        let next = next_question.clone();
        let show_answer = show_answer.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                if *show_answer {
                    // 如果已经显示答案，则跳转到下一题
                    next.emit(web_sys::MouseEvent::new("click").unwrap());
                } else {
                    // 如果未显示答案，则提交答案
                    submit.emit(web_sys::MouseEvent::new("click").unwrap());
                }
            }
        })
    };

    // 渲染
    if *loading {
        return html! {
            <div class="container">
                <div class="loading">{"加载中..."}</div>
            </div>
        };
    }

    // 获取当前单词
    let current_word = if *current_mode == "review" && !app_state.dynamic_review_pool.is_empty() {
        app_state.dynamic_review_pool.get(app_state.dynamic_review_index)
            .and_then(|word_str| {
                app_state.all_words.iter().find(|w| w.word == *word_str)
            })
    } else {
        app_state.all_words.get(app_state.new_words_index)
    };

    // 更新全局当前单词，供 JS 调用
    if let Some(word) = current_word {
        set_current_word(word.word.clone());
    }

    let is_quiz_mode = *current_mode == "review";

    // 计算卡片样式
    let card_class = if let Some(word) = current_word {
        let is_mastered = app_state.mastered_words.contains(&word.word);
        let is_difficult = app_state.difficult_words.contains(&word.word);
        let is_cached = app_state.cache_pool.contains(&word.word);

        let base_class = if is_mastered {
            "card mastered"
        } else if is_difficult {
            "card difficult"
        } else if is_cached {
            "card cached"
        } else {
            "card"
        };

        if *is_flipped {
            format!("{} flipped", base_class)
        } else {
            base_class.to_string()
        }
    } else {
        "card".to_string()
    };

    // 按钮禁用状态
    let buttons_disabled = app_state.is_locked;
    let button_opacity = if buttons_disabled { "0.5" } else { "1.0" };

    // 差异对比函数
    let compute_diff = |user: &str, correct: &str| -> Html {
        let user_chars: Vec<char> = user.chars().collect();
        let correct_chars: Vec<char> = correct.chars().collect();
        let user_lower: Vec<char> = user.to_lowercase().chars().collect();
        let correct_lower: Vec<char> = correct.to_lowercase().chars().collect();
        let max_len = user_chars.len().max(correct_chars.len());
        let mut user_result = Vec::new();
        let mut correct_result = Vec::new();

        for i in 0..max_len {
            match (user_chars.get(i), correct_chars.get(i), user_lower.get(i), correct_lower.get(i)) {
                (Some(&u), Some(&c), Some(&u_lower), Some(&c_lower)) if u_lower == c_lower => {
                    user_result.push(html! { <span style="background: #C8E6C9; color: #1B5E20; padding: 2px 4px; margin: 0 1px; border-radius: 3px;">{u}</span> });
                    correct_result.push(html! { <span style="background: #C8E6C9; color: #1B5E20; padding: 2px 4px; margin: 0 1px; border-radius: 3px;">{c}</span> });
                }
                (Some(&u), Some(&c), _, _) => {
                    user_result.push(html! { <span style="background: #FFCDD2; color: #B71C1C; padding: 2px 4px; margin: 0 1px; border-radius: 3px; font-weight: bold;">{u}</span> });
                    correct_result.push(html! { <span style="background: #C8E6C9; color: #1B5E20; padding: 2px 4px; margin: 0 1px; border-radius: 3px; font-weight: bold;">{c}</span> });
                }
                (Some(&u), None, _, _) => {
                    user_result.push(html! { <span style="background: #FFCDD2; color: #B71C1C; padding: 2px 4px; margin: 0 1px; border-radius: 3px; font-weight: bold; text-decoration: line-through;">{u}</span> });
                }
                (None, Some(&c), _, _) => {
                    user_result.push(html! { <span style="background: #FFF9C4; color: #F57F17; padding: 2px 4px; margin: 0 1px; border-radius: 3px;">{"_"}</span> });
                    correct_result.push(html! { <span style="background: #C8E6C9; color: #1B5E20; padding: 2px 4px; margin: 0 1px; border-radius: 3px; font-weight: bold;">{c}</span> });
                }
                _ => {}
            }
        }

        html! {
            <div style="display: flex; flex-direction: column; gap: 15px; align-items: center;">
                <div style="display: flex; flex-direction: column; align-items: flex-start; width: 100%;">
                    <div style="color: #B71C1C; font-size: 0.9rem; margin-bottom: 5px; font-weight: 600;">{"- 你的输入"}</div>
                    <div style="font-family: 'Courier New', monospace; font-size: 1.8rem; background: white; padding: 15px 20px; border-radius: 8px; border-left: 4px solid #f44336; width: 100%; overflow-x: auto;">
                        {user_result}
                    </div>
                </div>
                <div style="display: flex; flex-direction: column; align-items: flex-start; width: 100%;">
                    <div style="color: #1B5E20; font-size: 0.9rem; margin-bottom: 5px; font-weight: 600;">{"+ 正确答案"}</div>
                    <div style="font-family: 'Courier New', monospace; font-size: 1.8rem; background: white; padding: 15px 20px; border-radius: 8px; border-left: 4px solid #4CAF50; width: 100%; overflow-x: auto;">
                        {correct_result}
                    </div>
                </div>
            </div>
        }
    };

    html! {
        <div class="container">
            <div class="stats">
                <div class="stat-card">
                    <div class="stat-value">{app_state.mastered_words.len()}</div>
                    <div class="stat-label">{"已掌握"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{app_state.difficult_words.len()}</div>
                    <div class="stat-label">{"难词本"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{app_state.dynamic_review_pool.len()}</div>
                    <div class="stat-label">{"复习池"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{app_state.cache_pool.len()}</div>
                    <div class="stat-label">{"缓存池"}</div>
                </div>
            </div>

            if let Some(word) = current_word {
                <div class="card-container">
                    if is_quiz_mode {
                        <div class={card_class.clone()}>
                            if !*show_answer {
                                <div class="quiz-mode">
                                    <div class="quiz-hint">{"请根据中文释义拼写单词"}</div>
                                    <div class="translations">
                                        {for word.translations.iter().map(|t| html! {
                                            <div class="translation">
                                                {&t.translation}
                                                if let Some(wt) = &t.word_type {
                                                    <span class="type">{format!("[{}]", wt)}</span>
                                                }
                                            </div>
                                        })}
                                    </div>
                                    <input
                                        ref={input_ref}
                                        type="text"
                                        class="word-input"
                                        placeholder="输入单词..."
                                        value={(*user_input).clone()}
                                        oninput={on_input_change}
                                        onkeydown={on_keypress.clone()}
                                        disabled={buttons_disabled}
                                    />
                                    <button
                                        class="btn-submit"
                                        onclick={submit_answer}
                                        disabled={buttons_disabled}
                                        style={format!("opacity: {}", button_opacity)}
                                    >
                                        {"提交答案"}
                                    </button>
                                </div>
                            } else {
                                <div class="quiz-result" tabindex="0" onkeydown={on_keypress}>
                                    <div class="result-title">{"拼写对比"}</div>
                                    <div style="width: 100%; max-width: 600px; margin: 20px auto;">
                                        {compute_diff(&user_input, &word.word)}
                                    </div>
                                    <button
                                        class="btn-primary"
                                        onclick={next_question}
                                        disabled={buttons_disabled}
                                        style={format!("opacity: {}; margin-top: 20px;", button_opacity)}
                                    >
                                        {"下一题 (Enter)"}
                                    </button>
                                </div>
                            }
                        </div>
                    } else {
                        <div class={card_class} onclick={flip_card}>
                            <div class="card-front">
                                <div class="word">{&word.word}</div>
                                <div class="flip-hint">{"点击卡片查看释义"}</div>
                            </div>
                            <div class="card-back">
                                <div class="word">{&word.word}</div>
                                <div class="translations">
                                    {for word.translations.iter().map(|t| html! {
                                        <div class="translation">
                                            {&t.translation}
                                            if let Some(wt) = &t.word_type {
                                                <span class="type">{format!("[{}]", wt)}</span>
                                            }
                                        </div>
                                    })}
                                </div>
                                if !word.phrases.is_empty() {
                                    <div class="phrases">
                                        <h3>{"常用短语"}</h3>
                                        {for word.phrases.iter().take(5).map(|p| html! {
                                            <div class="phrase-item">
                                                <span class="phrase">{&p.phrase}</span>
                                                <span class="phrase-translation">{&p.translation}</span>
                                            </div>
                                        })}
                                    </div>
                                }
                            </div>
                        </div>
                    }
                </div>
            } else {
                <div class="card-container">
                    <div class="card">
                        <div class="word">{"没有单词了！"}</div>
                    </div>
                </div>
            }

            <div class="controls">
                <button
                    class="btn-success"
                    onclick={mark_mastered}
                    disabled={buttons_disabled}
                    style={format!("opacity: {}", button_opacity)}
                >
                    {"✓ 已掌握"}
                </button>
                <button
                    class="btn-warning"
                    onclick={mark_difficult}
                    disabled={buttons_disabled}
                    style={format!("opacity: {}", button_opacity)}
                >
                    {"★ 难词"}
                </button>
            </div>


        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}