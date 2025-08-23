use std::collections::HashMap;

struct DialogBuilder {
    start_node: DialogNodeId,
    nodes: HashMap<DialogNodeId, DialogNode>,
}

impl DialogBuilder {
    pub fn new(start_node: DialogNode) -> Self {
        let mut nodes = HashMap::new();
        let start_node_id = start_node.id().clone();
        nodes.insert(start_node_id.clone(), start_node);
        DialogBuilder {
            start_node: start_node_id,
            nodes,
        }
    }

    pub fn build(self) -> Result<Dialog, DialogError> {
        Ok(Dialog {
            start_node: self.start_node,
            nodes: self.nodes,
        })
    }

    pub fn add_node(mut self, node: DialogNode) -> Self {
        self.nodes.insert(node.id().clone(), node);
        self
    }

    pub fn add_link(mut self, link: DialogLink) -> Self {
        self.nodes.get_mut(link.from()).unwrap().add_link(link);
        self
    }
}

struct Dialog {
    start_node: DialogNodeId,
    nodes: HashMap<DialogNodeId, DialogNode>,
}

impl Dialog {
    pub fn start_node(&self) -> &DialogNode {
        self.nodes.get(&self.start_node).unwrap()
    }

    pub fn start(&self) -> DialogExecutor {
        DialogExecutor::new(&self)
    }

    pub fn get_node(&self, id: &DialogNodeId) -> &DialogNode {
        self.nodes.get(id).unwrap()
    }
}

struct DialogExecutor<'d> {
    dialog: &'d Dialog,
    current: DialogNodeId,
    path: Vec<(DialogNodeId, usize)>,
}

impl<'d> DialogExecutor<'d> {
    pub fn new(dialog: &'d Dialog) -> Self {
        DialogExecutor {
            dialog,
            current: dialog.start_node().id().clone(),
            path: Vec::new(),
        }
    }

    pub fn current_node(&self) -> &DialogNode {
        self.dialog.get_node(&self.current)
    }

    pub fn choices<'dd>(&'dd mut self) -> DialogExecChoices<'dd, 'd>
    where
        'd: 'dd,
    {
        let current_node_history = self
            .path
            .iter()
            .filter(|(id, _)| id == self.current_node().id())
            .map(|(_, index)| *index);
        DialogExecChoices::new(self, current_node_history.collect())
    }

    fn choose(&mut self, index: usize) {
        self.path.push((self.current_node().id().clone(), index));
        let chosen_link = self.current_node().links.get(index).unwrap();
        self.current = chosen_link.to().clone();
    }
}

struct DialogExecChoices<'dd, 'd> {
    parent: &'dd mut DialogExecutor<'d>,
    history: Vec<usize>,
}

impl<'dd, 'd> DialogExecChoices<'dd, 'd> {
    pub fn new(parent: &'dd mut DialogExecutor<'d>, history: Vec<usize>) -> Self {
        DialogExecChoices { parent, history }
    }

    pub fn all(&'dd self) -> impl Iterator<Item = (usize, &'dd DialogText)> + use<'dd, 'd> {
        self.parent
            .current_node()
            .links
            .iter()
            .enumerate()
            .filter(move |(index, link)| {
                link.condition == DialogLinkCondition::None
                    || (link.condition == DialogLinkCondition::OnlyIfNotYetChosen
                        && !self.history.contains(index))
            })
            .map(move |(index, link)| (index, link.text()))
    }

    pub fn get(self, index: usize) -> Option<DialogExecChoice<'dd, 'd>> {
        match self.parent.current_node().links.get(index) {
            Some(link) => Some(DialogExecChoice::new(link.clone(), self, index)),
            None => None,
        }
    }

    fn choose(self, index: usize) {
        self.parent.choose(index);
    }
}

struct DialogExecChoice<'dd, 'd> {
    link: DialogLink,
    parent: DialogExecChoices<'dd, 'd>,
    index: usize,
}

impl<'dd, 'd> DialogExecChoice<'dd, 'd> {
    pub fn new(link: DialogLink, parent: DialogExecChoices<'dd, 'd>, index: usize) -> Self {
        DialogExecChoice {
            link,
            parent,
            index,
        }
    }

    pub fn choose(self) {
        self.parent.choose(self.index);
    }
}

#[derive(PartialEq, Eq, Debug)]
enum DialogError {}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct DialogNodeId {
    id: String,
}

impl<S> From<S> for DialogNodeId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        DialogNodeId { id: value.into() }
    }
}

struct DialogNode {
    id: DialogNodeId,
    text: DialogText,
    links: Vec<DialogLink>,
}

impl DialogNode {
    pub fn new(id: impl Into<DialogNodeId>, text: impl Into<DialogText>) -> Self {
        DialogNode {
            id: id.into(),
            text: text.into(),
            links: Vec::new(),
        }
    }

    pub fn id(&self) -> &DialogNodeId {
        &self.id
    }
    pub fn text(&self) -> &DialogText {
        &self.text
    }
    pub fn links(&self) -> &Vec<DialogLink> {
        &self.links
    }
    pub fn add_link(&mut self, link: DialogLink) {
        self.links.push(link)
    }
}

#[derive(Clone)]
struct DialogLink {
    from: DialogNodeId,
    to: DialogNodeId,
    text: DialogText,
    condition: DialogLinkCondition,
}

impl DialogLink {
    pub fn new(
        from: impl Into<DialogNodeId>,
        to: impl Into<DialogNodeId>,
        text: impl Into<DialogText>,
        condition: DialogLinkCondition,
    ) -> Self {
        DialogLink {
            from: from.into(),
            to: to.into(),
            text: text.into(),
            condition,
        }
    }

    pub fn from(&self) -> &DialogNodeId {
        &self.from
    }
    pub fn to(&self) -> &DialogNodeId {
        &self.to
    }
    pub fn condition(&self) -> &DialogLinkCondition {
        &self.condition
    }
    pub fn text(&self) -> &DialogText {
        &self.text
    }
}

#[derive(PartialEq, Eq, Clone)]
enum DialogLinkCondition {
    None,
    OnlyIfNotYetChosen,
}

#[derive(Clone)]
enum DialogText {
    PlainText(String),
}

impl DialogText {
    pub fn as_plain_str(&self) -> &str {
        match self {
            Self::PlainText(plain_text) => plain_text,
        }
    }
}

impl<S> From<S> for DialogText
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        DialogText::PlainText(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn straight_dialog() {
        let dialog = DialogBuilder::new(DialogNode::new("Start", "Hello, World!"))
            .add_node(DialogNode::new("Second", "Goodbye, World!"))
            .add_link(DialogLink::new(
                "Start",
                "Second",
                "Hi and bye!",
                DialogLinkCondition::None,
            ))
            .build()
            .unwrap();
        assert_eq!(dialog.start_node().id(), &DialogNodeId::from("Start"));
        assert_eq!(dialog.start_node().text().as_plain_str(), "Hello, World!");
        let mut dialog_in_progress = dialog.start();
        assert_eq!(
            dialog_in_progress.current_node().id(),
            &DialogNodeId::from("Start")
        );
        assert_eq!(
            dialog_in_progress.current_node().text().as_plain_str(),
            "Hello, World!"
        );
        dialog_in_progress.choices().get(0).unwrap().choose();
        assert_eq!(
            dialog_in_progress.current_node().id(),
            &DialogNodeId::from("Second")
        );
        assert_eq!(
            dialog_in_progress.current_node().text().as_plain_str(),
            "Goodbye, World!"
        );
        let choices = dialog_in_progress.choices();
        let no_choices: Vec<(usize, &DialogText)> = choices.all().collect();
        assert_eq!(no_choices.len(), 0);
    }

    #[test]
    fn branched_dialog() {
        let dialog = DialogBuilder::new(DialogNode::new("Start", "Hello, World!"))
            .add_node(DialogNode::new("End", "Goodbye, World!"))
            .add_node(DialogNode::new("Branch", "Nice to meet you!"))
            .add_link(DialogLink::new(
                "Start",
                "End",
                "Hi and bye!",
                DialogLinkCondition::None,
            ))
            .add_link(DialogLink::new(
                "Start",
                "Branch",
                "Hi!",
                DialogLinkCondition::None,
            ))
            .add_link(DialogLink::new(
                "Branch",
                "End",
                "Goodbye",
                DialogLinkCondition::None,
            ))
            .build()
            .unwrap();
        let mut dialog_in_progress = dialog.start();
        dialog_in_progress.choices().get(1).unwrap().choose();
        assert_eq!(dialog_in_progress.current_node().id(), &"Branch".into());
        dialog_in_progress.choices().get(0).unwrap().choose();
        assert_eq!(dialog_in_progress.current_node().id(), &"End".into());
    }

    #[test]
    fn cyclic_dialog() {
        let dialog = DialogBuilder::new(DialogNode::new("Start", "Hello, World!"))
            .add_node(DialogNode::new("End", "Goodbye, World!"))
            .add_node(DialogNode::new("Branch", "Nice to meet you!"))
            .add_link(DialogLink::new(
                "Start",
                "End",
                "Hi and bye!",
                DialogLinkCondition::None,
            ))
            .add_link(DialogLink::new(
                "Start",
                "Branch",
                "Hi!",
                DialogLinkCondition::OnlyIfNotYetChosen,
            ))
            .add_link(DialogLink::new(
                "Branch",
                "End",
                "Goodbye",
                DialogLinkCondition::None,
            ))
            .add_link(DialogLink::new(
                "Branch",
                "Start",
                "Go Back",
                DialogLinkCondition::None,
            ))
            .build()
            .unwrap();
        let mut dialog_in_progress = dialog.start();
        dialog_in_progress.choices().get(1).unwrap().choose();
        assert_eq!(dialog_in_progress.current_node().id(), &"Branch".into());
        dialog_in_progress.choices().get(1).unwrap().choose();
        assert_eq!(dialog_in_progress.current_node().id(), &"Start".into());
        let choices = dialog_in_progress.choices();
        let single_choice: Vec<(usize, &DialogText)> = choices.all().collect();
        assert_eq!(single_choice.len(), 1);
    }
}
