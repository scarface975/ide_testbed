use dominator::{html, svg, Dom};
use dominator_bulma::{block, icon_text};
use futures_signals::signal::{self, Signal, SignalExt};

const ICON_SVG_PATH: &str =
    "M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,\
    19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,\
    9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,\
    7 12,5 9.5,5Z";

#[derive(Default)]
pub struct Search {

}

impl Search {
    pub fn tooltip(&self) -> &'static str {
        "Search"
    }

    pub fn icon(&self, active: impl Signal<Item = bool> + 'static) -> Dom {
        let active = active.broadcast();
        svg!("svg", {
            .attr("viewBox", "0 0 24 24")
            .class_signal("has-fill-white", active.signal())
            .class_signal("has-fill-grey", signal::not(active.signal()))
            .child(svg!("path", {
                .attr("d", ICON_SVG_PATH)
            }))
        })
    }

    pub fn render(&self) -> dominator::Dom {
        block!({
            .class("has-background-white-ter")
            .style("height", "100vh")
            .child(block!("p-3", "m-0", {
                .child(icon_text!({
                    .child(html!("span", {
                        .style("font-size", ".75em")
                        .style("letter-spacing", ".1em")
                        .style("text-transform", "uppercase")
                        .text("Search")
                    }))
                }))
            }))
        })
    }
}
