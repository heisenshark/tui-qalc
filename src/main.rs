pub mod async_text_view;
pub mod expression_view;
use crate::async_text_view::AsyncTextView;
use cursive::direction::Direction;
use cursive::event::{Event, Key};
use cursive::theme::{self, Color, ColorStyle, Style};
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::view::{ScrollStrategy, Selector};
use cursive::views::{EditView, LinearLayout, OnEventView, ScrollView, SelectView};
use cursive::Cursive;
use cursive::{traits::*, views};
use expression_view::{create_expression_view, open_history};
use itertools::Itertools;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let (mut history_file, mut history_lines) = open_history();
    let mut siv = cursive::default();
    let mut palete = cursive::theme::Palette::terminal_default();
    palete.set_color("Highlight", Color::Dark(theme::BaseColor::Red));
    siv.set_theme(theme::Theme {
        palette: palete,
        // borders: theme::BorderStyle::Outset,
        ..Default::default()
    });
    // siv.update_theme();
    let cb_sink = siv.cb_sink().clone();
    siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());
    let (tx, rx) = mpsc::channel();
    let expression_value = Arc::new(Mutex::new("".to_owned()));
    let ex_ref = expression_value.to_owned();
    thread::spawn(move || {
        update_message(&tx, cb_sink, &ex_ref);
    });
    let ex_ref = expression_value.clone();
    let mut history_inner: SelectView<String> =
        SelectView::<String>::new().on_submit(move |s: &mut Cursive, data: &str| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.set_content(data);
            let mut lock = ex_ref.lock().unwrap();
            *lock = data.into();
            let _ = s.focus_name("edit_view");
        });

    let iter_his = history_lines
        .chunks_mut(2)
        .into_iter()
        .map(|chunk| {
            let mut chiter = chunk.iter();
            let value = chiter.next().unwrap().to_string();
            let mut ss = String::new();
            let result = chiter.next().unwrap_or_else(|| &mut ss);
            Style::primary();
            // Style::merge(styles)
            let sss = ColorStyle::new(Color::TerminalDefault, Color::TerminalDefault);
            let stringg =
                SpannedString::single_span(format!("{} {}", value, result), Style::from(sss));
            let mut styledstring = StyledString::new();
            styledstring.append_styled(format!("{} {}", value, result), Style::terminal_default());
            // stringg.width();
            // SpannedString::new()

            // StyledString::from(format!("{} {}", valuer result));
            return (stringg, value);
        })
        .into_iter();
    history_inner.add_all(iter_his);
    let mut history_view = OnEventView::new(
        history_inner
            .with(|h| {
                h.select_down(1000);
            })
            .with_name("history"),
    )
    .on_event('j', |s| {
        let mut hist = s.find_name::<SelectView>("history").unwrap();
        hist.select_down(1);
    })
    .on_event('k', |s| {
        let mut hist = s.find_name::<SelectView>("history").unwrap();
        hist.select_up(1);
    })
    .on_event(Event::Key(Key::PageDown), |s| {
        let mut hist = s.find_name::<SelectView>("history").unwrap();
        hist.select_down(10);
    })
    .on_event(Event::Key(Key::PageUp), |s| {
        let mut hist = s.find_name::<SelectView>("history").unwrap();
        hist.select_up(10);
    })
    .on_pre_event(Event::Ctrl(Key::Down), |s| {
        let _ = s.focus_name("edit_view");
    })
    .on_pre_event(Event::CtrlChar('k'), |s| {
            let mut history = s
                .find_name::<ScrollView<LinearLayout>>("history_scroller")
                .unwrap();
        let _ = s.focus_name("edit_view");
    })
    .on_pre_event(Event::CtrlChar('j'), |s| {
        let _ = s.focus_name("edit_view");
    });
    let ex_ref = expression_value.to_owned();
    let on_edit = move |_: &mut Cursive, b: &str, _: usize| {
        let mut lock = ex_ref.lock().unwrap();
        *lock = b.into();
    };
    let mut hf = Mutex::new(history_file);
    let expression_view = create_expression_view(on_edit.clone(), hf);
    let result_preview = AsyncTextView::new("", "".to_owned(), rx).center();
    let mut expression_view = views::Panel::new(expression_view);
    expression_view.set_title("Expression");
    let mut history_layout = views::Panel::new(
        LinearLayout::vertical()
            .child(history_view)
            .scrollable()
            .scroll_y(true)
            .with(|l| {
                l.scroll_to_bottom();
            })
            .with_name("history_scroller")
            .full_height(),
    );
    history_layout.set_title("history");
    let layout = LinearLayout::vertical()
        .child(history_layout)
        .child(expression_view)
        .child(result_preview)
        .full_width()
        .with(|s| {
            // thread::sleep(Duration::from_millis(10000));
            let mut history = s.find_name::<SelectView>("history").unwrap();
            history.select_down(100000);
            let mut history = s
                .find_name::<ScrollView<LinearLayout>>("history_scroller")
                .unwrap();
            s.focus_view(&Selector::Name("edit_view"));
        });
    siv.add_fullscreen_layer(layout);
    let mut runner = siv.try_runner().unwrap();
    runner.step();
    runner.step();
    runner.step();
    runner.step();
    thread::sleep(Duration::from_millis(10));
    runner.step();
    thread::sleep(Duration::from_millis(10));
    runner.find_name::<ScrollView<LinearLayout>>("history_scroller").unwrap().scroll_to_bottom();
    runner.step();
    runner.run();
}

#[cached::proc_macro::cached]
pub fn qalc_cache(equation: String) -> String {
    match Command::new("qalc").arg(equation).output() {
        Ok(out) => {
            return String::from_utf8(out.stdout).unwrap();
        }
        Err(_) => {
            return "error???!!!".to_owned();
        }
    }
}

fn update_message(tx: &mpsc::Sender<String>, cb_sink: cursive::CbSink, vedit_ref: &Mutex<String>) {
    loop {
        thread::sleep(std::time::Duration::from_millis(40));
        let lock = vedit_ref.lock().unwrap();
        let fn_val = qalc_cache(lock.to_owned());
        if tx.send(fn_val).is_err() {
            return;
        }
        cb_sink.send(Box::new(Cursive::noop)).unwrap();
    }
}
