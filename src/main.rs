use core::panic;
use std::borrow::Borrow;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use crossterm::event::KeyCode;
use cursive::event::Key;
use cursive::theme::{PaletteStyle, Style, StyleType};
use cursive::view::ViewWrapper;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, NamedView, SelectView, TextView,
};
use cursive::Cursive;
use cursive::{traits::*, Vec2};

fn main() {
    let x = Command::new("qalc")
        .arg("1 +1 1+1 +1 ")
        .output()
        .unwrap()
        .stdout;
    let mut siv = cursive::default();
    println!("{:?}", String::from_utf8(x));
    let history: Vec<String> = vec![];

    // return;
    // let select = SelectView::<String>::new()
    //     .on_submit(on_submit)
    //     .with_name("select")
    //     .fixed_size((10, 5));
    //
    // let buttons = LinearLayout::vertical()
    //     .child(Button::new("Add new", add_name))
    //     .child(Button::new("Delete", delete_name))
    //     .child(DummyView)
    //     .child(Button::new("Quit", Cursive::quit))
    //
    // siv.add_layer(Dialog::around(LinearLayout::horizontal()
    //         .child(select)
    //         .child(DummyView)
    //         .child(buttons))
    //     .title("Select a profile"));
    // panic!("dupa");
    // "d.".sub== "error";
    siv.add_global_callback(Key::Enter, |s| {
        let e = s.find_name::<EditView>("editv").unwrap();
        s.call_on_name("history", |history_view: &mut SelectView<String>| {
            history_view.add_item(e.get_content().to_string(), e.get_content().to_string());
        });
    });
    let xd: NamedView<TextView> = TextView::new("").with_name("expression");
    let mut history_view: NamedView<SelectView<String>> = SelectView::new().with_name("history");
    let mut dupa = EditView::new()
        .on_edit(|s, text, _| {
            let dupa = s.call_on_name("expression", |textobj: &mut TextView| {
                if text.len() <= 0 {
                    return;
                }
                textobj.set_content_wrap(true);
                match Command::new("qalc").arg(text).output() {
                    Ok(out) => {
                        let mut stout = String::from_utf8(out.stdout).unwrap();

                        if stout.matches("error").count() >= 1 {
                            textobj.set_style(Style::highlight());
                        } else {
                            textobj.set_style(Style::primary());
                        }
                        textobj.set_content(stout);
                    }
                    Err(e) => {
                        textobj.set_content(format!("qualc errored {:?}", e));
                    }
                }
            });
        })
        .with_name("editv");
    let layout = LinearLayout::vertical()
        .child(dupa)
        .child(xd)
        .child(history_view)
        .full_screen();
    siv.add_layer(layout);
    siv.run();
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("name")
                .fixed_width(10),
        )
        .title("Enter a new name")
        .button("Ok", |s| {
            let name = s
                .call_on_name("name", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove")),
        Some(focus) => {
            select.remove_item(focus);
        }
    }
}

fn on_submit(s: &mut Cursive, name: &str) {
    // s.pop_layer();
    s.add_layer(
        Dialog::text(format!("Name: {}\nAwesome: yes", name))
            .title(format!("{}'s info", name))
            .button("Quit", |s| {
                s.pop_layer();
            }),
    );
}
