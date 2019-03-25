use super::AstNode;

pub trait AstVisitor {
    fn visit(node: &AstNode);
}
