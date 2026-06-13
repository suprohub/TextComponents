#[cfg(feature = "custom")]
use crate::custom::CustomContent;
use crate::{
    content::{Content, NbtSource, Object, ObjectPlayer, Resolvable},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent, Interactivity},
    translation::TranslatedMessage,
};
use std::borrow::Cow;

extern crate self as text_components;

#[cfg(feature = "build")]
pub mod build;
pub mod content;
#[cfg(feature = "custom")]
pub mod custom;
pub mod fmt;
pub mod format;
pub mod interactivity;
#[cfg(feature = "minimessage")]
pub mod minimessage;
#[cfg(feature = "minimessage")]
#[cfg(test)]
mod minimessage_tests;
#[cfg(feature = "nbt")]
pub mod nbt;
pub mod parse;
pub mod resolving;
pub mod translation;

/// A recursive rich text format with interaction capabilities.
/// ### Styling
/// Any type implementing [Into]<[TextComponent]> can be styled into a\
/// TextComponent using the trait [format::Modifier] like this:
/// ```
/// // Plain text component
/// TextComponent::plain("Plain text").color(Color::Red);
/// // String slice
/// "String Slice".bold(true);
/// // Compiled translation (No arguments)
/// TRANSLATION_TEST.italic(true);
/// ```
/// ### Interactivity
/// Text that can be inserted into the chat with Shift+Click:
/// ```
/// component.insert("Insert text here!");
/// ```
/// Data that can be displayed by hovering the text:
/// ```
/// component.hover_event(HoverEvent::show_text("Click me!"));
/// ```
/// A event triggered when the user clicks the text:
/// ```
/// component.click_event(
///     ClickEvent::open_url("https://www.minecraft.net/")
/// );
/// ```
/// ### Children
/// ```
/// component.add_child("Child 1");
/// component.add_children(vec![
///     "Child 2".color("#bf00ff"),
///     CHILD_THREE.italic(true),
/// ]);
/// ```
/// ### Display
/// [TextComponent] implements [Display](std::fmt::Display) for easy logging, this means that\
/// `format!("{}", component)` will return the text component as plain text resolved by the default resolver,\
/// if you want a pretty text `{:p}` can be used instead for this proupose.
/// Using this methods is not recommended when the component will be sent to a player.
/// ### Building
/// A [TextComponent] needs to be built into another format before sending it\
/// anywhere, which requires a [TextResolutor](crate::build::TextResolutor)
/// and a [BuildTarget](crate::build::BuildTarget):
/// ```
/// let component = TextComponent::plain("Component to build");
/// component.build(resolutor, target);
/// ```
/// If the "serde" feature is enabled a [TextComponent] can be serialized with:
/// ```
/// let component = TextComponent::plain("Component to build");
/// component.resolve(resolutor).serialize(serializer);
/// ```
/// A function can be attached to a [BuildTarget](crate::build::BuildTarget) for easy access:
/// ```
/// let component = TextComponent::plain("Component to build");
/// // Builds with TextBuilder a plain String
/// component.to_plain(resolutor);
/// // Build with RichTextBuilder a decorated String
/// component.to_pretty(resolutor);
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "databake", derive(::databake::Bake))]
#[cfg_attr(feature = "databake", databake(path = text_components))]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct RawTextComponent<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub content: Content<'a>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", rename = "extra", default)
    )]
    pub children: Vec<RawTextComponent<'a>>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub format: Format<'a>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub interactions: Interactivity<'a>,
}

pub type TextComponent = RawTextComponent<'static>;

// Constructors
impl<'a> RawTextComponent<'a> {
    /// Creates an empty [TextComponent], useful to make it the parent.
    pub const fn new() -> Self {
        RawTextComponent {
            content: Content::Text {
                text: Cow::Borrowed(""),
            },
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] of a plain text at compile time.
    /// ## Example
    /// ```
    /// // Results in "Test Component"
    /// TextComponent::const_plain("Test Component");
    /// ```
    pub const fn const_plain(text: &'a str) -> Self {
        RawTextComponent {
            content: Content::Text {
                text: Cow::Borrowed(text),
            },
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] of a plain text.
    /// ## Example
    /// ```
    /// // Results in "Test Component"
    /// TextComponent::plain("Test Component");
    /// ```
    /// This is equivalent of doing:
    /// ```
    /// let component: TextComponent = "Test Component".into();
    /// ```
    pub fn plain(text: impl Into<Cow<'a, str>>) -> Self {
        RawTextComponent {
            content: Content::Text { text: text.into() },
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] of a [TranslatedMessage], it's recommended using a compiled
    /// [Translation](crate::translation::Translation) which forces you to give it the right amount of arguments.
    /// ## Examples
    /// #### For a translation without arguments:
    /// ```
    /// // Results in "Diamond Sword"
    /// TextComponent::translated(ITEM_MINECRAFT_DIAMOMD_SWORD.msg());
    /// ```
    /// This is equivalent of doing:
    /// ```
    /// let component: TextComponent = ITEM_MINECRAFT_DIAMOND_SWORD.into()
    /// ```
    /// or
    /// ```
    /// ITEM_MINECRAFT_DIAMOND_SWORD.msg().component()
    /// ```
    /// #### For a translation with 2 arguments:
    /// ```
    /// // Results in "The Rust compiler was killed by you using magic".
    /// TextComponent::translated(DEATH_ATTACK_INDIRECT_MAGIC.message(["The Rust compiler", "you"]));
    /// ```
    pub const fn translated(message: TranslatedMessage<'a>) -> Self {
        RawTextComponent {
            content: Content::Translate(message),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] with an image from a resource pack in it.\
    /// * `sprite` - The path to the texture, starting from the atlas\
    /// * `atlas` - The atlas where the texture belongs, if it's [None] will default to "minecraft:blocks"
    /// ## Example
    /// ```
    /// // Displays the Diamond Sword sprite
    /// TextComponent::atlas("item/diamond_sword", Some("minecraft:items"));
    /// ```
    pub fn atlas(sprite: impl Into<Cow<'a, str>>, atlas: Option<impl Into<Cow<'a, str>>>) -> Self {
        RawTextComponent {
            content: Content::Object(Object::Atlas {
                atlas: atlas.map(Into::into),
                sprite: sprite.into(),
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
    /// Creates a [TextComponent] with the head of a player in it.
    /// * `player` - A [ObjectPlayer] containing the required info
    /// * `hat` - Whether to display the hat layer
    /// ## Example
    /// ```
    /// // Displays the head of Jeb_
    /// TextComponent::player_head(ObjectPlayer::name("Jeb_"), true);
    /// ```
    pub const fn player_head(player: ObjectPlayer<'a>, hat: bool) -> Self {
        RawTextComponent {
            content: Content::Object(Object::Player { player, hat }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] that will contain the value of a Scoreboard.
    /// * `selector` - Describes the player to get the data (Needs to be only 1 entity)\
    ///   The character '*' can be used to show the receiver player data
    /// * `objective` - The internal name of the scoreboard to show
    /// ## Example
    /// ```
    /// // Displays the 'deaths' scoreboard value of the nearest player
    /// TextComponent::scoreboard("@p", "deaths");
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn scoreboard(
        selector: impl Into<Cow<'a, str>>,
        objective: impl Into<Cow<'a, str>>,
    ) -> Self {
        RawTextComponent {
            content: Content::Resolvable(Resolvable::Scoreboard {
                selector: selector.into(),
                objective: objective.into(),
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] containing a entity or group of entities.
    /// * `selector` - The selector of the entities to display
    /// * `separator` - The component separating multiple entities. If [None] will be a grey comma
    /// ## Example
    /// ```
    /// // Displays all the players name separated by a space
    /// TextComponent::entity("@a", Some(" ".into()));
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn entity(selector: impl Into<Cow<'a, str>>, separator: Option<Self>) -> Self {
        RawTextComponent {
            content: Content::Resolvable(Resolvable::Entity {
                selector: selector.into(),
                separator: match separator {
                    Some(separator) => Box::new(separator),
                    None => Box::new(", ".color(Color::Gray)),
                },
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] containing the data of a Nbt tag.
    /// * `path` - The Nbt path of the tag to show
    /// * `source` - A [NbtSource] indicating where to search the nbt tag
    /// * `interpret` - If [true](bool) the Nbt data will be read as it's a text component
    /// * `separator` - The component separating multiple Nbt tags. If [None] will be a comma
    /// ## Example
    /// ```
    /// // Displays the nearest player health
    /// TextComponent::nbt("Health", NbtSource::entity("@p"), false, None);
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn nbt(
        path: impl Into<Cow<'a, str>>,
        source: NbtSource<'a>,
        interpret: bool,
        separator: Option<Self>,
    ) -> Self {
        RawTextComponent {
            content: Content::Resolvable(Resolvable::NBT {
                path: path.into(),
                interpret: if interpret { Some(true) } else { None },
                separator: match separator {
                    Some(separator) => Box::new(separator),
                    None => Box::new(", ".into()),
                },
                source,
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    #[cfg(feature = "custom")]
    pub fn custom(content: impl CustomContent<'a> + 'a) -> RawTextComponent<'a> {
        RawTextComponent {
            content: Content::Custom(content.as_data()),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    #[cfg(feature = "minimessage")]
    pub fn minimessage(text: impl Into<&'a str>) -> RawTextComponent<'a> {
        use crate::minimessage::Parser;
        Parser::parse(text.into())
    }
}

impl<'a> Default for RawTextComponent<'a> {
    fn default() -> Self {
        RawTextComponent::new()
    }
}

impl<'a> From<&'a str> for RawTextComponent<'a> {
    fn from(value: &'a str) -> Self {
        RawTextComponent::const_plain(value)
    }
}
impl<'a> From<String> for RawTextComponent<'a> {
    fn from(value: String) -> Self {
        RawTextComponent::plain(value)
    }
}

pub trait Modifier<'a> {
    type Output;
    /// Adds a child at the end of a text component
    fn add_child<T: Into<RawTextComponent<'a>>>(self, child: T) -> Self::Output;
    /// Appends a [vec] of [Into]<[TextComponent]> as children of this component
    fn add_children<T: Into<RawTextComponent<'a>>>(self, children: Vec<T>) -> Self::Output;
    /// Sets the Shift+Click chat insertion string
    fn insertion<T: Into<Cow<'a, str>>>(self, insertion: T) -> Self::Output;
    /// Sets the [ClickEvent] for this component
    fn click_event(self, click: ClickEvent<'a>) -> Self::Output;
    /// Sets the [HoverEvent] for this component
    fn hover_event(self, hover: HoverEvent<'a>) -> Self::Output;
    /// Sets the [Color] of this component
    /// * If you want to use a hex code check [color_hex](TextComponent::color_hex)
    fn color(self, color: Color) -> Self::Output;
    /// Sets the color of this component from a 6 digit hex color
    /// * If you want to use a predefined color check [color](TextComponent::color)
    fn color_hex(self, color: &'a str) -> Self::Output;
    /// Sets the font used to display this component
    fn font<F: Into<Cow<'a, str>>>(self, font: F) -> Self::Output;
    /// Makes this component **bold**
    fn bold(self, value: bool) -> Self::Output;
    /// Makes this component *italic*
    fn italic(self, value: bool) -> Self::Output;
    /// Makes this component underlined
    fn underlined(self, value: bool) -> Self::Output;
    /// Makes this component ~~strikethrough~~
    fn strikethrough(self, value: bool) -> Self::Output;
    /// Makes this component obfuscated
    fn obfuscated(self, value: bool) -> Self::Output;
    /// Sets the shadow color of this component
    fn shadow_color(self, a: u8, r: u8, g: u8, b: u8) -> Self::Output;
    /// Sets all the format of this component to the default
    fn reset(self) -> Self::Output;
}

impl<'a, T: Into<RawTextComponent<'a>> + Sized> Modifier<'a> for T {
    type Output = RawTextComponent<'a>;
    fn add_child<F: Into<RawTextComponent<'a>>>(self, child: F) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.children.push(child.into());
        component
    }
    fn add_children<F: Into<RawTextComponent<'a>>>(self, children: Vec<F>) -> RawTextComponent<'a> {
        let mut component = self.into();
        for child in children {
            component.children.push(child.into());
        }
        component
    }

    fn insertion<R: Into<Cow<'a, str>>>(self, insertion: R) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.interactions.insertion = Some(insertion.into());
        component
    }
    fn click_event(self, click: ClickEvent<'a>) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.interactions.click = Some(click);
        component
    }
    fn hover_event(self, hover: HoverEvent<'a>) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.interactions.hover = Some(hover);
        component
    }

    fn color(self, color: Color) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.color(color);
        component
    }
    fn color_hex(self, color: &'a str) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.color_hex(color);
        component
    }
    fn font<F: Into<Cow<'a, str>>>(self, font: F) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.font(font);
        component
    }
    fn bold(self, value: bool) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.bold(value);
        component
    }
    fn italic(self, value: bool) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.italic(value);
        component
    }
    fn underlined(self, value: bool) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.underlined(value);
        component
    }
    fn strikethrough(self, value: bool) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.strikethrough(value);
        component
    }
    fn obfuscated(self, value: bool) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.obfuscated(value);
        component
    }
    fn shadow_color(self, a: u8, r: u8, g: u8, b: u8) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.shadow_color(a, r, g, b);
        component
    }
    fn reset(self) -> RawTextComponent<'a> {
        let mut component = self.into();
        component.format = component.format.reset();
        component
    }
}

impl<'a> Modifier<'a> for &'a mut RawTextComponent<'a> {
    type Output = &'a mut RawTextComponent<'a>;
    fn add_child<T: Into<RawTextComponent<'a>>>(self, child: T) -> &'a mut RawTextComponent<'a> {
        self.children.push(child.into());
        self
    }

    fn add_children<T: Into<RawTextComponent<'a>>>(
        self,
        children: Vec<T>,
    ) -> &'a mut RawTextComponent<'a> {
        for child in children {
            self.children.push(child.into());
        }
        self
    }

    fn insertion<T: Into<Cow<'a, str>>>(self, insertion: T) -> &'a mut RawTextComponent<'a> {
        self.interactions.insertion = Some(insertion.into());
        self
    }

    fn click_event(self, click: ClickEvent<'a>) -> &'a mut RawTextComponent<'a> {
        self.interactions.click = Some(click);
        self
    }

    fn hover_event(self, hover: HoverEvent<'a>) -> &'a mut RawTextComponent<'a> {
        self.interactions.hover = Some(hover);
        self
    }

    fn color(self, color: Color) -> &'a mut RawTextComponent<'a> {
        self.format.color = Some(color);
        self
    }

    fn color_hex(self, color: &str) -> &'a mut RawTextComponent<'a> {
        if let Some(color) = Color::from_hex(color) {
            self.format.color = Some(color);
        }
        self
    }

    fn font<F: Into<Cow<'a, str>>>(self, font: F) -> &'a mut RawTextComponent<'a> {
        self.format.font = Some(font.into());
        self
    }

    fn bold(self, value: bool) -> &'a mut RawTextComponent<'a> {
        self.format.bold = Some(value);
        self
    }

    fn italic(self, value: bool) -> &'a mut RawTextComponent<'a> {
        self.format.italic = Some(value);
        self
    }

    fn underlined(self, value: bool) -> &'a mut RawTextComponent<'a> {
        self.format.underlined = Some(value);
        self
    }

    fn strikethrough(self, value: bool) -> &'a mut RawTextComponent<'a> {
        self.format.strikethrough = Some(value);
        self
    }

    fn obfuscated(self, value: bool) -> &'a mut RawTextComponent<'a> {
        self.format.obfuscated = Some(value);
        self
    }

    fn shadow_color(self, a: u8, r: u8, g: u8, b: u8) -> &'a mut RawTextComponent<'a> {
        self.format.shadow_color = Some(Format::parse_shadow_color(a, r, g, b));
        self
    }

    fn reset(self) -> &'a mut RawTextComponent<'a> {
        self.format.color = Some(Color::White);
        self.format.font = Some(Cow::Borrowed("minecraft:default"));
        self.format.bold = Some(false);
        self.format.italic = Some(false);
        self.format.underlined = Some(false);
        self.format.strikethrough = Some(false);
        self.format.obfuscated = Some(false);
        self.format.shadow_color = None;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolving::{BuildTarget, NoResolutor, TextResolutor};
    use std::borrow::Cow;

    struct StringTarget;
    impl<'a> BuildTarget<'a> for StringTarget {
        type Result = String;
        fn build_component<R: TextResolutor<'a> + ?Sized>(
            &self,
            _resolutor: &R,
            component: &RawTextComponent<'a>,
        ) -> Self::Result {
            fn extract_text(comp: &RawTextComponent) -> String {
                let mut s = match &comp.content {
                    Content::Text { text } => text.to_string(),
                    Content::Translate(msg) => msg.key.to_string(),
                    _ => String::new(),
                };
                for child in &comp.children {
                    s.push_str(&extract_text(child));
                }
                s
            }
            extract_text(component)
        }
    }

    #[test]
    fn test_new_empty_component() {
        let component = TextComponent::new();
        assert_eq!(
            component.content,
            Content::Text {
                text: Cow::Borrowed("")
            }
        );
        assert!(component.children.is_empty());
    }

    #[test]
    fn test_plain_text() {
        let component = TextComponent::plain("Hello");
        assert_eq!(
            component.content,
            Content::Text {
                text: Cow::Borrowed("Hello")
            }
        );
    }

    #[test]
    fn test_const_plain() {
        let component = TextComponent::const_plain("World");
        assert_eq!(
            component.content,
            Content::Text {
                text: Cow::Borrowed("World")
            }
        );
    }

    #[test]
    fn test_player_head() {
        let component = TextComponent::player_head(ObjectPlayer::name("Jeb_"), true);
        assert!(matches!(
            component.content,
            Content::Object(Object::Player { .. })
        ));
    }

    #[test]
    fn test_scoreboard() {
        let component = TextComponent::scoreboard("@p", "deaths");
        assert!(matches!(
            component.content,
            Content::Resolvable(Resolvable::Scoreboard { .. })
        ));
    }

    #[test]
    fn test_add_child_and_children() {
        let parent = TextComponent::plain("Parent")
            .add_child("Child1")
            .add_children(vec!["Child2", "Child3"]);
        assert_eq!(parent.children.len(), 3);
        assert_eq!(
            parent.children[0].content,
            Content::Text {
                text: Cow::Borrowed("Child1")
            }
        );
    }

    #[test]
    fn test_insertion() {
        let component = TextComponent::plain("text").insertion("insert me");
        assert_eq!(
            component.interactions.insertion,
            Some(Cow::Borrowed("insert me"))
        );
    }

    #[test]
    fn test_click_event() {
        let event = ClickEvent::open_url("http://example.com");
        let component = TextComponent::plain("link").click_event(event.clone());
        assert_eq!(component.interactions.click, Some(event));
    }

    #[test]
    fn test_hover_event() {
        let hover = HoverEvent::show_text("tooltip");
        let component = TextComponent::plain("hover").hover_event(hover.clone());
        assert_eq!(component.interactions.hover, Some(hover));
    }

    #[test]
    fn test_color_and_hex() {
        let component = TextComponent::plain("colored").color_hex("#00ff00");
        assert!(component.format.color.is_some());
    }

    #[test]
    fn test_formatting_bold_italic_underline() {
        let component = TextComponent::plain("f")
            .bold(true)
            .italic(true)
            .underlined(true)
            .strikethrough(false);
        assert_eq!(component.format.bold, Some(true));
        assert_eq!(component.format.italic, Some(true));
        assert_eq!(component.format.underlined, Some(true));
        assert_eq!(component.format.strikethrough, Some(false));
    }

    #[test]
    fn test_reset_formatting() {
        let component = TextComponent::plain("x")
            .color(Color::Blue)
            .bold(true)
            .italic(true)
            .reset();
        assert_eq!(component.format.color, Some(Color::White));
        assert_eq!(component.format.bold, Some(false));
        assert_eq!(component.format.italic, Some(false));
        assert_eq!(
            component.format.font,
            Some(Cow::Borrowed("minecraft:default"))
        );
    }

    #[test]
    fn test_resolve_scoreboard_noresolutor() {
        let component = TextComponent::scoreboard("@p", "score");
        let resolved = component.resolve(&NoResolutor);
        assert!(
            matches!(resolved.content, Content::Text { text } if text.contains("[Score: score]"))
        );
    }

    #[test]
    fn test_resolve_entity_noresolutor() {
        let component = TextComponent::entity("@a", None);
        let resolved = component.resolve(&NoResolutor);
        assert!(
            matches!(resolved.content, Content::Text { text } if text.contains("[Entity: @a]"))
        );
    }

    #[test]
    fn test_build_plain_text() {
        let component = TextComponent::plain("Hello world");
        let result = component.build(&NoResolutor, StringTarget);
        assert_eq!(result, "Hello world");
    }
}
