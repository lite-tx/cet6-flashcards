mod models;

use models::{Word, StudyProgress};
use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;

const STORAGE_KEY: &str = "cet6_progress";

#[function_component(App)]
fn app() -> Html {
    let words = use_state(|| Vec::<Word>::new());
    let current_index = use_state(|| 0);
    let is_flipped = use_state(|| false);
    let mastered = use_state(|| Vec::<String>::new());
    let difficult = use_state(|| Vec::<String>::new());
    let loading = use_state(|| true);
    let user_input = use_state(|| String::new());
    let show_answer = use_state(|| false);
    let input_ref = use_node_ref();

    // 加载数据
    {
        let words = words.clone();
        let loading = loading.clone();
        let current_index = current_index.clone();
        let mastered = mastered.clone();
        let difficult = difficult.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // 从LocalStorage加载进度
                if let Ok(progress) = LocalStorage::get::<StudyProgress>(STORAGE_KEY) {
                    current_index.set(progress.current_index);
                    mastered.set(progress.mastered);
                    difficult.set(progress.difficult);
                }

                // 加载JSON数据
                web_sys::console::log_1(&"开始加载JSON...".into());
                match Request::get("4-CET6-顺序.json")
                    .send()
                    .await
                {
                    Ok(response) => {
                        web_sys::console::log_1(&"网络请求成功，开始解析JSON...".into());
                        match response.json::<Vec<Word>>().await {
                            Ok(data) => {
                                web_sys::console::log_1(&format!("JSON解析成功！加载了 {} 个单词", data.len()).into());
                                words.set(data);
                            }
                            Err(e) => {
                                web_sys::console::log_1(&format!("JSON解析失败: {:?}", e).into());
                            }
                        }
                        loading.set(false);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("网络请求失败: {:?}", e).into());
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    // 保存进度
    let save_progress = {
        let current_index = current_index.clone();
        let mastered = mastered.clone();
        let difficult = difficult.clone();

        Callback::from(move |_| {
            let progress = StudyProgress {
                current_index: *current_index,
                mastered: (*mastered).clone(),
                difficult: (*difficult).clone(),
            };
            let _ = LocalStorage::set(STORAGE_KEY, progress);
        })
    };

    // 翻转卡片
    let flip_card = {
        let is_flipped = is_flipped.clone();
        Callback::from(move |_| {
            is_flipped.set(!*is_flipped);
        })
    };

    // 上一个单词
    let prev_word = {
        let current_index = current_index.clone();
        let is_flipped = is_flipped.clone();
        let save_progress = save_progress.clone();
        let user_input = user_input.clone();
        let show_answer = show_answer.clone();

        Callback::from(move |_| {
            if *current_index > 0 {
                current_index.set(*current_index - 1);
                is_flipped.set(false);
                user_input.set(String::new());
                show_answer.set(false);
                save_progress.emit(());
            }
        })
    };

    // 下一个单词
    let next_word = {
        let current_index = current_index.clone();
        let words = words.clone();
        let is_flipped = is_flipped.clone();
        let save_progress = save_progress.clone();
        let user_input = user_input.clone();
        let show_answer = show_answer.clone();

        Callback::from(move |_| {
            if *current_index < words.len() - 1 {
                current_index.set(*current_index + 1);
                is_flipped.set(false);
                user_input.set(String::new());
                show_answer.set(false);
                save_progress.emit(());
            }
        })
    };

    // 标记为已掌握（切换）
    let mark_mastered = {
        let mastered = mastered.clone();
        let difficult = difficult.clone();
        let words = words.clone();
        let current_index = current_index.clone();
        let save_progress = save_progress.clone();

        Callback::from(move |_| {
            if let Some(word) = words.get(*current_index) {
                let mut new_mastered = (*mastered).clone();
                let mut new_difficult = (*difficult).clone();

                if let Some(pos) = new_mastered.iter().position(|w| w == &word.word) {
                    // 已存在，移除
                    new_mastered.remove(pos);
                } else {
                    // 不存在，添加
                    new_mastered.push(word.word.clone());
                    // 从难词列表中移除（互斥）
                    if let Some(pos) = new_difficult.iter().position(|w| w == &word.word) {
                        new_difficult.remove(pos);
                    }
                }
                mastered.set(new_mastered);
                difficult.set(new_difficult);
                save_progress.emit(());
            }
        })
    };

    // 标记为难词（切换）
    let mark_difficult = {
        let difficult = difficult.clone();
        let mastered = mastered.clone();
        let words = words.clone();
        let current_index = current_index.clone();
        let save_progress = save_progress.clone();

        Callback::from(move |_| {
            if let Some(word) = words.get(*current_index) {
                let mut new_difficult = (*difficult).clone();
                let mut new_mastered = (*mastered).clone();

                if let Some(pos) = new_difficult.iter().position(|w| w == &word.word) {
                    // 已存在，移除
                    new_difficult.remove(pos);
                } else {
                    // 不存在，添加
                    new_difficult.push(word.word.clone());
                    // 从已掌握列表中移除（互斥）
                    if let Some(pos) = new_mastered.iter().position(|w| w == &word.word) {
                        new_mastered.remove(pos);
                    }
                }
                difficult.set(new_difficult);
                mastered.set(new_mastered);
                save_progress.emit(());
            }
        })
    };

    // 随机单词（只在已掌握和难词中随机）
    let random_word = {
        let current_index = current_index.clone();
        let words = words.clone();
        let mastered = mastered.clone();
        let difficult = difficult.clone();
        let is_flipped = is_flipped.clone();
        let save_progress = save_progress.clone();
        let user_input = user_input.clone();
        let show_answer = show_answer.clone();

        Callback::from(move |_| {
            // 收集所有已掌握和难词的单词索引
            let marked_indices: Vec<usize> = words.iter()
                .enumerate()
                .filter(|(_, word)| {
                    mastered.contains(&word.word) || difficult.contains(&word.word)
                })
                .map(|(idx, _)| idx)
                .collect();

            if !marked_indices.is_empty() {
                let random_idx = (js_sys::Math::random() * marked_indices.len() as f64) as usize;
                current_index.set(marked_indices[random_idx]);
                is_flipped.set(false);
                user_input.set(String::new());
                show_answer.set(false);
                save_progress.emit(());
            }
        })
    };

    // 处理用户输入
    let on_input_change = {
        let user_input = user_input.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            user_input.set(input.value());
        })
    };

    // 提交答案
    let submit_answer = {
        let show_answer = show_answer.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            show_answer.set(true);
        })
    };

    // 处理键盘事件
    let on_keypress = {
        let show_answer = show_answer.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" {
                show_answer.set(true);
            }
        })
    };

    // 自动聚焦到输入框（测试模式下）
    // 必须在任何条件返回之前调用所有hooks
    {
        let input_ref = input_ref.clone();
        let show_answer_val = *show_answer;
        let current_index_val = *current_index;
        let mastered = mastered.clone();
        let difficult = difficult.clone();
        let words = words.clone();

        use_effect_with((current_index_val, show_answer_val), move |_| {
            // 判断当前单词是否需要测试模式
            if let Some(word) = words.get(current_index_val) {
                let is_quiz_mode = mastered.contains(&word.word) || difficult.contains(&word.word);
                if is_quiz_mode && !show_answer_val {
                    if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                        let _ = input.focus();
                    }
                }
            }
            || ()
        });
    }

    if *loading {
        return html! {
            <div class="container">
                <div class="loading">{"加载中..."}</div>
            </div>
        };
    }

    let current_word = words.get(*current_index);

    // 简单的字符串diff函数
    let compute_diff = |user: &str, correct: &str| -> Html {
        let user_chars: Vec<char> = user.chars().collect();
        let correct_chars: Vec<char> = correct.chars().collect();
        let mut result = Vec::new();

        let max_len = user_chars.len().max(correct_chars.len());

        for i in 0..max_len {
            match (user_chars.get(i), correct_chars.get(i)) {
                (Some(&u), Some(&c)) if u == c => {
                    // 正确的字符 - 绿色
                    result.push(html! {
                        <span style="color: #4CAF50; font-weight: bold;">{u}</span>
                    });
                }
                (Some(&u), Some(&c)) => {
                    // 错误的字符 - 显示用户输入（红色）和正确答案（绿色）
                    result.push(html! {
                        <>
                            <span style="color: #f44336; font-weight: bold; text-decoration: line-through;">{u}</span>
                            <span style="color: #4CAF50; font-weight: bold;">{c}</span>
                        </>
                    });
                }
                (Some(&u), None) => {
                    // 多余的字符 - 红色删除线
                    result.push(html! {
                        <span style="color: #f44336; font-weight: bold; text-decoration: line-through;">{u}</span>
                    });
                }
                (None, Some(&c)) => {
                    // 缺失的字符 - 绿色下划线
                    result.push(html! {
                        <span style="color: #4CAF50; font-weight: bold; text-decoration: underline;">{c}</span>
                    });
                }
                _ => {}
            }
        }

        html! { <>{result}</> }
    };

    // 判断当前单词是否需要测试模式
    let is_quiz_mode = if let Some(word) = current_word {
        mastered.contains(&word.word) || difficult.contains(&word.word)
    } else {
        false
    };

    // 计算卡片状态
    let card_class = if let Some(word) = current_word {
        let is_mastered = mastered.contains(&word.word);
        let is_difficult = difficult.contains(&word.word);
        if is_mastered {
            classes!("card", "mastered", is_flipped.then(|| "flipped"))
        } else if is_difficult {
            classes!("card", "difficult", is_flipped.then(|| "flipped"))
        } else {
            classes!("card", is_flipped.then(|| "flipped"))
        }
    } else {
        classes!("card")
    };

    html! {
        <div class="container">
            <div class="header">
                <h1>{"CET6 背单词工具"}</h1>
                <p>{format!("共 {} 个单词", words.len())}</p>
            </div>

            <div class="stats">
                <div class="stat-card">
                    <div class="stat-value">{mastered.len()}</div>
                    <div class="stat-label">{"已掌握"}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{difficult.len()}</div>
                    <div class="stat-label">{"难词本"}</div>
                </div>
            </div>

            if let Some(word) = current_word {
                <div class="card-container">
                    if is_quiz_mode {
                        // 测试模式
                        <div class={card_class}>
                            if !*show_answer {
                                // 显示中文，等待用户输入
                                <div class="quiz-mode">
                                    <div class="quiz-hint">{"请根据中文释义拼写单词"}</div>
                                    <div class="translations">
                                        {word.translations.iter().map(|t| {
                                            html! {
                                                <div class="translation">
                                                    {&t.translation}
                                                    if let Some(wt) = &t.word_type {
                                                        <span class="type">{format!("[{}]", wt)}</span>
                                                    }
                                                </div>
                                            }
                                        }).collect::<Html>()}
                                    </div>
                                    <input
                                        ref={input_ref}
                                        type="text"
                                        class="word-input"
                                        placeholder="输入单词..."
                                        value={(*user_input).clone()}
                                        oninput={on_input_change}
                                        onkeypress={on_keypress}
                                    />
                                    <button class="btn-submit" onclick={submit_answer.clone()}>
                                        {"提交答案"}
                                    </button>
                                </div>
                            } else {
                                // 显示答案和diff
                                <div class="quiz-result">
                                    <div class="result-title">{"你的答案："}</div>
                                    <div class="diff-result">
                                        {compute_diff(&user_input, &word.word)}
                                    </div>
                                    <div class="correct-answer">
                                        {"正确答案: "}<strong>{&word.word}</strong>
                                    </div>
                                    <div class="translations" style="margin-top: 20px;">
                                        {word.translations.iter().map(|t| {
                                            html! {
                                                <div class="translation">
                                                    {&t.translation}
                                                    if let Some(wt) = &t.word_type {
                                                        <span class="type">{format!("[{}]", wt)}</span>
                                                    }
                                                </div>
                                            }
                                        }).collect::<Html>()}
                                    </div>
                                    if !word.phrases.is_empty() {
                                        <div class="phrases">
                                            <h3>{"常用短语"}</h3>
                                            {word.phrases.iter().take(5).map(|p| {
                                                html! {
                                                    <div class="phrase-item">
                                                        <span class="phrase">{&p.phrase}</span>
                                                        <span class="phrase-translation">{&p.translation}</span>
                                                    </div>
                                                }
                                            }).collect::<Html>()}
                                        </div>
                                    }
                                </div>
                            }
                        </div>
                    } else {
                        // 正常模式 - 翻转卡片
                        <div class={card_class} onclick={flip_card.clone()}>
                            <div class="card-front">
                                <div class="word">{&word.word}</div>
                                <div class="flip-hint">{"点击卡片查看释义"}</div>
                            </div>
                            <div class="card-back">
                                <div class="word">{&word.word}</div>
                                <div class="translations">
                                    {word.translations.iter().map(|t| {
                                        html! {
                                            <div class="translation">
                                                {&t.translation}
                                                if let Some(wt) = &t.word_type {
                                                    <span class="type">{format!("[{}]", wt)}</span>
                                                }
                                            </div>
                                        }
                                    }).collect::<Html>()}
                                </div>
                                if !word.phrases.is_empty() {
                                    <div class="phrases">
                                        <h3>{"常用短语"}</h3>
                                        {word.phrases.iter().take(5).map(|p| {
                                            html! {
                                                <div class="phrase-item">
                                                    <span class="phrase">{&p.phrase}</span>
                                                    <span class="phrase-translation">{&p.translation}</span>
                                                </div>
                                            }
                                        }).collect::<Html>()}
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
                <button class="btn-nav" onclick={prev_word} disabled={*current_index == 0}>
                    {"← 上一个"}
                </button>
                <button class="btn-success" onclick={mark_mastered}>
                    {"✓ 已掌握"}
                </button>
                <button class="btn-warning" onclick={mark_difficult}>
                    {"★ 难词"}
                </button>
                <button class="btn-primary" onclick={random_word}>
                    {"🎲 随机"}
                </button>
                <button class="btn-nav" onclick={next_word}
                    disabled={*current_index >= words.len().saturating_sub(1)}>
                    {"下一个 →"}
                </button>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
