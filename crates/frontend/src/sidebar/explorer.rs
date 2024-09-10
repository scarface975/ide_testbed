use std::rc::Rc;

use dominator::{clone, events, html, svg, Dom};
use dominator_bulma::{block, icon, icon_text};
use futures_signals::{signal::{self, Mutable, Signal, SignalExt}, signal_vec::SignalVecExt};

use crate::vfs::Directory;

const ICON_SVG_PATH: &str =
    "M16 0H8C6.9 0 6 .9 6 2V18C6 19.1 6.9 20 8 20H20C21.1 20 22 19.1 22 \
     18V6L16 0M20 18H8V2H15V7H20V18M4 4V22H20V24H4C2.9 24 2 23.1 2 22V4H4Z";

fn folder_open_icon() -> Dom {
    const FOLDER_OPEN_ICON: &str = "M2,7 12,17 22,7Z";
    svg!("svg", {
        .attr("pointer-events", "none")
        .attr("height", "1em")
        .attr("viewBox", "0 0 24 24")
        .child(svg!("path", {
            .attr("d", FOLDER_OPEN_ICON)
        }))
    })
}

fn folder_closed_icon() -> Dom {
    const FOLDER_CLOSED_ICON: &str = "M 7,22 17,12 7,2 Z";
    svg!("svg", {
        .attr("pointer-events", "none")
        .attr("height", "1em")
        .attr("viewBox", "0 0 24 24")
        .child(svg!("path", {
            .attr("d", FOLDER_CLOSED_ICON)
        }))
    })
}

fn file_icon() -> Dom {
    const FILE_ICON_PATH: &str = "M14,2H6A2,2 0 0,0 4,4V20A2,2 0 \
        0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z";
    svg!("svg", {
        .attr("pointer-events", "none")
        .attr("height", "1em")
        .attr("viewBox", "0 0 24 24")
        .child(svg!("path", {
            .attr("d", FILE_ICON_PATH)
        }))
    })
}

fn render_contents(
    directory: &Rc<Directory>,
    workspace_command_tx: &crate::WorkspaceCommandSender,
) -> Dom {
    let directories = directory.directories
        .signal_vec_cloned()
        .sort_by_cloned(|left_directory, right_directory|
            left_directory.name.lock_ref().cmp(&*right_directory.name.lock_ref()))
        .map(clone!(workspace_command_tx => move |directory| {
            let expanded = Mutable::new(true);
            html!("li", {
                .class("pl-5")
                .class("pt-1")
                .child(icon_text!({
                    .style("cursor", "pointer")
                    .event(clone!(expanded => move |_: events::PointerDown| {
                        let mut expanded = expanded.lock_mut();
                        *expanded = !*expanded;
                    }))
                    .child(icon!("mr-0", {
                        .child_signal(expanded.signal_ref(|expanded| match expanded {
                            true => folder_open_icon(),
                            false => folder_closed_icon(),
                        }.into()))
                    }))
                    .child(html!("span", {
                        .text_signal(directory.name.signal_cloned())
                    }))
                }))
                .child_signal(expanded.signal_ref(clone!(directory, workspace_command_tx => move |expanded| match expanded {
                    true => Some(render_contents(&directory, &workspace_command_tx)),
                    _ => None
                })))
            })
        }));

    let files = directory.files
        .signal_vec_cloned()
        .sort_by_cloned(|left_file, right_file|
            left_file.name.lock_ref().cmp(&*right_file.name.lock_ref()))
        .map(clone!(workspace_command_tx => move |file| html!("li", {
            .class("pl-5")
            .class("pt-1")
            .child(icon_text!({
                .style("cursor", "pointer")
                .event(clone!(workspace_command_tx, file => move |_: events::PointerDown| {
                    workspace_command_tx
                        .unbounded_send(crate::WorkspaceCommand::OpenFile(file.clone()))
                        .unwrap()
                }))
                .child(icon!("mr-0", {
                    .child(file_icon())
                }))
                .child(html!("span", {
                    .text_signal(file.name.signal_cloned())
                }))
            }))
        })));

    html!("ul", {
        .children_signal_vec(directories)
        .children_signal_vec(files)
    })
}

pub struct Explorer {
    workspace: Rc<Directory>
}

impl Default for Explorer {
    fn default() -> Self {
        Self {
            workspace: crate::PROJECT.with(|workspace| Rc::clone(workspace))
        }
    }
}

impl Explorer {
    pub fn render(this: &Rc<Explorer>, workspace_command_tx: &crate::WorkspaceCommandSender) -> dominator::Dom {
        let expanded = Mutable::new(true);
        block!({
            .class("has-background-white-ter")
            .style("height", "100vh")
            .child(block!("p-3", "m-0", {
                .child(icon_text!({
                    .child(html!("span", {
                        .style("font-size", ".75em")
                        .style("letter-spacing", ".1em")
                        .style("text-transform", "uppercase")
                        .text("Explorer")
                    }))
                }))
            }))
            // project listing
            .child(html!("ul", {
                .child(html!("li", {
                    .class("pl-2")
                    .child(icon_text!({
                        .style("cursor", "pointer")
                        .event(clone!(expanded => move |_: events::PointerDown| {
                            let mut expanded = expanded.lock_mut();
                            *expanded = !*expanded;
                        }))
                        .child(icon!("mr-0", {
                            .child_signal(expanded.signal_ref(|expanded| match expanded {
                                true => folder_open_icon(),
                                false => folder_closed_icon(),
                            }.into()))
                        }))
                        .child(html!("span", {
                            .text_signal(this.workspace.name.signal_cloned())
                        }))
                    }))
                    .child_signal(expanded.signal_ref(clone!(this, workspace_command_tx => move |expanded| match expanded {
                        true => Some(render_contents(&this.workspace, &workspace_command_tx)),
                        _ => None
                    })))
                }))
            }))
        })
    }

    pub fn tooltip(&self) -> &'static str {
        "Explorer"
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
}





