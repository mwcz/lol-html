use lol_html::html_content::Element;
use lol_html::{element, rewrite_str, RewriteStrSettings};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

fn main() {
    const HTML: &str = "<span>Short</span><meta><span><b>13</b><div> characters</span>";
    println!("{HTML}");
    let html = HTML;

    let depth = Arc::new(AtomicUsize::new(0));

    let html = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*", |el: &mut Element<'_, '_>| {
                // Truncate string for each new span.
                if depth.load(std::sync::atomic::Ordering::Relaxed) == 0 {
                    println!(
                        "{} starts at {:?}",
                        el.start_tag().name(),
                        el.start_tag().range()
                    );
                    if el.can_have_content() {
                        depth.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        if let Some(handlers) = el.end_tag_handlers() {
                            let depth = depth.clone();
                            handlers.push(Box::new(move |end| {
                                depth.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                                // replace the end tag with an uppercase version
                                println!("{} ends at {:?}", end.name(), end.range());
                                Ok(())
                            }));
                        }
                    }
                }
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();

    println!("{html}");
}
