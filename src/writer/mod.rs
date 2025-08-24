use super::engine;

pub fn serialize_dialog(dialog: &engine::Dialog) -> String {
    let mut output = String::new();
    let start_node_id = dialog.start_node().id().clone();
    output.push_str(&serialize_node(dialog.start_node()));
    for node in dialog.all_nodes() {
        if node.id() != &start_node_id {
            output.push_str(&serialize_node(node));
        }
    }
    output
}

fn serialize_node(node: &engine::DialogNode) -> String {
    let mut output = String::new();
    let mut link_text = String::new();
    let mut first = true;
    for link in node.links() {
        if !first {
            link_text.push_str(" | ");
        } else {
            link_text.push_str("\n---\n");
        }
        link_text.push_str(&format!(
            "{{{target}}} {text}",
            target = link.to().clone().to_string(),
            text = link.text().as_plain_str()
        ));
        first = false;
    }
    output.push_str(&format!(
        r#"Name: {name}
{text}{link_text}
===

"#,
        name = node.id().clone().to_string(),
        text = node.text().as_plain_str(),
        link_text = link_text
    ));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn render_simple_dialog() {
        let mut builder = engine::DialogBuilder::new(engine::DialogNode::new_with_links(
            "Start",
            "Hello, World!",
            vec![engine::DialogLink::new(
                "Start",
                "End",
                "Hi!",
                engine::DialogLinkCondition::None,
            )],
        ));
        builder = builder.add_node(engine::DialogNode::new("End", "Bye!"));
        let dialog = builder.build().unwrap();
        assert_eq!(
            serialize_dialog(&dialog),
            r#"Name: Start
Hello, World!
---
{End} Hi!
===

Name: End
Bye!
===

"#
        );
    }
}
