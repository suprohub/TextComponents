use std::borrow::Cow;

use crate::{
    RawTextComponent,
    content::{Content, NbtSource, Object, Resolvable},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent},
    minimessage::*,
};

fn first_child<'a>(comp: &'a RawTextComponent<'a>) -> &'a RawTextComponent<'a> {
    comp.children.first().expect("expected at least one child")
}

fn children<'a>(comp: &'a RawTextComponent<'a>) -> &'a [RawTextComponent<'a>] {
    &comp.children
}

#[test]
fn plain_text() {
    let root = parse("Hello");
    let child = first_child(&root);
    assert_eq!(
        child.content,
        Content::Text {
            text: Cow::Borrowed("Hello")
        }
    );
    assert!(child.format.color.is_none());
    assert!(child.format.bold.is_none());
    assert!(child.interactions.click.is_none());
}

#[test]
fn color_named() {
    let root = parse("<red>Test");
    let child = first_child(&root);
    assert_eq!(child.format.color, Some(Color::Red));
    assert_eq!(child.children.len(), 1);
    assert_eq!(
        child.children[0].content,
        Content::Text {
            text: Cow::Borrowed("Test")
        }
    );
}

#[test]
fn color_hex() {
    let root = parse("<#00ff00>Green");
    let child = first_child(&root);
    assert_eq!(child.format.color, Some(Color::Rgb(0, 255, 0)));
}

#[test]
fn color_nested() {
    let root = parse("<yellow>Hello <blue>World</blue>!");
    let top_child = first_child(&root);
    assert_eq!(
        top_child.content,
        Content::Text {
            text: Cow::Borrowed("")
        }
    );
    assert_eq!(top_child.format.color, Some(Color::Yellow));
    assert_eq!(top_child.children.len(), 3);

    let hello = &top_child.children[0];
    assert_eq!(
        hello.content,
        Content::Text {
            text: Cow::Borrowed("Hello ")
        }
    );

    let blue_wrapper = &top_child.children[1];
    assert_eq!(blue_wrapper.format.color, Some(Color::Blue));
    assert_eq!(blue_wrapper.children.len(), 1);
    let world = &blue_wrapper.children[0];
    assert_eq!(
        world.content,
        Content::Text {
            text: Cow::Borrowed("World")
        }
    );

    let excl = &top_child.children[2];
    assert_eq!(
        excl.content,
        Content::Text {
            text: Cow::Borrowed("!")
        }
    );
}

#[test]
fn bold() {
    let root = parse("<bold>Bold text");
    let child = first_child(&root);
    assert_eq!(child.format.bold, Some(true));
}

#[test]
fn not_bold() {
    let root = parse("<!bold>Not bold");
    let child = first_child(&root);
    assert_eq!(child.format.bold, Some(false));
}

#[test]
fn italic_aliases() {
    for tag in &["i", "em", "italic"] {
        let root = parse(&format!("<{}>Italic</{}>", tag, tag));
        let child = first_child(&root);
        assert_eq!(child.format.italic, Some(true), "failed for tag {}", tag);
    }
}

#[test]
fn underlined() {
    let root = parse("<u>Under</u>");
    let child = first_child(&root);
    assert_eq!(child.format.underlined, Some(true));
}

#[test]
fn strikethrough() {
    let root = parse("<st>Strike</st>");
    let child = first_child(&root);
    assert_eq!(child.format.strikethrough, Some(true));
}

#[test]
fn obfuscated() {
    let root = parse("<obf>Obfuscated</obf>");
    let child = first_child(&root);
    assert_eq!(child.format.obfuscated, Some(true));
}

#[test]
fn negation_underlined() {
    let root = parse("<!u>Not underlined");
    let child = first_child(&root);
    assert_eq!(child.format.underlined, Some(false));
}

#[test]
fn reset_clears_style() {
    let root = parse("<yellow><bold>Hello <reset>world!");
    let kids = children(&root);
    assert_eq!(kids.len(), 2);

    let yellow = &kids[0];
    assert_eq!(yellow.format.color, Some(Color::Yellow));
    assert!(yellow.format.bold.is_none());
    assert_eq!(yellow.children.len(), 1);

    let bold = &yellow.children[0];
    assert_eq!(bold.format.bold, Some(true));
    assert_eq!(bold.children.len(), 1);
    assert_eq!(
        bold.children[0].content,
        Content::Text {
            text: Cow::Borrowed("Hello ")
        }
    );

    let world = &kids[1];
    assert!(world.format.color.is_none());
    assert!(world.format.bold.is_none());
    assert_eq!(
        world.content,
        Content::Text {
            text: Cow::Borrowed("world!")
        }
    );
}

#[test]
fn shadow_named() {
    let root = parse("<shadow:red>Shadow");
    let child = first_child(&root);
    let expected = Format::parse_shadow_color(64, 255, 85, 85);
    assert_eq!(child.format.shadow_color, Some(expected));
}

#[test]
fn shadow_alpha() {
    let root = parse("<shadow:aqua:0.5>Test");
    let child = first_child(&root);
    let expected = Format::parse_shadow_color(128, 85, 255, 255);
    assert_eq!(child.format.shadow_color, Some(expected));
}

#[test]
fn shadow_hex() {
    let root = parse("<shadow:#FF0000>Red shadow");
    let child = first_child(&root);
    let expected = Format::parse_shadow_color(64, 255, 0, 0);
    assert_eq!(child.format.shadow_color, Some(expected));
}

#[test]
fn shadow_hex_with_alpha() {
    let root = parse("<shadow:#FF000080>Red shadow alpha");
    let child = first_child(&root);
    let expected = Format::parse_shadow_color(0x80, 255, 0, 0);
    assert_eq!(child.format.shadow_color, Some(expected));
}

#[test]
fn shadow_disable() {
    let root = parse("<!shadow>No shadow");
    let child = first_child(&root);
    assert_eq!(child.format.shadow_color, Some(0));
}

#[test]
fn verbose_color() {
    for tag in &["color", "c", "colour"] {
        let root = parse(&format!("<{}:blue>Blue</{}>", tag, tag));
        let child = first_child(&root);
        assert_eq!(child.format.color, Some(Color::Blue), "tag {}", tag);
    }
}

#[test]
fn click_run_command() {
    let root = parse("<click:run_command:/seed>Click");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::RunCommand {
            command: Cow::Owned("/seed".into())
        })
    );
}

#[test]
fn click_open_url() {
    let root = parse("<click:open_url:https://example.com>Link");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::OpenUrl {
            url: Cow::Owned("https://example.com".into())
        })
    );
}

#[test]
fn click_suggest_command() {
    let root = parse("<click:suggest_command:/help>Suggest");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::SuggestCommand {
            command: Cow::Owned("/help".into())
        })
    );
}

#[test]
fn click_change_page() {
    let root = parse("<click:change_page:3>Page 3");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::ChangePage { page: 3 })
    );
}

#[test]
fn click_copy_to_clipboard() {
    let root = parse("<click:copy_to_clipboard:secret>Copy");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::CopyToClipboard {
            value: Cow::Owned("secret".into())
        })
    );
}

#[test]
fn click_show_dialog() {
    let root = parse("<click:show_dialog:dialog_id>Dialog");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.click,
        Some(ClickEvent::ShowDialog {
            dialog: Cow::Owned("dialog_id".into())
        })
    );
}

#[cfg(feature = "custom")]
#[test]
fn click_custom() {
    let root = parse("<click:custom:my_action>Custom");
    let child = first_child(&root);
    match &child.interactions.click {
        Some(ClickEvent::Custom(data)) => {
            assert_eq!(data.id, "my_action");
        }
        _ => panic!("expected custom click event"),
    }
}

#[test]
fn hover_show_text() {
    let root = parse("<hover:show_text:'<red>test'>Hover");
    let child = first_child(&root);
    match &child.interactions.hover {
        Some(HoverEvent::ShowText { value }) => {
            let inner = value;
            let inner_child = inner.children.first().unwrap();
            assert_eq!(inner_child.format.color, Some(Color::Red));
            assert_eq!(
                inner_child.children[0].content,
                Content::Text {
                    text: Cow::Borrowed("test")
                }
            );
        }
        _ => panic!("expected show_text hover event"),
    }
}

#[test]
fn hover_show_item() {
    let root = parse("<hover:show_item:stone:3:tag>Item");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.hover,
        Some(HoverEvent::ShowItem {
            id: Cow::Owned("stone".into()),
            count: Some(3),
            components: Some(Cow::Owned("tag".into())),
        })
    );
}

#[test]
fn hover_show_entity() {
    let uuid_str = "1f085b2d-9548-4159-a8c7-f3ccdf0c2054";
    let root = parse(&format!("<hover:show_entity:cow:{}:Name>Entity", uuid_str));
    let child = first_child(&root);
    match &child.interactions.hover {
        Some(HoverEvent::ShowEntity { id, uuid, name }) => {
            assert_eq!(id.as_ref(), "cow");
            assert_eq!(*uuid, uuid::Uuid::parse_str(uuid_str).unwrap());
            let name_comp = name.as_ref().unwrap();
            let name_text = name_comp.children.first().unwrap();
            assert_eq!(
                name_text.content,
                Content::Text {
                    text: Cow::Borrowed("Name")
                }
            );
        }
        _ => panic!("expected show_entity hover event"),
    }
}

#[test]
fn insertion() {
    let root = parse("<insert:test>Insert");
    let child = first_child(&root);
    assert_eq!(
        child.interactions.insertion,
        Some(Cow::Owned("test".into()))
    );
}

#[test]
fn font() {
    let root = parse("<font:uniform>Uniform text");
    let child = first_child(&root);
    assert_eq!(child.format.font, Some(Cow::Owned("uniform".into())));
}

#[test]
fn font_with_namespace() {
    let root = parse("<font:myfont:custom_font>Custom");
    let child = first_child(&root);
    assert_eq!(
        child.format.font,
        Some(Cow::Owned("myfont:custom_font".into()))
    );
}

#[test]
fn keybind() {
    let root = parse("<key:key.jump>");
    let child = first_child(&root);
    assert_eq!(
        child.content,
        Content::Keybind {
            keybind: Cow::Owned("key.jump".into())
        }
    );
}

#[test]
fn translate() {
    let root = parse("<lang:block.minecraft.diamond_block>");
    let child = first_child(&root);
    match &child.content {
        Content::Translate(msg) => {
            assert_eq!(msg.key, "block.minecraft.diamond_block");
            assert!(msg.fallback.is_none());
            assert!(msg.args.is_none());
        }
        _ => panic!("expected translation"),
    }
}

#[test]
fn translate_with_args() {
    let root = parse("<lang:commands.drop.success.single:'<red>1':'<blue>Stone'>");
    let child = first_child(&root);
    match &child.content {
        Content::Translate(msg) => {
            assert_eq!(msg.key, "commands.drop.success.single");
            let args = msg.args.as_ref().unwrap();
            assert_eq!(args.len(), 2);
            let arg1 = &args[0];
            let red_child = arg1.children.first().unwrap();
            assert_eq!(red_child.format.color, Some(Color::Red));
            assert_eq!(
                red_child.children[0].content,
                Content::Text {
                    text: Cow::Borrowed("1")
                }
            );
            let arg2 = &args[1];
            let blue_child = arg2.children.first().unwrap();
            assert_eq!(blue_child.format.color, Some(Color::Blue));
            assert_eq!(
                blue_child.children[0].content,
                Content::Text {
                    text: Cow::Borrowed("Stone")
                }
            );
        }
        _ => panic!("expected translation"),
    }
}

#[test]
fn translate_with_fallback() {
    let root = parse("<lang_or:my.key:Fallback>");
    let child = first_child(&root);
    match &child.content {
        Content::Translate(msg) => {
            assert_eq!(msg.key, "my.key");
            assert_eq!(msg.fallback, Some(Cow::Owned("Fallback".into())));
            assert!(msg.args.is_none());
        }
        _ => panic!("expected translation with fallback"),
    }
}

#[test]
fn newline() {
    let root = parse("Line1<newline>Line2");
    let kids = children(&root);
    assert_eq!(kids.len(), 1);
    assert_eq!(
        kids[0].content,
        Content::Text {
            text: Cow::Borrowed("Line1\nLine2")
        }
    );
}

#[test]
fn selector() {
    let root = parse("<sel:@a>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::Entity {
            selector,
            separator: _,
        }) => {
            assert_eq!(selector, "@a");
        }
        _ => panic!("expected entity selector"),
    }
}

#[test]
fn selector_with_separator() {
    let root = parse("<sel:@a:', '>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::Entity {
            selector,
            separator,
        }) => {
            assert_eq!(selector, "@a");
            let sep_text = separator.children.first().unwrap();
            assert_eq!(
                sep_text.content,
                Content::Text {
                    text: Cow::Borrowed(", ")
                }
            );
        }
        _ => panic!("expected entity selector with separator"),
    }
}

#[test]
fn score() {
    let root = parse("<score:player:deaths>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::Scoreboard {
            selector,
            objective,
        }) => {
            assert_eq!(selector, "player");
            assert_eq!(objective, "deaths");
        }
        _ => panic!("expected scoreboard"),
    }
}

#[test]
fn nbt_entity() {
    let root = parse("<nbt:entity:@s:Health>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::NBT {
            path,
            source,
            interpret,
            separator: _,
        }) => {
            assert_eq!(path, "Health");
            assert_eq!(*source, NbtSource::Entity(Cow::Owned("@s".into())));
            assert!(interpret.is_none());
        }
        _ => panic!("expected nbt"),
    }
}

#[test]
fn nbt_with_interpret() {
    let root = parse("<nbt:block:12 34 56:Items:, :interpret>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::NBT {
            source, interpret, ..
        }) => {
            assert!(*interpret == Some(true));
            assert_eq!(*source, NbtSource::Block(Cow::Owned("12 34 56".into())));
        }
        _ => panic!("expected nbt with interpret"),
    }
}

#[test]
fn nbt_with_separator() {
    let root = parse("<nbt:storage:foo:bar:', ':interpret>");
    let child = first_child(&root);
    match &child.content {
        Content::Resolvable(Resolvable::NBT {
            separator,
            source,
            interpret,
            ..
        }) => {
            assert_eq!(*source, NbtSource::Storage(Cow::Owned("foo".into())));
            assert!(*interpret == Some(true));
            let sep_text = separator.children.first().unwrap();
            assert_eq!(
                sep_text.content,
                Content::Text {
                    text: Cow::Borrowed(", ")
                }
            );
        }
        _ => panic!("expected nbt with separator"),
    }
}

#[test]
fn sprite_full() {
    let root = parse("<sprite:blocks:item/diamond_sword>");
    let child = first_child(&root);
    match &child.content {
        Content::Object(Object::Atlas { atlas, sprite }) => {
            assert_eq!(atlas.as_deref(), Some("blocks"));
            assert_eq!(sprite, "item/diamond_sword");
        }
        _ => panic!("expected sprite"),
    }
}

#[test]
fn sprite_only() {
    let root = parse("<sprite:item/emerald>");
    let child = first_child(&root);
    match &child.content {
        Content::Object(Object::Atlas { atlas, sprite }) => {
            assert!(atlas.is_none());
            assert_eq!(sprite, "item/emerald");
        }
        _ => panic!("expected sprite"),
    }
}

#[test]
fn head_by_name() {
    let root = parse("<head:Strokkur24>");
    let child = first_child(&root);
    match &child.content {
        Content::Object(Object::Player { player, hat }) => {
            assert!(hat);
            assert_eq!(player.name, Some("Strokkur24".into()));
        }
        _ => panic!("expected player head"),
    }
}

#[test]
fn head_no_outer_layer() {
    let root = parse("<head:Strokkur24:false>");
    let child = first_child(&root);
    match &child.content {
        Content::Object(Object::Player { player: _, hat }) => assert!(!hat),
        _ => panic!("expected head"),
    }
}

#[test]
fn head_by_uuid() {
    let uuid_str = "1f085b2d-9548-4159-a8c7-f3ccdf0c2054";
    let root = parse(&format!("<head:{}>", uuid_str));
    let child = first_child(&root);
    assert!(matches!(
        child.content,
        Content::Object(Object::Player { .. })
    ));
}

#[cfg(feature = "custom")]
#[test]
fn rainbow() {
    let root = parse("<rainbow>hello</rainbow>");
    let child = first_child(&root);
    match &child.content {
        Content::Custom(data) => assert_eq!(data.id, "rainbow"),
        _ => panic!("expected rainbow custom element"),
    }
}

#[cfg(feature = "custom")]
#[test]
fn gradient() {
    let root = parse("<gradient>hello</gradient>");
    let child = first_child(&root);
    match &child.content {
        Content::Custom(data) => assert_eq!(data.id, "gradient"),
        _ => panic!("expected gradient"),
    }
}

#[cfg(feature = "custom")]
#[test]
fn transition() {
    let root = parse("<transition>hello</transition>");
    let child = first_child(&root);
    match &child.content {
        Content::Custom(data) => assert_eq!(data.id, "transition"),
        _ => panic!("expected transition"),
    }
}

#[cfg(feature = "custom")]
#[test]
fn pride() {
    let root = parse("<pride>hello</pride>");
    let child = first_child(&root);
    match &child.content {
        Content::Custom(data) => assert_eq!(data.id, "pride"),
        _ => panic!("expected pride"),
    }
}

#[test]
fn self_closing_tag() {
    let root = parse("<yellow/>Hello");
    let kids = children(&root);
    assert_eq!(kids.len(), 2);
    assert_eq!(kids[0].format.color, Some(Color::Yellow));
    assert_eq!(
        kids[0].content,
        Content::Text {
            text: Cow::Borrowed("")
        }
    );
    assert_eq!(
        kids[1].content,
        Content::Text {
            text: Cow::Borrowed("Hello")
        }
    );
}

#[test]
fn unclosed_tag() {
    let root = parse("<yellow>Hello");
    let child = first_child(&root);
    assert_eq!(child.format.color, Some(Color::Yellow));
    assert_eq!(
        child.children[0].content,
        Content::Text {
            text: Cow::Borrowed("Hello")
        }
    );
}

#[test]
fn escape_backslash() {
    let root = parse(r"\\<red>test");
    let kids = children(&root);
    assert_eq!(kids.len(), 2);
    assert_eq!(
        kids[0].content,
        Content::Text {
            text: Cow::Owned("\\".into())
        }
    );
    let red_wrapper = &kids[1];
    assert_eq!(red_wrapper.format.color, Some(Color::Red));
    assert_eq!(red_wrapper.children.len(), 1);
    assert_eq!(
        red_wrapper.children[0].content,
        Content::Text {
            text: Cow::Borrowed("test")
        }
    );
}

#[test]
fn unknown_tag_ignored() {
    let root = parse("<unknown>test</unknown>");
    let child = first_child(&root);
    assert_eq!(
        child.content,
        Content::Text {
            text: Cow::Owned("test".into())
        }
    );
}

#[test]
fn mixed_formatting() {
    let root = parse("<bold><italic>Text</italic></bold>");
    let bold = first_child(&root);
    assert_eq!(bold.format.bold, Some(true));
    let italic = &bold.children[0];
    assert_eq!(italic.format.italic, Some(true));
    let text = &italic.children[0];
    assert_eq!(
        text.content,
        Content::Text {
            text: Cow::Borrowed("Text")
        }
    );
}

#[test]
fn quoted_args_with_escaped_quote() {
    let root = parse(r"<hover:show_text:'It\'s a test'>Hover");
    let child = first_child(&root);
    match &child.interactions.hover {
        Some(HoverEvent::ShowText { value }) => {
            let inner_child = value.children.first().unwrap();
            assert_eq!(
                inner_child.content,
                Content::Text {
                    text: Cow::Owned("It's a test".into())
                }
            );
        }
        _ => panic!("expected hover"),
    }
}
