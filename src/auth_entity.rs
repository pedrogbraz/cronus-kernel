//! Implicit auth entity.
//!
//! `auth { entity User }` names the table that signup/login read and write.
//! When the program does not declare that entity, the runtime synthesizes it
//! so a fresh app has a working signup/login instead of "no such table".

use crate::parser::{self, AstNode, EntityNode};

/// The fields every synthesized auth entity gets.
const AUTH_ENTITY_FIELDS: &str = "  email    email! unique\n  password string! sensitive\n  role     string\n  name     string\n";

/// Builds the implicit auth entity named `name` through the parser, so it is
/// exactly what the user would get by writing it out.
fn synthesize(name: &str) -> Option<EntityNode> {
    let src = format!("entity {} {{\n{}}}\n", name, AUTH_ENTITY_FIELDS);
    parser::parse(&src).ok()?.into_iter().find_map(|n| match n {
        AstNode::Entity(e) => Some(e),
        _ => None,
    })
}

/// Appends the auth entity to `entities` when `auth_entity` names one that is
/// not declared (case-insensitive, same rule as `session::auth_table`).
/// Returns `true` when an entity was synthesized.
pub(crate) fn ensure_declared(entities: &mut Vec<EntityNode>, auth_entity: Option<&str>) -> bool {
    let Some(name) = auth_entity.map(str::trim).filter(|n| !n.is_empty()) else {
        return false;
    };
    if entities.iter().any(|e| e.name.eq_ignore_ascii_case(name)) {
        return false;
    }
    match synthesize(name) {
        Some(entity) => {
            entities.push(entity);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entities_and_auth(src: &str) -> (Vec<EntityNode>, Option<String>) {
        let mut entities = Vec::new();
        let mut auth = None;
        for node in parser::parse(src).expect("parse") {
            match node {
                AstNode::Entity(e) => entities.push(e),
                AstNode::Auth(a) => auth = Some(a.entity),
                _ => {}
            }
        }
        (entities, auth)
    }

    #[test]
    fn undeclared_auth_entity_is_synthesized_with_login_fields() {
        let (mut entities, auth) = entities_and_auth(
            "app \"A\" { port 5950 }\nauth {\n  entity User\n  login email + password\n  session jwt\n}\nentity Note {\n  title string!\n}\n",
        );
        assert!(ensure_declared(&mut entities, auth.as_deref()));
        let user = entities.iter().find(|e| e.name == "User").expect("User");
        let field = |n: &str| user.fields.iter().find(|f| f.name == n).expect(n);
        assert!(field("email").required && field("email").unique);
        assert!(field("password").required && field("password").sensitive);
        assert!(!field("role").required);
        assert!(!field("name").required);
    }

    #[test]
    fn synthesized_auth_entity_backs_signup_and_login() {
        let src = "app \"A\" { port 5950 }\nauth {\n  entity User\n  login email + password\n  session jwt\n  roles [member]\n}\nentity Note {\n  title string!\n}\n";
        let (mut entities, auth) = entities_and_auth(src);
        ensure_declared(&mut entities, auth.as_deref());
        let db = crate::database::CronusDB::open_memory().expect("memory db");
        db.migrate(&entities).expect("migrate");
        let row = db
            .insert(
                "User",
                &serde_json::json!({"name": "Ana", "email": "ana@x.test", "password": "h", "role": "member"}),
            )
            .expect("insert into synthesized User table");
        assert_eq!(row["email"], "ana@x.test");
        assert!(db
            .find_by_field("User", "email", "ana@x.test")
            .unwrap()
            .is_some());
    }

    #[test]
    fn declared_auth_entity_is_left_alone() {
        let (mut entities, auth) = entities_and_auth(
            "app \"A\" { port 5950 }\nauth {\n  entity user\n  login email + password\n  session jwt\n}\nentity User {\n  email email! unique\n  password string! sensitive\n  team string\n}\n",
        );
        assert!(!ensure_declared(&mut entities, auth.as_deref()));
        assert_eq!(entities.len(), 1);
        assert!(entities[0].fields.iter().any(|f| f.name == "team"));
    }

    #[test]
    fn no_auth_block_synthesizes_nothing() {
        let mut entities = Vec::new();
        assert!(!ensure_declared(&mut entities, None));
        assert!(entities.is_empty());
    }
}
