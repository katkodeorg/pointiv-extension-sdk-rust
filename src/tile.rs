//! Declarative tile widgets rendered by Pointiv around the popup command bar.
//!
//! A tile is data. Your extension returns a [`TileUi`] tree, the host validates
//! it and renders native widgets. Declare the tile in `pointiv-extension.json`
//! with a `"tiles"` block, then export a `render_tile` function:
//!
//! ```rust,no_run
//! use pointiv_extension_sdk::prelude::*;
//!
//! #[plugin_fn]
//! pub fn render_tile(Json(input): Json<TileRenderInput>) -> FnResult<Json<TileUi>> {
//!     let tile = TileUi::new("Todos")
//!         .subtitle(format!("as of {}", input.now))
//!         .badge("2 open", TileTone::Warn)
//!         .row(RowBuilder::new("Buy milk").action("Done", "todo done 1"))
//!         .row(RowBuilder::new("Ship SDK").secondary("due Friday"))
//!         .footer("Refresh", "todo list");
//!     Ok(Json(tile))
//! }
//! ```
//!
//! The host calls `render_tile` when the popup opens and again after a tile
//! action runs, with a 3 second budget and storage-only host access. Action
//! commands are dispatched through your normal `execute` function with the
//! exact command string the tile declared.
//!
//! Limits enforced by the host: 16 nodes per tile, nesting depth 3, 16KB of
//! JSON, title 80 chars, subtitle 120, text 300, badge and action labels 40,
//! action commands 512, 2 actions per row, 3 footer actions.

use serde::{Deserialize, Serialize};

/// Input passed to your `render_tile` function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRenderInput {
    /// Current time as an RFC3339 timestamp.
    pub now: String,
}

/// Color tone for text, badges and countdowns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileTone {
    #[default]
    Neutral,
    Ok,
    Warn,
    Danger,
}

/// A clickable action. The host runs `command` through your `execute` function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileAction {
    pub label: String,
    pub command: String,
}

impl TileAction {
    pub fn new(label: impl Into<String>, command: impl Into<String>) -> Self {
        Self { label: label.into(), command: command.into() }
    }
}

/// A small status chip attached to a row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileBadge {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<TileTone>,
}

/// One node in a tile body. Tagged with `"type"` on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TileNode {
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<TileTone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        muted: Option<bool>,
    },
    Row {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        secondary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        badge: Option<TileBadge>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        actions: Vec<TileAction>,
    },
    Badge {
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<TileTone>,
    },
    Progress {
        /// Fill fraction between 0 and 1.
        value: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    Countdown {
        /// RFC3339 timestamp the host counts down to.
        deadline: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<TileTone>,
    },
    Divider,
    Component {
        /// Catalog name, e.g. `"mui-button"`. See the TILES.md component catalog.
        component: String,
        #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
        props: serde_json::Map<String, serde_json::Value>,
        /// Child nodes. Only `"mui-stack"` accepts children.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<TileNode>,
    },
}

/// The full tile returned by `render_tile`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileUi {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub body: Vec<TileNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<TileAction>,
}

impl TileUi {
    /// Start a tile with a title and an empty body.
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), subtitle: None, body: Vec::new(), footer: Vec::new() }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Append a plain text node.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.body.push(TileNode::Text { text: text.into(), tone: None, muted: None });
        self
    }

    /// Append a text node with a tone. Set `muted` for de-emphasized styling.
    pub fn text_toned(mut self, text: impl Into<String>, tone: TileTone, muted: bool) -> Self {
        self.body.push(TileNode::Text {
            text: text.into(),
            tone: Some(tone),
            muted: if muted { Some(true) } else { None },
        });
        self
    }

    /// Append a row built with [`RowBuilder`].
    pub fn row(mut self, row: RowBuilder) -> Self {
        self.body.push(row.build());
        self
    }

    /// Append a standalone badge node.
    pub fn badge(mut self, label: impl Into<String>, tone: TileTone) -> Self {
        self.body.push(TileNode::Badge { label: label.into(), tone: Some(tone) });
        self
    }

    /// Append a progress bar. `value` must be within 0..=1; the host rejects
    /// the whole tile if it is out of range.
    pub fn progress(mut self, value: f32) -> Self {
        self.body.push(TileNode::Progress { value, label: None });
        self
    }

    /// Append a labeled progress bar.
    pub fn progress_labeled(mut self, value: f32, label: impl Into<String>) -> Self {
        self.body.push(TileNode::Progress { value, label: Some(label.into()) });
        self
    }

    /// Append a countdown to an RFC3339 deadline.
    pub fn countdown(mut self, deadline: impl Into<String>) -> Self {
        self.body.push(TileNode::Countdown { deadline: deadline.into(), label: None, tone: None });
        self
    }

    /// Append a labeled, toned countdown.
    pub fn countdown_labeled(
        mut self,
        deadline: impl Into<String>,
        label: impl Into<String>,
        tone: TileTone,
    ) -> Self {
        self.body.push(TileNode::Countdown {
            deadline: deadline.into(),
            label: Some(label.into()),
            tone: Some(tone),
        });
        self
    }

    /// Append a horizontal divider.
    pub fn divider(mut self) -> Self {
        self.body.push(TileNode::Divider);
        self
    }

    /// Append a catalog component node.
    ///
    /// `props` should be a JSON object, typically built with `serde_json::json!`:
    ///
    /// ```rust,no_run
    /// # use pointiv_extension_sdk::tile::TileUi;
    /// # use serde_json::json;
    /// TileUi::new("T").component("mui-chip", json!({ "label": "beta", "size": "small" }))
    /// # ;
    /// ```
    pub fn component(mut self, name: impl Into<String>, props_json: serde_json::Value) -> Self {
        let props = match props_json {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };
        self.body.push(TileNode::Component { component: name.into(), props, children: Vec::new() });
        self
    }

    /// Append any node directly. Escape hatch for shapes the helpers do not cover,
    /// such as an `mui-stack` component with children.
    pub fn node(mut self, node: TileNode) -> Self {
        self.body.push(node);
        self
    }

    /// Append a footer action. The host rejects the whole tile if it has more
    /// than 3 footer actions.
    pub fn footer(mut self, label: impl Into<String>, command: impl Into<String>) -> Self {
        self.footer.push(TileAction::new(label, command));
        self
    }
}

/// Builder for [`TileNode::Row`].
///
/// ```rust,no_run
/// # use pointiv_extension_sdk::tile::{RowBuilder, TileTone, TileUi};
/// TileUi::new("Todos").row(
///     RowBuilder::new("Buy milk")
///         .secondary("added today")
///         .badge("due", TileTone::Warn)
///         .action("Done", "todo done 1"),
/// )
/// # ;
/// ```
#[derive(Debug, Clone)]
pub struct RowBuilder {
    text: String,
    secondary: Option<String>,
    badge: Option<TileBadge>,
    actions: Vec<TileAction>,
}

impl RowBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), secondary: None, badge: None, actions: Vec::new() }
    }

    pub fn secondary(mut self, secondary: impl Into<String>) -> Self {
        self.secondary = Some(secondary.into());
        self
    }

    pub fn badge(mut self, label: impl Into<String>, tone: TileTone) -> Self {
        self.badge = Some(TileBadge { label: label.into(), tone: Some(tone) });
        self
    }

    /// Add a row action. The host rejects the whole tile if a row has more
    /// than 2 actions.
    pub fn action(mut self, label: impl Into<String>, command: impl Into<String>) -> Self {
        self.actions.push(TileAction::new(label, command));
        self
    }

    pub fn build(self) -> TileNode {
        TileNode::Row {
            text: self.text,
            secondary: self.secondary,
            badge: self.badge,
            actions: self.actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_serializes_to_wire_format() {
        let tile = TileUi::new("Todos")
            .badge("2 open", TileTone::Warn)
            .row(RowBuilder::new("milk").action("Done", "todo done 1"))
            .footer("Refresh", "todo list");
        let json = serde_json::to_value(&tile).unwrap();
        assert_eq!(json["title"], "Todos");
        assert_eq!(json["body"][0]["type"], "badge");
        assert_eq!(json["body"][0]["tone"], "warn");
        assert_eq!(json["body"][1]["type"], "row");
        assert_eq!(json["body"][1]["actions"][0]["command"], "todo done 1");
        assert_eq!(json["footer"][0]["label"], "Refresh");
        // Unset optionals stay off the wire.
        assert!(json["body"][1].get("secondary").is_none());
        assert!(json.get("subtitle").is_none());
    }

    #[test]
    fn wire_json_round_trips() {
        let raw = r#"{"title":"T","body":[{"type":"text","text":"hi","tone":"ok"},{"type":"divider"}],"footer":[{"label":"Go","command":"go"}]}"#;
        let tile: TileUi = serde_json::from_str(raw).unwrap();
        assert_eq!(tile.body.len(), 2);
        let back = serde_json::to_string(&tile).unwrap();
        assert!(back.contains(r#""type":"divider""#));
    }
}
