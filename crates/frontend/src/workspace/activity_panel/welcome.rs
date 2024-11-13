use std::rc::Rc;

use dominator::{html, svg, Dom};
use dominator_bulma::{block, column, columns, content};
use futures_signals::signal::{self, Signal, SignalExt};

const MAX_CONTENT_WIDTH: u32 = 850;

pub struct Welcome {}

impl Welcome {
    pub fn new() -> Welcome {
        Welcome {}
    }

    pub fn render(
        _this: &Rc<Welcome>,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> impl Signal<Item = Option<Dom>> {
        let content_max_width = width
            .map(|width| (((width as f32) * 0.8) as u32).min(MAX_CONTENT_WIDTH))
            .broadcast();
      
        let dom = columns!("is-mobile", "is-gapless", "is-centered", {
            .style("overflow-y", "scroll")
            .style_signal("height", height.map(|height| format!("{height}px")))
            .child(column!("is-narrow", {
                .child(block!("py-6", {
                    .style_signal("max-width", content_max_width.signal_ref(|width| format!("{width}px")))
                    .child(content!({
                        .child(html!("h1", { .text("Web-based IDE") })) 
                    }))
                }))
            }))
        });
        signal::always(dom.into())
    }
        
    pub fn label(&self) -> Dom {
        html!("span", {
            .text("Welcome")
        })
    }
    
    pub fn icon(&self) -> Dom {
        const PATH: &str = "M12 \
            16C13.1 16 14 16.9 14 18S13.1 20 12 20 10 19.1 10 18 10.9 16 12 16M12 \
            10C13.1 10 14 10.9 14 12S13.1 14 12 14 10 13.1 10 12 10.9 10 12 10M12 \
            4C13.1 4 14 4.9 14 6S13.1 8 12 8 10 7.1 10 6 10.9 4 12 4M6 \
            16C7.1 16 8 16.9 8 18S7.1 20 6 20 4 19.1 4 18 4.9 16 6 16M6 \
            10C7.1 10 8 10.9 8 12S7.1 14 6 14 4 13.1 4 12 4.9 10 6 10M6 \
            4C7.1 4 8 4.9 8 6S7.1 8 6 8 4 7.1 4 6 4.9 4 6 4M18 \
            16C19.1 16 20 16.9 20 18S19.1 20 18 20 16 19.1 16 18 16.9 16 18 16M18 \
            10C19.1 10 20 10.9 20 12S19.1 14 18 14 16 13.1 16 12 16.9 10 18 10M18 \
            4C19.1 4 20 4.9 20 6S19.1 8 18 8 16 7.1 16 6 16.9 4 18 4Z";
        svg!("svg", {
            .attr("height", "2em")
            .attr("viewBox", "0 0 24 24")
            .class("has-fill-info")
            .child(svg!("path", {
                .attr("d", PATH)
            }))
        })
    }
}
