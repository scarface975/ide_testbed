use std::{sync::Arc, time::{Duration, UNIX_EPOCH}};

use chrono::DateTime;
use dominator::{html, Dom};
use dominator_bulma::{block, icon_text, tag};
use futures_signals::signal_vec::SignalVecExt;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Default)]
pub struct Console {}

impl Console {
    pub fn render(&self) -> dominator::Dom {
        block!({
            .style("height", "100%")
            .class("has-background-white-ter")
            .child(block!("p-3", "m-0", {
                .child(icon_text!({
                    .child(html!("span", {
                        .class("is-size-7")
                        .class("is-uppercase")
                        .style("letter-spacing", ".1em")
                        .text("Log messages")
                    }))
                }))
            }))
            .child(block!("p-2", {
                .style("overflow-y", "scroll")
                .style("height", "calc(100% - 48px)") // 48 px for the block above
                .class("has-background-white")
                .children_signal_vec(crate::GLOBAL_LOG.with(|messages| messages
                    .signal_vec_cloned().map(render_entry)))
                .scroll_top_signal(crate::GLOBAL_LOG.with(|messages| messages
                    .signal_vec_cloned().to_signal_map(|_| Some(i32::MAX))))
            }))
        })
    }
}

fn render_entry(message: Arc<str>) -> Dom {
    static PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\[([A-Z]+)\] \[([0-9]+\.[0-9]+)] \[([^\]]+)\]: (.+)$").unwrap());
    let structured_message = PATTERN.captures(&message)
        .and_then(|captures| match (captures.get(1), captures.get(2), captures.get(3), captures.get(4)) {
            (Some(category), Some(timestamp), Some(node), Some(message)) => Some((category, timestamp, node, message)),
            _ => None
        })
        .map(|(category, timestamp, node, message)| {
            html!("p", {
                .class("p-1")
                .child(render_category(category.as_str()))
                .child(render_timestamp(timestamp.as_str()))
                .child(render_node(node.as_str()))
                .child(render_message(message.as_str()))
            })
        });
    
    structured_message.unwrap_or(html!("p", { .text(&message) }))
}

fn render_message(node: &str) -> Dom {
    tag!("is-white", {
        .class("is-size-7")
        .text(&node)
    })
}

fn render_node(node: &str) -> Dom {
    tag!("is-white", {
        .class("is-size-7")
        .text(&node)
    })
}

fn render_timestamp(timestamp: &str) -> Dom {
    if let Ok(duration) = timestamp.parse::<f64>().map(Duration::from_secs_f64) {
        let datetime = DateTime::<chrono::Local>::from(UNIX_EPOCH + duration)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        tag!("is-white", {
            .class("is-size-7")
            .text(&datetime)
        })
    }
    else {
        tag!("is-white", {
            .class("is-size-7")
            .text(&timestamp)
        })
    }
}

fn render_category(category: &str) -> Dom {
    tag!("is-light", {
        .apply(|builder| match category {
            "INFO" => builder.text("info").class("is-success"),
            "WARN" => builder.text("warn").class("is-warning"),
            "ERROR" => builder.text("error").class("is-danger"),
            _ => builder.text("unknown")
        })
        .class("is-size-7")
        .class("is-uppercase")
        .style("min-width", "65px")
        .style("letter-spacing", ".1em")
    })
}
