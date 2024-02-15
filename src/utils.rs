use robotics_lib::world::tile::Content;

pub fn is_content_rock(content: &Content) -> bool {
    match content {
        Content::Rock(_) => true,
        _ => false,
    }
}
