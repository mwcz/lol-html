use lol_html::html_content::Element;
use lol_html::{element, rewrite_str, RewriteStrSettings};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn main() {
    const HTML: &str = "<span>Short</span><meta><span><b>13</b><div> characters</div></span>";
    println!("{HTML}");
    let html = HTML;

    let depth = Arc::new(AtomicUsize::new(0));

    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*", |el: &mut Element<'_, '_>| {
                println!(
                    "{}{} <{}> @ {:?}",
                    "  ".repeat(depth.load(Ordering::Acquire)),
                    depth.load(Ordering::Relaxed),
                    el.start_tag().name(),
                    el.start_tag().range()
                );
                if el.can_have_content() {
                    depth.fetch_add(1, Ordering::Acquire);

                    if let Some(handlers) = el.end_tag_handlers() {
                        let depth = depth.clone();
                        handlers.push(Box::new(move |end| {
                            depth.fetch_sub(1, Ordering::Acquire);

                            println!(
                                "{}{} </{}> @ {:?}",
                                "  ".repeat(depth.load(Ordering::Acquire)),
                                depth.load(Ordering::Acquire),
                                end.name(),
                                end.range()
                            );
                            Ok(())
                        }));
                    }
                }
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();
}
